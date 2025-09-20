//! File-based locking utilities for agent concurrency control

use std::fs;
use std::time::{Duration, Instant};
use crate::utils::errors::exit_with;

/// File-based lock for agent operations
pub struct AgentLock {
    lock_file: String,
    _lock_guard: Option<fs::File>,
}

impl AgentLock {
    /// Create a new agent lock
    pub fn new(project: &str, agent: &str) -> Self {
        let lock_dir = format!("./locks/{}", project);
        let _ = fs::create_dir_all(&lock_dir);
        let lock_file = format!("{}/{}.lock", lock_dir, agent);
        
        Self {
            lock_file,
            _lock_guard: None,
        }
    }
    
    /// Acquire the lock with timeout
    pub fn acquire(&mut self, timeout: Duration) -> Result<(), Box<dyn std::error::Error>> {
        let start = Instant::now();
        
        while start.elapsed() < timeout {
            match fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&self.lock_file)
            {
                Ok(file) => {
                    // Write PID and timestamp to lock file
                    let pid = std::process::id();
                    let timestamp = chrono::Utc::now().to_rfc3339();
                    let lock_info = format!("pid={}\ntimestamp={}\n", pid, timestamp);
                    let _ = file.set_len(0);
                    let mut file = file;
                    let _ = std::io::Write::write_all(&mut file, lock_info.as_bytes());
                    
                    self._lock_guard = Some(file);
                    return Ok(());
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    // Lock exists, check if it's stale
                    if self.is_stale_lock()? {
                        self.force_release()?;
                        continue;
                    }
                    
                    // Wait a bit before retrying
                    std::thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Err(e) => {
                    return Err(format!("Failed to create lock file '{}': {}", self.lock_file, e).into());
                }
            }
        }
        
        Err(format!("Failed to acquire lock '{}' within timeout", self.lock_file).into())
    }
    
    /// Check if the lock is stale (process no longer exists)
    fn is_stale_lock(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let lock_content = fs::read_to_string(&self.lock_file)?;
        
        for line in lock_content.lines() {
            if line.starts_with("pid=") {
                let pid_str = &line[4..];
                if let Ok(pid) = pid_str.parse::<u32>() {
                    // Check if process exists
                    return Ok(!is_process_running(pid));
                }
            }
        }
        
        // If we can't parse PID, consider it stale
        Ok(true)
    }
    
    /// Force release a stale lock
    fn force_release(&self) -> Result<(), Box<dyn std::error::Error>> {
        fs::remove_file(&self.lock_file)?;
        Ok(())
    }
    
    /// Release the lock
    pub fn release(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(_) = self._lock_guard.take() {
            fs::remove_file(&self.lock_file)?;
        }
        Ok(())
    }
}

impl Drop for AgentLock {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

/// Check if a process is running (Unix-specific)
fn is_process_running(pid: u32) -> bool {
    // On Unix systems, kill with signal 0 checks if process exists
    unsafe {
        libc::kill(pid as i32, 0) == 0
    }
}

/// Execute a function with agent lock
pub fn with_agent_lock<F, R>(
    project: &str, 
    agent: &str, 
    timeout: Duration, 
    f: F
) -> Result<R, Box<dyn std::error::Error>>
where
    F: FnOnce() -> Result<R, Box<dyn std::error::Error>>,
{
    let mut lock = AgentLock::new(project, agent);
    
    match lock.acquire(timeout) {
        Ok(_) => {
            let result = f();
            let _ = lock.release();
            result
        }
        Err(e) => {
            let _ = exit_with::<()>(8, format!("Failed to acquire agent lock for {}/{}: {}. Another agent operation may be in progress.", project, agent, e));
            unreachable!()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    use std::time::Duration;
    
    #[test]
    fn test_agent_lock_basic() {
        let temp_dir = TempDir::new().unwrap();
        let lock_file = temp_dir.path().join("test.lock");
        
        // Test lock acquisition
        let mut lock = AgentLock {
            lock_file: lock_file.to_string_lossy().to_string(),
            _lock_guard: None,
        };
        
        assert!(lock.acquire(Duration::from_secs(1)).is_ok());
        assert!(lock.release().is_ok());
    }
    
    #[test]
    fn test_agent_lock_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let lock_file = temp_dir.path().join("test.lock");
        
        // Create a lock file manually with current PID to simulate active lock
        let current_pid = std::process::id();
        fs::write(&lock_file, format!("pid={}\ntimestamp=2023-01-01T00:00:00Z\n", current_pid)).unwrap();
        
        let mut lock = AgentLock {
            lock_file: lock_file.to_string_lossy().to_string(),
            _lock_guard: None,
        };
        
        // Should timeout since lock exists and is held by current process
        let result = lock.acquire(Duration::from_millis(100));
        assert!(result.is_err());
    }
    
    #[test]
    fn test_with_agent_lock() {
        let result = with_agent_lock("test", "agent", Duration::from_secs(1), || {
            Ok::<i32, Box<dyn std::error::Error>>(42)
        });
        
        assert_eq!(result.unwrap(), 42);
    }
}
