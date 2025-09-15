## Security and Tool Allowlists

Principles
- Safe-by-default; deny dangerous operations unless explicitly listed.
- Use provider-native allowlists/permission flags when available.

Per-provider
- Claude Code
  - Enforce `--allowed-tools "<list>"` per role.
  - Use `--permission-mode plan|acceptEdits` depending on role risk.
- Gemini CLI
  - Apply `--allowed-tools` and `--allowed-mcp-server-names` if supported.
  - If unsupported, warn and fall back to system prompt constraints.
- Cursor Agent
  - Do NOT allow `--force` (explicitly forbidden).
  - Constrain via system prompts and sandboxed working directories.

Validation
- On startup, validate that tools in role allowlists are known for the chosen provider.
- Refuse configuration with unknown tools (fail fast).

Sandboxing
- Run agents in dedicated working directories per role.
- Limit file system permissions (umask) and environment exposure.
