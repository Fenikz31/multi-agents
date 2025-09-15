## Logging (NDJSON)

Location
- Per-agent file: `./logs/{project}/{role}.ndjson` (append-only).

Required fields per line
```json
{"ts":"2025-09-15T14:03:21.123Z","level":"info","project_id":"demo","agent_role":"backend","agent_id":"backend","provider":"gemini","session_id":"gemini:demo:backend:...","broadcast_id":null,"direction":"agent","event":"stdout_line","message_id":"...","text":"First response line","dur_ms":12}
```

Events
- `start`: agent process started (no `text`).
- `stdout_line`: a line from provider stdout.
- `stderr_line`: a line from provider stderr.
- `end`: agent process terminated (include `exit_code`).

Practices
- UTF-8 only, no ANSI codes.
- One JSON object per line; flush on newline.
- Include correlation IDs (`session_id`, `broadcast_id`) and durations when available.
- Rotate/age logs by size/time to limit disk usage.
