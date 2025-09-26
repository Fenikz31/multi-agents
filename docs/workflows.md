# Multi-Agents Workflows

## Overview

This document describes the key workflows and usage patterns for Multi-Agents CLI. For detailed command reference, see [`docs/cli-reference.md`](cli-reference.md). For tmux integration details, see [`docs/tmux.md`](tmux.md).

## First-Run Flow

### Initial Setup

When running critical commands (`agent`, `send`, `tui`) for the first time, the CLI provides guided setup:

#### 1. Environment Validation
```bash
# Check if environment is ready
multi-agents doctor

# Expected output:
# ✅ gemini CLI found (version 2.0)
# ✅ claude CLI found (version 3.0)  
# ✅ cursor-agent CLI found (version 1.0)
# ✅ tmux found (version 3.3)
# ✅ git found (version 2.40)
# ✅ All providers available
```

#### 2. Configuration Setup
```bash
# Create initial configuration files
multi-agents config init

# Expected output:
# Created ./config/project.yaml
# Created ./config/providers.yaml
# 
# Next steps:
# 1. Edit ./config/project.yaml to define your projects and agents
# 2. Edit ./config/providers.yaml to configure provider settings
# 3. Run 'multi-agents config validate' to check your configuration
```

#### 3. Database Initialization
```bash
# Initialize SQLite database
multi-agents db init

# Expected output:
# Database initialized at ./data/multi-agents.sqlite3
# Tables created: projects, agents, sessions, messages, tasks, broadcasts
```

#### 4. Project and Agent Creation
```bash
# Create a new project
multi-agents project add --name demo

# Add agents to the project
multi-agents agent add --project demo --name backend --role backend --provider gemini --model 2.0
multi-agents agent add --project demo --name frontend --role frontend --provider claude --model opus
multi-agents agent add --project demo --name devops --role devops --provider cursor --model gpt-4
```

#### 5. Start Agent REPL
```bash
# Start backend agent
multi-agents agent run --project demo --agent backend

# Expected output:
# Starting agent backend in tmux session proj:demo
# Window created: backend:backend
# Logging to: ./logs/demo/backend.ndjson
# Agent started successfully
```

#### 6. Launch TUI (Optional)
```bash
# Open terminal interface
multi-agents tui --project demo
```

### Configuration Files

#### `config/project.yaml`
```yaml
projects:
  - name: demo
    description: "Demo project for testing"
    agents:
      - name: backend
        role: backend
        provider: gemini
        model: 2.0
        system_prompt: "You are a backend developer specializing in API design and database optimization."
        allowed_tools: ["code", "database", "api"]
      - name: frontend
        role: frontend
        provider: claude
        model: opus
        system_prompt: "You are a frontend developer focused on React and modern web technologies."
        allowed_tools: ["code", "ui", "css"]
```

#### `config/providers.yaml`
```yaml
providers:
  gemini:
    command: "gemini"
    flags:
      model: "--model"
      system_prompt: "--system-prompt"
    default_model: "2.0"
  claude:
    command: "claude"
    flags:
      model: "--model"
      system_prompt: "--system-prompt"
    default_model: "opus"
  cursor:
    command: "cursor-agent"
    flags:
      model: "--model"
      system_prompt: "--system-prompt"
    default_model: "gpt-4"
```

## Core Workflows

### One-Shot User Send

#### Basic Flow
1. **Resolve target**: `@role` → agent → provider; load allowlist from `project.yaml`
2. **Compose command**: Build provider command using placeholders from `providers.yaml`
3. **Enforce concurrency**: Maximum 3 one-shot executions (FIFO queue with global semaphore)
4. **Execute with timeout**: 120s default, stream stdout to user and log NDJSON per line
5. **Show progress**: Display spinner by default (`--no-progress` to disable)
6. **Handle sessions**: 
   - If `--to <conversation_id>`, reuse existing session (validate; fallback to creation if expired/invalid)
   - Otherwise, auto-create session (Claude/Gemini ID valid; Cursor via `create-chat`)
   - Update `last_activity` and record `provider_session_id` when available
7. **Cursor headless**: Force `--output-format stream-json`, parse `assistant.message.content[].text` deltas and terminate on `result` event

#### Examples
```bash
# Send to specific agent
multi-agents send --to backend --message "Implement user authentication with JWT tokens"

# Send to all agents in a role
multi-agents send --to @backend --message "Database schema has been updated, please review"

# Send to all agents
multi-agents send --to @all --message "Starting deployment to production"

# Send to existing session
multi-agents send --to conv_1234567890abcdef --message "Continue with the previous task"

# Custom timeout and JSON output
multi-agents send --to backend --message "Run comprehensive tests" --timeout-ms 300000 --format json
```

### Agent REPL (tmux)

#### Startup Flow
1. **Validate inputs**: Check project/agent exist in database
2. **Ensure tmux session**: `proj:{project}` exists (create if missing)
3. **Create window**: `{role}:{agent}` with provider REPL command and system prompt
4. **Activate logging**: `pipe-pane -o` to `./logs/{project}/{role}.ndjson`
5. **Emit start event**: Write NDJSON `start` event with agent/provider metadata
6. **Monitor output**: Append `stdout_line` per provider stdout line
7. **Handle termination**: Emit `end` event with `dur_ms` and status

#### Interaction Flow
1. **Send messages**: Use `tmux send-keys` to window `{role}:{agent}`
2. **Monitor logs**: Real-time NDJSON events in log files
3. **Broadcast support**: Send identical keystrokes to multiple windows
4. **Session persistence**: tmux session survives agent stops

#### Examples
```bash
# Start backend agent
multi-agents agent run --project demo --agent backend

# Attach to session to interact manually
multi-agents agent attach --project demo

# Send message to agent
multi-agents broadcast --project demo --message "Check database status" --mode repl --to @backend

# Stop agent
multi-agents agent stop --project demo --agent backend
```

### Broadcast Workflows

#### One-Shot Broadcast
- **Fan-out**: Send to all group members with concurrency=3
- **Persistence**: Shared `broadcast_id` across all messages
- **Progress tracking**: Global spinner shows overall progress
- **NDJSON compatibility**: Works with structured logging

#### REPL Broadcast
- **Target selection**: `@role|@all|agent1,agent2` to select targets
- **Keystroke sending**: Identical keystrokes to each `{role}:{agent}` window
- **Status aggregation**: Per-target status tracking with summary
- **NDJSON correlation**: Events per agent with shared `broadcast_id`

#### Examples
```bash
# Broadcast to all agents (oneshot)
multi-agents broadcast --project demo --message "Starting deployment" --mode oneshot

# Broadcast to specific role (repl)
multi-agents broadcast --project demo --message "Check status" --mode repl --to @backend

# Broadcast to specific agents (repl)
multi-agents broadcast --project demo --message "Update configuration" --mode repl --to backend,frontend

# JSON output with status
multi-agents broadcast --project demo --message "Status check" --mode oneshot --format json
```

## Advanced Workflows

### Routing and Supervisor

#### Role-Based Routing
- **`@role`**: Routes to all agents with that role
- **`@all`**: Expands to group `all` (all agents in project)
- **Specific agent**: Direct routing by agent name
- **Session ID**: Direct routing to existing conversation

#### Supervisor Pattern
- **Message subscription**: Supervisor receives all messages
- **Task re-routing**: Can redirect tasks between agents
- **Coordination**: Orchestrates complex multi-agent workflows
- **Log monitoring**: Real-time monitoring of agent activities
- **Metrics computation**: Analysis of routed events and performance
- **Event aggregation**: Multi-role log aggregation and filtering

#### Examples
```bash
# Route to all backend agents
multi-agents send --to @backend --message "Review API changes"

# Route to all agents
multi-agents send --to @all --message "Project status update"

# Route to specific agent
multi-agents send --to backend --message "Implement new endpoint"

# Route to existing session
multi-agents send --to conv_1234567890abcdef --message "Continue previous task"
```

#### Supervisor Monitoring Workflow

The supervisor provides comprehensive monitoring capabilities for multi-agent systems:

##### Real-time Log Monitoring
```bash
# Monitor routed events in real-time
# Supervisor automatically tracks all send --to @role and @all operations
multi-agents send --to @backend --message "Database migration started"
# → Generates routed event in ./logs/{project}/backend.ndjson

multi-agents send --to @all --message "System maintenance in 5 minutes"
# → Generates routed events in all role log files
```

##### Metrics Analysis
The supervisor computes comprehensive metrics for routed events:
- **Total routed events**: Count of all routing operations
- **Per-role breakdown**: Events distributed by agent role
- **Unique broadcasts**: Number of distinct broadcast operations
- **P95 latency**: 95th percentile latency per broadcast
- **Top roles**: Roles sorted by activity level

##### Log Aggregation
```bash
# Supervisor can aggregate logs from multiple roles
# Example: Monitor all backend and frontend activities
# Logs are automatically sorted by timestamp and filtered by event type
```

##### Integration with CLI Commands
The supervisor integrates seamlessly with existing CLI commands:
- All `send --to @role` and `send --to @all` commands generate routed events
- Events are stored in NDJSON format for easy parsing and analysis
- Real-time monitoring enables immediate feedback on system performance

### Git Context Integration

#### Context Collection
- **Git status**: Current repository state
- **Git diff**: Uncommitted changes
- **Git log**: Last 5 commits
- **Size limits**: Respect reasonable limits for prompt injection
- **Secret redaction**: Attempt to redact sensitive information

#### Usage
```bash
# Get Git status
multi-agents context git --status

# Get Git diff
multi-agents context git --diff

# Get Git log
multi-agents context git --log

# Use with send (future feature)
multi-agents send --to backend --message "Review these changes" --include-git
```

### Session Management

#### Session Lifecycle
1. **Creation**: Auto-created on first `send` or explicit `session start`
2. **Resumption**: Use `session resume` with conversation ID
3. **Cleanup**: Automatic cleanup of sessions >24h old
4. **Persistence**: Provider session IDs stored when available

#### Examples
```bash
# Start new session
multi-agents session start --project demo --agent backend
# Output: conversation_id=conv_1234567890abcdef

# Resume existing session
multi-agents session resume --conversation-id conv_1234567890abcdef

# List active sessions
multi-agents session list --project demo

# Clean up old sessions
multi-agents session cleanup --dry-run
```

## Error Handling and Recovery

### Common Error Scenarios

#### Configuration Missing
```bash
# Error: Config files not found
multi-agents send --to backend --message "Hello"
# Output: Configuration files missing. Run 'multi-agents config init' first.

# Solution
multi-agents config init
multi-agents config validate
```

#### Provider Unavailable
```bash
# Error: Provider CLI not found
multi-agents send --to backend --message "Hello"
# Output: Provider 'gemini' not available. Run 'multi-agents doctor' to check.

# Solution
multi-agents doctor
# Install missing provider CLI
```

#### tmux Errors
```bash
# Error: tmux session not found
multi-agents agent attach --project demo
# Output: Session 'proj:demo' not found. Run 'multi-agents agent run' first.

# Solution
multi-agents agent run --project demo --agent backend
```

### Recovery Strategies

#### Idempotent Operations
- **Safe retry**: All commands safe to run multiple times
- **No side effects**: Repeated execution doesn't cause issues
- **State consistency**: Operations maintain consistent state

#### Graceful Degradation
- **Partial failures**: Continue on non-critical failures
- **Clear feedback**: Actionable error messages
- **Fallback options**: Alternative approaches when possible

## Best Practices

### Project Organization
1. **Clear naming**: Use descriptive project and agent names
2. **Role separation**: Separate concerns by agent roles
3. **Configuration**: Keep config files in version control
4. **Documentation**: Document agent purposes and capabilities

### Session Management
1. **Regular cleanup**: Clean up old sessions periodically
2. **Session reuse**: Reuse existing sessions when possible
3. **Proper shutdown**: Stop agents cleanly when done
4. **Log monitoring**: Monitor NDJSON logs for issues

### Error Handling
1. **Check environment**: Run `doctor` regularly
2. **Validate config**: Use `config validate` before operations
3. **Monitor logs**: Watch for error patterns in logs
4. **Graceful recovery**: Use retry mechanisms appropriately

### Performance Optimization
1. **Concurrency limits**: Respect the 3-execution limit
2. **Timeout tuning**: Adjust timeouts based on use case
3. **Log rotation**: Implement log rotation for long-running projects
4. **Resource monitoring**: Monitor system resources during heavy usage

## Integration Patterns

### CI/CD Integration
```bash
# In CI pipeline
multi-agents doctor --format json > doctor-report.json
multi-agents send --to @all --message "Build completed successfully" --format json
```

### Development Workflow
```bash
# Daily development routine
multi-agents agent run --project my-app --agent backend
multi-agents agent run --project my-app --agent frontend
multi-agents tui --project my-app
```

### Monitoring and Observability
```bash
# Health check
multi-agents doctor --snapshot health-check.json

# Session monitoring
multi-agents session list --project demo --format json

# Log analysis
tail -f ./logs/demo/backend.ndjson | jq '.event'
```

See [`docs/cli-reference.md`](cli-reference.md) for complete command reference and [`docs/tmux.md`](tmux.md) for detailed tmux integration.
