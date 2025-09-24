## Configuration

Files
- `project.yaml`: roles, prompts, providers, allowlists, groups.
- `providers.yaml`: per-provider command templates and flags.
- `.env`: optional secrets if CLIs require them.

Default locations and resolution
- Priority: CLI flags > ENV > defaults.
- ENV:
  - `MULTI_AGENTS_PROJECT_FILE`, `MULTI_AGENTS_PROVIDERS_FILE` point to explicit files.
  - `MULTI_AGENTS_CONFIG_DIR` points to a directory containing `project.(yaml|yml)` and `providers.(yaml|yml)`.
  - `MULTI_AGENTS_HOME` sets application home directory (affects config, DB, logs paths).
  - `XDG_CONFIG_HOME` sets XDG config directory (defaults to `$HOME/.config`).
- Defaults (if nothing provided): `./config/project.yaml|yml`, `./config/providers.yaml|yml`.
- If no resolvable file is found: exit 6 (config missing).

Path Resolution (XDG-compliant)
The CLI uses centralized path resolution following XDG Base Directory specification:

**Configuration Directory Resolution:**
1. `MULTI_AGENTS_CONFIG_DIR` - Explicit override
2. `MULTI_AGENTS_HOME/config` - Application-specific home
3. `XDG_CONFIG_HOME/multi-agents` - XDG config directory
4. `$HOME/.config/multi-agents` - XDG fallback
5. `./config` - Development fallback

**Database Path Resolution:**
1. `MULTI_AGENTS_DB` - Explicit database file path
2. `MULTI_AGENTS_HOME/multi-agents.sqlite3` - Application-specific home
3. `XDG_DATA_HOME/multi-agents/multi-agents.sqlite3` - XDG data directory
4. `$HOME/.local/share/multi-agents/multi-agents.sqlite3` - XDG fallback
5. `./data/multi-agents.sqlite3` - Development fallback

**Logs Path Resolution:**
1. `MULTI_AGENTS_LOGS_DIR` - Explicit logs directory
2. `MULTI_AGENTS_HOME/logs` - Application-specific home
3. `XDG_DATA_HOME/multi-agents/logs` - XDG data directory
4. `$HOME/.local/share/multi-agents/logs` - XDG fallback
5. `./logs` - Development fallback

Bootstrap
- `multi-agents config init [--dir <path>] [--force]` scaffolds minimal `project.yaml` and `providers.yaml` under the target directory (default `./config`). Existing files are not overwritten unless `--force`.

TUI Auto-Seed
- When the TUI starts and no projects are found in the database, it automatically attempts to seed from configuration files:
  1. Resolves configuration paths using the same priority system
  2. Reads and parses `project.yaml`
  3. Synchronizes project and agents to the database
  4. Reloads the project list for display
- This ensures that existing configuration files are automatically imported when starting the TUI for the first time.
- Auto-seed is non-fatal: if configuration files are missing or invalid, the TUI will start with an empty project list.

project.yaml (minimal example)
```yaml
project: demo-app
agents:
  - name: supervisor
    role: supervisor
    provider: claude
    model: sonnet-4
    allowed_tools: ["Search", "Edit"]
    system_prompt: >
      Coordinate agents, route tasks, write concise summaries.
  - name: backend
    role: backend
    provider: gemini
    model: gemini-1.5-flash
    allowed_tools: ["Edit", "Bash(git:status)"]
    system_prompt: >
      Backend engineer. Respond in up to 5 bullet points.
  - name: frontend
    role: frontend
    provider: cursor-agent
    model: gpt-5
    allowed_tools: ["Edit", "Search"]
    system_prompt: >
      Frontend engineer. Prioritize accessibility and clarity.
groups:
  - name: all
    members: ["supervisor", "backend", "frontend"]
```

providers.yaml (minimal example)
```yaml
providers:
  claude:
    cmd: "claude"
    oneshot_args: ["-p","--print","--output-format","text","{prompt}","--session-id","{session_id}","--allowed-tools","{allowed_tools}","--permission-mode","plan"]
    repl_args: []  # "claude" REPL; resume via -r {session_id}
  cursor-agent:
    cmd: "cursor-agent"
    oneshot_args: ["-p","--output-format","text","--resume","{chat_id}","{prompt}"]
    repl_args: ["agent","--resume","{chat_id}"]
    forbid_flags: ["--force"]
Complete examples
- See `examples/project-complete.yaml` and `examples/providers-complete.yaml`

Invalid examples (for testing)
- `examples/project-invalid-missing-fields.yaml`
- `examples/providers-invalid-placeholders.yaml`
  gemini:
    cmd: "gemini"
    oneshot_args: ["{prompt}"]
    repl_args: ["-i","{system_prompt}","--allowed-tools","{allowed_tools}"]
```

Validation
- `multi-agents config validate --project-file project.yaml --providers-file providers.yaml`.
- Fails on missing roles, unknown tools per provider, or malformed placeholders.
- Additional semantic rules (M0-03):
  - Providers:
    - `claude`: `{prompt}` in oneshot args; `{session_id}` recommended; `{allowed_tools}` if `allowlist_flag` set.
    - `cursor*`: `{prompt}` in oneshot args; `{chat_id}` in oneshot & repl args.
    - `gemini`: `{prompt}` in oneshot args; `{system_prompt}` in repl args; `{allowed_tools}` if `allowlist_flag` set.
  - Project:
    - `schema_version == 1`.
    - Agent names unique; provider keys must exist in providers.yaml.
    - For `claude`/`gemini`, `allowed_tools` must not be empty; `system_prompt` non-empty.
    - Group members must reference existing agent names.

JSON Schemas
- Generated from Rust models (Serde + schemars):
  - `docs/specs/schemas/project.schema.json`
  - `docs/specs/schemas/providers.schema.json`
- Regenerate with:
```bash
cargo run -p schema-gen -- --out-dir docs/specs/schemas
```
