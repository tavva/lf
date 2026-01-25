# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`lf` is a Rust CLI for the Langfuse LLM observability platform. It queries traces, sessions, observations, scores, and metrics via the Langfuse API, with output in table, JSON, CSV, or markdown formats.

**API Reference:** https://cloud.langfuse.com/generated/api/openapi.yml

## Development Commands

```bash
cargo check          # Quick compilation check
cargo build          # Debug build
cargo build --release  # Optimised release build (uses LTO, strips symbols)
cargo run -- <args>  # Run with arguments, e.g. cargo run -- traces list --limit 10
cargo install --path .  # Install to ~/.cargo/bin/lf
cargo test            # Run all tests
```

## Architecture

**Command flow:** `main.rs` parses CLI args with clap, dispatches to command modules, which use `LangfuseClient` for API calls, then format output via formatters.

```
src/
├── main.rs         # CLI entry point, routes to Commands enum
├── client.rs       # LangfuseClient - HTTP client with basic auth, handles pagination
├── config.rs       # Profile-based config (~/.config/langfuse/config.yml)
├── types.rs        # API response structs (Trace, Session, Observation, Score, Dataset, etc.)
├── commands/       # One module per resource (traces, sessions, observations, scores, metrics, prompts, datasets, config)
└── formatters/     # Output renderers (table, json, csv, markdown)
```

**Configuration resolution:** CLI args → environment variables → config file → defaults

**CLI structure:**
```
lf config {setup,set,show,list}
lf traces {list,get}
lf sessions {list,show}
lf observations {list,get}
lf scores {list,get,create}
lf metrics query
lf prompts {list,get,create-text,create-chat,label,delete}
lf datasets {list,get,create,items,item-get,item-create,runs,run-get}
```

## Key Patterns

- Each command module has a clap-derived enum with an `execute()` method returning `anyhow::Result<()>`
- `LangfuseClient` methods return typed response structs from `types.rs`
- Formatters receive `serde_json::Value` and render to stdout
- Config supports multiple named profiles with credentials (public key, secret key, host URL)

## More info

Issues are stored in GitHub.
