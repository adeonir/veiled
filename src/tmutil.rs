use std::path::{Path, PathBuf};
use std::process::Command;

const XATTR_KEY: &str = "com.apple.metadata:com_apple_backup_excludeItem";

// Binary plist value that tmutil sets for the exclude attribute.
// Equivalent to: bplist00 with string "com.apple.backupd"
const XATTR_VALUE: [u8; 61] = [
    0x62, 0x70, 0x6C, 0x69, 0x73, 0x74, 0x30, 0x30, 0x5F, 0x10, 0x11, 0x63, 0x6F, 0x6D, 0x2E, 0x61,
    0x70, 0x70, 0x6C, 0x65, 0x2E, 0x62, 0x61, 0x63, 0x6B, 0x75, 0x70, 0x64, 0x08, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1C,
];

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
    xattr::set(path, XATTR_KEY, &XATTR_VALUE)
        .map_err(|e| format!("failed to set exclusion on {}: {e}", path.display()))
}

pub fn add_exclusions(paths: &[PathBuf]) -> Result<(), String> {
    let mut errors = Vec::new();
    for path in paths {
        if let Err(e) = add_exclusion(path) {
            errors.push(e);
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

pub fn remove_exclusion(path: &Path) -> Result<(), String> {
    match xattr::remove(path, XATTR_KEY) {
        Ok(()) => Ok(()),
        Err(e) if e.raw_os_error() == Some(93) => Ok(()),
        Err(e) => Err(format!(
            "failed to remove exclusion from {}: {e}",
            path.display()
        )),
    }
}

pub fn remove_exclusions(paths: &[PathBuf]) -> Result<(), String> {
    let mut errors = Vec::new();
    for path in paths {
        if let Err(e) = remove_exclusion(path) {
            errors.push(e);
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

pub fn are_excluded(paths: &[PathBuf]) -> Vec<bool> {
    paths.iter().map(|p| is_excluded(p)).collect()
}

fn is_excluded(path: &Path) -> bool {
    xattr::get(path, XATTR_KEY)
        .ok()
        .flatten()
        .is_some_and(|val| val == XATTR_VALUE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_excluded_returns_false_for_nonexistent() {
        assert!(!is_excluded(Path::new(
            "/nonexistent/path/that/does/not/exist"
        )));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn add_and_check_exclusion() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();

        assert!(!is_excluded(path));
        add_exclusion(path).unwrap();
        assert!(is_excluded(path));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn remove_exclusion_clears_attribute() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();

        add_exclusion(path).unwrap();
        assert!(is_excluded(path));

        remove_exclusion(path).unwrap();
        assert!(!is_excluded(path));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn remove_exclusion_on_non_excluded_is_ok() {
        let dir = tempfile::tempdir().unwrap();
        assert!(remove_exclusion(dir.path()).is_ok());
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn are_excluded_batch() {
        let dir1 = tempfile::tempdir().unwrap();
        let dir2 = tempfile::tempdir().unwrap();

        add_exclusion(dir1.path()).unwrap();

        let results = are_excluded(&[dir1.path().to_path_buf(), dir2.path().to_path_buf()]);
        assert_eq!(results, vec![true, false]);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn add_exclusions_batch() {
        let dir1 = tempfile::tempdir().unwrap();
        let dir2 = tempfile::tempdir().unwrap();

        add_exclusions(&[dir1.path().to_path_buf(), dir2.path().to_path_buf()]).unwrap();

        assert!(is_excluded(dir1.path()));
        assert!(is_excluded(dir2.path()));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn remove_exclusions_batch() {
        let dir1 = tempfile::tempdir().unwrap();
        let dir2 = tempfile::tempdir().unwrap();

        add_exclusions(&[dir1.path().to_path_buf(), dir2.path().to_path_buf()]).unwrap();
        remove_exclusions(&[dir1.path().to_path_buf(), dir2.path().to_path_buf()]).unwrap();

        assert!(!is_excluded(dir1.path()));
        assert!(!is_excluded(dir2.path()));
    }
}
