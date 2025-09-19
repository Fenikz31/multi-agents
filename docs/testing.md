## Testing Strategy

The CLI includes a comprehensive test suite with 24 tests organized into unit and integration tests:

### Test Organization
- **Unit tests** (`tests/unit/`): Test individual modules and utilities
  - `utils_tests.rs`: Configuration resolution, UUID generation, constants
  - `logging_tests.rs`: NDJSON event handling, ANSI escape sequence removal
  - `providers_tests.rs`: Provider management functionality
- **Integration tests** (`tests/integration/`): Test complete command workflows
  - `config_tests.rs`: Configuration file resolution and validation
  - `db_tests.rs`: Database operations and project/agent management
  - `tmux_tests.rs`: tmux operations, timeouts, and retry logic
  - `doctor_tests.rs`: Environment validation (placeholder)
  - `send_tests.rs`: Message sending workflows (placeholder)
  - `session_tests.rs`: Session management (placeholder)
  - `agent_tests.rs`: Agent lifecycle management (placeholder)

### Test Categories

**Smoke tests**
- Provider availability and flags (doctor): complete in < 10s total.
- One-shot send per provider returns non-empty text in < 2s (hello world).

**Session tests**
- Claude: `--session-id` resumes correctly.
- Cursor: `create-chat` then `--resume` reuses chat.
- Gemini: internal session ID works; transcript persisted.

**Broadcast tests**
- With >3 agents, ensure only 3 concurrent executions; others queue.
- NDJSON entries present for each agent with same `broadcast_id`.

**TUI tests**
- Keyboard navigation; Kanban + Sessions + Detail render; clean exit.

**Resilience tests**
- Timeout handling (return code 5) and clear error message.
- Provider CLI non-zero exit propagates with code 4.
- Missing config yields code 6; tmux issues yield code 8.

**Snapshots**
- Normalize provider text (strip ANSI, trim) and compare snapshots for regressions.

### Running Tests
```bash
# Run all tests
cargo test --package multi-agents-cli

# Run only unit tests
cargo test --package multi-agents-cli tests::unit

# Run only integration tests
cargo test --package multi-agents-cli tests::integration
```
