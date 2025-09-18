## DB Integration for REPL — Spec

### Scope
- Align REPL lifecycle with existing SQLite schema (see `docs/data-model.md`, `crates/db`).

### Entities involved
- projects, agents, sessions, messages, broadcasts (read-only for M4, write minimal if needed).

### Session model
- For REPL, optionally create a `sessions` row with `type=repl`, `status=active`, `agent_id`, `provider`, `created_at`, `last_activity`.
- Store `provider_session_id` if/when the provider exposes a persistent session ID; otherwise null.

### Messages mapping
- NDJSON is the source of truth for REPL logs in M4; optional ingestion pipeline can map NDJSON lines to `messages` for querying.
- Minimal requirement: ensure `message_id` uniqueness and correlatable fields (agent_id, timestamps).

### Cleanup
- `session cleanup` (>24h) applies to REPL sessions as well; state transitions to `inactive` on `end` event or explicit stop.

### Out of scope (M4)
- Full ingestion of NDJSON into DB (schedule for M7).
- Complex indexing/analytics on REPL content.


