## Data Model (SQLite)

Tables
- projects(id TEXT PRIMARY KEY, name TEXT NOT NULL, created_at TEXT NOT NULL)
- agents(id TEXT PRIMARY KEY, project_id TEXT NOT NULL, role TEXT NOT NULL, provider TEXT NOT NULL, model TEXT NOT NULL, allowed_tools_json TEXT NOT NULL, system_prompt TEXT NOT NULL, created_at TEXT NOT NULL)
 - agents(id TEXT PRIMARY KEY, project_id TEXT NOT NULL, name TEXT NOT NULL, role TEXT NOT NULL, provider TEXT NOT NULL, model TEXT NOT NULL, allowed_tools_json TEXT NOT NULL, system_prompt TEXT NOT NULL, created_at TEXT NOT NULL)
- sessions(id TEXT PRIMARY KEY, project_id TEXT NOT NULL, agent_id TEXT NOT NULL, provider TEXT NOT NULL, provider_session_id TEXT, created_at TEXT NOT NULL)
- messages(id TEXT PRIMARY KEY, session_id TEXT NOT NULL, sender TEXT NOT NULL, content TEXT NOT NULL, broadcast_id TEXT, created_at TEXT NOT NULL)
- tasks(id TEXT PRIMARY KEY, project_id TEXT NOT NULL, title TEXT NOT NULL, status TEXT NOT NULL, assignee_agent_id TEXT, created_at TEXT NOT NULL)

Indexes
- projects(name)
- agents(project_id, role)
 - agents(project_id, name) UNIQUE
- sessions(project_id, created_at)
- messages(session_id, created_at)
- tasks(project_id, status, created_at)

Conventions
- Timestamps ISO-8601 UTC.
- `provider_session_id`: Claude `session_id` or Cursor `chat_id`; Gemini one-shot may be null.
- `broadcast_id`: shared across messages originating from a broadcast.
 - PRAGMAs enabled: `foreign_keys=ON`, `journal_mode=WAL`, `busy_timeout=3000ms`.
