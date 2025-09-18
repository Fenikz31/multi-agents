## tmux Conventions

### Naming
- Session: `proj:{project}`
- Windows: `{role}:{agent}`
- Panes: one pane per agent REPL

### Logging
- `tmux pipe-pane -o` to `./logs/{project}/{role}.ndjson`
- Append-only, rotate via external tooling

### Timeouts
- tmux actions (create/attach/stop): default 5s (see `config/defaults.yaml`)

### Recommended commands
- Create session if missing: `tmux has-session -t proj:{project} || tmux new-session -d -s proj:{project}`
- Create window: `tmux new-window -t proj:{project} -n {role}:{agent} -- <provider_cmd>`
- Pipe pane once (append-only): `tmux pipe-pane -ot proj:{project}:{role}:{agent} 'cat >> ./logs/{project}/{role}.ndjson'`
- Emit start event (example): `printf '%s\n' '{"ts":"...","event":"start", ...}' >> ./logs/{project}/{role}.ndjson`
- Send keys: `tmux send-keys -t proj:{project}:{role}:{agent} "{text}" C-m`
- Attach: `tmux attach -t proj:{project}`
- Stop window: `tmux kill-window -t proj:{project}:{role}:{agent}`

### REPL startup flow (reference)
1. Ensure tmux session `proj:{project}` exists.
2. Create window `{role}:{agent}` with `<provider_cmd>` running the provider in interactive mode.
3. Activate `pipe-pane -o` to `./logs/{project}/{role}.ndjson`.
4. Write `start` NDJSON event with agent/provider metadata.
5. On each stdout line, one `stdout_line` event is appended (no ANSI, UTF‑8).
6. On termination, write `end` with `dur_ms` and status.

### Error handling and timeouts
- Default timeout per tmux action: 5s (see `config/defaults.yaml`).
- On timeout: return code 5; on tmux failure: return code 8.
- Idempotency: `pipe-pane -o` prevents duplicate piping; re-running should not create duplicates.
