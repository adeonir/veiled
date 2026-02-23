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

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn generate_plist(binary_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let binary = escape_xml(&binary_path.display().to_string());
    let log = escape_xml(&log_dir()?.display().to_string());

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

fn bootout() -> Result<(), String> {
    let output = Command::new("launchctl")
        .args(["bootout", &service_target()])
        .output()
        .map_err(|e| format!("failed to run launchctl: {e}"))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);

    if stderr.contains("Could not find service") {
        return Ok(());
    }

    Err(stderr.trim().to_string())
}

fn kill_service() -> Result<(), String> {
    let output = Command::new("launchctl")
        .args(["kill", "SIGTERM", &service_target()])
        .output()
        .map_err(|e| format!("failed to run launchctl: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(stderr.trim().to_string())
    }
}

pub fn restart() -> Result<bool, Box<dyn std::error::Error>> {
    if !is_installed()? {
        return Ok(false);
    }

    uninstall()?;

    let binary_path =
        std::env::current_exe().map_err(|e| format!("failed to resolve binary path: {e}"))?;

    let plist = generate_plist(&binary_path)?;
    install(&plist)?;

    Ok(true)
}

pub fn uninstall() -> Result<(), Box<dyn std::error::Error>> {
    let path = plist_path()?;

    if let Err(reason) = bootout() {
        let _ = kill_service();

        if let Err(retry) = bootout() {
            return Err(format!("failed to stop service: {reason} (retry: {retry})").into());
        }
    }

    if let Err(e) = fs::remove_file(&path)
        && e.kind() != std::io::ErrorKind::NotFound
    {
        return Err(e.into());
    }

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

    #[test]
    fn domain_target_has_gui_uid_format() {
        let target = domain_target();
        assert!(
            target.starts_with("gui/"),
            "expected gui/ prefix, got: {target}"
        );
        let uid_str = target.strip_prefix("gui/").unwrap();
        assert!(
            uid_str.parse::<u32>().is_ok(),
            "expected numeric uid, got: {uid_str}"
        );
    }

    #[test]
    fn service_target_has_gui_uid_label_format() {
        let target = service_target();
        let expected_suffix = format!("/{LABEL}");
        assert!(
            target.ends_with(&expected_suffix),
            "expected {expected_suffix} suffix, got: {target}"
        );
        assert!(
            target.starts_with("gui/"),
            "expected gui/ prefix, got: {target}"
        );
    }

    #[test]
    fn current_uid_is_nonzero_in_user_context() {
        assert!(current_uid() > 0);
    }

    #[test]
    fn escape_xml_replaces_ampersand() {
        assert_eq!(escape_xml("a&b"), "a&amp;b");
    }

    #[test]
    fn escape_xml_replaces_angle_brackets() {
        assert_eq!(escape_xml("a<b>c"), "a&lt;b&gt;c");
    }

    #[test]
    fn escape_xml_replaces_quotes() {
        assert_eq!(escape_xml(r#"a"b'c"#), "a&quot;b&apos;c");
    }

    #[test]
    fn escape_xml_handles_multiple_special_chars() {
        assert_eq!(escape_xml(r#"<a&b>"c'd"#), "&lt;a&amp;b&gt;&quot;c&apos;d");
    }

    #[test]
    fn escape_xml_leaves_normal_string_unchanged() {
        let path = "/usr/local/bin/veiled";
        assert_eq!(escape_xml(path), path);
    }

    #[test]
    fn generate_plist_escapes_special_chars_in_path() {
        let plist = generate_plist(Path::new("/opt/my&app/veiled")).unwrap();
        assert!(plist.contains("<string>/opt/my&amp;app/veiled</string>"));
        assert!(!plist.contains("<string>/opt/my&app/veiled</string>"));
    }
}
