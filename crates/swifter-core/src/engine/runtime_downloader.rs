use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

pub const DEFAULT_SWIFTER_USER_AGENT: &str =
    concat!("Swifter/", env!("CARGO_PKG_VERSION"), " (+https://github.com/msandypr/swifter)");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DownloadProgress {
    pub stage: String,
    pub percent: f64,
    pub downloaded_mb: f64,
    pub total_mb: f64,
    pub speed_mb: f64,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadEndpoint {
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub is_api_asset: bool,
}

#[derive(Debug, Clone)]
pub struct RuntimeAssetSpec {
    pub name: &'static str,
    pub repo: &'static str,
    pub tag: &'static str,
    pub asset_filename: &'static str,
    pub default_size: u64,
    pub primary_url: &'static str,
    pub fallback_urls: &'static [&'static str],
}

pub const WINE_RUNTIME_SPEC: RuntimeAssetSpec = RuntimeAssetSpec {
    name: "wine",
    repo: "Gcenx/macOS_Wine_builds",
    tag: "11.18",
    asset_filename: "wine-devel-11.18-osx64.tar.xz",
    default_size: 190_974_384,
    primary_url: "https://github.com/Gcenx/macOS_Wine_builds/releases/download/11.18/wine-devel-11.18-osx64.tar.xz",
    fallback_urls: &[
        "https://github.com/Gcenx/macOS_Wine_builds/releases/download/10.16/wine-devel-10.16-osx64.tar.xz",
        "https://github.com/Gcenx/macOS_Wine_builds/releases/download/9.18/wine-devel-9.18-osx64.tar.xz",
    ],
};

pub const DXVK_SPEC: RuntimeAssetSpec = RuntimeAssetSpec {
    name: "dxvk",
    repo: "Gcenx/DXVK-macOS",
    tag: "v1.10.3-20230507-repack",
    asset_filename: "dxvk-macOS-async-v1.10.3-20230507-repack.tar.gz",
    default_size: 2_828_000,
    primary_url: "https://github.com/Gcenx/DXVK-macOS/releases/download/v1.10.3-20230507-repack/dxvk-macOS-async-v1.10.3-20230507-repack.tar.gz",
    fallback_urls: &[
        "https://github.com/Gcenx/DXVK-macOS/releases/download/v1.10.3-20230507/dxvk-macOS-async-v1.10.3-20230507.tar.gz",
    ],
};

pub const WINEMETALGL_SPEC: RuntimeAssetSpec = RuntimeAssetSpec {
    name: "winemetalgl",
    repo: "metalsharp/WineMetalGL",
    tag: "winemetalgl-3.7.0",
    asset_filename: "winemetalgl-3.7.0-x86_64.tar.gz",
    default_size: 15_700_000,
    primary_url: "https://github.com/metalsharp/WineMetalGL/releases/download/winemetalgl-3.7.0/winemetalgl-3.7.0-x86_64.tar.gz",
    fallback_urls: &[
        "https://github.com/metalsharp/WineMetalGL/releases/download/winemetalgl-3.6.0/winemetalgl-3.6.0-x86_64.tar.gz",
    ],
};

pub const VCREDIST_SPEC: RuntimeAssetSpec = RuntimeAssetSpec {
    name: "vcredist",
    repo: "",
    tag: "",
    asset_filename: "vc_redist.x64.exe",
    default_size: 25_358_000,
    primary_url: "https://aka.ms/vs/17/release/vc_redist.x64.exe",
    fallback_urls: &[
        "https://download.visualstudio.microsoft.com/download/pr/9b460d3c-9749-418e-a22c-a81387d3744d/F8A6BE4CEE5BA9A4F99831CF84577B5EEACBF84F2EAF1D5A67406C6B6FBFE785/VC_redist.x64.exe",
    ],
};

#[derive(Deserialize, Debug)]
struct GitHubAssetJson {
    name: String,
    size: u64,
    url: String,
    browser_download_url: String,
}

#[derive(Deserialize, Debug)]
struct GitHubReleaseJson {
    #[serde(default)]
    assets: Vec<GitHubAssetJson>,
}

/// Parses Content-Range or Content-Length from an HTTP response header dump
pub fn parse_content_range_or_length(headers: &str) -> Option<u64> {
    for line in headers.lines().rev() {
        let trimmed = line.trim();
        let lower = trimmed.to_ascii_lowercase();

        // 1. Content-Range: bytes 0-0/190974384
        if lower.starts_with("content-range:") {
            if let Some(slash_idx) = trimmed.rfind('/') {
                let size_str = trimmed[slash_idx + 1..].trim();
                if let Ok(val) = size_str.parse::<u64>() {
                    if val > 0 {
                        return Some(val);
                    }
                }
            }
        }

        // 2. Content-Length: 190974384
        if lower.starts_with("content-length:") {
            if let Some(colon_idx) = trimmed.find(':') {
                let size_str = trimmed[colon_idx + 1..].trim();
                if let Ok(val) = size_str.parse::<u64>() {
                    if val > 0 {
                        return Some(val);
                    }
                }
            }
        }
    }
    None
}

/// Extracts asset endpoint & size information from GitHub API release JSON
pub fn extract_asset_info_from_release_json(
    json_str: &str,
    target_filename: &str,
) -> Option<(DownloadEndpoint, DownloadEndpoint, u64)> {
    let release: GitHubReleaseJson = serde_json::from_str(json_str).ok()?;
    for asset in release.assets {
        if asset.name.eq_ignore_ascii_case(target_filename) {
            let api_endpoint = DownloadEndpoint {
                url: asset.url,
                headers: vec![("Accept".to_string(), "application/octet-stream".to_string())],
                is_api_asset: true,
            };
            let direct_endpoint = DownloadEndpoint {
                url: asset.browser_download_url,
                headers: vec![],
                is_api_asset: false,
            };
            return Some((api_endpoint, direct_endpoint, asset.size));
        }
    }
    None
}

/// Resolves candidate download endpoints and queries real dynamic content size
pub fn resolve_asset_endpoints(spec: &RuntimeAssetSpec, user_agent: &str) -> (Vec<DownloadEndpoint>, u64) {
    let mut endpoints: Vec<DownloadEndpoint> = Vec::new();
    let mut dynamic_size = None;

    // 1. Try querying GitHub Releases API if repo is configured
    if !spec.repo.is_empty() {
        let api_url = if spec.tag.is_empty() || spec.tag == "latest" {
            format!("https://api.github.com/repos/{}/releases/latest", spec.repo)
        } else {
            format!("https://api.github.com/repos/{}/releases/tags/{}", spec.repo, spec.tag)
        };

        let output = Command::new("/usr/bin/curl")
            .arg("-s")
            .arg("--connect-timeout")
            .arg("4")
            .arg("-m")
            .arg("6")
            .arg("-A")
            .arg(user_agent)
            .arg(&api_url)
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                if let Ok(json_text) = std::str::from_utf8(&out.stdout) {
                    if let Some((api_ep, direct_ep, size)) =
                        extract_asset_info_from_release_json(json_text, spec.asset_filename)
                    {
                        endpoints.push(api_ep);
                        endpoints.push(direct_ep);
                        dynamic_size = Some(size);
                    }
                }
            }
        }
    }

    // 2. Add Primary URL if not already added
    if !endpoints.iter().any(|e| e.url == spec.primary_url) {
        endpoints.push(DownloadEndpoint {
            url: spec.primary_url.to_string(),
            headers: vec![],
            is_api_asset: false,
        });
    }

    // 3. Add Fallback Mirror URLs
    for fallback in spec.fallback_urls {
        if !endpoints.iter().any(|e| &e.url == fallback) {
            endpoints.push(DownloadEndpoint {
                url: fallback.to_string(),
                headers: vec![],
                is_api_asset: false,
            });
        }
    }

    // 4. If dynamic size was not determined via GitHub API, probe via Range header on the first direct endpoint
    let final_size = match dynamic_size {
        Some(s) if s > 0 => s,
        _ => {
            let mut probed = None;
            for ep in &endpoints {
                if ep.is_api_asset {
                    continue;
                }
                let mut cmd = Command::new("/usr/bin/curl");
                cmd.arg("-s")
                    .arg("-L")
                    .arg("-D")
                    .arg("-")
                    .arg("-r")
                    .arg("0-0")
                    .arg("--connect-timeout")
                    .arg("4")
                    .arg("-m")
                    .arg("6")
                    .arg("-A")
                    .arg(user_agent)
                    .arg("-o")
                    .arg("/dev/null")
                    .arg(&ep.url);

                if let Ok(out) = cmd.output() {
                    if out.status.success() {
                        if let Ok(hdr_text) = std::str::from_utf8(&out.stdout) {
                            if let Some(parsed) = parse_content_range_or_length(hdr_text) {
                                probed = Some(parsed);
                                break;
                            }
                        }
                    }
                }
            }
            probed.unwrap_or(spec.default_size)
        }
    };

    (endpoints, final_size)
}

/// Verifies that the archive matches expected magic bytes
pub fn verify_archive_magic(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err("Target file does not exist.".to_string());
    }

    let mut file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut header = [0u8; 6];
    let read_bytes = file.read(&mut header).map_err(|e| format!("Failed to read magic bytes: {}", e))?;

    if read_bytes < 2 {
        return Err("File is too short or empty.".to_string());
    }

    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

    if filename.ends_with(".tar.xz") || ext == "xz" {
        // xz magic bytes: \xFD 7zXZ \x00 -> [0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00]
        if read_bytes >= 6 && &header[..6] == [0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00] {
            return Ok(());
        }
        return Err("File is not a valid .tar.xz archive (magic bytes mismatch).".to_string());
    } else if filename.ends_with(".tar.gz") || ext == "gz" {
        // gzip magic bytes: [0x1F, 0x8B]
        if header[0] == 0x1F && header[1] == 0x8B {
            return Ok(());
        }
        return Err("File is not a valid .tar.gz archive (magic bytes mismatch).".to_string());
    } else if ext == "exe" {
        // PE DOS magic bytes: "MZ" -> [0x4D, 0x5A]
        if header[0] == 0x4D && header[1] == 0x5A {
            return Ok(());
        }
        return Err("File is not a valid Windows executable (magic bytes mismatch).".to_string());
    }

    Ok(())
}

/// Downloads a runtime asset using fallback endpoints with dynamic speed and progress callbacks
pub fn download_file_with_fallback<F>(
    endpoints: &[DownloadEndpoint],
    dest_path: &Path,
    expected_size: u64,
    user_agent: &str,
    progress_callback: F,
) -> Result<(), String>
where
    F: Fn(DownloadProgress),
{
    let part_path = PathBuf::from(format!("{}.part", dest_path.display()));
    let total_mb = expected_size as f64 / (1024.0 * 1024.0);
    let mut last_error = String::new();

    for (idx, endpoint) in endpoints.iter().enumerate() {
        let _ = std::fs::remove_file(&part_path);
        let _ = std::fs::remove_file(dest_path);

        let mirror_name = if endpoint.is_api_asset {
            "GitHub API Mirror (Fast CDN)"
        } else if endpoint.url.contains("github.com") {
            "GitHub Release Direct"
        } else if endpoint.url.contains("microsoft") || endpoint.url.contains("aka.ms") {
            "Microsoft CDN"
        } else {
            "Secondary Fallback Mirror"
        };

        progress_callback(DownloadProgress {
            stage: "downloading".to_string(),
            percent: 0.0,
            downloaded_mb: 0.0,
            total_mb,
            speed_mb: 0.0,
            message: format!(
                "Connecting to {} (Mirror {}/{})...",
                mirror_name,
                idx + 1,
                endpoints.len()
            ),
        });

        let mut cmd = Command::new("/usr/bin/curl");
        cmd.arg("-L")
            .arg("-f")
            .arg("-A")
            .arg(user_agent)
            .arg("--connect-timeout")
            .arg("12")
            .arg("--retry")
            .arg("2")
            .arg("-o")
            .arg(&part_path);

        for (k, v) in &endpoint.headers {
            cmd.arg("-H").arg(format!("{}: {}", k, v));
        }

        cmd.arg(&endpoint.url);

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                last_error = format!("Failed to spawn curl: {}", e);
                continue;
            }
        };

        let mut prev_bytes: u64 = 0;
        let mut last_tick = Instant::now();
        let mut download_succeeded = false;

        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    if status.success() {
                        download_succeeded = true;
                    } else {
                        last_error = format!("Endpoint failed with curl exit code {:?}", status.code());
                    }
                    break;
                }
                Ok(None) => {
                    let now = Instant::now();
                    let elapsed = now.duration_since(last_tick).as_secs_f64().max(0.1);

                    let current_bytes = std::fs::metadata(&part_path).map(|m| m.len()).unwrap_or(0);
                    let downloaded_mb = current_bytes as f64 / (1024.0 * 1024.0);
                    let percent = if expected_size > 0 {
                        ((current_bytes as f64 / expected_size as f64) * 100.0).min(99.0)
                    } else {
                        50.0
                    };

                    let speed_mb = ((current_bytes.saturating_sub(prev_bytes)) as f64 / elapsed) / (1024.0 * 1024.0);
                    prev_bytes = current_bytes;
                    last_tick = now;

                    progress_callback(DownloadProgress {
                        stage: "downloading".to_string(),
                        percent,
                        downloaded_mb,
                        total_mb,
                        speed_mb,
                        message: format!(
                            "{:.1} MB / {:.1} MB ({:.1} MB/s)",
                            downloaded_mb, total_mb, speed_mb
                        ),
                    });

                    std::thread::sleep(Duration::from_millis(400));
                }
                Err(e) => {
                    last_error = format!("Process monitor error: {}", e);
                    break;
                }
            }
        }

        if download_succeeded && part_path.exists() {
            let actual_len = std::fs::metadata(&part_path).map(|m| m.len()).unwrap_or(0);
            // Verify minimum size (at least 70% of expected)
            let min_expected = (expected_size * 7) / 10;
            if actual_len >= min_expected {
                if let Err(e) = verify_archive_magic(&part_path) {
                    last_error = format!("Archive verification error: {}", e);
                    let _ = std::fs::remove_file(&part_path);
                    continue;
                }

                if std::fs::rename(&part_path, dest_path).is_ok() {
                    progress_callback(DownloadProgress {
                        stage: "downloading".to_string(),
                        percent: 100.0,
                        downloaded_mb: actual_len as f64 / (1024.0 * 1024.0),
                        total_mb,
                        speed_mb: 0.0,
                        message: format!("Download complete ({:.1} MB).", actual_len as f64 / (1024.0 * 1024.0)),
                    });
                    return Ok(());
                }
            } else {
                last_error = format!(
                    "Downloaded file is too small ({} bytes vs expected {} bytes).",
                    actual_len, expected_size
                );
            }
        }

        let _ = std::fs::remove_file(&part_path);
    }

    Err(format!(
        "All download mirrors failed. Last error: {}",
        if last_error.is_empty() {
            "Connection interrupted or timed out"
        } else {
            &last_error
        }
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_content_range() {
        let headers = "HTTP/1.1 206 Partial Content\r\nContent-Type: application/octet-stream\r\nContent-Range: bytes 0-0/190974384\r\nConnection: keep-alive\r\n";
        assert_eq!(parse_content_range_or_length(headers), Some(190974384));
    }

    #[test]
    fn test_parse_content_length() {
        let headers = "HTTP/2 200\r\nserver: GitHub.com\r\ncontent-length: 2828000\r\ncontent-type: application/x-gzip\r\n";
        assert_eq!(parse_content_range_or_length(headers), Some(2828000));
    }

    #[test]
    fn test_extract_asset_info_from_release_json() {
        let json = r#"{
            "tag_name": "11.18",
            "assets": [
                {
                    "name": "other-file.txt",
                    "size": 100,
                    "url": "https://api.github.com/assets/1",
                    "browser_download_url": "https://github.com/download/other-file.txt"
                },
                {
                    "name": "wine-devel-11.18-osx64.tar.xz",
                    "size": 190974384,
                    "url": "https://api.github.com/assets/588640823",
                    "browser_download_url": "https://github.com/download/wine-devel-11.18-osx64.tar.xz"
                }
            ]
        }"#;

        let res = extract_asset_info_from_release_json(json, "wine-devel-11.18-osx64.tar.xz");
        assert!(res.is_some());
        let (api_ep, direct_ep, size) = res.unwrap();
        assert_eq!(size, 190974384);
        assert!(api_ep.is_api_asset);
        assert_eq!(api_ep.url, "https://api.github.com/assets/588640823");
        assert!(!direct_ep.is_api_asset);
        assert_eq!(direct_ep.url, "https://github.com/download/wine-devel-11.18-osx64.tar.xz");
    }

    #[test]
    fn test_verify_archive_magic() {
        let temp_dir = std::env::temp_dir().join(format!("swifter_magic_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);

        // 1. Valid xz magic
        let xz_path = temp_dir.join("test.tar.xz");
        let mut f1 = File::create(&xz_path).unwrap();
        f1.write_all(&[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00, 0x11, 0x22]).unwrap();
        assert!(verify_archive_magic(&xz_path).is_ok());

        // 2. Corrupted xz
        let bad_xz = temp_dir.join("bad.tar.xz");
        let mut f2 = File::create(&bad_xz).unwrap();
        f2.write_all(b"<html>404 Not Found</html>").unwrap();
        assert!(verify_archive_magic(&bad_xz).is_err());

        // 3. Valid gzip magic
        let gz_path = temp_dir.join("test.tar.gz");
        let mut f3 = File::create(&gz_path).unwrap();
        f3.write_all(&[0x1F, 0x8B, 0x08, 0x00]).unwrap();
        assert!(verify_archive_magic(&gz_path).is_ok());

        // 4. Valid PE exe magic
        let exe_path = temp_dir.join("test.exe");
        let mut f4 = File::create(&exe_path).unwrap();
        f4.write_all(b"MZ\x90\x00\x03\x00\x00\x00").unwrap();
        assert!(verify_archive_magic(&exe_path).is_ok());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
