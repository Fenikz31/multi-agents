## CLI Reference (planned)

Environment
- Linux/WSL2; CLIs in PATH: `gemini`, `claude`, `cursor-agent`, `tmux`, `git`.

Global behavior
- Concurrency: max 3 one-shot executions (FIFO queue).
- Default timeouts: 120s per send; doctor 2s/provider (10s global); 5s tmux actions.
- Exit codes: 0 OK; 1 generic; 2 invalid input; 3 provider unavailable; 4 provider CLI error; 5 timeout; 6 config missing; 7 DB error; 8 tmux error.
  - See canonical spec: `docs/specs/errors-and-timeouts.md` (human) and `config/defaults.yaml` (machine-readable).

Commands
- `multi-agents doctor [--format text|json] [--ndjson-sample <path>]`
  - Verify CLIs, versions, flags; short timeouts.
  - Optional: `--ndjson-sample` runs a NDJSON self-check (one JSON per line, UTF‑8, no ANSI, required fields).
  - Exit codes: 0 OK; 1 degraded (flags manquants); 2 NDJSON invalide; 3 providers manquants; 5 timeout.
  - Optional: `--snapshot <path>` writes the full JSON report to a file (directories created if needed)
- `multi-agents config validate --project-file <path> --providers-file <path>`
- `multi-agents db init`
- `multi-agents project add --name <name>`
- `multi-agents agent add --project <name> --name <name> --role <role> --provider <prov> --model <model>`
- `multi-agents session start --project <name> --agent <name>` → prints `conversation_id=<id>`
- `multi-agents session list --project <name>`
- `multi-agents session resume --conversation-id <id>`
- `multi-agents send --conversation-id <id> --message "..."`
- `multi-agents send --to @role|@all --message "..."`
- `multi-agents agent run|attach|stop --project <name> --agent <name>`
- `multi-agents broadcast --project <name> --message "..." --mode oneshot|repl`
- `multi-agents tui --project <name>`
- `multi-agents context git --status|--diff|--log`

Notes
- Provider flags derive from `providers.yaml` and role allowlists in `project.yaml`.
- Use `--verbose` for detailed diagnostics and queue state.
