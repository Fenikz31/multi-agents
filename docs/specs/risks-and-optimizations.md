## Risks, Mitigations and Optimizations — M4

### Risks / Blockers
- tmux not installed or unusable (WSL2/CI): `agent attach` impossible → exit 8; mitigation: upfront `doctor` check; guidance message.
- Missing write permissions to `./logs/...`: `pipe-pane` fails; mitigation: suggest `--logs-dir`, `--no-logs`.
- Providers missing from PATH: `doctor` must block; clear error mapping to 3.
- Concurrency duplicates: multiple `agent run` racing → duplicate windows/pipes; mitigation: per-agent file lock.
- Large/ANSI outputs: log bloat and parsing issues; mitigation: strip ANSI, cap line length.

### Optimizations
- Per-agent lock (file-based) to serialize tmux operations.
- Healthcheck after REPL start (e.g., provider `--version`) to confirm ready state.
- CLI options: `--logs-dir`, `--no-logs`, `--workdir`.
- Metrics in NDJSON: startup duration, tmux categorized failures.
- Prepare M5: abstraction for multi-target `send-keys`, aggregate statuses, add `broadcast_id` correlation.


