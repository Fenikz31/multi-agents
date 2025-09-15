# Product Brief — Multi-Agents CLI

Status: Draft
Owner: @fenikz
Last updated: 2025-09-15
Source: basic-idea.md

## Problem Statement
Developers need a terminal-first way to coordinate multiple AI coding agents (Gemini CLI, Claude Code, Cursor Agent) across parallel sessions, preserving context per project and enabling fast collaboration and oversight without web dashboards.

## Goals
- Single Rust CLI orchestrating official CLIs; Linux/WSL2 first.
- Multiple agents with roles; user↔agent and agent↔agent messaging.
- Track and resume sessions (conversation IDs) across projects.
- Terminal dashboard (TUI) to monitor tasks, sessions, and logs.
- Broadcast messages to groups (e.g., @all).
- Operational guardrails (tool allowlists, timeouts, concurrency caps).

## Non‑Goals (initial MVP)
- No HTTP SDK integrations (CLI-only).
- No SSO/auth or remote multi-user server.
- No cost tracking/quotas.

## Users and Primary Use Cases
- Solo developer on Linux/WSL2 coordinating backend, frontend, devops, documentation agents in parallel.
- Architect/supervisor agent routing tasks and summarizing progress.
- Quick fan-out announcements (broadcast) to all agents.

## Key Requirements
- CLI-first UX (bash/zsh); agents run as REPLs in tmux panes.
- Roles and prompts defined in YAML; tool allowlists per role.
- Conversation IDs: persist provider session/chat IDs when available (Claude `session_id`, Cursor `chat_id`).
- SQLite persistence for projects, agents, sessions, messages, tasks, broadcasts.
- TUI with Kanban (ToDo/Doing/Done), sessions list, and session detail.
- Broadcast in two modes: one-shot (spawn) and REPL (tmux send-keys).
- Structured logging per agent in NDJSON; correlation IDs throughout.
- Concurrency limit: max 3 one-shot executions; deterministic timeouts.
- Git context (read-only) optionally injected into prompts.

## Success Criteria
- Doctor validates environment and config in <10s with clear outputs.
- One-shot send returns non-empty text for Gemini, Claude, Cursor within 120s.
- tmux agents log NDJSON (`start`, `stdout_line`, `end`) to per-role files.
- Broadcast to >3 agents queues correctly (concurrency=3) and persists a shared `broadcast_id`.
- TUI renders Kanban, sessions, and scrollable session detail; exits cleanly.

## Constraints and Assumptions
- Linux/WSL2, tmux installed; CLIs in PATH: `gemini`, `claude`, `cursor-agent`, `git`.
- No offline mode initially (Ollama/LM Studio out of scope for MVP).
- No cloud privacy restrictions (using official CLIs as installed).

## High-Level Solution Outline
- Orchestrator CLI in Rust using templates to invoke `gemini`, `claude`, `cursor-agent`.
- Session Manager mapping internal `conversation_id` to provider `session_id`/`chat_id`.
- tmux Manager: one pane per agent/role; REPL startup; keystroke broadcast.
- SQLite storage; NDJSON logs under `./logs/{project}/{role}.ndjson`.
- TUI (ratatui) for Kanban/sessions/log tail.
- Security: tool allowlists per role; forbid dangerous flags (e.g., Cursor `--force`).

## Roadmap and Acceptance
See `docs/roadmap.md` for milestone-by-milestone acceptance contracts (commands, inputs/outputs, formats, timeouts, exit codes).

## Open Questions (post-MVP)
- When to enable internal parsing of provider stream-json for more robust persistence.
- Whether to add zellij as an alternative to tmux by default.
- Depth and format of Git context injection per role.

## Example Interactions (reference)
- Start a session and send a message (one-shot):
  - `multi-agents session start --project demo --agent backend`
  - `multi-agents send --conversation-id <id> --message "Implement /health"`
- Run agents in tmux and broadcast:
  - `multi-agents agent run --project demo --agent backend`
  - `multi-agents broadcast --project demo --message "Standup in 10m" --mode oneshot`
