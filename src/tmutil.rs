use std::path::{Path, PathBuf};
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

pub fn are_excluded(paths: &[PathBuf]) -> Result<Vec<bool>, String> {
    if paths.is_empty() {
        return Ok(vec![]);
    }

    let output = Command::new("tmutil")
        .arg("isexcluded")
        .args(paths)
        .output()
        .map_err(|e| format!("failed to run tmutil: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let results = parse_are_excluded(&stdout);

    if results.len() != paths.len() {
        return Err(format!(
            "tmutil returned {} results for {} paths",
            results.len(),
            paths.len()
        ));
    }

    Ok(results)
}

fn parse_are_excluded(output: &str) -> Vec<bool> {
    output
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| line.starts_with("[Excluded]"))
        .collect()
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
    output
        .lines()
        .find(|line| !line.is_empty())
        .is_some_and(|line| line.starts_with("[Excluded]"))
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
    fn parses_batch_excluded_output() {
        let output = "[Excluded]      /Users/dev/project/node_modules\n[Excluded]      /Users/dev/project/target\n";
        let results = parse_are_excluded(output);
        assert_eq!(results, vec![true, true]);
    }

    #[test]
    fn parses_batch_mixed_output() {
        let output = "[Excluded]      /Users/dev/project/node_modules\n[NotExcluded]   /Users/dev/project/src\n[Excluded]      /Users/dev/project/target\n";
        let results = parse_are_excluded(output);
        assert_eq!(results, vec![true, false, true]);
    }

    #[test]
    fn parses_batch_empty_output() {
        let results = parse_are_excluded("");
        assert!(results.is_empty());
    }

    #[test]
    fn parses_batch_all_not_excluded() {
        let output =
            "[NotExcluded]   /Users/dev/project/src\n[NotExcluded]   /Users/dev/project/docs\n";
        let results = parse_are_excluded(output);
        assert_eq!(results, vec![false, false]);
    }

    #[test]
    fn parse_is_excluded_only_checks_first_line() {
        let output = "[Excluded]      /Users/dev/node_modules\n[NotExcluded]   /Users/dev/src\n";
        assert!(parse_is_excluded(output));

        let output = "[NotExcluded]   /Users/dev/src\n[Excluded]      /Users/dev/node_modules\n";
        assert!(!parse_is_excluded(output));
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
