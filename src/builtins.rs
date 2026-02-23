/// Known development artifact directory names that should be excluded from Time Machine backups.
///
/// Some names are generic and may match non-artifact directories. These are
/// annotated with "generic" below. Veiled only matches top-level directory
/// names inside search paths, which limits false positives to projects that
/// use these names for committed source code.
const BUILTIN_DIRS: &[&str] = &[
    // JavaScript / TypeScript
    "node_modules",
    ".next",
    ".nuxt",
    "dist",  // generic: may match non-JS distribution directories
    "build", // generic: may match C/Make or other compiled output with source
    "out",   // generic: may match custom output directories with committed files
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
    "target", // generic: Rust/Cargo convention, but some projects use for other purposes
    ".gradle",
    // Go / PHP
    "vendor", // generic: Go vendor may contain committed source code
    // iOS / Swift
    "Pods",
    ".build",
    // IDEs and misc
    ".idea",
    "tmp", // generic: may match project-level temp directories with relevant data
    ".tmp",
];

pub fn is_builtin(name: &str) -> bool {
    BUILTIN_DIRS.contains(&name)
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
}
