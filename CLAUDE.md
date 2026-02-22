# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Veiled CLI is a macOS CLI that automatically excludes development artifacts (node_modules, target, .venv, etc.) from Time Machine backups using `tmutil`. It scans project directories, identifies ignorable paths via `.gitignore` and built-in rules, and runs as a daily `launchd` daemon.

## Commands

```bash
cargo build                    # Debug build
cargo build --release          # Release build (strip + LTO)
cargo test                     # Run all tests (unit + integration)
cargo test help_displays       # Run a single test by name
cargo fmt -- --check           # Check formatting
cargo clippy -- -D warnings    # Lint with pedantic clippy (warnings are errors)
```

Pre-commit hooks (via lefthook) run fmt, clippy, and tests in parallel on every commit.

## Architecture

```
src/
  main.rs          # Entrypoint: parses CLI args, sets up OnceLock<bool> verbose global, runs FDA probe before tmutil commands, dispatches to command modules
  cli.rs           # clap derive structs: Cli (Parser with global --verbose flag) and Commands (Subcommand enum)
  builtins.rs      # Static list of known dev artifact directory names (node_modules, target, .venv, etc.)
  config.rs        # Config load/save from ~/.config/veiled/config.json with tilde expansion
  daemon.rs        # launchd plist generation, install/uninstall/status for the daily agent
  registry.rs      # Tracks managed exclusions in ~/.config/veiled/registry.json (add/remove/list/contains) with exclusive file locking via LockedRegistry
  disksize.rs      # Recursive directory size calculation and human-readable formatting (MB/GB)
  scanner.rs       # Scans search paths: git ls-files for repos, directory traversal for non-git dirs, dedup + tmutil filtering
  tmutil.rs        # Wraps macOS tmutil commands (addexclusion, removeexclusion, isexcluded) with structured results; check_access() probes FDA permissions
  updater.rs       # GitHub Releases version check, binary download with SHA-256 checksum validation and atomic replacement
  commands/
    mod.rs          # Re-exports all command modules
    run.rs          # Scan and exclude new paths (spinner + summary + rate-limited auto-update check with 24h cooldown)
    list.rs         # Print all managed exclusion paths
    status.rs       # Show daemon state and exclusion count
    add.rs          # Add custom directory to exclusions (validates path, updates config + registry + tmutil)
    remove.rs       # Remove a directory from exclusions (unregisters from registry + config + tmutil)
    reset.rs        # Remove all exclusions (confirmation prompt, --yes to bypass)
    start.rs        # Install binary to ~/.local/bin and activate the launchd daemon
    stop.rs         # Deactivate daemon and remove the launch agent plist
    update.rs       # Check for updates and install the latest version from GitHub Releases
tests/
  cli.rs           # Integration tests using assert_cmd + predicates (runs the compiled binary)
```

The CLI uses clap derive macros. Each subcommand is a variant in `Commands` enum (cli.rs), and `main.rs` matches on it to call the corresponding `commands::{name}::execute()` function. Doc comments on enum variants become the `--help` descriptions. The top-level `about` text is derived from the Cargo.toml `description` field at compile time. All commands return `Result<(), Box<dyn std::error::Error>>`; main catches errors, prints them in red via `console::style`, and exits non-zero.

Config uses `#[serde(default, rename_all = "camelCase")]` so JSON fields are camelCase while Rust fields are snake_case. Partial configs fill missing fields from defaults. All path fields undergo tilde expansion after loading. The tmutil module isolates stdout parsing from command execution so parsing logic is testable cross-platform.

Scanner combines two strategies: `git ls-files --ignored --exclude-standard` for git repos, and direct directory traversal for non-git dirs. Both filter through `builtins::is_builtin()`. Results are deduplicated and filtered against `tmutil::is_excluded` to skip already-excluded paths. When `--verbose` is active, scanner logs git failures, skipped directories, and empty results to stderr.

Data files live in `~/.config/veiled/`: `config.json` (user settings) and `registry.json` (managed exclusions, cached saved bytes, and last update check timestamp). Both Config and Registry use a `load_from`/`save_to` pattern that accepts a `&Path` argument, allowing unit tests to use `tempfile::TempDir` instead of touching the real config directory. Integration tests in `tests/cli.rs` use `assert_cmd` with `cargo_bin_cmd!("veiled")` to run the compiled binary.

## Workflow

- Always use the `spec-driven` skill for feature implementation (initialize, plan, tasks, implement, validate)
- Always use the `git-helpers` skill for commits, code reviews, and pull requests

## Conventions

- Rust edition 2024, rustfmt edition 2024
- Clippy pedantic lints enabled (`[lints.clippy]` in Cargo.toml)
- Release profile: `strip = true`, `lto = true`
- Indentation: 4 spaces for Rust, 2 spaces for everything else (see .editorconfig)
- Terminal output: `console` crate for colors, `indicatif` for spinners
- HTTP requests: `ureq` crate for GitHub API calls (updater)
- File locking: `fs2` crate for exclusive flock on registry
- Checksums: `sha2` crate for SHA-256 binary validation
