/// Known development artifact directory names that should be excluded from Time Machine backups.
const BUILTIN_DIRS: &[&str] = &[
    // JavaScript / TypeScript
    "node_modules",
    ".next",
    ".nuxt",
    "dist",
    "build",
    "out",
    ".turbo",
    ".cache",
    ".vite",
    ".vercel",
    ".output",
    ".parcel-cache",
    "coverage",
    ".nyc_output",
    // Python
    ".venv",
    "venv",
    "__pycache__",
    ".mypy_cache",
    ".pytest_cache",
    // Rust / Java / JVM
    "target",
    ".gradle",
    // Go / PHP
    "vendor",
    // iOS / Swift
    "Pods",
    ".build",
    // IDEs and misc
    ".idea",
    "tmp",
    ".tmp",
];

pub fn is_builtin(name: &str) -> bool {
    BUILTIN_DIRS.contains(&name)
}

pub fn list() -> &'static [&'static str] {
    BUILTIN_DIRS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_known_directory() {
        assert!(is_builtin("node_modules"));
        assert!(is_builtin("target"));
        assert!(is_builtin(".venv"));
        assert!(is_builtin("Pods"));
    }

    #[test]
    fn rejects_unknown_directory() {
        assert!(!is_builtin("src"));
        assert!(!is_builtin("README.md"));
        assert!(!is_builtin(""));
    }

    #[test]
    fn match_is_case_sensitive() {
        assert!(!is_builtin("Node_Modules"));
        assert!(!is_builtin("TARGET"));
    }

    #[test]
    fn list_returns_all_entries() {
        let entries = list();
        assert!(entries.len() > 20);
        assert!(entries.contains(&"node_modules"));
    }
}
