## Workflows

One-shot user send
1) Resolve `@role` → agent → provider; load allowlist.
2) Compose provider command (placeholders from `providers.yaml`).
3) Spawn process with timeout; capture stdout (text).
4) Persist user and agent messages; index by `conversation_id`.

Agent REPL (tmux)
1) Create tmux session `proj:{project}` if missing.
2) Create window `{role}:{agent}`; run provider REPL with system prompt.
3) Pipe pane to `./logs/{project}/{role}.ndjson`.
4) Send messages via `tmux send-keys`.

Broadcast
- One-shot: fan-out to all group members with concurrency=3; persist shared `broadcast_id`.
- REPL: send same keystrokes to each pane; NDJSON records one event per agent.

Routing and supervisor
- `@role` routes to that agent; `@all` expands to group `all`.
- `supervisor` subscribes to all messages; can re-route tasks.

Git context (opt-in)
- Collect `git status/diff/log -n 5` and inject into the prompt template.
- Respect size limits and redact secrets when possible.
