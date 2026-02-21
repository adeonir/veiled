# veiled

A macOS CLI to automatically exclude development artifacts from Time Machine backups.

Time Machine doesn't support wildcard exclusion rules -- only absolute paths. That means `node_modules`, `.next`, `dist`, `target`, `.venv`, and other build artifacts get backed up by default, making backups slower and larger than they need to be.

**veiled** scans your project directories, identifies what doesn't need to be backed up using `.gitignore`, a built-in list of known directories, and custom exclusions, then applies the exclusions via `tmutil`. It runs as a daily daemon with zero intervention after setup.

## Install

```sh
brew install adeonir/tap/veiled
```

## Usage

```sh
veiled start     # Install binary and activate daemon
veiled stop      # Deactivate daemon and remove plist
veiled run       # Run a scan manually
veiled list      # List all paths excluded by veiled
veiled reset     # Remove all exclusions managed by veiled
veiled add .dir  # Add a directory to the custom exclusion list
veiled status    # Show daemon state and exclusion stats
veiled update    # Update binary to the latest version
```

## How it works

1. Scans `searchPaths` (default: `~/Projects`) for Git repositories
2. Uses `git ls-files --ignored` to find paths each project already ignores
3. Falls back to a built-in list for directories outside Git repos
4. Checks each path with `tmutil isexcluded` before applying `tmutil addexclusion`
5. Keeps an internal registry so `reset` only removes what veiled managed

The daemon runs daily at 3:00 AM via `launchd`, picking up new projects automatically.

## License

MIT
