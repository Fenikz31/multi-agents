## Glossary

- Agent: role-bound assistant process (one-shot or REPL in tmux).
- Broadcast: fan-out of the same message to a group (e.g., @all).
- Conversation ID: internal identifier mapping to provider session/chat when supported.
- NDJSON: newline-delimited JSON log format; one JSON object per line.
- Provider: external CLI (gemini, claude, cursor-agent) invoked by the orchestrator.
- REPL: long-running interactive process in a tmux pane.
- Role: function such as backend, frontend, devops, supervisor.
- Session: persisted context for a conversation with a provider.
- TUI: terminal user interface (ratatui).
