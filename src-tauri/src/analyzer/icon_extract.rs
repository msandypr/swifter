use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use image::{DynamicImage, ImageFormat, RgbaImage};
use icns::{IconFamily, IconType, Image as IcnsImage, PixelFormat};
use pelite::pe32::{Pe as Pe32, PeFile as PeFile32};
use pelite::pe64::{Pe as Pe64, PeFile as PeFile64};
use pelite::Wrap;
use sha2::{Digest, Sha256};

/// Extracts the highest quality icon from the Windows executable and generates a macOS .icns file
pub fn extract_and_generate_icns(exe_path: &Path) -> Result<PathBuf, String> {
    let bytes = std::fs::read(exe_path)
        .map_err(|e| format!("Failed to read binary for icon extraction: {}", e))?;

    // Create cache directory ~/.cache/swifter/icons/
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("swifter")
        .join("icons");
    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create icon cache directory: {}", e))?;

    // Hash binary path for stable cache key
    let mut hasher = Sha256::new();
    hasher.update(exe_path.to_string_lossy().as_bytes());
    if bytes.len() > 4096 {
        hasher.update(&bytes[..4096]);
    } else {
        hasher.update(&bytes);
    }
    let hash_hex = format!("{:x}", hasher.finalize());
    let icns_path = cache_dir.join(format!("{}.icns", hash_hex));

    // If cached icns already exists and is not empty, reuse it
    if icns_path.exists() {
        if let Ok(meta) = icns_path.metadata() {
            if meta.len() > 0 {
                return Ok(icns_path);
            }
        }
    }

    // Attempt to extract the icon from PE resources
    let loaded_image = extract_best_image_from_pe(&bytes);

    let base_image = match loaded_image {
        Ok(img) => img,
        Err(_) => {
            // Generate fallback icon
            generate_fallback_game_icon()
        }
    };

    // Pack into .icns with macOS standard resolutions
    pack_image_to_icns(&base_image, &icns_path)?;

    Ok(icns_path)
}

fn extract_best_image_from_pe(bytes: &[u8]) -> Result<DynamicImage, String> {
    let wrap = Wrap::<PeFile32, PeFile64>::from_bytes(bytes)
        .map_err(|e| format!("Invalid PE binary: {:?}", e))?;

    match wrap {
        Wrap::T32(pe) => extract_best_image_from_resources(pe.resources().ok()),
        Wrap::T64(pe) => extract_best_image_from_resources(pe.resources().ok()),
    }
}

fn extract_best_image_from_resources(resources: Option<pelite::resources::Resources>) -> Result<DynamicImage, String> {
    let resources = resources.ok_or_else(|| "No resources found in PE".to_string())?;

    let mut best_image: Option<DynamicImage> = None;
    let mut best_res = 0u32;

    for group in resources.icons().filter_map(Result::ok) {
        // First check individual entries for high-res PNGs (often 256x256)
        for entry in group.1.entries() {
            if let Ok(raw_bytes) = group.1.image(entry.nId) {
                if raw_bytes.starts_with(b"\x89PNG") {
                    if let Ok(img) = image::load_from_memory_with_format(raw_bytes, ImageFormat::Png) {
                        let res = img.width().max(img.height());
                        if res > best_res {
                            best_res = res;
                            best_image = Some(img);
                        }
                    }
                }
            }
        }

        // Try reassembling the entire ICO group
        let mut ico_bytes = Vec::new();
        if group.1.write(&mut ico_bytes).is_ok() {
            if let Ok(img) = image::load_from_memory_with_format(&ico_bytes, ImageFormat::Ico) {
                let res = img.width().max(img.height());
                if res > best_res {
                    best_res = res;
                    best_image = Some(img);
                }
            }
        }
    }

    best_image.ok_or_else(|| "No valid icon image found in PE resources".to_string())
}

/// Converts DynamicImage into a multi-resolution Apple .icns container
pub fn pack_image_to_icns(base_img: &DynamicImage, output_path: &Path) -> Result<(), String> {
    let mut family = IconFamily::new();

    let target_sizes: &[(u32, IconType)] = &[
        (16, IconType::RGBA32_16x16),
        (32, IconType::RGBA32_32x32),
        (64, IconType::RGBA32_64x64),
        (128, IconType::RGBA32_128x128),
        (256, IconType::RGBA32_256x256),
        (512, IconType::RGBA32_512x512),
        (1024, IconType::RGBA32_512x512_2x),
    ];

    for &(size, icon_type) in target_sizes {
        let resized = base_img.resize_exact(size, size, image::imageops::FilterType::Lanczos3);
        let rgba = resized.to_rgba8();
        let icns_image = IcnsImage::from_data(PixelFormat::RGBA, size, size, rgba.into_raw())
            .map_err(|e| format!("Failed to create ICNS image frame {}x{}: {}", size, size, e))?;

        family
            .add_icon_with_type(&icns_image, icon_type)
            .map_err(|e| format!("Failed to add icon type to family: {}", e))?;
    }

    let file = File::create(output_path)
        .map_err(|e| format!("Failed to create icns file at {}: {}", output_path.display(), e))?;
    let mut writer = BufWriter::new(file);
    family
        .write(&mut writer)
        .map_err(|e| format!("Failed to encode .icns: {}", e))?;

    Ok(())
}

/// Generates a modern gradient fallback icon if the executable has no icon
fn generate_fallback_game_icon() -> DynamicImage {
    let width = 512;
    let height = 512;
    let mut img = RgbaImage::new(width, height);

    let center_x = (width / 2) as f32;
    let center_y = (height / 2) as f32;
    let radius = 220.0f32;

    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist <= radius {
                // Smooth gradient from deep purple to indigo
                let t = (x + y) as f32 / (width + height) as f32;
                let r = (60.0 * (1.0 - t) + 120.0 * t) as u8;
                let g = (40.0 * (1.0 - t) + 80.0 * t) as u8;
                let b = (180.0 * (1.0 - t) + 240.0 * t) as u8;

                // Subtle game controller d-pad cross in the center
                let is_center_cross = (dx.abs() < 40.0 && dy.abs() < 12.0) || (dy.abs() < 40.0 && dx.abs() < 12.0);
                if is_center_cross {
                    img.put_pixel(x, y, image::Rgba([255, 255, 255, 230]));
                } else {
                    img.put_pixel(x, y, image::Rgba([r, g, b, 255]));
                }
            } else if dist <= radius + 2.0 {
                // Antialiasing edge
                let alpha = ((radius + 2.0 - dist) / 2.0 * 255.0) as u8;
                img.put_pixel(x, y, image::Rgba([80, 50, 200, alpha]));
            } else {
                img.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
            }
        }
    }

    DynamicImage::ImageRgba8(img)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_icon_and_pack_icns() {
        let icon = generate_fallback_game_icon();
        assert_eq!(icon.width(), 512);
        assert_eq!(icon.height(), 512);

        let temp_icns = std::env::temp_dir().join("swifter_test_icon.icns");
        let _ = std::fs::remove_file(&temp_icns);

        let res = pack_image_to_icns(&icon, &temp_icns);
        assert!(res.is_ok(), "Failed to pack icns: {:?}", res);
        assert!(temp_icns.exists());
        let meta = std::fs::metadata(&temp_icns).unwrap();
        assert!(meta.len() > 1000, "ICNS file should contain packed image elements");

        let _ = std::fs::remove_file(&temp_icns);
    }
}
