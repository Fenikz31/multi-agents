## Workflows

One-shot user send
1) Resolve `@role` → agent → provider; load allowlist.
2) Compose provider command (placeholders from `providers.yaml`).
3) Enforce concurrency=3; spawn with timeout; stream stdout to user and log NDJSON per line.
4) Afficher un spinner de progression par défaut (`--no-progress` pour désactiver).
5) Persist user and agent messages; index by `conversation_id`.
6) **Cursor headless**: force `--output-format stream-json`, parse `assistant.message.content[].text` deltas, terminate on `result` event.

Agent REPL (tmux)
1) Create tmux session `proj:{project}` if missing.
2) Create window `{role}:{agent}`; run provider REPL with system prompt.
3) Pipe pane to `./logs/{project}/{role}.ndjson` using `pipe-pane -o`.
4) Send messages via `tmux send-keys`.
5) See `docs/tmux.md` for detailed commands.

Broadcast
- One-shot: fan-out to all group members avec concurrency=3; persist shared `broadcast_id`.
- REPL: send same keystrokes to each pane; NDJSON records one event per agent.
- Spinner affiche la progression globale quand activé (par défaut ON).

Routing and supervisor
- `@role` routes to that agent; `@all` expands to group `all`.
- `supervisor` subscribes to all messages; can re-route tasks.

Git context (opt-in)
- Collect `git status/diff/log -n 5` and inject into the prompt template.
- Respect size limits and redact secrets when possible.
