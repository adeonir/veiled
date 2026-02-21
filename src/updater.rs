use std::fs;
use std::os::unix::fs::PermissionsExt;

use serde::Deserialize;

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
    let asset = response
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| format!("no binary available for this platform ({asset_name})"))?;

    download_and_replace(&asset.browser_download_url)?;

    Ok(UpdateResult {
        updated: true,
        old_version: old,
        new_version: new,
    })
}

fn download_and_replace(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let binary_path =
        std::env::current_exe().map_err(|e| format!("failed to resolve binary path: {e}"))?;

    let parent = binary_path
        .parent()
        .ok_or("failed to resolve binary directory")?;

    let temp_path = parent.join(".veiled-update");

    let bytes = ureq::get(url)
        .header("User-Agent", "veiled")
        .call()
        .map_err(|e| format!("failed to download update: {e}"))?
        .into_body()
        .read_to_vec()
        .map_err(|e| format!("failed to read download: {e}"))?;

    fs::write(&temp_path, &bytes)?;
    fs::set_permissions(&temp_path, fs::Permissions::from_mode(0o755))?;
    fs::rename(&temp_path, &binary_path)?;

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
}
