# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.4] - 2026-01-20

### Changed

- Removed macOS Intel target (macos-13 runner retired)
- Supported targets: Linux x86_64, macOS ARM, Windows x86_64

## [0.7.3] - 2026-01-20

### Changed

- Simplified release builds to native targets only: Linux x86_64, macOS (Intel + ARM), Windows x86_64

## [0.7.2] - 2026-01-20

### Fixed

- Fixed release CI: use `cross` for Linux cross-compilation, `macos-13` for Intel Mac

## [0.7.1] - 2026-01-20

### Fixed

- Fixed cross-compilation builds by using vendored libgit2

## [0.7.0] - 2026-01-20

### Added

- `self update` command to check for and install updates from GitHub releases
  - `skilo self update` to update to the latest version
  - `skilo self update --check` to check for updates without installing
  - `skilo self update --yes` to skip confirmation prompt
  - Detects cargo installations and warns about potential version conflicts
- GitHub Actions release workflow to build binaries on tag push
  - Builds for Linux (x86_64, aarch64, musl), macOS (x86_64, aarch64), Windows (x86_64)
  - Automatically creates GitHub releases with attached binaries
- Install script for quick setup via `curl | sh`
- Installation instructions in README

### Dependencies

- Added `reqwest` for HTTP requests
- Added `flate2` for gzip decompression
- Added `tar` for archive extraction
- Added `zip` (Windows) for zip archive extraction

## [0.6.0] - 2026-01-20

### Added

- Git repository caching for faster repeated installs
  - Bare repos cached in `~/.skilo/git/db/`
  - Checkouts cached in `~/.skilo/git/checkouts/`
  - Offline mode via `SKILO_OFFLINE=1` environment variable
- `cache` command to manage git cache
  - `skilo cache` shows cache status and disk usage
  - `skilo cache path` shows cache directory location
  - `skilo cache clean` removes old checkouts (default: 30 days)
  - `skilo cache clean --all` removes entire cache
- Environment variables for cache configuration
  - `SKILO_HOME` to override `~/.skilo/`
  - `SKILO_CACHE` to override `~/.skilo/git/`
- Multi-agent support: install skills to multiple agents simultaneously
  - `--agent all` to install to all detected agents
  - Multiple `--agent` flags: `--agent claude --agent cursor`
- Global installation support with `--global` / `-g` flag for `add`, `new`, and `list` commands
- `list` command to show installed skills
  - `skilo list` for project skills
  - `skilo list --global` for global skills
  - `skilo list --all` for both project and global
  - `skilo list /path/to/project` to specify project directory
- `agents` command to list detected AI coding agents
  - Shows project and global agent installations
  - `--verbose` flag for feature support matrix
- Agent detection: automatically detect installed agents by checking for config directories
- Agent feature matrix: track which agents support `context:fork`, hooks, `allowed-tools`
- Compatibility warnings when installing skills with features not supported by target agent
- New `scope` module for project/global path resolution

### Changed

- `AgentSelection` enum replaces `Option<Agent>` for clearer semantics

### Dependencies

- Added `dirs` for cross-platform home directory detection

## [0.5.0] - 2026-01-19

### Added

- `add` command to install skills from git repositories or local paths
- Support for various source formats: GitHub shorthand (`owner/repo`), full URLs, SSH URLs, and local paths
- `--skill` option to install specific skills by name
- `--list` option to list available skills without installing
- `--yes` option for non-interactive/CI-friendly installation
- `--branch` and `--tag` options for specifying git refs
- `--agent` option to specify target agent for installation
- `--output` option for custom install directory
- Direct skill path support (`https://github.com/owner/repo/tree/main/skills/my-skill`)
- New `[add]` configuration section in `.skilorc.toml`
- Support for 14 AI coding agents: Claude Code, OpenCode, Codex, Cursor, Amp, Kilo Code, Roo Code, Goose, Gemini CLI, Antigravity, GitHub Copilot, Clawdbot, Droid, Windsurf
- New `agent` module with `Agent` enum for managing agent-specific paths

### Dependencies

- Added `git2` for git operations
- Added `url` for URL parsing
- Added `dialoguer` for interactive prompts
- Added `tempfile` as runtime dependency

## [0.4.0] - 2026-01-14

### Added

- `read-properties` command to output skill metadata as JSON
- `to-prompt` command to generate `<available_skills>` XML for agent system prompts

## [0.3.0] - 2026-01-13

### Changed

- Refactored validator to use pluggable rule architecture

### Added

- Configurable lint rules via `[lint.rules]` in `.skilorc.toml`
- Rules can be set to `true` (default), `false` (disabled), or a number (custom threshold)
- Threshold-based rules with configurable limits:
  - `name_length` (E002) - default 64
  - `description_length` (E005) - default 1024
  - `compatibility_length` (E006) - default 500
  - `body_length` (W001) - default 500
- Boolean rules:
  - `name_format` (E001), `name_directory` (E003)
  - `description_required` (E004), `references_exist` (E009)
  - `script_executable` (W002), `script_shebang` (W003)

## [0.2.0] - 2026-01-13

### Changed

- Renamed project from `skillz` to `skilo`
- Renamed `SkillzError` to `SkiloError`
- Renamed `SKILLZ_CONFIG` environment variable to `SKILO_CONFIG`

### Added

- Table formatting support in output

## [0.1.0] - Initial Release

### Added

- `skilo new` command to scaffold new skills from templates
- `skilo lint` command to validate skills against the specification
- `skilo fmt` command to format SKILL.md files
- `skilo check` command to run all validations
- `skilo validate` command (alias for `lint --strict`)
- Support for hello-world, minimal, full, and script-based templates
- Support for Python, Bash, JavaScript, and TypeScript scripts
- Configuration via `.skilorc.toml`
- Text, JSON, and SARIF output formats
