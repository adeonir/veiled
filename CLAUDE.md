# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

veiled is a macOS CLI that automatically excludes development artifacts (node_modules, target, .venv, etc.) from Time Machine backups using `tmutil`. It scans project directories, identifies ignorable paths via `.gitignore` and built-in rules, and runs as a daily `launchd` daemon.

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
  main.rs          # Entrypoint: parses CLI args, dispatches to command modules, handles errors with exit codes
  cli.rs           # clap derive structs: Cli (Parser) and Commands (Subcommand enum)
  builtins.rs      # Static list of known dev artifact directory names (node_modules, target, .venv, etc.)
  config.rs        # Config load/save from ~/.config/veiled/config.json with tilde expansion
  registry.rs      # Tracks managed exclusions in ~/.config/veiled/registry.json (add/remove/list/contains)
  scanner.rs       # Scans search paths: git ls-files for repos, directory traversal for non-git dirs, dedup + tmutil filtering
  tmutil.rs        # Wraps macOS tmutil commands (addexclusion, removeexclusion, isexcluded) with structured results
  commands/
    mod.rs          # Re-exports all command modules
    run.rs          # Scan and exclude new paths (spinner + summary)
    list.rs         # Print all managed exclusion paths
    status.rs       # Show count of managed exclusions
    add.rs          # Add custom directory to exclusions (validates path, updates config + registry + tmutil)
    reset.rs        # Remove all exclusions (confirmation prompt, --yes to bypass)
    start.rs        # Stub (feature 005 -- daemon)
    stop.rs         # Stub (feature 005 -- daemon)
    update.rs       # Stub (feature 006 -- updater)
tests/
  cli.rs           # Integration tests using assert_cmd + predicates (runs the compiled binary)
```

The CLI uses clap derive macros. Each subcommand is a variant in `Commands` enum (cli.rs), and `main.rs` matches on it to call the corresponding `commands::{name}::execute()` function. Doc comments on enum variants become the `--help` descriptions. All commands return `Result<(), Box<dyn std::error::Error>>`; main catches errors, prints them in red via `console::style`, and exits non-zero.

Config uses `#[serde(default, rename_all = "camelCase")]` so JSON fields are camelCase while Rust fields are snake_case. Partial configs fill missing fields from defaults. All path fields undergo tilde expansion after loading. The tmutil module isolates stdout parsing from command execution so parsing logic is testable cross-platform.

Scanner combines two strategies: `git ls-files --ignored --exclude-standard` for git repos, and direct directory traversal for non-git dirs. Both filter through `builtins::is_builtin()`. Results are deduplicated and filtered against `tmutil::is_excluded` to skip already-excluded paths.

## Workflow

- Always use the `spec-driven` skill for feature implementation (initialize, plan, tasks, implement, validate)
- Always use the `git-helpers` skill for commits, code reviews, and pull requests

## Conventions

- Rust edition 2024, rustfmt edition 2024
- Clippy pedantic lints enabled (`[lints.clippy]` in Cargo.toml)
- Release profile: `strip = true`, `lto = true`
- Indentation: 4 spaces for Rust, 2 spaces for everything else (see .editorconfig)
- Terminal output: `console` crate for colors, `indicatif` for spinners
- Stub commands use `#[allow(clippy::unnecessary_wraps)]` since they return Result but only ever Ok
