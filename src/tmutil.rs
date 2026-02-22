use std::path::Path;
use std::process::Command;

pub fn check_access() -> Result<(), String> {
    let output = Command::new("tmutil")
        .arg("isexcluded")
        .arg("/")
        .output()
        .map_err(|e| format!("failed to run tmutil: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(stderr.trim().to_string())
    }
}

pub fn add_exclusion(path: &Path) -> Result<(), String> {
    let output = Command::new("tmutil")
        .arg("addexclusion")
        .arg(path)
        .output()
        .map_err(|e| format!("failed to run tmutil: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("tmutil addexclusion failed: {stderr}"))
    }
}

pub fn remove_exclusion(path: &Path) -> Result<(), String> {
    let output = Command::new("tmutil")
        .arg("removeexclusion")
        .arg(path)
        .output()
        .map_err(|e| format!("failed to run tmutil: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("tmutil removeexclusion failed: {stderr}"))
    }
}

pub fn is_excluded(path: &Path) -> Result<bool, String> {
    let output = Command::new("tmutil")
        .arg("isexcluded")
        .arg(path)
        .output()
        .map_err(|e| format!("failed to run tmutil: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_is_excluded(&stdout))
}

// tmutil outputs `[Excluded] /path` or `[NotExcluded] /path`
fn parse_is_excluded(output: &str) -> bool {
    output.contains("[Excluded]") && !output.contains("[NotExcluded]")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_excluded_output() {
        let output = "[Excluded]      /Users/dev/project/node_modules\n";
        assert!(parse_is_excluded(output));
    }

    #[test]
    fn parses_not_excluded_output() {
        let output = "[NotExcluded]   /Users/dev/project/src\n";
        assert!(!parse_is_excluded(output));
    }

    #[test]
    fn parses_empty_output() {
        assert!(!parse_is_excluded(""));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn remove_exclusion_on_nonexistent_path() {
        let result = remove_exclusion(Path::new("/nonexistent/path/that/does/not/exist"));
        assert!(result.is_err());
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn is_excluded_on_nonexistent_path() {
        let result = is_excluded(Path::new("/nonexistent/path/that/does/not/exist"));
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
