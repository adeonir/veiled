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
  config.rs        # Config load/save from ~/.config/veiled/config.toml with tilde expansion and exclusive file locking
  daemon.rs        # launchd plist generation, install/uninstall/status for the daily agent
  registry.rs      # Tracks managed exclusions in ~/.config/veiled/registry.json (add/remove/list/contains) with exclusive file locking via LockedRegistry
  disksize.rs      # Parallel directory size calculation and human-readable formatting (KB/MB/GB)
  scanner.rs       # Scans search paths: parallel git ls-files --directory for repos (all gitignored dirs), directory traversal for non-git dirs (builtin names), dedup
  tmutil.rs        # Manages Time Machine exclusions via xattr (add/remove/check); check_access() probes FDA permissions via tmutil process
  updater.rs       # GitHub Releases version check, binary download with SHA-256 checksum validation and atomic replacement
  commands/
    mod.rs          # Re-exports all command modules
    run.rs          # Scan and exclude new paths (spinner + summary + rate-limited auto-update check with 24h cooldown)
    list.rs         # Print all managed exclusion paths
    status.rs       # Show daemon state and exclusion count
    add.rs          # Add custom directory to exclusions (validates path, updates config + registry + tmutil)
    remove.rs       # Remove a directory from exclusions (unregisters from registry + config + tmutil)
    reset.rs        # Remove all exclusions (confirmation prompt, --yes to bypass)
    start.rs        # Activate the launchd daemon (runs initial scan only if registry is empty)
    stop.rs         # Deactivate daemon and remove the launch agent plist
    update.rs       # Check for updates, install the latest version, and restart/activate the daemon
tests/
  cli.rs           # Integration tests using assert_cmd + predicates (runs the compiled binary)
```

The CLI uses clap derive macros. Each subcommand is a variant in `Commands` enum (cli.rs), and `main.rs` matches on it to call the corresponding `commands::{name}::execute()` function. Doc comments on enum variants become the `--help` descriptions. The top-level `about` text is derived from the Cargo.toml `description` field at compile time. All commands return `Result<(), Box<dyn std::error::Error>>`; main catches errors, prints them in red via `console::style`, and exits non-zero.

Config uses `#[serde(default)]` with TOML format and `snake_case` keys. Partial configs fill missing fields from defaults. All path fields undergo tilde expansion after loading (tilde notation is preserved on save). Legacy `config.json` files are automatically migrated to `config.toml` on first load. The tmutil module uses the `xattr` crate to directly read/write the `com.apple.metadata:com_apple_backup_excludeItem` extended attribute instead of spawning tmutil processes, making add/remove/check operations near-instant.

Scanner combines two strategies: `git ls-files --ignored --others --exclude-standard --directory` for git repos (captures all gitignored directories), and direct directory traversal for non-git dirs (matches `builtins::is_builtin()` names). Individual files are skipped to preserve recoverable data in backups. Traverse also descends into git repos to find builtin directories that may not be in `.gitignore`. Git repos are scanned in parallel (8 thread chunks). Results are deduplicated. When `--verbose` is active, scanner logs git failures, skipped directories, and empty results to stderr.

Data files live in `~/.config/veiled/`: `config.toml` (user settings) and `registry.json` (managed exclusions, cached saved bytes, and last update check timestamp). Both Config and Registry use exclusive file locking and a `load_from`/`save_to` pattern that accepts a `&Path` argument, allowing unit tests to use `tempfile::TempDir` instead of touching the real config directory. Integration tests in `tests/cli.rs` use `assert_cmd` with `cargo_bin_cmd!("veiled")` to run the compiled binary.

## Quality Gates

All three must pass before any change is considered complete:

1. `cargo fmt -- --check`
2. `cargo clippy -- -D warnings`
3. `cargo test`

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
- Extended attributes: `xattr` crate for Time Machine exclusion management
