use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;

use console::style;

use crate::builtins;
use crate::config::Config;
use crate::verbose;

pub fn scan(config: &Config, on_found: &dyn Fn(usize)) -> Vec<PathBuf> {
    let candidates = collect_paths(config, on_found);

    if verbose() && candidates.is_empty() {
        eprintln!(
            "{} scan found no paths to evaluate",
            style("verbose:").dim()
        );
    }

    candidates
}

fn collect_paths(config: &Config, on_found: &dyn Fn(usize)) -> Vec<PathBuf> {
    let mut paths: HashSet<PathBuf> =
        traverse(&config.search_paths, &config.ignore_paths, on_found)
            .into_iter()
            .collect();

    for extra in &config.extra_exclusions {
        let path = PathBuf::from(extra);
        if path.is_dir() {
            paths.insert(path);
        }
    }

    let mut results: Vec<PathBuf> = paths.into_iter().collect();
    results.sort();
    results
}

pub fn parse_git_ignored(repo_path: &Path, output: &str) -> Vec<PathBuf> {
    let mut dirs = HashSet::new();

    for line in output.split('\0') {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let mut prefix = PathBuf::new();
        for component in Path::new(line).components() {
            prefix.push(component);
            let name = component.as_os_str().to_string_lossy();
            if builtins::is_builtin(&name) {
                dirs.insert(repo_path.join(&prefix));
                break;
            }
        }
    }

    dirs.into_iter().collect()
}

pub fn scan_git_repo(repo_path: &Path) -> Vec<PathBuf> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args([
            "ls-files",
            "--ignored",
            "--others",
            "--exclude-standard",
            "-z",
        ])
        .output();

    let Ok(output) = output else {
        if verbose() {
            eprintln!(
                "{} git command failed in {}",
                style("verbose:").dim(),
                repo_path.display()
            );
        }
        return vec![];
    };

    if !output.status.success() {
        if verbose() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!(
                "{} git ls-files failed in {}: {}",
                style("verbose:").dim(),
                repo_path.display(),
                stderr.trim()
            );
        }
        return vec![];
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_git_ignored(repo_path, &stdout)
}

pub fn traverse(
    search_paths: &[String],
    ignore_paths: &[String],
    on_found: &dyn Fn(usize),
) -> Vec<PathBuf> {
    let ignore_set: HashSet<PathBuf> = ignore_paths.iter().map(PathBuf::from).collect();
    let mut results = Vec::new();
    let mut git_repos = Vec::new();
    let mut stack: Vec<PathBuf> = search_paths.iter().map(PathBuf::from).collect();

    while let Some(dir) = stack.pop() {
        if !dir.is_dir() {
            if verbose() {
                eprintln!(
                    "{} skipping non-existent path: {}",
                    style("verbose:").dim(),
                    dir.display()
                );
            }
            continue;
        }

        if ignore_set.contains(&dir) {
            continue;
        }

        if dir.join(".git").is_dir() {
            git_repos.push(dir);
            continue;
        }

        let Ok(entries) = fs::read_dir(&dir) else {
            if verbose() {
                eprintln!(
                    "{} cannot read directory: {}",
                    style("verbose:").dim(),
                    dir.display()
                );
            }
            continue;
        };

        for entry in entries.flatten() {
            let Ok(ft) = entry.file_type() else {
                continue;
            };
            if !ft.is_dir() || ft.is_symlink() {
                continue;
            }
            let path = entry.path();
            if let Some(name) = path.file_name()
                && builtins::is_builtin(&name.to_string_lossy())
            {
                results.push(path);
                on_found(results.len());
            } else {
                stack.push(path);
            }
        }
    }

    let chunk_size = (git_repos.len() / 8).max(1);
    let chunks: Vec<Vec<PathBuf>> = git_repos
        .chunks(chunk_size)
        .map(<[PathBuf]>::to_vec)
        .collect();

    let handles: Vec<_> = chunks
        .into_iter()
        .map(|chunk| {
            thread::spawn(move || {
                chunk
                    .iter()
                    .flat_map(|repo| scan_git_repo(repo))
                    .collect::<Vec<_>>()
            })
        })
        .collect();

    for handle in handles {
        if let Ok(paths) = handle.join() {
            results.extend(paths);
            on_found(results.len());
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn parse_git_ignored_extracts_builtin_dirs() {
        let repo = Path::new("/Users/dev/project");
        let output =
            "node_modules/express/index.js\0node_modules/.package-lock.json\0src/main.rs\0";

        let results = parse_git_ignored(repo, output);

        assert_eq!(results.len(), 1);
        assert!(results.contains(&repo.join("node_modules")));
    }

    #[test]
    fn parse_git_ignored_filters_non_builtin() {
        let repo = Path::new("/Users/dev/project");
        let output = "logs/app.log\0src/generated/types.ts\0";

        let results = parse_git_ignored(repo, output);

        assert!(results.is_empty());
    }

    #[test]
    fn parse_git_ignored_handles_empty_output() {
        let repo = Path::new("/Users/dev/project");
        let results = parse_git_ignored(repo, "");

        assert!(results.is_empty());
    }

    #[test]
    fn parse_git_ignored_deduplicates_same_dir() {
        let repo = Path::new("/Users/dev/project");
        let output = "target/debug/veiled\0target/release/veiled\0target/.rustc_info.json\0";

        let results = parse_git_ignored(repo, output);

        assert_eq!(results.len(), 1);
        assert!(results.contains(&repo.join("target")));
    }

    #[test]
    fn parse_git_ignored_handles_multiple_builtin_dirs() {
        let repo = Path::new("/Users/dev/project");
        let output = "node_modules/pkg/index.js\0target/debug/bin\0.next/cache/webpack\0";

        let results = parse_git_ignored(repo, output);

        assert_eq!(results.len(), 3);
        assert!(results.contains(&repo.join("node_modules")));
        assert!(results.contains(&repo.join("target")));
        assert!(results.contains(&repo.join(".next")));
    }

    #[test]
    fn parse_git_ignored_finds_nested_builtin_in_monorepo() {
        let repo = Path::new("/Users/dev/monorepo");
        let output = "packages/api/node_modules/express/index.js\0packages/api/node_modules/.package-lock.json\0";

        let results = parse_git_ignored(repo, output);

        assert_eq!(results.len(), 1);
        assert!(results.contains(&repo.join("packages/api/node_modules")));
    }

    #[test]
    fn parse_git_ignored_finds_multiple_nested_builtins() {
        let repo = Path::new("/Users/dev/monorepo");
        let output = "packages/api/node_modules/pkg/index.js\0apps/web/.next/cache/file\0apps/web/dist/bundle.js\0";

        let results = parse_git_ignored(repo, output);

        assert_eq!(results.len(), 3);
        assert!(results.contains(&repo.join("packages/api/node_modules")));
        assert!(results.contains(&repo.join("apps/web/.next")));
        assert!(results.contains(&repo.join("apps/web/dist")));
    }

    #[test]
    fn parse_git_ignored_deduplicates_nested_builtins() {
        let repo = Path::new("/Users/dev/monorepo");
        let output = "packages/api/node_modules/a/index.js\0packages/api/node_modules/b/index.js\0";

        let results = parse_git_ignored(repo, output);

        assert_eq!(results.len(), 1);
        assert!(results.contains(&repo.join("packages/api/node_modules")));
    }

    #[test]
    fn parse_git_ignored_handles_paths_with_special_chars() {
        let repo = Path::new("/Users/dev/project");
        let output = "node_modules/.pnpm/@fastify+send@4.1.0/node_modules/send/index.js\0";

        let results = parse_git_ignored(repo, output);

        assert_eq!(results.len(), 1);
        assert!(results.contains(&repo.join("node_modules")));
    }

    #[test]
    fn scan_git_repo_finds_ignored_builtin_dirs() {
        let dir = TempDir::new().unwrap();
        let repo = dir.path();

        Command::new("git").arg("init").arg(repo).output().unwrap();
        fs::write(repo.join(".gitignore"), "node_modules/\ntarget/\n").unwrap();
        fs::create_dir(repo.join("node_modules")).unwrap();
        fs::write(repo.join("node_modules/pkg.json"), "{}").unwrap();
        fs::create_dir(repo.join("target")).unwrap();
        fs::write(repo.join("target/output"), "bin").unwrap();
        fs::create_dir(repo.join("src")).unwrap();
        fs::write(repo.join("src/main.rs"), "fn main() {}").unwrap();

        let results = scan_git_repo(repo);

        assert_eq!(results.len(), 2);
        assert!(results.contains(&repo.join("node_modules")));
        assert!(results.contains(&repo.join("target")));
    }

    #[test]
    fn scan_git_repo_returns_empty_for_non_git_dir() {
        let dir = TempDir::new().unwrap();
        let results = scan_git_repo(dir.path());

        assert!(results.is_empty());
    }

    #[test]
    fn traverse_finds_builtin_in_git_repo() {
        let dir = TempDir::new().unwrap();
        let repo = dir.path().join("project");
        fs::create_dir(&repo).unwrap();

        Command::new("git").arg("init").arg(&repo).output().unwrap();
        fs::write(repo.join(".gitignore"), "node_modules/\n").unwrap();
        fs::create_dir(repo.join("node_modules")).unwrap();
        fs::write(repo.join("node_modules/pkg.json"), "{}").unwrap();

        let results = traverse(&[dir.path().to_string_lossy().into_owned()], &[], &|_| {});

        assert!(results.iter().any(|p| p.ends_with("node_modules")));
    }

    #[test]
    fn traverse_finds_builtin_in_non_git_dir() {
        let dir = TempDir::new().unwrap();
        let project = dir.path().join("project");
        fs::create_dir(&project).unwrap();
        fs::create_dir(project.join("node_modules")).unwrap();

        let results = traverse(&[dir.path().to_string_lossy().into_owned()], &[], &|_| {});

        assert!(results.iter().any(|p| p.ends_with("node_modules")));
    }

    #[test]
    fn traverse_skips_ignore_paths() {
        let dir = TempDir::new().unwrap();
        let ignored = dir.path().join("ignored");
        fs::create_dir(&ignored).unwrap();
        fs::create_dir(ignored.join("node_modules")).unwrap();

        let results = traverse(
            &[dir.path().to_string_lossy().into_owned()],
            &[ignored.to_string_lossy().into_owned()],
            &|_| {},
        );

        assert!(results.is_empty());
    }

    #[test]
    fn traverse_skips_nonexistent_search_path() {
        let results = traverse(&["/nonexistent/search/path".to_string()], &[], &|_| {});

        assert!(results.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn traverse_skips_symlink_loops() {
        let dir = TempDir::new().unwrap();
        let project = dir.path().join("project");
        fs::create_dir(&project).unwrap();
        fs::create_dir(project.join("node_modules")).unwrap();

        std::os::unix::fs::symlink(&project, project.join("link")).unwrap();

        let results = traverse(&[dir.path().to_string_lossy().into_owned()], &[], &|_| {});

        assert_eq!(results.len(), 1);
        assert!(results[0].ends_with("node_modules"));
    }

    #[test]
    fn traverse_does_not_descend_into_builtin_dirs() {
        let dir = TempDir::new().unwrap();
        let project = dir.path().join("project");
        fs::create_dir(&project).unwrap();
        let nm = project.join("node_modules");
        fs::create_dir(&nm).unwrap();
        // nested builtin inside node_modules should not appear separately
        fs::create_dir(nm.join("target")).unwrap();

        let results = traverse(&[dir.path().to_string_lossy().into_owned()], &[], &|_| {});

        assert_eq!(results.len(), 1);
        assert!(results[0].ends_with("node_modules"));
    }

    fn test_config(
        search_paths: Vec<String>,
        ignore_paths: Vec<String>,
        extra_exclusions: Vec<String>,
    ) -> Config {
        Config {
            search_paths,
            ignore_paths,
            extra_exclusions,
            auto_update: false,
        }
    }

    #[test]
    fn collect_paths_includes_traversed() {
        let dir = TempDir::new().unwrap();
        let project = dir.path().join("project");
        fs::create_dir(&project).unwrap();
        fs::create_dir(project.join("node_modules")).unwrap();

        let config = test_config(
            vec![dir.path().to_string_lossy().into_owned()],
            vec![],
            vec![],
        );

        let results = collect_paths(&config, &|_| {});

        assert!(results.iter().any(|p| p.ends_with("node_modules")));
    }

    #[test]
    fn collect_paths_includes_extra_exclusions() {
        let dir = TempDir::new().unwrap();
        let extra = dir.path().join("extra_cache");
        fs::create_dir(&extra).unwrap();

        let config = test_config(vec![], vec![], vec![extra.to_string_lossy().into_owned()]);

        let results = collect_paths(&config, &|_| {});

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], extra);
    }

    #[test]
    fn collect_paths_skips_nonexistent_extra_exclusions() {
        let config = test_config(vec![], vec![], vec!["/nonexistent/extra/path".to_string()]);

        let results = collect_paths(&config, &|_| {});

        assert!(results.is_empty());
    }

    #[test]
    fn collect_paths_deduplicates_results() {
        let dir = TempDir::new().unwrap();
        let project = dir.path().join("project");
        fs::create_dir(&project).unwrap();
        let nm = project.join("node_modules");
        fs::create_dir(&nm).unwrap();

        let config = test_config(
            vec![dir.path().to_string_lossy().into_owned()],
            vec![],
            vec![nm.to_string_lossy().into_owned()],
        );

        let results = collect_paths(&config, &|_| {});

        assert_eq!(
            results
                .iter()
                .filter(|p| p.ends_with("node_modules"))
                .count(),
            1
        );
    }

    #[test]
    fn collect_paths_returns_sorted_results() {
        let dir = TempDir::new().unwrap();
        let project = dir.path().join("project");
        fs::create_dir(&project).unwrap();
        fs::create_dir(project.join("target")).unwrap();
        fs::create_dir(project.join("node_modules")).unwrap();
        fs::create_dir(project.join(".venv")).unwrap();

        let config = test_config(
            vec![dir.path().to_string_lossy().into_owned()],
            vec![],
            vec![],
        );

        let results = collect_paths(&config, &|_| {});
        let sorted: Vec<_> = {
            let mut s = results.clone();
            s.sort();
            s
        };

        assert_eq!(results, sorted);
    }
}
