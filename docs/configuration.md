## Configuration

Files
- `project.yaml`: roles, prompts, providers, allowlists, groups.
- `providers.yaml`: per-provider command templates and flags.
- `.env`: optional secrets if CLIs require them.

project.yaml (example)
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
    provider: cursor
    model: gpt-5
    allowed_tools: ["Edit", "Search"]
    system_prompt: >
      Frontend engineer. Prioritize accessibility and clarity.
groups:
  - name: all
    members: ["supervisor", "backend", "frontend"]
```

providers.yaml (example)
```yaml
providers:
  claude:
    cmd: "claude"
    oneshot_args: ["-p","--print","--output-format","text","{prompt}","--session-id","{session_id}","--allowed-tools","{allowed_tools}","--permission-mode","plan"]
    repl_args: []  # "claude" REPL; resume via -r {session_id}
  cursor:
    cmd: "cursor-agent"
    oneshot_args: ["-p","--output-format","text","--resume","{chat_id}","{prompt}"]
    repl_args: ["agent","--resume","{chat_id}"]
    forbid_flags: ["--force"]
  gemini:
    cmd: "gemini"
    oneshot_args: ["{prompt}"]
    repl_args: ["-i","{system_prompt}","--allowed-tools","{allowed_tools}"]
```

Validation
- `multi-agents config validate --project-file project.yaml --providers-file providers.yaml`.
- Fails on missing roles, unknown tools per provider, or malformed placeholders.

JSON Schemas
- Generated from Rust models (Serde + schemars):
  - `docs/specs/schemas/project.schema.json`
  - `docs/specs/schemas/providers.schema.json`
- Regenerate with:
```bash
cargo run -p schema-gen -- --out-dir docs/specs/schemas
```
