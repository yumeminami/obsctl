# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

_No unreleased changes yet._

## [0.1.2] - 2025-02-19

### Added
- Installer script (`scripts/install.sh`) that fetches the latest GitHub release and installs both binaries.
- Pre-commit configuration for `cargo fmt` and `cargo clippy`.
- CI release matrix that builds Linux/macOS artifacts for x86-64 and ARM64 and publishes a GitHub release automatically.

### Changed
- Renamed the MCP server binary from `mcp_server` to `obsctl_mcp` and updated documentation accordingly.
- Search commands now emit actionable messages when `rg` or `fzf` is missing from `PATH`.

### Fixed
- Ensured release archives include both `obsctl` and `obsctl_mcp` executables.

## [0.1.1] - 2025-02-18

### Added
- `obsctl version` command with JSON/verbose output.
- Build metadata embedded via `build.rs`.
- CI release workflow now publishes artifacts only on tagged builds.

## [0.1.0] - 2025-02-18

### Added
- Initial CLI scaffolding for notes, tasks, search, and config management.
- MCP server exposing append, task update, search, and summarization tools.
- Default templates and configuration bootstrap.
