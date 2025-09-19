# tmux Integration Guide

## Overview

Multi-Agents CLI uses tmux to manage agent REPL sessions, providing persistent interactive environments for AI agents. Each agent runs in its own tmux window with structured logging and broadcast capabilities.

## Naming Conventions

### Session Naming
- **Format**: `proj:{project}`
- **Example**: `proj:demo`, `proj:my-web-app`
- **Purpose**: Groups all windows for a project in a single session

### Window Naming
- **Format**: `{role}:{agent}`
- **Example**: `backend:api-server`, `frontend:react-app`, `devops:deployer`
- **Purpose**: Identifies agent role and specific instance

### Pane Structure
- **One pane per agent**: Each agent gets exactly one pane in its window
- **No splitting**: Agents run in dedicated single-pane windows
- **Consistent layout**: Predictable structure for automation

## Logging Architecture

### NDJSON Log Files
- **Path**: `./logs/{project}/{role}.ndjson`
- **Format**: One JSON object per line, UTF-8 encoded
- **Append-only**: New events are appended, never overwritten
- **Rotation**: Handled by external tooling (logrotate, etc.)

### Pipe-Pane Integration
```bash
# Activate logging for a pane
tmux pipe-pane -ot proj:{project}:{role}:{agent} 'cat >> ./logs/{project}/{role}.ndjson'
```

**Key Features:**
- `-o` flag: Only output (stdout) is captured, not input
- `-t` target: Specifies the exact pane to monitor
- Append-only: `>>` ensures no data loss on restart
- Idempotent: Safe to run multiple times without duplication

## Timeout Configuration

### Default Timeouts
- **tmux actions**: 5s per action (create/attach/stop)
- **Configuration**: See `config/defaults.yaml`
- **Override**: Use `--timeout-ms` flag in agent commands

### Timeout Scenarios
1. **Session creation**: `tmux new-session` timeout
2. **Window creation**: `tmux new-window` timeout  
3. **Pane piping**: `tmux pipe-pane` timeout
4. **Key sending**: `tmux send-keys` timeout
5. **Window killing**: `tmux kill-window` timeout

## Core tmux Commands

### Session Management
```bash
# Check if session exists
tmux has-session -t proj:{project}

# Create session if missing (detached)
tmux new-session -d -s proj:{project}

# Attach to session
tmux attach -t proj:{project}

# List sessions
tmux list-sessions
```

### Window Management
```bash
# Create new window with command
tmux new-window -t proj:{project} -n {role}:{agent} -- <provider_cmd>

# List windows in session
tmux list-windows -t proj:{project}

# Kill specific window
tmux kill-window -t proj:{project}:{role}:{agent}
```

### Pane Operations
```bash
# Pipe pane output to log file
tmux pipe-pane -ot proj:{project}:{role}:{agent} 'cat >> ./logs/{project}/{role}.ndjson'

# Send keys to pane
tmux send-keys -t proj:{project}:{role}:{agent} "{text}" C-m

# Send Enter key
tmux send-keys -t proj:{project}:{role}:{agent} C-m
```

## REPL Startup Flow

### Complete Agent Startup Sequence
1. **Validate inputs**: Check project/agent exist in database
2. **Check session**: `tmux has-session -t proj:{project}`
3. **Create session**: If missing, `tmux new-session -d -s proj:{project}`
4. **Check window**: Verify `{role}:{agent}` window doesn't exist
5. **Create window**: `tmux new-window -t proj:{project} -n {role}:{agent} -- <provider_cmd>`
6. **Activate logging**: `tmux pipe-pane -ot proj:{project}:{role}:{agent} 'cat >> ./logs/{project}/{role}.ndjson'`
7. **Emit start event**: Write NDJSON `start` event with metadata
8. **Monitor output**: Capture stdout lines as `stdout_line` events
9. **Handle termination**: Write `end` event with duration and status

### Provider Command Examples
```bash
# Gemini REPL
gemini chat --model 2.0 --system-prompt "You are a backend developer..."

# Claude REPL  
claude chat --model opus --system-prompt "You are a frontend developer..."

# Cursor REPL
cursor-agent chat --model gpt-4 --system-prompt "You are a DevOps engineer..."
```

## NDJSON Event Schema

### Start Event
```json
{
  "ts": "2025-01-15T14:03:21.123Z",
  "level": "info",
  "project_id": "demo",
  "agent_role": "backend", 
  "agent_id": "api-server",
  "provider": "gemini",
  "session_id": "proj:demo:backend:api-server",
  "broadcast_id": null,
  "direction": "agent",
  "event": "start",
  "message_id": null,
  "text": "Agent started with provider gemini model 2.0",
  "dur_ms": null
}
```

### Stdout Line Event
```json
{
  "ts": "2025-01-15T14:03:22.456Z",
  "level": "info", 
  "project_id": "demo",
  "agent_role": "backend",
  "agent_id": "api-server", 
  "provider": "gemini",
  "session_id": "proj:demo:backend:api-server",
  "broadcast_id": null,
  "direction": "agent",
  "event": "stdout_line",
  "message_id": null,
  "text": "I'll help you implement the user authentication system.",
  "dur_ms": null
}
```

### End Event
```json
{
  "ts": "2025-01-15T14:05:30.789Z",
  "level": "info",
  "project_id": "demo", 
  "agent_role": "backend",
  "agent_id": "api-server",
  "provider": "gemini", 
  "session_id": "proj:demo:backend:api-server",
  "broadcast_id": null,
  "direction": "agent",
  "event": "end",
  "message_id": null,
  "text": "Agent terminated normally",
  "dur_ms": 129666
}
```

## Error Handling and Recovery

### Exit Code Mapping
- **0**: Success
- **2**: Invalid input (project/agent not found)
- **5**: Timeout (tmux action exceeded limit)
- **8**: tmux error (command failed, cleaned stderr)

### Common Error Scenarios

#### Session Already Exists
```bash
# Safe to run multiple times
tmux has-session -t proj:demo || tmux new-session -d -s proj:demo
```

#### Window Already Exists
```bash
# Check before creating
if ! tmux list-windows -t proj:demo | grep -q "backend:api-server"; then
    tmux new-window -t proj:demo -n backend:api-server -- gemini chat
fi
```

#### Pane Already Piped
```bash
# pipe-pane -o is idempotent
tmux pipe-pane -ot proj:demo:backend:api-server 'cat >> ./logs/demo/backend.ndjson'
```

### Retry Logic
- **Single retry**: For race-prone sequences (new-session → new-window)
- **Exponential backoff**: Not implemented (keep it simple)
- **Idempotent operations**: All commands safe to retry

### Error Message Cleaning
```bash
# Raw tmux error
tmux: can't find session proj:nonexistent

# Cleaned error message  
Session 'proj:nonexistent' not found. Run 'multi-agents agent run --project <name> --agent <name>' first.
```

## Broadcast Integration

### Send Keys to Multiple Agents
```bash
# Send to specific role
for window in $(tmux list-windows -t proj:demo -F "#W" | grep "^backend:"); do
    tmux send-keys -t proj:demo:$window "Check database status" C-m
done

# Send to all agents
for window in $(tmux list-windows -t proj:demo -F "#W"); do
    tmux send-keys -t proj:demo:$window "Starting deployment" C-m
done
```

### Broadcast Status Tracking
- **Per-agent status**: Track success/failure for each target
- **Aggregate reporting**: Summary of broadcast results
- **NDJSON correlation**: Shared `broadcast_id` across events

## Best Practices

### Session Lifecycle
1. **Create once**: Session created on first agent start
2. **Persist**: Session survives agent stops
3. **Clean shutdown**: Manual cleanup when project complete

### Window Management
1. **One agent per window**: No window sharing
2. **Descriptive names**: Use role:agent format consistently
3. **Clean termination**: Proper shutdown on agent stop

### Logging Strategy
1. **Append-only**: Never truncate log files
2. **UTF-8 encoding**: Ensure proper character handling
3. **No ANSI escapes**: Clean text output only
4. **External rotation**: Use logrotate or similar

### Error Recovery
1. **Idempotent operations**: Safe to retry commands
2. **Graceful degradation**: Continue on non-critical failures
3. **Clear error messages**: Actionable feedback for users

## Troubleshooting

### Common Issues

#### Session Not Found
```bash
# Check if session exists
tmux has-session -t proj:demo

# List all sessions
tmux list-sessions

# Create session manually
tmux new-session -d -s proj:demo
```

#### Window Not Found
```bash
# List windows in session
tmux list-windows -t proj:demo

# Check window name format
tmux list-windows -t proj:demo -F "#W"
```

#### Logging Not Working
```bash
# Check if pipe-pane is active
tmux list-panes -t proj:demo:backend:api-server -F "#{pane_id} #{pane_pipe}"

# Manually activate logging
tmux pipe-pane -ot proj:demo:backend:api-server 'cat >> ./logs/demo/backend.ndjson'
```

#### Provider Not Starting
```bash
# Test provider command directly
gemini chat --model 2.0

# Check provider availability
multi-agents doctor

# Verify system prompt
echo "You are a backend developer..." | gemini chat --model 2.0
```

### Debug Commands
```bash
# Verbose tmux output
tmux -v new-session -d -s proj:demo

# Check tmux version
tmux -V

# List all tmux processes
ps aux | grep tmux

# Check log file permissions
ls -la ./logs/demo/backend.ndjson
```

## Integration with Multi-Agents CLI

### Command Mapping
- `multi-agents agent run` → Session/window creation + REPL startup
- `multi-agents agent attach` → `tmux attach -t proj:{project}`
- `multi-agents agent stop` → `tmux kill-window -t proj:{project}:{role}:{agent}`
- `multi-agents broadcast` → `tmux send-keys` to multiple windows

### Configuration Integration
- **Timeouts**: From `config/defaults.yaml`
- **Provider commands**: From `config/providers.yaml`
- **Agent roles**: From `config/project.yaml`
- **Log paths**: Computed from project/role

### Database Integration
- **Session tracking**: Store tmux session state in SQLite
- **Agent metadata**: Link tmux windows to database records
- **Log correlation**: Connect NDJSON events to database sessions

See [`docs/cli-reference.md`](cli-reference.md) for complete command reference and [`docs/workflows.md`](workflows.md) for usage patterns.
