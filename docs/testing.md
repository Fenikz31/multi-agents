## Testing Strategy

Smoke tests
- Provider availability and flags (doctor): complete in < 10s total.
- One-shot send per provider returns non-empty text in < 2s (hello world).

Session tests
- Claude: `--session-id` resumes correctly.
- Cursor: `create-chat` then `--resume` reuses chat.
- Gemini: internal session ID works; transcript persisted.

Broadcast tests
- With >3 agents, ensure only 3 concurrent executions; others queue.
- NDJSON entries present for each agent with same `broadcast_id`.

TUI tests
- Keyboard navigation; Kanban + Sessions + Detail render; clean exit.

Resilience tests
- Timeout handling (return code 5) and clear error message.
- Provider CLI non-zero exit propagates with code 4.
- Missing config yields code 6; tmux issues yield code 8.

Snapshots
- Normalize provider text (strip ANSI, trim) and compare snapshots for regressions.
