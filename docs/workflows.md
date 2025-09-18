## Workflows

One-shot user send
1) Resolve `@role` → agent → provider; load allowlist.
2) Compose provider command (placeholders from `providers.yaml`).
3) Enforce concurrency=3; spawn with timeout; stream stdout to user and log NDJSON per line.
4) Afficher un spinner de progression par défaut (`--no-progress` pour désactiver).
5) Sessions: si `--to <conversation_id>`, réutiliser la session (valider; fallback création si `expired/invalid`). Sinon, créer automatiquement une session (Claude/Gemini ID valide; Cursor via `create-chat`). Mettre à jour `last_activity` et enregistrer `provider_session_id` quand dispo.
6) **Cursor headless**: force `--output-format stream-json`, parse `assistant.message.content[].text` (deltas) et terminer sur l’événement `result`.

Agent REPL (tmux)
1) Ensure tmux session `proj:{project}` exists (create if missing).
2) Create window `{role}:{agent}`; start provider REPL (`<provider_cmd>`) with system prompt.
3) Activate `pipe-pane -o` to `./logs/{project}/{role}.ndjson`.
4) Emit `start` event; then append `stdout_line` per provider stdout line; on termination, emit `end` with `dur_ms`.
5) Send messages via `tmux send-keys` to the window `{role}:{agent}`.
6) See `docs/tmux.md` for detailed commands and timeout/error policy.

Broadcast
- One-shot: fan-out to all group members avec concurrency=3; persist shared `broadcast_id`.
- REPL: select targets (`@role|@all|list`), send identical keystrokes to each `{role}:{agent}` window; NDJSON records events per agent with a shared `broadcast_id`.
- Spinner affiche la progression globale quand activé (par défaut ON). Compatible avec les sorties NDJSON.

Routing and supervisor
- `@role` routes to that agent; `@all` expands to group `all`.
- `supervisor` subscribes to all messages; can re-route tasks.

Git context (opt-in)
- Collect `git status/diff/log -n 5` and inject into the prompt template.
- Respect size limits and redact secrets when possible.

First-run flow (desired)
- If configuration files are missing at invocation of critical commands (`agent`, `send`, `tui`):
  1) Suggest running `multi-agents doctor` to validate environment and CLIs in PATH.
  2) Suggest `multi-agents config init` to scaffold minimal `project.yaml` and `providers.yaml`.
  3) Run `multi-agents db init` to initialize SQLite.
  4) Guide the user to create a project and an agent, e.g.: `multi-agents project add --name demo` then `multi-agents agent add --project demo --name backend --role backend --provider gemini --model 2.0`.
  5) Propose starting a REPL: `multi-agents agent run --project demo --agent backend`, and optionally open the TUI.

