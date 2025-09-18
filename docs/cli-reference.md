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
  - Shows a progress spinner during checks.
- `multi-agents config validate [--project-file <path>] [--providers-file <path>]`
  - Path resolution (priority): flags > ENV (`MULTI_AGENTS_PROJECT_FILE`, `MULTI_AGENTS_PROVIDERS_FILE`, `MULTI_AGENTS_CONFIG_DIR`) > defaults (`./config/project.(yaml|yml)`, `./config/providers.(yaml|yml)`).
  - Missing resolved file(s) → exit 6 (config missing).
- `multi-agents config init [--dir <path>] [--force]`
  - Scaffold minimal `project.yaml` and `providers.yaml` (default dir `./config`).
  - Won't overwrite existing files unless `--force`.
- `multi-agents db init`
  - Initialize SQLite database (idempotent). Exit codes: 0 OK; 7 db_error.
- `multi-agents project add --name <name>`
- `multi-agents agent add --project <name> --name <name> --role <role> --provider <prov> --model <model>`
  - Exit codes: 0 OK; 2 invalid_input; 7 db_error. Prints created IDs.
- `multi-agents session start --project <name> --agent <name>` → prints `conversation_id=<id>`
- `multi-agents session list --project <name> [--agent <name>] [--provider <prov>] [--format text|json]`
  - Liste les sessions par projet avec filtres. Par défaut `status=active`, `limit=50`, tri `created_at DESC`.
  - Format `text` (table) ou `json` (objets). Retourne `id`, `provider`, `status`, `created_at`, `last_activity`, `provider_session_id`.
- `multi-agents session resume --conversation-id <id> [--timeout-ms 5000]`
  - Valide et reprend une session. Timeout 5s. Erreurs normalisées si `expired/invalid`.
- `multi-agents session cleanup [--project-file <path>] [--dry-run] [--format text|json]`
  - Supprime les sessions inactives (>24h) selon `last_activity` ou `created_at`. En `--dry-run`, affiche sans supprimer.
- `multi-agents send [--project-file <path>] [--providers-file <path>] --to @role|@all|<agent> --message "..." [--timeout-ms <millis>] [--format text|json]`
  - Uses same path resolution as `config validate`. Missing → exit 6.
  - Optional: `--timeout-ms` overrides 120s default; `--format json` prints a minimal JSON status.
  - Shows a progress spinner by default; disable with `--no-progress`.
  - **Cursor headless**: automatically uses `--output-format stream-json` and parses `assistant.message.content[].text` deltas plus final `result` event for clean termination.
  - Sessions: `--to <conversation_id>` cible une session existante. Sans session, création automatique par provider (Claude/Gemini ID valide, Cursor `create-chat`). Met à jour `last_activity` et, si disponible, `provider_session_id`.
- `multi-agents agent run|attach|stop --project <name> --agent <name>`
-   - Agent REPL lifecycle (tmux). Subcommands and flags:
-     - `run` — create tmux session/window if missing and start provider REPL
-       - Flags: `--project <name>` (required), `--agent <name>` (required), `--role <role>` (optional), `--provider <prov>` (optional), `--model <model>` (optional), `--workdir <path>` (optional), `--no-logs` (optional), `--timeout-ms <int>` (default 5000)
-       - Exit codes: 0 OK; 2 invalid input; 5 timeout; 8 tmux error
-       - Behavior: ensures session `proj:{project}`, window `{role}:{agent}`, 1 pane; starts provider REPL; if logging enabled, pipes pane to `./logs/{project}/{role}.ndjson` using `pipe-pane -o`
-       - Examples:
-         - `multi-agents agent run --project demo --agent backend`
-         - `multi-agents agent run --project demo --agent writer --provider claude --model opus --timeout-ms 8000`
-     - `attach` — attach the current terminal to tmux session
-       - Flags: `--project <name>` (required)
-       - Exit codes: 0 OK; 8 tmux error
-       - Behavior: `tmux attach -t proj:{project}` (prints guidance when non-interactive)
-       - Example: `multi-agents agent attach --project demo`
-     - `stop` — stop a specific agent REPL by killing its tmux window
-       - Flags: `--project <name>` (required), `--agent <name>` (required)
-       - Exit codes: 0 OK (idempotent even if window missing); 8 tmux error
-       - Behavior: `tmux kill-window -t proj:{project}:{role}:{agent}`; does not kill session
-       - Example: `multi-agents agent stop --project demo --agent backend`
- `multi-agents broadcast --project <name> --message "..." --mode oneshot|repl`
  - REPL mode (preview):
    - `--to @role|@all|agent1,agent2` to select targets
    - Sends identical keystrokes to each target tmux window `{role}:{agent}`
    - Aggregates per-target status; prints summary; optional JSON output
    - Exit codes: 0 OK; 2 invalid input; 8 tmux error
- `multi-agents tui --project <name>`
- `multi-agents context git --status|--diff|--log`

Notes
- Provider flags derive from `providers.yaml` and role allowlists in `project.yaml`.
- Use `--verbose` for detailed diagnostics and queue state.
 - Synchronisation automatique YAML → DB: à chaque `send`/`session start`, les projets/agents du `project.yaml` sont assurés en base (idempotent).
 - tmux conventions: session `proj:{project}`, window `{role}:{agent}`, 1 pane per agent; logs path `./logs/{project}/{role}.ndjson`.
 - NDJSON events for REPL: `start`, `stdout_line`, `end` with required fields (see `docs/roadmap.md`).
