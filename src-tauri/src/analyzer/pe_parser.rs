use std::path::Path;
use pelite::pe32::{Pe as Pe32, PeFile as PeFile32};
use pelite::pe64::{Pe as Pe64, PeFile as PeFile64};
use pelite::Wrap;

#[derive(Debug, Clone)]
pub struct PeMetadata {
    pub is_64bit: bool,
    pub product_name: Option<String>,
    pub file_description: Option<String>,
    pub company_name: Option<String>,
    pub original_filename: Option<String>,
}

/// Known anti-cheat kernel driver or service signatures
const ANTICHEAT_SIGNATURES: &[(&str, &str)] = &[
    ("vgk.sys", "Riot Vanguard (kernel-level anti-cheat)"),
    ("easyanticheat.sys", "Easy Anti-Cheat (kernel-level driver)"),
    ("easyanticheat_x64.sys", "Easy Anti-Cheat (kernel-level driver)"),
    ("bedaisy.sys", "BattlEye (kernel-level anti-cheat)"),
    ("beservice.exe", "BattlEye Service"),
    ("mhyprot2.sys", "miHoYo Anti-Cheat"),
    ("ace_loader.sys", "Tencent Anti-Cheat Expert"),
    ("blackcipher.aes", "Nexon BlackCipher"),
    ("gamemon.des", "nProtect GameGuard"),
    ("dbdata.dll", "Denuvo Anti-Tamper (unsupported on Wine)"),
    ("denuvo64.dll", "Denuvo Anti-Tamper (unsupported on Wine)"),
];

/// Inspects a PE binary file and extracts architecture and product metadata
pub fn parse_pe_binary(exe_path: &Path) -> Result<PeMetadata, String> {
    let bytes = std::fs::read(exe_path)
        .map_err(|e| format!("Failed to read executable at {}: {}", exe_path.display(), e))?;

    if bytes.len() < 64 {
        return Err("File too small to be a valid Windows executable".into());
    }

    // Wrap automatically parses both 32-bit and 64-bit PE binaries
    let wrap = Wrap::<PeFile32, PeFile64>::from_bytes(&bytes)
        .map_err(|e| format!("Invalid or corrupted PE executable: {:?}", e))?;

    match wrap {
        Wrap::T32(pe) => {
            let (product_name, file_description, company_name, original_filename) = extract_version_strings_32(&pe);
            Ok(PeMetadata {
                is_64bit: false,
                product_name,
                file_description,
                company_name,
                original_filename,
            })
        }
        Wrap::T64(pe) => {
            let (product_name, file_description, company_name, original_filename) = extract_version_strings_64(&pe);
            Ok(PeMetadata {
                is_64bit: true,
                product_name,
                file_description,
                company_name,
                original_filename,
            })
        }
    }
}

fn extract_version_strings_32(pe: &PeFile32) -> (Option<String>, Option<String>, Option<String>, Option<String>) {
    extract_from_resources(pe.resources().ok())
}

fn extract_version_strings_64(pe: &PeFile64) -> (Option<String>, Option<String>, Option<String>, Option<String>) {
    extract_from_resources(pe.resources().ok())
}

fn extract_from_resources(resources: Option<pelite::resources::Resources>) -> (Option<String>, Option<String>, Option<String>, Option<String>) {
    let resources = match resources {
        Some(res) => res,
        None => return (None, None, None, None),
    };

    let version_info = match resources.version_info() {
        Ok(v) => v,
        Err(_) => return (None, None, None, None),
    };

    // Version info contains StringFileInfo with language keys
    let mut product_name = None;
    let mut file_description = None;
    let mut company_name = None;
    let mut original_filename = None;

    let file_info = version_info.file_info();
    for string_map in file_info.strings.values() {
        if product_name.is_none() {
            if let Some(v) = string_map.get("ProductName") {
                if !v.trim().is_empty() {
                    product_name = Some(v.trim().to_string());
                }
            }
        }
        if file_description.is_none() {
            if let Some(v) = string_map.get("FileDescription") {
                if !v.trim().is_empty() {
                    file_description = Some(v.trim().to_string());
                }
            }
        }
        if company_name.is_none() {
            if let Some(v) = string_map.get("CompanyName") {
                if !v.trim().is_empty() {
                    company_name = Some(v.trim().to_string());
                }
            }
        }
        if original_filename.is_none() {
            if let Some(v) = string_map.get("OriginalFilename") {
                if !v.trim().is_empty() {
                    original_filename = Some(v.trim().to_string());
                }
            }
        }
    }

    (product_name, file_description, company_name, original_filename)
}

/// Checks if any file in the game directory matches known kernel anti-cheat drivers
pub fn check_anticheat_guard(game_folder: &Path) -> Option<String> {
    for entry in walkdir::WalkDir::new(game_folder)
        .max_depth(4)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let file_name = entry.file_name().to_string_lossy().to_lowercase();
        for (sig, desc) in ANTICHEAT_SIGNATURES {
            if file_name == *sig || file_name.starts_with(sig) {
                return Some(format!(
                    "{} detected ({}) at {}. Kernel-level anti-cheat cannot run under Wine/macOS.",
                    desc, sig, entry.path().display()
                ));
            }
        }
    }
    None
}
