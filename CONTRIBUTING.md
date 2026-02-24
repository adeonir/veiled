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

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feat/my-feature`)
3. Make your changes and ensure all quality gates pass
4. Commit using [conventional commits](https://www.conventionalcommits.org/) (e.g. `feat:`, `fix:`, `refactor:`)
5. Push to your fork and open a pull request against `main`

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
