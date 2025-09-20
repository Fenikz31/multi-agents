# Multi-Agents Broadcast — Practical Examples

This document provides real-world examples and use cases for the Multi-Agents Broadcast functionality. These examples demonstrate how to effectively use broadcast commands in various scenarios.

## Table of Contents

- [Development Workflows](#development-workflows)
- [DevOps Operations](#devops-operations)
- [Project Management](#project-management)
- [Incident Response](#incident-response)
- [Code Review Process](#code-review-process)
- [Testing Coordination](#testing-coordination)
- [Documentation Updates](#documentation-updates)
- [Scripts and Automation](#scripts-and-automation)

## Development Workflows

### Daily Standup Automation

**Scenario:** Automate daily standup coordination with your development team.

```bash
#!/bin/bash
# daily-standup.sh

PROJECT="my-project"
DATE=$(date +%Y-%m-%d)

echo "Starting daily standup for $DATE..."

# Send standup reminder
multi-agents broadcast oneshot \
  --project "$PROJECT" \
  --to "@role:developer" \
  --message "Daily standup in 15 minutes! Please prepare your updates:
  
  📅 Date: $DATE
  ⏰ Time: 9:00 AM
  
  Please share:
  ✅ What you completed yesterday
  🎯 What you're working on today
  🚫 Any blockers or issues
  📊 Overall progress (1-10)
  
  Meeting link: https://meet.company.com/standup" \
  --format text \
  --progress

echo "Standup reminder sent to all developers!"
```

### Feature Branch Coordination

**Scenario:** Coordinate feature development across multiple agents.

```bash
#!/bin/bash
# feature-coordination.sh

PROJECT="my-project"
FEATURE="user-dashboard"
BRANCH="feature/$FEATURE"

echo "Starting feature development: $FEATURE"

# 1. Assign tasks to different agents
multi-agents broadcast oneshot \
  --project "$PROJECT" \
  --to "backend-dev" \
  --message "🎯 Backend Task for $FEATURE:
  
  - Create API endpoints for dashboard data
  - Implement user preferences storage
  - Add authentication middleware
  - Branch: $BRANCH
  - Deadline: End of week
  
  Requirements doc: ./docs/features/$FEATURE.md" \
  --format text

multi-agents broadcast oneshot \
  --project "$PROJECT" \
  --to "frontend-dev" \
  --message "🎯 Frontend Task for $FEATURE:
  
  - Design dashboard UI components
  - Implement data visualization
  - Add responsive design
  - Branch: $BRANCH
  - Deadline: End of week
  
  Design mockups: ./designs/$FEATURE.fig" \
  --format text

multi-agents broadcast oneshot \
  --project "$PROJECT" \
  --to "devops" \
  --message "🎯 DevOps Task for $FEATURE:
  
  - Set up staging environment
  - Configure monitoring for new endpoints
  - Prepare deployment pipeline
  - Branch: $BRANCH
  - Deadline: End of week" \
  --format text

echo "Feature tasks assigned successfully!"
```

### Code Review Automation

**Scenario:** Automate code review requests and follow-ups.

```bash
#!/bin/bash
# code-review.sh

PROJECT="my-project"
PR_NUMBER="$1"
AUTHOR="$2"
BRANCH="$3"

if [ -z "$PR_NUMBER" ] || [ -z "$AUTHOR" ] || [ -z "$BRANCH" ]; then
  echo "Usage: $0 <PR_NUMBER> <AUTHOR> <BRANCH>"
  exit 1
fi

echo "Setting up code review for PR #$PR_NUMBER..."

# Request review from senior developers
multi-agents broadcast oneshot \
  --project "$PROJECT" \
  --to "senior-dev-1,senior-dev-2,tech-lead" \
  --message "🔍 Code Review Request
  
  PR: #$PR_NUMBER
  Author: $AUTHOR
  Branch: $BRANCH
  Link: https://github.com/company/repo/pull/$PR_NUMBER
  
  Please review:
  - Code quality and best practices
  - Security considerations
  - Performance implications
  - Test coverage
  
  Priority: High
  Deadline: 24 hours
  
  Thanks! 🙏" \
  --format text

# Notify author about review request
multi-agents broadcast oneshot \
  --project "$PROJECT" \
  --to "$AUTHOR" \
  --message "✅ Code review requested for PR #$PR_NUMBER
  
  Reviewers: senior-dev-1, senior-dev-2, tech-lead
  Expected completion: 24 hours
  
  You'll be notified when reviews are complete!" \
  --format text

echo "Code review process initiated!"
```

## DevOps Operations

### Deployment Coordination

**Scenario:** Coordinate deployment activities across environments.

```bash
#!/bin/bash
# deployment-coord.sh

PROJECT="my-project"
VERSION="$1"
ENVIRONMENT="$2"

if [ -z "$VERSION" ] || [ -z "$ENVIRONMENT" ]; then
  echo "Usage: $0 <VERSION> <ENVIRONMENT>"
  exit 1
fi

echo "Starting deployment coordination for v$VERSION to $ENVIRONMENT..."

# Notify all teams about deployment
multi-agents broadcast oneshot \
  --project "$PROJECT" \
  --to "@all" \
  --message "🚀 DEPLOYMENT NOTIFICATION
  
  Version: v$VERSION
  Environment: $ENVIRONMENT
  Scheduled: $(date)
  Duration: ~30 minutes
  
  Impact:
  - Brief service interruption expected
  - New features: User dashboard, API improvements
  - Bug fixes: Authentication issues, performance
  
  Please:
  - Monitor your services
  - Report any issues immediately
  - Update status in #deployment channel
  
  Rollback plan: Available if needed" \
  --format text

# Start deployment process
echo "Deployment started..."

# Monitor deployment progress
multi-agents broadcast oneshot \
  --project "$PROJECT" \
  --to "@role:devops" \
  --message "Deployment monitoring for v$VERSION:
  
  Please check:
  - Service health status
  - Error rates
  - Performance metrics
  - Database connectivity
  
  Report status every 5 minutes" \
  --format text

echo "Deployment coordination complete!"
```

### Infrastructure Monitoring

**Scenario:** Set up automated infrastructure monitoring alerts.

```bash
#!/bin/bash
# infrastructure-monitor.sh

PROJECT="my-project"

echo "Setting up infrastructure monitoring..."

# Create monitoring alerts
multi-agents broadcast oneshot \
  --project "$PROJECT" \
  --to "@role:devops" \
  --message "🔍 Infrastructure Monitoring Setup
  
  New monitoring rules activated:
  
  🚨 Critical Alerts:
  - CPU usage > 80% for 5 minutes
  - Memory usage > 90% for 3 minutes
  - Disk space < 10% available
  - Database connection failures
  
  ⚠️ Warning Alerts:
  - Response time > 2 seconds
  - Error rate > 1%
  - Queue depth > 1000 items
  
  📊 Monitoring Dashboard:
  - URL: https://monitoring.company.com
  - Credentials: Check password manager
  - Refresh rate: 30 seconds
  
  Please verify all systems are green!" \
  --format text

echo "Infrastructure monitoring configured!"
```

## Project Management

### Sprint Planning

**Scenario:** Coordinate sprint planning activities.

```bash
#!/bin/bash
# sprint-planning.sh

PROJECT="my-project"
SPRINT_NUMBER="$1"
SPRINT_GOALS="$2"

if [ -z "$SPRINT_NUMBER" ] || [ -z "$SPRINT_GOALS" ]; then
  echo "Usage: $0 <SPRINT_NUMBER> <SPRINT_GOALS>"
  exit 1
fi

echo "Starting sprint $SPRINT_NUMBER planning..."

# Send sprint planning invitation
multi-agents broadcast oneshot \
  --project "$PROJECT" \
  --to "@all" \
  --message "📋 Sprint $SPRINT_NUMBER Planning
  
  🎯 Sprint Goals:
  $SPRINT_GOALS
  
  📅 Planning Session:
  - Date: Tomorrow, 10:00 AM
  - Duration: 2 hours
  - Location: Conference Room A
  - Remote: https://meet.company.com/sprint-planning
  
  📝 Preparation Required:
  - Review backlog items
  - Estimate story points
  - Identify dependencies
  - Prepare questions
  
  📊 Sprint Metrics:
  - Velocity target: 40 story points
  - Capacity: 5 developers
  - Duration: 2 weeks
  
  See you there! 🚀" \
  --format text

echo "Sprint planning invitation sent!"
```

### Retrospective Coordination

**Scenario:** Organize sprint retrospective meetings.

```bash
#!/bin/bash
# retrospective.sh

PROJECT="my-project"
SPRINT_NUMBER="$1"

if [ -z "$SPRINT_NUMBER" ]; then
  echo "Usage: $0 <SPRINT_NUMBER>"
  exit 1
fi

echo "Organizing sprint $SPRINT_NUMBER retrospective..."

# Send retrospective invitation
multi-agents broadcast oneshot \
  --project "$PROJECT" \
  --to "@role:developer" \
  --message "🔄 Sprint $SPRINT_NUMBER Retrospective
  
  📅 Meeting Details:
  - Date: Friday, 3:00 PM
  - Duration: 1 hour
  - Location: Conference Room B
  - Remote: https://meet.company.com/retrospective
  
  🎯 Discussion Topics:
  - What went well?
  - What could be improved?
  - Action items for next sprint
  - Process improvements
  
  📊 Sprint Results:
  - Stories completed: 8/10
  - Bugs fixed: 15
  - Velocity: 38 story points
  - Team satisfaction: 8/10
  
  Please come prepared with your thoughts! 💭" \
  --format text

echo "Retrospective invitation sent!"
```

## Incident Response

### Production Incident Alert

**Scenario:** Handle production incidents with coordinated response.

```bash
#!/bin/bash
# incident-alert.sh

PROJECT="my-project"
SEVERITY="$1"
DESCRIPTION="$2"

if [ -z "$SEVERITY" ] || [ -z "$DESCRIPTION" ]; then
  echo "Usage: $0 <SEVERITY> <DESCRIPTION>"
  exit 1
fi

echo "Alerting teams about $SEVERITY incident..."

# Send incident alert
multi-agents broadcast oneshot \
  --project "$PROJECT" \
  --to "@all" \
  --message "🚨 PRODUCTION INCIDENT ALERT
  
  Severity: $SEVERITY
  Description: $DESCRIPTION
  Time: $(date)
  Incident ID: INC-$(date +%Y%m%d%H%M%S)
  
  Impact:
  - User login failures
  - API response timeouts
  - Database connection issues
  
  Response Team:
  - Incident Commander: @tech-lead
  - Backend: @backend-dev
  - Frontend: @frontend-dev
  - DevOps: @devops
  
  Actions Required:
  - Check service status
  - Review recent deployments
  - Monitor error logs
  - Update #incident-response channel
  
  War Room: https://meet.company.com/incident-response
  
  Stay calm and follow the runbook! 📋" \
  --format text

echo "Incident alert sent to all teams!"
```

### Post-Incident Review

**Scenario:** Organize post-incident review meetings.

```bash
#!/bin/bash
# post-incident-review.sh

PROJECT="my-project"
INCIDENT_ID="$1"

if [ -z "$INCIDENT_ID" ]; then
  echo "Usage: $0 <INCIDENT_ID>"
  exit 1
fi

echo "Organizing post-incident review for $INCIDENT_ID..."

# Send review invitation
multi-agents broadcast oneshot \
  --project "$PROJECT" \
  --to "@all" \
  --message "📋 Post-Incident Review: $INCIDENT_ID
  
  📅 Meeting Details:
  - Date: Tomorrow, 2:00 PM
  - Duration: 1.5 hours
  - Location: Conference Room C
  - Remote: https://meet.company.com/post-incident
  
  🎯 Review Objectives:
  - Timeline reconstruction
  - Root cause analysis
  - Impact assessment
  - Prevention measures
  
  📊 Incident Summary:
  - Duration: 2 hours 15 minutes
  - Users affected: 1,200
  - Revenue impact: $5,000
  - Root cause: Database connection pool exhaustion
  
  📝 Preparation:
  - Review incident timeline
  - Prepare your observations
  - Think about improvements
  - Bring relevant logs
  
  Let's learn and improve! 🚀" \
  --format text

echo "Post-incident review invitation sent!"
```

## Code Review Process

### Automated Review Assignment

**Scenario:** Automate code review assignments based on file changes.

```bash
#!/bin/bash
# auto-review-assignment.sh

PROJECT="my-project"
PR_NUMBER="$1"
CHANGED_FILES="$2"

if [ -z "$PR_NUMBER" ] || [ -z "$CHANGED_FILES" ]; then
  echo "Usage: $0 <PR_NUMBER> <CHANGED_FILES>"
  exit 1
fi

echo "Assigning reviewers for PR #$PR_NUMBER..."

# Analyze changed files and assign reviewers
if echo "$CHANGED_FILES" | grep -q "backend/"; then
  BACKEND_REVIEWERS="backend-dev,senior-backend-dev"
else
  BACKEND_REVIEWERS=""
fi

if echo "$CHANGED_FILES" | grep -q "frontend/"; then
  FRONTEND_REVIEWERS="frontend-dev,senior-frontend-dev"
else
  FRONTEND_REVIEWERS=""
fi

if echo "$CHANGED_FILES" | grep -q "infrastructure/"; then
  DEVOPS_REVIEWERS="devops,senior-devops"
else
  DEVOPS_REVIEWERS=""
fi

# Assign reviewers
REVIEWERS="$BACKEND_REVIEWERS,$FRONTEND_REVIEWERS,$DEVOPS_REVIEWERS"
REVIEWERS=$(echo "$REVIEWERS" | sed 's/,,/,/g' | sed 's/^,//' | sed 's/,$//')

if [ -n "$REVIEWERS" ]; then
  multi-agents broadcast oneshot \
    --project "$PROJECT" \
    --to "$REVIEWERS" \
    --message "🔍 Code Review Assignment
  
    PR: #$PR_NUMBER
    Changed Files: $CHANGED_FILES
    
    You've been assigned as a reviewer based on the files you modified.
    
    Please review:
    - Code quality and best practices
    - Security considerations
    - Performance implications
    - Test coverage
    
    Deadline: 48 hours
    Link: https://github.com/company/repo/pull/$PR_NUMBER
    
    Thanks for your help! 🙏" \
    --format text

  echo "Reviewers assigned: $REVIEWERS"
else
  echo "No specific reviewers found for changed files"
fi
```

## Testing Coordination

### Test Environment Setup

**Scenario:** Coordinate testing activities across different environments.

```bash
#!/bin/bash
# test-coordination.sh

PROJECT="my-project"
TEST_PHASE="$1"
FEATURE="$2"

if [ -z "$TEST_PHASE" ] || [ -z "$FEATURE" ]; then
  echo "Usage: $0 <TEST_PHASE> <FEATURE>"
  exit 1
fi

echo "Coordinating $TEST_PHASE testing for $FEATURE..."

# Phase 1: Unit Testing
if [ "$TEST_PHASE" = "unit" ]; then
  multi-agents broadcast oneshot \
    --project "$PROJECT" \
    --to "@role:developer" \
    --message "🧪 Unit Testing Phase: $FEATURE
    
    Please run unit tests for your components:
    
    Backend:
    - API endpoint tests
    - Business logic tests
    - Database integration tests
    - Command: npm run test:unit
    
    Frontend:
    - Component tests
    - Hook tests
    - Utility function tests
    - Command: npm run test:unit
    
    Deadline: End of day
    Report results in #testing channel" \
    --format text

# Phase 2: Integration Testing
elif [ "$TEST_PHASE" = "integration" ]; then
  multi-agents broadcast oneshot \
    --project "$PROJECT" \
    --to "@role:developer" \
    --message "🔗 Integration Testing Phase: $FEATURE
    
    Please run integration tests:
    
    Backend:
    - API integration tests
    - Database integration tests
    - External service tests
    - Command: npm run test:integration
    
    Frontend:
    - Component integration tests
    - API integration tests
    - User flow tests
    - Command: npm run test:integration
    
    Environment: Staging
    Deadline: Tomorrow EOD
    Report results in #testing channel" \
    --format text

# Phase 3: End-to-End Testing
elif [ "$TEST_PHASE" = "e2e" ]; then
  multi-agents broadcast oneshot \
    --project "$PROJECT" \
    --to "@role:developer" \
    --message "🎯 End-to-End Testing Phase: $FEATURE
    
    Please run E2E tests:
    
    Test Scenarios:
    - User registration flow
    - Feature functionality
    - Error handling
    - Performance testing
    
    Environment: Staging
    Tools: Playwright, Cypress
    Command: npm run test:e2e
    
    Deadline: 2 days
    Report results in #testing channel" \
    --format text
fi

echo "Testing coordination complete!"
```

## Documentation Updates

### Documentation Review

**Scenario:** Coordinate documentation updates and reviews.

```bash
#!/bin/bash
# doc-review.sh

PROJECT="my-project"
DOC_TYPE="$1"
DOC_PATH="$2"

if [ -z "$DOC_TYPE" ] || [ -z "$DOC_PATH" ]; then
  echo "Usage: $0 <DOC_TYPE> <DOC_PATH>"
  exit 1
fi

echo "Coordinating documentation review for $DOC_TYPE..."

# Send documentation review request
multi-agents broadcast oneshot \
  --project "$PROJECT" \
  --to "@role:developer" \
  --message "📚 Documentation Review Request
  
  Type: $DOC_TYPE
  Path: $DOC_PATH
  Author: $(git config user.name)
  
  Please review:
  - Technical accuracy
  - Clarity and completeness
  - Code examples
  - Screenshots and diagrams
  
  Review Guidelines:
  - Check for typos and grammar
  - Verify code examples work
  - Ensure screenshots are up-to-date
  - Suggest improvements
  
  Deadline: 3 days
  Submit feedback via PR comments
  
  Thanks for helping improve our docs! 📖" \
  --format text

echo "Documentation review request sent!"
```

## Scripts and Automation

### Daily Status Report

**Scenario:** Generate daily status reports from all team members.

```bash
#!/bin/bash
# daily-status-report.sh

PROJECT="my-project"
DATE=$(date +%Y-%m-%d)
REPORT_FILE="status-report-$DATE.json"

echo "Generating daily status report for $DATE..."

# Collect status from all team members
multi-agents broadcast oneshot \
  --project "$PROJECT" \
  --to "@all" \
  --message "📊 Daily Status Report - $DATE
  
  Please provide your status update:
  
  1. Tasks completed yesterday
  2. Tasks planned for today
  3. Any blockers or issues
  4. Overall project health (1-10)
  5. Additional notes
  
  Format: JSON
  Deadline: 9:00 AM
  
  Your response will be included in the daily report." \
  --format json > "$REPORT_FILE"

# Process and format the report
echo "Processing status report..."

# Send summary to management
multi-agents broadcast oneshot \
  --project "$PROJECT" \
  --to "tech-lead,project-manager" \
  --message "📈 Daily Status Summary - $DATE
  
  Report generated: $REPORT_FILE
  Team members: $(jq '.results | length' "$REPORT_FILE")
  Average health score: $(jq '.results | map(.response | fromjson.health) | add / length' "$REPORT_FILE")
  
  Full report available at: ./reports/$REPORT_FILE" \
  --format text

echo "Daily status report complete: $REPORT_FILE"
```

### Weekly Retrospective

**Scenario:** Automate weekly retrospective data collection.

```bash
#!/bin/bash
# weekly-retrospective.sh

PROJECT="my-project"
WEEK=$(date +%Y-W%U)

echo "Collecting weekly retrospective data for week $WEEK..."

# Collect retrospective feedback
multi-agents broadcast oneshot \
  --project "$PROJECT" \
  --to "@role:developer" \
  --message "🔄 Weekly Retrospective - Week $WEEK
  
  Please share your feedback:
  
  What went well this week?
  What could be improved?
  What should we start doing?
  What should we stop doing?
  What should we continue doing?
  
  Additional comments or suggestions?
  
  Your feedback is anonymous and will help improve our team processes.
  
  Deadline: Friday 5:00 PM" \
  --format json > "retrospective-$WEEK.json"

echo "Weekly retrospective data collected: retrospective-$WEEK.json"
```

## Advanced Examples

### Conditional Broadcasting

**Scenario:** Send different messages based on team roles or conditions.

```bash
#!/bin/bash
# conditional-broadcast.sh

PROJECT="my-project"
EVENT_TYPE="$1"

case "$EVENT_TYPE" in
  "deployment")
    multi-agents broadcast oneshot \
      --project "$PROJECT" \
      --to "@role:devops" \
      --message "🚀 Deployment Event
  
      Please monitor:
      - Service health
      - Error rates
      - Performance metrics
      
      Report any issues immediately!" \
      --format text
    ;;
  
  "code-review")
    multi-agents broadcast oneshot \
      --project "$PROJECT" \
      --to "@role:developer" \
      --message "🔍 Code Review Event
      
      New PRs available for review:
      - Check your assigned reviews
      - Provide feedback within 24 hours
      - Follow code review guidelines" \
      --format text
    ;;
  
  "incident")
    multi-agents broadcast oneshot \
      --project "$PROJECT" \
      --to "@all" \
      --message "🚨 Incident Event
      
      Production issue detected:
      - Check service status
      - Review recent changes
      - Report to incident channel
      - Follow incident response procedures" \
      --format text
    ;;
  
  *)
    echo "Usage: $0 <deployment|code-review|incident>"
    exit 1
    ;;
esac
```

### Scheduled Broadcasting

**Scenario:** Use cron jobs for scheduled broadcasts.

```bash
#!/bin/bash
# scheduled-broadcast.sh

PROJECT="my-project"
HOUR=$(date +%H)

case "$HOUR" in
  "09")
    # Morning standup reminder
    multi-agents broadcast oneshot \
      --project "$PROJECT" \
      --to "@role:developer" \
      --message "☀️ Good morning! Standup in 15 minutes at 9:15 AM" \
      --format text
    ;;
  
  "17")
    # End of day check
    multi-agents broadcast oneshot \
      --project "$PROJECT" \
      --to "@role:developer" \
      --message "🌅 End of day check:
      
      Please update:
      - Task status
      - Tomorrow's priorities
      - Any blockers
      
      Have a great evening! 🌙" \
      --format text
    ;;
  
  "18")
    # Weekend reminder
    if [ "$(date +%u)" -eq 5 ]; then
      multi-agents broadcast oneshot \
        --project "$PROJECT" \
        --to "@all" \
        --message "🎉 Happy Friday!
        
        Weekend reminder:
        - Commit your changes
        - Update documentation
        - Plan next week's tasks
        
        Have a great weekend! 🏖️" \
        --format text
    fi
    ;;
esac
```

---

*These examples demonstrate the flexibility and power of Multi-Agents Broadcast. Adapt them to your specific needs and workflows!*
