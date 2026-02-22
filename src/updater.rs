use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use serde::Deserialize;
use sha2::{Digest, Sha256};

const REPO: &str = "adeonir/veiled";

#[derive(Debug)]
pub struct UpdateResult {
    pub updated: bool,
    pub old_version: String,
    pub new_version: String,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

pub fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn platform_asset_name() -> String {
    let arch = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        _ => "x64",
    };
    format!("veiled-macos-{arch}")
}

fn parse_version(tag: &str) -> Result<semver::Version, Box<dyn std::error::Error>> {
    let version_str = tag.strip_prefix('v').unwrap_or(tag);
    Ok(semver::Version::parse(version_str)?)
}

fn parse_checksum(content: &str) -> Result<String, Box<dyn std::error::Error>> {
    let hex = content
        .split_whitespace()
        .next()
        .ok_or("empty checksum file")?;

    if hex.len() != 64 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("invalid SHA-256 digest: {hex}").into());
    }

    Ok(hex.to_lowercase())
}

fn compute_sha256(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

pub fn check() -> Result<UpdateResult, Box<dyn std::error::Error>> {
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");

    let response: Release = ureq::get(&url)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "veiled")
        .call()
        .map_err(|e| format!("failed to fetch latest release: {e}"))?
        .body_mut()
        .read_json()?;

    let old = current_version().to_string();
    let new = response.tag_name.clone();

    let current = parse_version(&old)?;
    let latest = parse_version(&new)?;

    if latest <= current {
        return Ok(UpdateResult {
            updated: false,
            old_version: old,
            new_version: new,
        });
    }

    let asset_name = platform_asset_name();
    let checksum_name = format!("{asset_name}.sha256");

    let binary_asset = response
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| format!("no binary available for this platform ({asset_name})"))?;

    let checksum_asset = response
        .assets
        .iter()
        .find(|a| a.name == checksum_name)
        .ok_or_else(|| format!("no checksum available for this platform ({checksum_name})"))?;

    download_and_replace(
        &binary_asset.browser_download_url,
        &checksum_asset.browser_download_url,
    )?;

    Ok(UpdateResult {
        updated: true,
        old_version: old,
        new_version: new,
    })
}

struct TempFile(PathBuf);

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn download_and_replace(
    binary_url: &str,
    checksum_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let binary_path =
        std::env::current_exe().map_err(|e| format!("failed to resolve binary path: {e}"))?;

    let parent = binary_path
        .parent()
        .ok_or("failed to resolve binary directory")?;

    let temp = TempFile(parent.join(".veiled-update"));

    let checksum_content = ureq::get(checksum_url)
        .header("User-Agent", "veiled")
        .call()
        .map_err(|e| format!("failed to download checksum: {e}"))?
        .into_body()
        .read_to_string()
        .map_err(|e| format!("failed to read checksum: {e}"))?;

    let expected = parse_checksum(&checksum_content)?;

    let bytes = ureq::get(binary_url)
        .header("User-Agent", "veiled")
        .call()
        .map_err(|e| format!("failed to download update: {e}"))?
        .into_body()
        .read_to_vec()
        .map_err(|e| format!("failed to read download: {e}"))?;

    let actual = compute_sha256(&bytes);

    if actual != expected {
        return Err(format!("checksum mismatch: expected {expected}, got {actual}").into());
    }

    fs::write(&temp.0, &bytes)?;
    fs::set_permissions(&temp.0, fs::Permissions::from_mode(0o755))?;
    fs::rename(&temp.0, &binary_path)?;

    // Rename succeeded, prevent Drop from removing the file
    std::mem::forget(temp);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_version_is_valid_semver() {
        let version = current_version();
        assert!(semver::Version::parse(version).is_ok());
    }

    #[test]
    fn platform_asset_name_contains_macos() {
        let name = platform_asset_name();
        assert!(name.starts_with("veiled-macos-"));
    }

    #[test]
    fn platform_asset_name_has_arch() {
        let name = platform_asset_name();
        assert!(name.ends_with("arm64") || name.ends_with("x64"));
    }

    #[test]
    fn parse_version_strips_v_prefix() {
        let version = parse_version("v1.2.3").unwrap();
        assert_eq!(version, semver::Version::new(1, 2, 3));
    }

    #[test]
    fn parse_version_handles_no_prefix() {
        let version = parse_version("1.2.3").unwrap();
        assert_eq!(version, semver::Version::new(1, 2, 3));
    }

    #[test]
    fn parse_version_rejects_invalid() {
        assert!(parse_version("not-a-version").is_err());
    }

    #[test]
    fn version_comparison_newer() {
        let current = parse_version("0.1.0").unwrap();
        let latest = parse_version("0.2.0").unwrap();
        assert!(latest > current);
    }

    #[test]
    fn version_comparison_same() {
        let current = parse_version("0.1.0").unwrap();
        let latest = parse_version("0.1.0").unwrap();
        assert!(latest <= current);
    }

    #[test]
    fn version_comparison_older() {
        let current = parse_version("0.2.0").unwrap();
        let latest = parse_version("0.1.0").unwrap();
        assert!(latest <= current);
    }

    #[test]
    fn deserialize_release_response() {
        let json = r#"{
            "tag_name": "v0.2.0",
            "assets": [
                {
                    "name": "veiled-macos-arm64",
                    "browser_download_url": "https://github.com/adeonir/veiled/releases/download/v0.2.0/veiled-macos-arm64"
                },
                {
                    "name": "veiled-macos-x64",
                    "browser_download_url": "https://github.com/adeonir/veiled/releases/download/v0.2.0/veiled-macos-x64"
                }
            ]
        }"#;

        let release: Release = serde_json::from_str(json).unwrap();
        assert_eq!(release.tag_name, "v0.2.0");
        assert_eq!(release.assets.len(), 2);
        assert_eq!(release.assets[0].name, "veiled-macos-arm64");
    }

    #[test]
    fn deserialize_release_with_no_assets() {
        let json = r#"{
            "tag_name": "v0.1.0",
            "assets": []
        }"#;

        let release: Release = serde_json::from_str(json).unwrap();
        assert!(release.assets.is_empty());
    }

    #[test]
    fn compute_sha256_produces_valid_hex() {
        let hash = compute_sha256(b"hello world");
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn compute_sha256_is_deterministic() {
        let a = compute_sha256(b"test data");
        let b = compute_sha256(b"test data");
        assert_eq!(a, b);
    }

    #[test]
    fn compute_sha256_differs_for_different_input() {
        let a = compute_sha256(b"hello");
        let b = compute_sha256(b"world");
        assert_ne!(a, b);
    }

    #[test]
    fn parse_checksum_extracts_hex_digest() {
        let content = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9  veiled-macos-arm64\n";
        let hex = parse_checksum(content).unwrap();
        assert_eq!(
            hex,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn parse_checksum_handles_bare_hex() {
        let content = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9\n";
        let hex = parse_checksum(content).unwrap();
        assert_eq!(
            hex,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn parse_checksum_normalizes_to_lowercase() {
        let content = "B94D27B9934D3E08A52E52D7DA7DABFAC484EFE37A5380EE9088F7ACE2EFCDE9  file\n";
        let hex = parse_checksum(content).unwrap();
        assert!(hex.chars().all(|c| !c.is_ascii_uppercase()));
    }

    #[test]
    fn parse_checksum_rejects_short_digest() {
        assert!(parse_checksum("abc123  file").is_err());
    }

    #[test]
    fn parse_checksum_rejects_empty() {
        assert!(parse_checksum("").is_err());
    }

    #[test]
    fn parse_checksum_rejects_non_hex() {
        let content = "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz  file";
        assert!(parse_checksum(content).is_err());
    }

    #[test]
    fn temp_file_cleanup_on_drop() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("temp-binary");
        fs::write(&path, b"data").unwrap();
        assert!(path.exists());

        let temp = TempFile(path.clone());
        drop(temp);

        assert!(!path.exists());
    }
}
