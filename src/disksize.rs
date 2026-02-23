use std::fs;
use std::path::{Path, PathBuf};

pub fn dir_size(path: &Path) -> u64 {
    let mut total = 0u64;
    let mut stack: Vec<PathBuf> = vec![path.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let Ok(ft) = entry.file_type() else {
                continue;
            };

            if ft.is_symlink() {
                continue;
            }

            if ft.is_dir() {
                stack.push(entry.path());
            } else {
                let Ok(metadata) = entry.metadata() else {
                    continue;
                };
                total = total.saturating_add(metadata.len());
            }
        }
    }

    total
}

pub fn calculate_total_size(paths: &[String]) -> u64 {
    paths
        .iter()
        .map(|p| dir_size(Path::new(p)))
        .fold(0u64, u64::saturating_add)
}

pub fn format_size(bytes: u64) -> String {
    const GB: f64 = 1_073_741_824.0;
    const MB: f64 = 1_048_576.0;
    const KB: f64 = 1_024.0;

    #[allow(clippy::cast_precision_loss)]
    let value = bytes as f64;

    if value >= GB {
        format!("{:.1} GB", value / GB)
    } else if value >= MB {
        format!("{:.1} MB", value / MB)
    } else {
        format!("{:.1} KB", value / KB)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn empty_dir_returns_zero() {
        let dir = TempDir::new().unwrap();
        assert_eq!(dir_size(dir.path()), 0);
    }

    #[test]
    fn single_file_returns_file_size() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("file.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello").unwrap();

        assert_eq!(dir_size(dir.path()), 5);
    }

    #[test]
    fn nested_dirs_sum_all_files() {
        let dir = TempDir::new().unwrap();

        let sub = dir.path().join("sub");
        fs::create_dir(&sub).unwrap();

        let mut f1 = File::create(dir.path().join("a.txt")).unwrap();
        f1.write_all(b"aaa").unwrap();

        let mut f2 = File::create(sub.join("b.txt")).unwrap();
        f2.write_all(b"bbbbb").unwrap();

        assert_eq!(dir_size(dir.path()), 8);
    }

    #[test]
    fn nonexistent_path_returns_zero() {
        assert_eq!(dir_size(Path::new("/nonexistent/path")), 0);
    }

    #[test]
    fn calculate_total_size_sums_multiple_dirs() {
        let d1 = TempDir::new().unwrap();
        let d2 = TempDir::new().unwrap();

        let mut f1 = File::create(d1.path().join("a.txt")).unwrap();
        f1.write_all(b"aaa").unwrap();

        let mut f2 = File::create(d2.path().join("b.txt")).unwrap();
        f2.write_all(b"bb").unwrap();

        let paths = vec![
            d1.path().to_string_lossy().into_owned(),
            d2.path().to_string_lossy().into_owned(),
        ];

        assert_eq!(calculate_total_size(&paths), 5);
    }

    #[test]
    fn calculate_total_size_skips_nonexistent() {
        let paths = vec![
            "/nonexistent/one".to_string(),
            "/nonexistent/two".to_string(),
        ];
        assert_eq!(calculate_total_size(&paths), 0);
    }

    #[cfg(unix)]
    #[test]
    fn dir_size_skips_symlinks() {
        let dir = TempDir::new().unwrap();
        let sub = dir.path().join("sub");
        fs::create_dir(&sub).unwrap();
        let mut f = File::create(sub.join("file.txt")).unwrap();
        f.write_all(b"hello").unwrap();

        std::os::unix::fs::symlink(dir.path(), dir.path().join("loop")).unwrap();
        assert_eq!(dir_size(dir.path()), 5);
    }

    #[test]
    fn format_size_kilobytes() {
        assert_eq!(format_size(0), "0.0 KB");
        assert_eq!(format_size(1_024), "1.0 KB");
        assert_eq!(format_size(524_288), "512.0 KB");
        assert_eq!(format_size(1_048_575), "1024.0 KB");
    }

    #[test]
    fn format_size_megabytes() {
        assert_eq!(format_size(1_048_576), "1.0 MB");
        assert_eq!(format_size(268_959_334), "256.5 MB");
    }

    #[test]
    fn format_size_gigabytes() {
        assert_eq!(format_size(1_073_741_824), "1.0 GB");
        assert_eq!(format_size(13_207_024_435), "12.3 GB");
    }
}
