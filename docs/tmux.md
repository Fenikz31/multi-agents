## tmux Conventions

### Naming
- Session: `proj:{project}`
- Windows: `{role}:{agent}`
- Panes: one pane per agent REPL

### Logging
- `tmux pipe-pane -o` to `./logs/{project}/{role}.ndjson`
- Append-only, rotate via external tooling

### Timeouts
- tmux actions (create/attach/stop): default 5s (see `config/defaults.yaml`)

### Recommended commands
- Create session if missing: `tmux has-session -t proj:{project} || tmux new-session -d -s proj:{project}`
- Create window: `tmux new-window -t proj:{project} -n {role}:{agent} -- <cmd>`
- Pipe pane once: `tmux pipe-pane -ot proj:{project}:{role}:{agent} 'cat >> ./logs/{project}/{role}.ndjson'`
- Send keys: `tmux send-keys -t proj:{project}:{role}:{agent} "{text}" C-m`
- Attach: `tmux attach -t proj:{project}`
