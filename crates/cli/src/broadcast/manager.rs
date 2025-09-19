//! Broadcast manager for multi-target operations

use std::time::{Duration, Instant};
use uuid::Uuid;
use crate::tmux::manager::TmuxManager;
use crate::logging::emit_metrics_event;
use super::targets::{BroadcastTarget, BroadcastResult, BroadcastSummary};

/// Broadcast manager for handling multi-target operations
pub struct BroadcastManager {
    tmux_manager: TmuxManager,
    project_name: String,
    broadcast_id: String,
}

impl BroadcastManager {
    /// Create new broadcast manager
    pub fn new(project_name: String, timeout: Duration) -> Self {
        Self {
            tmux_manager: TmuxManager::new(timeout),
            project_name,
            broadcast_id: Uuid::new_v4().to_string(),
        }
    }
    
    /// Execute broadcast to multiple targets
    pub fn broadcast_to_targets(
        &self,
        targets: &[String],
        message: &str,
        mode: BroadcastMode,
    ) -> Result<BroadcastSummary, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        let mut summary = BroadcastSummary::new(self.broadcast_id.clone());
        
        for target in targets {
            let target_start = Instant::now();
            let result = match mode {
                BroadcastMode::Oneshot => self.broadcast_oneshot(target, message),
                BroadcastMode::Repl => self.broadcast_repl(target, message),
            };
            
            let duration_ms = target_start.elapsed().as_millis() as u64;
            
            match result {
                Ok(_) => {
                    summary.add_result(BroadcastResult {
                        target: target.clone(),
                        success: true,
                        error: None,
                        duration_ms,
                    });
                }
                Err(e) => {
                    summary.add_result(BroadcastResult {
                        target: target.clone(),
                        success: false,
                        error: Some(e.to_string()),
                        duration_ms,
                    });
                }
            }
        }
        
        summary.total_duration_ms = start_time.elapsed().as_millis() as u64;
        
        // Emit metrics for broadcast operation
        if let Err(e) = emit_metrics_event(
            &self.project_name,
            "broadcast",
            "manager",
            "multi-agents",
            "broadcast",
            summary.total_duration_ms,
            summary.status(),
            Some(&format!("targets={}, successful={}, failed={}", 
                         summary.total_targets, summary.successful, summary.failed))
        ) {
            eprintln!("Warning: Failed to emit broadcast metrics: {}", e);
        }
        
        Ok(summary)
    }
    
    /// Broadcast in oneshot mode (spawn new processes)
    fn broadcast_oneshot(&self, target: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        // For M5, this would spawn new provider processes
        // For now, we'll use REPL mode as a placeholder
        self.broadcast_repl(target, message)
    }
    
    /// Broadcast in REPL mode (send keys to existing tmux windows)
    fn broadcast_repl(&self, target: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Parse target to get role and agent
        let parts: Vec<&str> = target.split(':').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid target format: {}", target).into());
        }
        
        let role = parts[0];
        let agent = parts[1];
        let session_name = format!("proj:{}", self.project_name);
        let window_name = format!("{}:{}", role, agent);
        
        // Send keys to tmux window
        self.tmux_manager.send_keys(&session_name, &window_name, message)?;
        
        Ok(())
    }
    
    /// Get broadcast ID
    pub fn broadcast_id(&self) -> &str {
        &self.broadcast_id
    }
}

/// Broadcast mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BroadcastMode {
    /// One-shot mode: spawn new processes
    Oneshot,
    /// REPL mode: send keys to existing tmux windows
    Repl,
}

impl BroadcastMode {
    /// Parse broadcast mode from string
    pub fn from_str(mode: &str) -> Result<Self, String> {
        match mode {
            "oneshot" => Ok(BroadcastMode::Oneshot),
            "repl" => Ok(BroadcastMode::Repl),
            _ => Err(format!("Invalid broadcast mode: {}", mode)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[test]
    fn test_broadcast_mode_parsing() {
        assert_eq!(BroadcastMode::from_str("oneshot").unwrap(), BroadcastMode::Oneshot);
        assert_eq!(BroadcastMode::from_str("repl").unwrap(), BroadcastMode::Repl);
        assert!(BroadcastMode::from_str("invalid").is_err());
    }
    
    #[test]
    fn test_broadcast_manager_creation() {
        let manager = BroadcastManager::new("test".to_string(), Duration::from_secs(5));
        assert_eq!(manager.project_name, "test");
        assert!(!manager.broadcast_id().is_empty());
    }
}
