//! Broadcast target resolution and management


/// Broadcast target types
#[derive(Debug, Clone, PartialEq)]
pub enum BroadcastTarget {
    /// All agents in project
    All,
    /// All agents with specific role
    Role(String),
    /// Specific agent
    Agent(String),
    /// Comma-separated list of agents
    AgentList(Vec<String>),
}

impl BroadcastTarget {
    /// Parse broadcast target from string
    pub fn from_str(target: &str) -> Result<Self, String> {
        match target {
            "@all" => Ok(BroadcastTarget::All),
            target if target.starts_with("@") => {
                let role = &target[1..];
                if role.is_empty() {
                    Err("Invalid role target: @".to_string())
                } else {
                    Ok(BroadcastTarget::Role(role.to_string()))
                }
            }
            target if target.contains(",") => {
                let agents: Vec<String> = target.split(",")
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if agents.is_empty() {
                    Err("Empty agent list".to_string())
                } else {
                    Ok(BroadcastTarget::AgentList(agents))
                }
            }
            target => Ok(BroadcastTarget::Agent(target.to_string())),
        }
    }
    
    /// Resolve target to list of agent names
    pub fn resolve_agents(&self, project_agents: &[db::Agent]) -> Result<Vec<String>, String> {
        match self {
            BroadcastTarget::All => {
                Ok(project_agents.iter().map(|a| a.name.clone()).collect())
            }
            BroadcastTarget::Role(role) => {
                let agents: Vec<String> = project_agents.iter()
                    .filter(|a| a.role == *role)
                    .map(|a| a.name.clone())
                    .collect();
                if agents.is_empty() {
                    Err(format!("No agents found with role '{}'", role))
                } else {
                    Ok(agents)
                }
            }
            BroadcastTarget::Agent(agent) => {
                if project_agents.iter().any(|a| a.name == *agent) {
                    Ok(vec![agent.clone()])
                } else {
                    Err(format!("Agent '{}' not found in project", agent))
                }
            }
            BroadcastTarget::AgentList(agents) => {
                let mut valid_agents = Vec::new();
                let mut invalid_agents = Vec::new();
                
                for agent in agents {
                    if project_agents.iter().any(|a| a.name == *agent) {
                        valid_agents.push(agent.clone());
                    } else {
                        invalid_agents.push(agent.clone());
                    }
                }
                
                if !invalid_agents.is_empty() {
                    Err(format!("Invalid agents: {}", invalid_agents.join(", ")))
                } else {
                    Ok(valid_agents)
                }
            }
        }
    }
}

/// Broadcast result for a single target
#[derive(Debug, Clone)]
pub struct BroadcastResult {
    pub target: String,
    pub success: bool,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// Aggregate broadcast results
#[derive(Debug, Clone)]
pub struct BroadcastSummary {
    pub total_targets: usize,
    pub successful: usize,
    pub failed: usize,
    pub results: Vec<BroadcastResult>,
    pub broadcast_id: String,
    pub total_duration_ms: u64,
}

impl BroadcastSummary {
    /// Create new broadcast summary
    pub fn new(broadcast_id: String) -> Self {
        Self {
            total_targets: 0,
            successful: 0,
            failed: 0,
            results: Vec::new(),
            broadcast_id,
            total_duration_ms: 0,
        }
    }
    
    /// Add result to summary
    pub fn add_result(&mut self, result: BroadcastResult) {
        self.total_targets += 1;
        if result.success {
            self.successful += 1;
        } else {
            self.failed += 1;
        }
        self.results.push(result);
    }
    
    /// Check if all targets succeeded
    pub fn is_success(&self) -> bool {
        self.failed == 0
    }
    
    /// Get overall status
    pub fn status(&self) -> &'static str {
        if self.is_success() {
            "success"
        } else if self.successful > 0 {
            "partial"
        } else {
            "failed"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use db::Agent;
    
    #[test]
    fn test_broadcast_target_parsing() {
        assert_eq!(BroadcastTarget::from_str("@all").unwrap(), BroadcastTarget::All);
        assert_eq!(BroadcastTarget::from_str("@backend").unwrap(), BroadcastTarget::Role("backend".to_string()));
        assert_eq!(BroadcastTarget::from_str("agent1").unwrap(), BroadcastTarget::Agent("agent1".to_string()));
        assert_eq!(BroadcastTarget::from_str("agent1,agent2").unwrap(), BroadcastTarget::AgentList(vec!["agent1".to_string(), "agent2".to_string()]));
    }
    
    #[test]
    fn test_broadcast_target_resolution() {
        let agents = vec![
            Agent {
                id: "1".to_string(),
                project_id: "1".to_string(),
                name: "backend1".to_string(),
                role: "backend".to_string(),
                provider: "gemini".to_string(),
                model: "2.0".to_string(),
                system_prompt: "".to_string(),
                allowed_tools: vec![],
            },
            Agent {
                id: "2".to_string(),
                project_id: "1".to_string(),
                name: "frontend1".to_string(),
                role: "frontend".to_string(),
                provider: "claude".to_string(),
                model: "opus".to_string(),
                system_prompt: "".to_string(),
                allowed_tools: vec![],
            },
        ];
        
        assert_eq!(BroadcastTarget::All.resolve_agents(&agents).unwrap(), vec!["backend1", "frontend1"]);
        assert_eq!(BroadcastTarget::Role("backend".to_string()).resolve_agents(&agents).unwrap(), vec!["backend1"]);
        assert_eq!(BroadcastTarget::Agent("backend1".to_string()).resolve_agents(&agents).unwrap(), vec!["backend1"]);
        assert_eq!(BroadcastTarget::AgentList(vec!["backend1".to_string(), "frontend1".to_string()]).resolve_agents(&agents).unwrap(), vec!["backend1", "frontend1"]);
    }
    
    #[test]
    fn test_broadcast_summary() {
        let mut summary = BroadcastSummary::new("test-123".to_string());
        
        summary.add_result(BroadcastResult {
            target: "agent1".to_string(),
            success: true,
            error: None,
            duration_ms: 100,
        });
        
        summary.add_result(BroadcastResult {
            target: "agent2".to_string(),
            success: false,
            error: Some("timeout".to_string()),
            duration_ms: 200,
        });
        
        assert_eq!(summary.total_targets, 2);
        assert_eq!(summary.successful, 1);
        assert_eq!(summary.failed, 1);
        assert!(!summary.is_success());
        assert_eq!(summary.status(), "partial");
    }
}
