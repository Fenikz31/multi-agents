## Purpose and Scope
- Build a Rust CLI that orchestrates official CLIs (Gemini, Claude Code, Cursor Agent) to run multiple AI agents in parallel.
- Terminal-first: user one-shot sends; agents run as REPLs inside tmux panes.
- Roles and system prompts in YAML; SQLite for state; NDJSON logs per agent.

## System Overview
- Orchestrator CLI (Rust): routes commands, composes provider invocations, enforces allowlists and concurrency (3).
- Providers (wrappers): templates for `gemini`, `claude`, `cursor-agent` in one-shot or REPL.
- Session Manager: maps internal `conversation_id` to provider `session_id`/`chat_id`.
- tmux Manager: one pane per agent; REPL startup; keystroke broadcast; log capture. See `docs/tmux.md` for naming and pipe-pane conventions.
- Store (SQLite): projects, agents, sessions, messages, tasks, broadcasts.
- TUI (ratatui): Kanban board, sessions list, session detail (NDJSON tail).

## Architecture

The CLI has been organized into a modular architecture:

### Module Organization
- **`cli/`**: Command definitions and parsing logic
- **`commands/`**: Implementation of all CLI commands (config, doctor, db, send, session, agent, init)
- **`utils/`**: Shared utilities (constants, error handling, config resolution, timeouts)
- **`tmux/`**: tmux session and window management with retry logic
- **`logging/`**: NDJSON event handling and logging utilities
- **`providers/`**: Provider management and integration
- **`tests/`**: Comprehensive test suite (unit and integration tests)

### Key Benefits
- **Maintainability**: Clear separation of concerns, easy navigation
- **Testability**: 24 tests covering all major functionality
- **Reusability**: Modular components can be used independently
- **Extensibility**: Easy to add new commands or providers

## Environment
- Linux/WSL2.
- Required CLIs: `gemini`, `claude`, `cursor-agent`, `tmux`, `git`.
- Database: `./data/multi-agents.sqlite3`.
- Logs: `./logs/{project}/{role}.ndjson`.

## Principles
- CLI-only integration; avoid HTTP SDKs.
- Safe-by-default via tool allowlists and sandboxing.
- Deterministic timeouts and backpressure.
- Structured observability (NDJSON + tracing).
