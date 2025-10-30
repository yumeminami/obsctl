# obsctl

Local-first CLI companion for Obsidian-style knowledge management with optional AI (MCP) assistance.

## Goals
- Capture daily notes quickly from the terminal.
- Track and update tasks stored as Markdown.
- Search a vault with ripgrep/fzf and future RAG indexing.
- Expose MCP functions so a local LLM can summarize or update notes.

## Quick Start

```bash
cargo run -- note add "Draft project outline"
cargo run -- task add "Review design doc" --due 2025-01-05 --priority high
cargo run -- search grep "control loop"
```

To persist configuration, run:

```bash
cargo run -- config init
```

This bootstraps `~/.obsctl/` with a vault structure:

```
~/.obsctl/
├── config.toml
└── vault/
    ├── Journal/
    ├── Tasks/tasks.md
    ├── Projects/
    └── templates/
        ├── daily.md
        └── task.md
```

Adjust paths in `config.toml` or use `cargo run -- config path --set <dir>` to relocate the vault.

## Commands

- `note add|open|list` – append entries and browse daily notes.
- `task add|done|list|clean` – maintain Markdown tasks with optional due date, recurrence, and priority markers.
- `search grep|fzf` – grep the vault or fuzzy-find file paths.
- `config init|path` – scaffold and inspect configuration.
- `version [--json|--verbose]` – show release information in plain text or JSON output.

Run `cargo run -- --help` for global options and per-command usage.

## MCP Server

- Start the server with `cargo run --bin mcp_server`.
- Exposes tools: `append_daily_note`, `update_task_status`, `query_knowledge`, `summarize_today`.
- Implements the Model Context Protocol using the official `rmcp` Rust SDK over stdio.
- Designed for local LLMs/agents that speak MCP to automate notebook updates.

## Changelog

See [`CHANGELOG.md`](CHANGELOG.md) for release history.

## Architecture Overview

- `src/cli` – clap-powered command parsing and handlers.
- `src/config` – loads/saves TOML config and ensures vault directories.
- `src/core` – services for notes (`vault`) and tasks.
- `src/search` – wrappers for ripgrep/fzf execution.
- `src/mcp` – MCP server implementation built on the rmcp SDK.
- `src/templates` – default Markdown templates for daily notes and tasks.

The design keeps filesystem logic in services so both CLI and MCP calls can reuse behavior.

## Development

```bash
cargo check
cargo fmt
cargo test
```

Future milestones include MCP client integration, richer indexing (e.g., tantivy/sqlite FTS), and RAG-powered context retrieval.
