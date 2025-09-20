# Multi-Agents Broadcast — User Guide

## Overview

The Multi-Agents Broadcast feature allows you to send messages to multiple agents simultaneously, either as one-shot commands or through tmux REPL sessions. This guide covers everything you need to know to effectively use broadcast functionality.

## Table of Contents

- [Quick Start](#quick-start)
- [Broadcast Modes](#broadcast-modes)
- [Target Selection](#target-selection)
- [Command Reference](#command-reference)
- [Common Use Cases](#common-use-cases)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)
- [FAQ](#faq)

## Quick Start

### Prerequisites

Before using broadcast, ensure you have:
- Multi-Agents CLI installed and configured
- At least one project with agents defined
- Providers (Claude, Gemini, Cursor Agent) available

### Basic Setup

1. **Check your environment:**
```bash
multi-agents doctor
```

2. **Initialize your project and agents:**
```bash
# Create a project
multi-agents project add --name "my-project"

# Add agents to your project
multi-agents agent add --project "my-project" --name "backend-dev" --role "developer" --provider "claude" --model "claude-3-5-sonnet-20241022"
multi-agents agent add --project "my-project" --name "frontend-dev" --role "developer" --provider "gemini" --model "gemini-1.5-pro"
multi-agents agent add --project "my-project" --name "devops" --role "devops" --provider "cursor-agent" --model "gpt-4o"
```

3. **Test broadcast functionality:**
```bash
# Send a message to all agents
multi-agents broadcast oneshot --project "my-project" --to "@all" --message "Hello team! Ready for today's work?"
```

## Broadcast Modes

### Oneshot Mode

Oneshot mode sends a single message to multiple agents and returns their responses immediately. This is ideal for:
- Quick questions or requests
- Status checks
- One-time commands
- When you need immediate responses

**Basic syntax:**
```bash
multi-agents broadcast oneshot --project <project> --to <targets> --message "<message>"
```

**Example:**
```bash
multi-agents broadcast oneshot \
  --project "my-project" \
  --to "@all" \
  --message "What's the status of your current task?" \
  --format json
```

### REPL Mode

REPL mode sends messages to agents running in tmux sessions. This is ideal for:
- Interactive conversations
- Long-running tasks
- When you want to maintain context
- Collaborative debugging sessions

**Basic syntax:**
```bash
multi-agents broadcast repl --project <project> --to <targets> --message "<message>"
```

**Example:**
```bash
# First, start agents in REPL mode
multi-agents agent run --project "my-project" --agent "backend-dev"
multi-agents agent run --project "my-project" --agent "frontend-dev"

# Then broadcast to them
multi-agents broadcast repl \
  --project "my-project" \
  --to "@all" \
  --message "Let's review the code changes together"
```

## Target Selection

Broadcast supports flexible target selection to send messages to specific agents or groups.

### Target Types

#### All Agents (`@all`)
Send to all agents in the project:
```bash
multi-agents broadcast oneshot --project "my-project" --to "@all" --message "Team standup in 10 minutes"
```

#### By Role (`@role:<role>`)
Send to all agents with a specific role:
```bash
# Send to all developers
multi-agents broadcast oneshot --project "my-project" --to "@role:developer" --message "Code review needed"

# Send to all devops agents
multi-agents broadcast oneshot --project "my-project" --to "@role:devops" --message "Deployment pipeline status?"
```

#### Specific Agents
Send to specific agents by name:
```bash
# Single agent
multi-agents broadcast oneshot --project "my-project" --to "backend-dev" --message "Check database connection"

# Multiple agents (comma-separated)
multi-agents broadcast oneshot --project "my-project" --to "backend-dev,frontend-dev" --message "Let's sync on the API changes"
```

#### Mixed Targets
Combine different target types:
```bash
multi-agents broadcast oneshot --project "my-project" --to "@role:developer,devops" --message "All hands meeting"
```

## Command Reference

### Common Options

| Option | Description | Default | Example |
|--------|-------------|---------|---------|
| `--project` | Project name (required) | - | `--project "my-project"` |
| `--to` | Target agents (required) | - | `--to "@all"` |
| `--message` | Message to send (required) | - | `--message "Hello team"` |
| `--timeout-ms` | Timeout in milliseconds | 5000 | `--timeout-ms 10000` |
| `--format` | Output format | text | `--format json` |
| `--progress` | Show progress bar | false | `--progress` |

### Advanced Options

| Option | Description | Default | Example |
|--------|-------------|---------|---------|
| `--project-file` | Custom project config | `./config/project.yaml` | `--project-file "./my-project.yaml"` |
| `--providers-file` | Custom providers config | `./config/providers.yaml` | `--providers-file "./my-providers.yaml"` |

### Output Formats

#### Text Format (Default)
```bash
multi-agents broadcast oneshot --project "my-project" --to "@all" --message "Status check"
```

Output:
```
✅ backend-dev: Database connection stable, all services running
✅ frontend-dev: UI components updated, tests passing
✅ devops: Infrastructure healthy, monitoring green
```

#### JSON Format
```bash
multi-agents broadcast oneshot --project "my-project" --to "@all" --message "Status check" --format json
```

Output:
```json
{
  "broadcast_id": "broadcast_1234567890",
  "project": "my-project",
  "message": "Status check",
  "results": [
    {
      "agent": "backend-dev",
      "success": true,
      "response": "Database connection stable, all services running",
      "duration_ms": 1200
    },
    {
      "agent": "frontend-dev", 
      "success": true,
      "response": "UI components updated, tests passing",
      "duration_ms": 800
    }
  ],
  "summary": {
    "total_agents": 3,
    "successful": 3,
    "failed": 0,
    "total_duration_ms": 2000
  }
}
```

## Common Use Cases

### 1. Daily Standup Coordination

**Scenario:** Coordinate daily standup with your development team.

```bash
# Send standup reminder
multi-agents broadcast oneshot \
  --project "my-project" \
  --to "@role:developer" \
  --message "Daily standup in 15 minutes. Please prepare your updates on:
  - What you completed yesterday
  - What you're working on today  
  - Any blockers or issues" \
  --format text
```

### 2. Code Review Requests

**Scenario:** Request code reviews from specific team members.

```bash
# Request review from senior developers
multi-agents broadcast oneshot \
  --project "my-project" \
  --to "senior-dev-1,senior-dev-2" \
  --message "Code review needed for PR #123: User authentication refactor
  - Branch: feature/auth-refactor
  - Files changed: auth.js, middleware.js, routes.js
  - Please review by EOD" \
  --format text
```

### 3. Incident Response

**Scenario:** Coordinate incident response across teams.

```bash
# Alert all teams about production issue
multi-agents broadcast oneshot \
  --project "my-project" \
  --to "@all" \
  --message "🚨 PRODUCTION INCIDENT 🚨
  - Issue: Database connection timeouts
  - Impact: User login failures
  - Status: Investigating
  - War room: #incident-response
  - Please check your services and report status" \
  --format text
```

### 4. Feature Planning Session

**Scenario:** Conduct collaborative feature planning.

```bash
# Start agents in REPL mode for interactive session
multi-agents agent run --project "my-project" --agent "backend-dev"
multi-agents agent run --project "my-project" --agent "frontend-dev"
multi-agents agent run --project "my-project" --agent "product-manager"

# Start planning discussion
multi-agents broadcast repl \
  --project "my-project" \
  --to "@all" \
  --message "Let's plan the new user dashboard feature. 
  I'll share the requirements document and we can discuss implementation approach."
```

### 5. Automated Status Reports

**Scenario:** Generate automated status reports from all agents.

```bash
# Create a script for daily status reports
#!/bin/bash
DATE=$(date +%Y-%m-%d)
multi-agents broadcast oneshot \
  --project "my-project" \
  --to "@all" \
  --message "Daily status report for $DATE:
  Please provide:
  1. Tasks completed today
  2. Tasks planned for tomorrow
  3. Any blockers or concerns
  4. Overall project health (1-10)" \
  --format json > "status-report-$DATE.json"
```

### 6. Testing Coordination

**Scenario:** Coordinate testing activities across different environments.

```bash
# Coordinate testing phases
multi-agents broadcast oneshot \
  --project "my-project" \
  --to "@role:developer" \
  --message "Testing phase 1 starting:
  - Unit tests: Run locally
  - Integration tests: Staging environment
  - Report results in 2 hours
  - Use #testing channel for updates" \
  --format text
```

## Best Practices

### 1. Message Clarity

**✅ Good:**
```bash
multi-agents broadcast oneshot \
  --project "my-project" \
  --to "@role:developer" \
  --message "Code review needed for PR #123: User authentication refactor
  - Branch: feature/auth-refactor
  - Files: auth.js, middleware.js, routes.js
  - Deadline: EOD today
  - Contact: @john-doe for questions"
```

**❌ Avoid:**
```bash
multi-agents broadcast oneshot \
  --project "my-project" \
  --to "@role:developer" \
  --message "Need review"
```

### 2. Target Selection

**✅ Use specific targets when possible:**
```bash
# Specific agents for specific tasks
multi-agents broadcast oneshot --project "my-project" --to "backend-dev,frontend-dev" --message "API integration discussion"
```

**✅ Use role-based targeting for team-wide messages:**
```bash
# All developers for development-related messages
multi-agents broadcast oneshot --project "my-project" --to "@role:developer" --message "New coding standards document"
```

### 3. Timeout Management

**✅ Set appropriate timeouts:**
```bash
# Quick status check - short timeout
multi-agents broadcast oneshot --project "my-project" --to "@all" --message "Status?" --timeout-ms 3000

# Complex analysis - longer timeout
multi-agents broadcast oneshot --project "my-project" --to "@all" --message "Analyze performance bottlenecks" --timeout-ms 30000
```

### 4. Output Format Selection

**✅ Use text format for human reading:**
```bash
multi-agents broadcast oneshot --project "my-project" --to "@all" --message "Status check" --format text
```

**✅ Use JSON format for automation:**
```bash
multi-agents broadcast oneshot --project "my-project" --to "@all" --message "Status check" --format json | jq '.results[].response'
```

### 5. Progress Monitoring

**✅ Use progress bars for long operations:**
```bash
multi-agents broadcast oneshot --project "my-project" --to "@all" --message "Complex analysis" --progress
```

### 6. Error Handling

**✅ Check exit codes in scripts:**
```bash
#!/bin/bash
if multi-agents broadcast oneshot --project "my-project" --to "@all" --message "Status check"; then
  echo "All agents responded successfully"
else
  echo "Some agents failed to respond"
  exit 1
fi
```

## Troubleshooting

### Common Issues

#### 1. "Project not found" Error

**Problem:** `Error: Project 'my-project' not found`

**Solution:**
```bash
# Check available projects
multi-agents project list

# Create the project if it doesn't exist
multi-agents project add --name "my-project"
```

#### 2. "No agents found" Error

**Problem:** `Error: No agents found for target '@all'`

**Solution:**
```bash
# Check available agents
multi-agents agent list --project "my-project"

# Add agents to the project
multi-agents agent add --project "my-project" --name "agent1" --role "developer" --provider "claude" --model "claude-3-5-sonnet-20241022"
```

#### 3. "Provider unavailable" Error

**Problem:** `Error: Provider 'claude' unavailable`

**Solution:**
```bash
# Check provider availability
multi-agents doctor

# Install missing providers or check configuration
# See provider-specific installation guides
```

#### 4. Timeout Errors

**Problem:** `Error: Timeout after 5000ms`

**Solution:**
```bash
# Increase timeout for complex operations
multi-agents broadcast oneshot --project "my-project" --to "@all" --message "Complex task" --timeout-ms 30000
```

#### 5. tmux Session Errors (REPL Mode)

**Problem:** `Error: tmux session not found`

**Solution:**
```bash
# Start agents in REPL mode first
multi-agents agent run --project "my-project" --agent "agent1"

# Then broadcast to them
multi-agents broadcast repl --project "my-project" --to "agent1" --message "Hello"
```

### Debug Mode

Enable verbose output for debugging:
```bash
# Set environment variable for verbose logging
export RUST_LOG=debug
multi-agents broadcast oneshot --project "my-project" --to "@all" --message "Test"
```

### Performance Issues

If broadcast operations are slow:

1. **Check concurrency limits:**
   - Default: 3 concurrent operations
   - Adjust in project configuration if needed

2. **Optimize target selection:**
   - Use specific agents instead of `@all` when possible
   - Use role-based targeting for efficiency

3. **Monitor resource usage:**
   - Check system resources during broadcast
   - Consider reducing timeout values

## FAQ

### Q: What's the difference between oneshot and repl modes?

**A:** Oneshot mode sends a single message and returns responses immediately, while REPL mode sends messages to agents running in tmux sessions for interactive conversations.

### Q: Can I broadcast to agents across different projects?

**A:** No, broadcast is limited to agents within a single project. Use multiple broadcast commands for different projects.

### Q: What's the maximum number of agents I can broadcast to?

**A:** There's no hard limit, but performance may degrade with many agents. The system handles up to 3 concurrent operations by default.

### Q: Can I use broadcast in scripts and automation?

**A:** Yes! Use JSON format for programmatic processing and check exit codes for error handling.

### Q: How do I handle agents that don't respond?

**A:** The system will report which agents failed. Check the error messages and ensure agents are properly configured and available.

### Q: Can I schedule broadcast messages?

**A:** Yes, use cron jobs or task schedulers to run broadcast commands at specific times.

### Q: Is there a way to see broadcast history?

**A:** Yes, check the NDJSON logs in `./logs/<project>/<role>.ndjson` for detailed broadcast history.

### Q: Can I customize the broadcast timeout per agent?

**A:** Currently, timeout is global per broadcast operation. Use the `--timeout-ms` option to set the timeout for all agents.

### Q: What happens if a provider is temporarily unavailable?

**A:** The system will report the error for that specific agent but continue with other agents. Check the error messages for details.

### Q: Can I use broadcast with custom provider configurations?

**A:** Yes, use the `--providers-file` option to specify custom provider configurations.

## Getting Help

- **Documentation:** See [CLI Reference](cli-reference.md) for complete command documentation
- **Configuration:** See [Configuration Guide](configuration.md) for setup details
- **Workflows:** See [Workflows Guide](workflows.md) for usage patterns
- **Issues:** Report bugs and feature requests in the project repository

---

*This guide covers the essential aspects of Multi-Agents Broadcast. For advanced usage and configuration options, refer to the [CLI Reference](cli-reference.md) and [Configuration Guide](configuration.md).*
