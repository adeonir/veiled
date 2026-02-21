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
veiled run
```

This scans your configured search paths and excludes all recognized development artifacts from Time Machine.

## Usage

```sh
veiled run       # Run a scan and exclude development artifacts
veiled list      # List all paths currently excluded by veiled
veiled status    # Show the number of managed exclusions
veiled add .dir  # Add a custom directory to the exclusion list
veiled reset     # Remove all exclusions managed by veiled
veiled start     # Install binary and activate the daily daemon
veiled stop      # Deactivate daemon and remove the launch agent
veiled update    # Check for updates and install the latest version
```

## Configuration

veiled stores its configuration at `~/.config/veiled/config.json`. If the file doesn't exist, it's created with default values on first run.

```json
{
  "searchPaths": ["~/Projects"],
  "extraExclusions": [],
  "ignorePaths": ["~/.Trash", "~/Library", "~/Downloads"],
  "autoUpdate": true
}
```

- **searchPaths** -- Directories to scan for projects. Defaults to `["~/Projects"]`.
- **extraExclusions** -- Additional directory names to exclude beyond the built-in list. Defaults to `[]`.
- **ignorePaths** -- Paths to skip entirely during scans. Defaults to `["~/.Trash", "~/Library", "~/Downloads"]`.
- **autoUpdate** -- Check for new versions automatically when running a scan. Defaults to `true`.

## Requirements

- macOS 12 (Monterey) or later
- Full Disk Access may be required for `tmutil` to manage exclusions in protected paths (System Settings > Privacy & Security > Full Disk Access)

## License

MIT
