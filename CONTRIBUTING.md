# Contributing

## Prerequisites

- Rust (latest stable)
- macOS (Time Machine features are macOS-only)
- [lefthook](https://github.com/evilmartians/lefthook) for pre-commit hooks

## Setup

```bash
git clone https://github.com/adeonir/veiled.git
cd veiled
lefthook install
cargo build
```

## Running

```bash
cargo run -- <command>       # Run in debug mode
cargo build --release        # Release build (strip + LTO)
```

## Tests

```bash
cargo test                   # All tests (unit + integration)
cargo test <test_name>       # Single test by name
```

## Quality Gates

These run automatically on every commit via lefthook:

```bash
cargo fmt -- --check         # Check formatting
cargo clippy -- -D warnings  # Lint (pedantic, warnings are errors)
```

To auto-fix formatting:

```bash
cargo fmt
```

## Code Style

- Rust edition 2024, rustfmt edition 2024
- Clippy pedantic lints enabled
- 4 spaces for Rust, 2 spaces for everything else (see `.editorconfig`)
- `console` crate for colored output, `indicatif` for spinners
- All commands return `Result<(), Box<dyn std::error::Error>>`
- Warnings go to stderr via `eprintln!` with `style("warning:").yellow().bold()`
