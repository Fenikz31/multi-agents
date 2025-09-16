## Roadmap and Acceptance Contract

Standards
- Concurrency: max 3 one-shot executions (global semaphore, FIFO).
- Timeouts: 120s per send; 2–10s doctor/detection; 5s tmux actions.
- Logging: NDJSON per agent at `./logs/{project}/{role}.ndjson`.
- Exit codes: 0 OK; 1 generic; 2 invalid input; 3 provider unavailable; 4 provider CLI error; 5 timeout; 6 config missing; 7 DB error; 8 tmux error.
- References: canonical spec in `docs/specs/errors-and-timeouts.md`; machine-readable defaults in `config/defaults.yaml`.

Milestones (M0–M9)
- M0 Config/Doctor
  - Commands: `multi-agents doctor`, `multi-agents config validate`.
  - Must: detect CLIs and key flags; validate YAML; exit 0 only when all required are OK.
  - Doctor options: `--format text|json`, `--ndjson-sample <path>`, `--snapshot <path>` (writes JSON report)
  - Timeouts: 2s/provider (version/help); 10s global.
- M1 Data Model (SQLite)
  - Commands: `multi-agents db init`, `project add`, `agent add`.
  - Must: create DB/tables; ISO-8601 timestamps; indexes present.
  - Timeout: 3s.
- M2 Providers One-shot
  - Commands: `session start`, `send`.
  - Must: 3 providers return plain text; apply allowlists; handle timeout deterministically.
  - Timeout: 120s per send.
- M3 Session Resume
  - Commands: `session list`, `session resume`.
  - Must: Claude `--session-id` reuse; Cursor `create-chat`/`--resume`; Gemini internal ID.
- M4 tmux Agents (REPL)
  - Commands: `agent run|attach|stop`.
  - Must: pane exists; NDJSON logs appended (`start`, `stdout_line`, `end`).
  - Timeout: 5s per action.
- M5 Broadcast
  - Commands: `broadcast --mode oneshot|repl`.
  - Must: concurrency=3 enforced; aggregate OK/Error/Timeout; persist `broadcast_id`.
- M6 TUI
  - Commands: `tui`.
  - Must: Kanban + Sessions + Detail; ≤5 Hz refresh; clean terminal exit.
- M7 Routing and Supervisor
  - Commands: `send --to @role|@all`.
  - Must: correct routing; supervisor receives system log entries.
- M8 Git Context (read-only)
  - Commands: `context git --status|--diff|--log`, `send --include-git`.
  - Must: render context; inject on demand; clear errors out of repo.
- M9 Robustness and Observability
  - Commands: `send --timeout ...`, `broadcast --verbose`.
  - Must: consistent error codes; tracing correlation; retries where applicable.

NDJSON schema (required fields)
```json
{"ts":"2025-09-15T14:03:21.123Z","level":"info","project_id":"demo","agent_role":"backend","agent_id":"backend","provider":"gemini","session_id":"gemini:demo:backend:...","broadcast_id":null,"direction":"agent","event":"stdout_line","message_id":"...","text":"First response line","dur_ms":12}
```

