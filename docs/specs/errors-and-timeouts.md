## Errors and Timeouts — Canonical Spec

### Exit codes (standardized)
- 0: ok
- 1: generic_error
- 2: invalid_input
- 3: provider_unavailable
- 4: provider_cli_error
- 5: timeout
- 6: missing_config
- 7: db_error
- 8: tmux_error

See machine-readable defaults in `config/defaults.yaml`.

### Timeouts (defaults)
- send: 120s
- doctor: 2s per provider (10s global)
- tmux action: 5s

### tmux errors and timeouts policy
- Any tmux command failure maps to exit code 8 (`tmux_error`) with cleaned stderr in the message.
- Each tmux action (has-session, new-session, new-window, pipe-pane, kill-window, attach) is bounded by the tmux action timeout (default 5s). On exceed, return 5 (`timeout`).
- One fast retry is allowed for race-prone sequences (e.g., new-session immediately followed by new-window).
- Idempotency: killing a non-existent window returns code 0 with a warning; re-piping with `-o` is safe.

### NDJSON logging constraints
- Append-only; 1 JSON object per line; UTF‑8; no ANSI escapes.
- Required fields at minimum (example subset): `ts`, `level`, `project_id`, `agent_role`, `agent_id`, `provider`, `event`, `text?`, `dur_ms?`, `broadcast_id?`, `session_id?`.

### Provider-level errors (context)
- Provider binary missing/unavailable: 3 (`provider_unavailable`).
- Provider process returns non-zero: 4 (`provider_cli_error`) for one-shot; for REPL, treat as `end` with error state, then propagate 4 if startup fails.

### Configuration detection
- Missing `project.yaml`/`providers.yaml` resolved paths → 6 (`missing_config`). CLI should suggest `multi-agents config init` and show searched locations.

# Error Codes and Default Timeouts (Canonical Spec)

Status: Stable
Schema version: 1

## Exit Codes

| Code | Name                   | Description                                      | Notes                 |
|-----:|------------------------|--------------------------------------------------|-----------------------|
| 0    | OK                     | Successful execution                             |                       |
| 1    | GENERIC_ERROR          | Unspecified error                                |                       |
| 2    | INVALID_INPUT          | Invalid CLI arguments or malformed config        |                       |
| 3    | PROVIDER_UNAVAILABLE   | Required provider CLI not found or unusable      | doctor, runtime       |
| 4    | PROVIDER_CLI_ERROR     | Provider CLI returned non‑zero exit              | capture stderr        |
| 5    | TIMEOUT                | Operation exceeded the configured timeout        | send/doctor/tmux      |
| 6    | MISSING_CONFIG         | Missing or unreadable required configuration     | YAML/env              |
| 7    | DB_ERROR               | Database error (reserved for M1+)                | not used in M0        |
| 8    | TMUX_ERROR             | tmux not installed or action failed              |                       |

Guidelines
- Map every failure to one of these codes; avoid ad‑hoc values.
- Provide actionable stderr with cause and remediation hints.
- Keep codes stable across versions; add new codes with care.

## Default Timeouts

| Operation                 | Default | Purpose                                    |
|---------------------------|---------|--------------------------------------------|
| send (one‑shot)           | 120s    | Upper bound for provider execution         |
| doctor per‑provider check | 2s      | Fast capability/version probe               |
| doctor global             | 10s     | Overall budget for doctor run              |
| tmux action               | 5s      | Create/attach/stop pane/window              |

Guidelines
- Make timeouts configurable via flags/env later; keep these as sane defaults.
- Use bounded concurrency (3) to avoid head‑of‑line blocking.

## Machine‑Readable Snapshot (JSON)

```json
{
  "schema_version": 1,
  "exit_codes": {
    "OK": 0,
    "GENERIC_ERROR": 1,
    "INVALID_INPUT": 2,
    "PROVIDER_UNAVAILABLE": 3,
    "PROVIDER_CLI_ERROR": 4,
    "TIMEOUT": 5,
    "MISSING_CONFIG": 6,
    "DB_ERROR": 7,
    "TMUX_ERROR": 8
  },
  "timeouts": {
    "send_ms": 120000,
    "doctor": {"per_provider_ms": 2000, "global_ms": 10000},
    "tmux_action_ms": 5000
  }
}
```
