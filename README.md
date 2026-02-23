# Veiled CLI

A macOS CLI to automatically exclude development artifacts from Time Machine backups.

Time Machine doesn't support wildcard exclusion rules -- only absolute paths. That means `node_modules`, `.next`, `dist`, `target`, `.venv`, and other build artifacts get backed up by default, making backups slower and larger than they need to be.

**veiled** scans your project directories, identifies what doesn't need to be backed up using `.gitignore`, a built-in list of known directories, and custom exclusions, then applies the exclusions via `tmutil`. It runs as a daily daemon with zero intervention after setup.

## Install

```sh
brew install adeonir/tap/veiled
```

## Quick start

```sh
veiled start
```

This activates the daily daemon. On first run, it also performs an immediate scan to exclude all recognized development artifacts from Time Machine.

## Usage

```sh
veiled run                # Run a scan and exclude development artifacts
veiled list               # List all paths currently excluded by veiled
veiled status             # Show daemon state, exclusion count, and saved space
veiled status --refresh   # Recalculate saved space from current exclusions
veiled add <path>         # Add a custom directory to the exclusion list
veiled remove <path>      # Remove a directory from the exclusion list
veiled reset              # Remove all exclusions managed by veiled
veiled reset --yes        # Skip confirmation prompt
veiled start              # Install binary and activate the daily daemon
veiled stop               # Deactivate daemon and remove the launch agent
veiled update             # Check for updates and install the latest version
veiled --verbose <cmd>    # Enable diagnostic output on stderr
```

## Configuration

veiled stores its configuration at `~/.config/veiled/config.toml`. If the file doesn't exist, it's created with default values on first run. Existing `config.json` files are automatically migrated on first load.

```toml
# Directories to scan for projects
search_paths = ["~/Projects", "~/Developer"]

# Additional paths to exclude beyond the built-in list
extra_exclusions = []

# Paths to skip entirely during scans
ignore_paths = ["~/.Trash", "~/Library", "~/Downloads"]

# Check for new versions automatically when running a scan
auto_update = true
```

- **search_paths** -- Directories to scan for projects. Defaults to `["~/Projects", "~/Developer"]`.
- **extra_exclusions** -- Additional directory names to exclude beyond the built-in list. Defaults to `[]`.
- **ignore_paths** -- Paths to skip entirely during scans. Defaults to `["~/.Trash", "~/Library", "~/Downloads"]`.
- **auto_update** -- Check for new versions automatically when running a scan. Defaults to `true`.

**veiled** checks for new versions automatically during scans and updates itself. You can disable this by setting `auto_update` to `false` in the configuration, or run `veiled update` manually at any time.

## Requirements

- macOS 12 (Monterey) or later
- Full Disk Access may be required for `tmutil` to manage exclusions in protected paths (System Settings > Privacy & Security > Full Disk Access)

## How it works

1. **Scans** your project directories looking for development artifacts
2. **Identifies** what to exclude using `.gitignore` rules, a built-in list of known directories, and any custom exclusions you define
3. **Applies** Time Machine exclusions via `tmutil` for each matched path
4. **Runs daily** as a background daemon, so new projects are covered automatically

All exclusions are tracked in a local registry, so you can list, review, or reset them at any time.

## License

MIT
