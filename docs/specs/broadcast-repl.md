## Broadcast — REPL Mode (Preparation for M5)

### Goal
- Fan-out the same input to multiple agents running in tmux REPL by sending identical keystrokes (`send-keys`) to each target window.

### CLI (preview)
- `multi-agents broadcast --project <name> --message "..." --mode repl [--to @role|@all|agent1,agent2]`
- Concurrency limit does not apply to REPL send-keys (instant), but status aggregation should be reported.
- Exit codes: 0 OK; 2 invalid input; 8 tmux error.

### Target selection
- `@role`: all agents with this role in the project.
- `@all`: every agent of the project.
- Explicit list: comma-separated agent names.

### tmux mapping
- For each selected agent, resolve `proj:{project}:{role}:{agent}` window and emit `tmux send-keys ... C-m`.
- Missing window: report as error for that agent; overall status OK if at least one success unless `--strict` is specified.

### NDJSON logging
- Each agent still logs independently to `./logs/{project}/{role}.ndjson`.
- Add a shared `broadcast_id` to correlate messages across agents.

### Error policy
- If tmux returns non-zero for a target, record an error entry; overall exit code 8 if all targets fail; else 0 (or 1 with summary) depending on UX decision.

### Observability
- Print a compact summary: targets attempted, successes, failures; optionally output JSON with per-target status.


