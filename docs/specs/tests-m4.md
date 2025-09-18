## Tests Plan — M4 tmux Agents (REPL)

### Unit tests
- CLI parsing for `agent run|attach|stop` (flags, missing args → exit 2).
- Timeout mapping: simulate tmux runner stall → exit 5.
- tmux errors mapping: non-zero return → exit 8 with cleaned stderr.
- Naming builder: session/window/pane strings per conventions.

### Contract tests (NDJSON)
- Validate `start`, `stdout_line`, `end` events schema (required fields, UTF‑8, no ANSI).
- Snapshot of example lines; ensure append-only behavior.

### Integration (simulated tmux)
- Mock `tmux` runner covering sequences: has-session→new-session→new-window→pipe-pane→send-keys.
- Idempotency: rerun `agent run` does not duplicate window/pipe.
- `stop` on missing window returns OK with warning.

### Docs validation
- Execute examples in `docs/tmux.md` with dry-run stubs to ensure they stay in sync.


