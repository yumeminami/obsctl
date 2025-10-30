# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `obsctl version` command with JSON/verbose output.
- Build metadata embedded via `build.rs`.
- CI release workflow now publishes artifacts only on tagged builds.

## [0.1.0] - 2025-02-18

### Added
- Initial CLI scaffolding for notes, tasks, search, and config management.
- MCP server exposing append, task update, search, and summarization tools.
- Default templates and configuration bootstrap.
