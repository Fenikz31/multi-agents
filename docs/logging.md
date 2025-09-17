## Logging (NDJSON)

Location
- Per-agent file: `./logs/{project}/{role}.ndjson` (append-only).

Example
```json
{"ts":"2025-09-15T14:03:21.123Z","project_id":"demo","agent_role":"backend","provider":"claude","session_id":"6b1f2a2e-4c3b-9a1e-8d2f-112233445566","direction":"system","event":"start"}
{"ts":"2025-09-15T14:03:21.456Z","project_id":"demo","agent_role":"backend","provider":"claude","session_id":"6b1f2a2e-4c3b-9a1e-8d2f-112233445566","direction":"agent","event":"stdout_line","text":"Hello! I'm Claude Code, ready to help..."}
{"ts":"2025-09-15T14:03:22.001Z","project_id":"demo","agent_role":"backend","provider":"claude","session_id":"6b1f2a2e-4c3b-9a1e-8d2f-112233445566","direction":"system","event":"end","exit_code":0}
```

Required fields per line
```json
{"ts":"2025-09-15T14:03:21.123Z","level":"info","project_id":"demo","agent_role":"backend","agent_id":"backend","provider":"gemini","session_id":"gemini:demo:backend:...","broadcast_id":null,"direction":"agent","event":"stdout_line","message_id":"...","text":"First response line","dur_ms":12}
```

Required keys
- `ts` (ISO-8601, millisecond precision)
- `project_id`
- `agent_role`
- `provider`
- `session_id`
- `direction` (one of: `user`, `agent`, `system`)
- `event` (see below)

Optional keys
- `level` (`info` par défaut)
- `agent_id`
- `broadcast_id`
- `message_id`
- `text`
- `dur_ms`
- `exit_code`

Events
- `start`: agent process started (no `text`).
- `stdout_line`: a line from provider stdout.
- `stderr_line`: a line from provider stderr.
- `end`: agent process terminated (include `exit_code`).
- `routed`: message routed by supervisor (carries `broadcast_id` or `message_id`).

Practices
- UTF-8 only, no ANSI codes.
- One JSON object per line; flush on newline.
- Include correlation IDs (`session_id`, `broadcast_id`) and durations when available.
- Rotate/age logs by size/time to limit disk usage.
