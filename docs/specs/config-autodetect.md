## Configuration Autodetect — Spec

### Resolution order
- Flags > ENV (`MULTI_AGENTS_PROJECT_FILE`, `MULTI_AGENTS_PROVIDERS_FILE`, `MULTI_AGENTS_CONFIG_DIR`) > defaults (`./config/project.(yaml|yml)`, `./config/providers.(yaml|yml)`).

### First-run behavior
- On critical commands (`send`, `agent`, `tui`): if resolved files are missing → exit 6 (`missing_config`) and suggest:
  1) `multi-agents doctor` (environment/CLIs check)
  2) `multi-agents config init [--dir ./config]` (scaffold files, non-destructive)
  3) `multi-agents db init`
  4) `multi-agents project add` / `multi-agents agent add`

### Validation command
- `multi-agents config validate` checks existence and YAML schemas; exit 6 if missing.

### Notes
- Provider flags derived from `providers.yaml`; allowlists per role from `project.yaml`.
- Automatic sync YAML → DB at `send`/`session start` (idempotent ensure).


