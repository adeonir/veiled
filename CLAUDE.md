# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

veiled is a macOS CLI that automatically excludes development artifacts (node_modules, target, .venv, etc.) from Time Machine backups using `tmutil`. It scans project directories, identifies ignorable paths via `.gitignore` and built-in rules, and runs as a daily `launchd` daemon.

## Commands

```bash
cargo build                    # Debug build
cargo build --release          # Release build (strip + LTO, produces ~600KB binary)
cargo test                     # Run all tests (unit + integration)
cargo test help_displays       # Run a single test by name
cargo fmt -- --check           # Check formatting
cargo clippy -- -D warnings    # Lint with pedantic clippy (warnings are errors)
```

Pre-commit hooks (via lefthook) run fmt, clippy, and tests in parallel on every commit.

## Architecture

```
src/
  main.rs          # Entrypoint: parses CLI args, dispatches to command modules
  cli.rs           # clap derive structs: Cli (Parser) and Commands (Subcommand enum)
  commands/
    mod.rs          # Re-exports all command modules
    {command}.rs    # One file per subcommand with pub fn execute()
tests/
  cli.rs           # Integration tests using assert_cmd (runs the compiled binary)
```

The CLI uses clap derive macros. Each subcommand is a variant in `Commands` enum (cli.rs), and `main.rs` matches on it to call the corresponding `commands::{name}::execute()` function. Doc comments on enum variants become the `--help` descriptions.

8 subcommands: start, stop, run, list, reset, add, status, update. All are stubs currently.

## Workflow

- Always use the `spec-driven` skill for feature implementation (initialize, plan, tasks, implement, validate)
- Always use the `git-helpers` skill for commits, code reviews, and pull requests

## Conventions

- Rust edition 2024, rustfmt edition 2024
- Clippy pedantic lints enabled (`[lints.clippy]` in Cargo.toml)
- Release profile: `strip = true`, `lto = true`
- Indentation: 4 spaces for Rust, 2 spaces for everything else (see .editorconfig)
