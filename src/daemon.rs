use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const LABEL: &str = "com.veiled.agent";

fn current_uid() -> u32 {
    // SAFETY: getuid() is a single syscall with no failure mode
    unsafe { libc::getuid() }
}

fn domain_target() -> String {
    format!("gui/{}", current_uid())
}

fn service_target() -> String {
    format!("gui/{}/{LABEL}", current_uid())
}

pub fn plist_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = dirs::home_dir().ok_or("could not determine home directory")?;
    Ok(home
        .join("Library/LaunchAgents")
        .join(format!("{LABEL}.plist")))
}

fn log_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = dirs::home_dir().ok_or("could not determine home directory")?;
    Ok(home.join(".config/veiled"))
}

pub fn generate_plist(binary_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let binary = binary_path.display();
    let log = log_dir()?.display().to_string();

    Ok(format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{binary}</string>
        <string>run</string>
    </array>
    <key>StartCalendarInterval</key>
    <dict>
        <key>Hour</key>
        <integer>3</integer>
        <key>Minute</key>
        <integer>0</integer>
    </dict>
    <key>RunAtLoad</key>
    <false/>
    <key>StandardOutPath</key>
    <string>{log}/stdout.log</string>
    <key>StandardErrorPath</key>
    <string>{log}/stderr.log</string>
</dict>
</plist>
"#
    ))
}

pub fn is_installed() -> Result<bool, Box<dyn std::error::Error>> {
    Ok(plist_path()?.exists())
}

pub fn install(plist_content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = plist_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let log = log_dir()?;
    fs::create_dir_all(&log)?;

    fs::write(&path, plist_content)?;

    let output = Command::new("launchctl")
        .args(["bootstrap", &domain_target()])
        .arg(&path)
        .output()
        .map_err(|e| format!("failed to run launchctl: {e}"))?;

    if !output.status.success() {
        fs::remove_file(&path).ok();
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("launchctl bootstrap failed: {stderr}").into());
    }

    Ok(())
}

pub fn kickstart() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("launchctl")
        .args(["kickstart", &service_target()])
        .output()
        .map_err(|e| format!("failed to run launchctl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("launchctl kickstart failed: {stderr}").into());
    }

    Ok(())
}

pub fn uninstall() -> Result<(), Box<dyn std::error::Error>> {
    let path = plist_path()?;

    let output = Command::new("launchctl")
        .args(["bootout", &service_target()])
        .output()
        .map_err(|e| format!("failed to run launchctl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("warning: launchctl bootout failed: {stderr}");
    }

    fs::remove_file(&path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plist_path_ends_with_label() {
        let path = plist_path().unwrap();
        assert!(path.ends_with(format!("{LABEL}.plist")));
    }

    #[test]
    fn plist_path_is_under_launch_agents() {
        let path = plist_path().unwrap();
        let parent = path.parent().unwrap();
        assert!(parent.ends_with("Library/LaunchAgents"));
    }

    #[test]
    fn generate_plist_contains_label() {
        let plist = generate_plist(Path::new("/usr/local/bin/veiled")).unwrap();
        assert!(plist.contains(&format!("<string>{LABEL}</string>")));
    }

    #[test]
    fn generate_plist_contains_binary_path() {
        let plist = generate_plist(Path::new("/opt/homebrew/bin/veiled")).unwrap();
        assert!(plist.contains("<string>/opt/homebrew/bin/veiled</string>"));
    }

    #[test]
    fn generate_plist_contains_run_argument() {
        let plist = generate_plist(Path::new("/usr/local/bin/veiled")).unwrap();
        assert!(plist.contains("<string>run</string>"));
    }

    #[test]
    fn generate_plist_has_calendar_interval() {
        let plist = generate_plist(Path::new("/usr/local/bin/veiled")).unwrap();
        assert!(plist.contains("<key>StartCalendarInterval</key>"));
        assert!(plist.contains("<key>Hour</key>"));
        assert!(plist.contains("<integer>3</integer>"));
        assert!(plist.contains("<key>Minute</key>"));
        assert!(plist.contains("<integer>0</integer>"));
    }

    #[test]
    fn generate_plist_run_at_load_is_false() {
        let plist = generate_plist(Path::new("/usr/local/bin/veiled")).unwrap();
        assert!(plist.contains("<false/>"));
    }

    #[test]
    fn generate_plist_has_log_paths() {
        let plist = generate_plist(Path::new("/usr/local/bin/veiled")).unwrap();
        assert!(plist.contains("<key>StandardOutPath</key>"));
        assert!(plist.contains("stdout.log"));
        assert!(plist.contains("<key>StandardErrorPath</key>"));
        assert!(plist.contains("stderr.log"));
    }

    #[test]
    fn is_installed_returns_result() {
        let _ = is_installed().unwrap();
    }
}
