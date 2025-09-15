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
