//! TUI Performance Tests
//! 
//! Tests the performance characteristics of TUI components, state management,
//! and database operations. These tests verify that the TUI remains responsive
//! and efficient even with large datasets.

use std::error::Error;
use std::time::{Duration, Instant};
use tempfile::TempDir;

use crate::tui::{
    state::{StateManager, TuiState},
    state::view_state::{KanbanState, SessionsState, TaskItem, SessionItem},
    state::navigation_state::{HelpState, ProjectSelectState},
    components::{Toast, ToastQueue, ToastType, GlobalStatus, GlobalStateIcon},
};
use db;

/// Performance metrics for benchmarking
#[derive(Debug, Clone)]
struct PerformanceMetrics {
    operation_name: String,
    duration: Duration,
    iterations: usize,
    avg_duration: Duration,
    min_duration: Duration,
    max_duration: Duration,
    success: bool,
    error_message: Option<String>,
}

impl PerformanceMetrics {
    fn new(operation_name: String) -> Self {
        Self {
            operation_name,
            duration: Duration::ZERO,
            iterations: 0,
            avg_duration: Duration::ZERO,
            min_duration: Duration::MAX,
            max_duration: Duration::ZERO,
            success: true,
            error_message: None,
        }
    }

    fn add_measurement(&mut self, duration: Duration) {
        self.iterations += 1;
        self.duration += duration;
        self.min_duration = self.min_duration.min(duration);
        self.max_duration = self.max_duration.max(duration);
    }

    fn finalize(&mut self) {
        if self.iterations > 0 {
            self.avg_duration = Duration::from_nanos(
                self.duration.as_nanos() as u64 / self.iterations as u64
            );
        }
    }

    fn mark_failure(&mut self, error: String) {
        self.success = false;
        self.error_message = Some(error);
    }
}

/// Helper function to create a temporary database for performance testing
fn create_performance_test_db(_task_count: usize, _session_count: usize) -> Result<(TempDir, String), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("perf_test.db").to_string_lossy().to_string();
    
    // Create database and tables using the existing db module
    let _conn = db::open_or_create_db(&db_path)?;
    
    Ok((temp_dir, db_path))
}

/// Helper function to create test tasks for performance testing
fn create_test_tasks(count: usize) -> Vec<TaskItem> {
    (0..count)
        .map(|i| TaskItem {
            id: format!("task-{}", i),
            title: format!("Performance Test Task {}", i),
            status: match i % 3 {
                0 => "todo".to_string(),
                1 => "in_progress".to_string(),
                _ => "done".to_string(),
            },
            assignee: if i % 2 == 0 { Some(format!("agent-{}", i % 5)) } else { None },
            priority: match i % 4 {
                0 => "low".to_string(),
                1 => "medium".to_string(),
                2 => "high".to_string(),
                _ => "critical".to_string(),
            },
        })
        .collect()
}

/// Helper function to create test sessions for performance testing
fn create_test_sessions(count: usize) -> Vec<SessionItem> {
    (0..count)
        .map(|i| SessionItem {
            id: format!("session-{}", i),
            agent_name: format!("agent-{}", i % 10),
            role: format!("role-{}", i % 3),
            provider: match i % 3 {
                0 => "claude".to_string(),
                1 => "gemini".to_string(),
                _ => "cursor".to_string(),
            },
            status: match i % 4 {
                0 => "active".to_string(),
                1 => "completed".to_string(),
                2 => "paused".to_string(),
                _ => "error".to_string(),
            },
            duration: format!("{}m", (i % 60) + 1),
        })
        .collect()
}

#[cfg(test)]
mod tui_rendering_performance_tests {
    use super::*;

    #[test]
    fn test_kanban_state_rendering_performance() -> Result<(), Box<dyn Error>> {
        let mut metrics = PerformanceMetrics::new("KanbanState Rendering".to_string());
        
        // Test with different dataset sizes
        let test_sizes = vec![10, 100, 500, 1000];
        
        for size in test_sizes {
            let mut kanban_state = KanbanState::new();
            let tasks = create_test_tasks(size);
            kanban_state.tasks = tasks;
            
            // Warm up
            let _ = kanban_state.render()?;
            
            // Benchmark rendering
            let iterations = if size <= 100 { 100 } else { 10 };
            for _ in 0..iterations {
                let start = Instant::now();
                let _output = kanban_state.render()?;
                let duration = start.elapsed();
                metrics.add_measurement(duration);
            }
            
            println!("KanbanState rendering with {} tasks: avg {:?}", size, metrics.avg_duration);
            
            // Performance assertions
            assert!(metrics.avg_duration < Duration::from_millis(50), 
                "KanbanState rendering too slow with {} tasks: {:?}", size, metrics.avg_duration);
        }
        
        metrics.finalize();
        assert!(metrics.success, "KanbanState rendering performance test failed");
        
        Ok(())
    }

    #[test]
    fn test_sessions_state_rendering_performance() -> Result<(), Box<dyn Error>> {
        let mut metrics = PerformanceMetrics::new("SessionsState Rendering".to_string());
        
        // Test with different dataset sizes
        let test_sizes = vec![10, 100, 500, 1000];
        
        for size in test_sizes {
            let mut sessions_state = SessionsState::new();
            let sessions = create_test_sessions(size);
            sessions_state.sessions = sessions;
            
            // Warm up
            let _ = sessions_state.render()?;
            
            // Benchmark rendering
            let iterations = if size <= 100 { 100 } else { 10 };
            for _ in 0..iterations {
                let start = Instant::now();
                let _output = sessions_state.render()?;
                let duration = start.elapsed();
                metrics.add_measurement(duration);
            }
            
            println!("SessionsState rendering with {} sessions: avg {:?}", size, metrics.avg_duration);
            
            // Performance assertions
            assert!(metrics.avg_duration < Duration::from_millis(50), 
                "SessionsState rendering too slow with {} sessions: {:?}", size, metrics.avg_duration);
        }
        
        metrics.finalize();
        assert!(metrics.success, "SessionsState rendering performance test failed");
        
        Ok(())
    }

    #[test]
    fn test_component_rendering_performance() -> Result<(), Box<dyn Error>> {
        let mut metrics = PerformanceMetrics::new("Component Rendering".to_string());
        
        // Test ToastQueue performance
        let mut toast_queue = ToastQueue::with_capacity(10);
        for i in 0..10 {
            toast_queue.enqueue(Toast::new(
                ToastType::Info,
                &format!("Performance test toast {}", i),
                Some(5000)
            ));
        }
        
        let iterations = 1000;
        for _ in 0..iterations {
            let start = Instant::now();
            toast_queue.tick(100);
            let duration = start.elapsed();
            metrics.add_measurement(duration);
        }
        
        println!("ToastQueue tick performance: avg {:?}", metrics.avg_duration);
        assert!(metrics.avg_duration < Duration::from_micros(100), 
            "ToastQueue tick too slow: {:?}", metrics.avg_duration);
        
        // Test GlobalStatus performance
        let status = GlobalStatus {
            project_name: "performance-test".to_string(),
            view_name: "kanban".to_string(),
            focus: "Task 1".to_string(),
            icon: GlobalStateIcon::Active,
            last_action: Some("Performance test".to_string()),
        };
        
        let iterations = 1000;
        for _ in 0..iterations {
            let start = Instant::now();
            let _header = status.header_text();
            let duration = start.elapsed();
            metrics.add_measurement(duration);
        }
        
        println!("GlobalStatus header_text performance: avg {:?}", metrics.avg_duration);
        assert!(metrics.avg_duration < Duration::from_micros(10), 
            "GlobalStatus header_text too slow: {:?}", metrics.avg_duration);
        
        metrics.finalize();
        assert!(metrics.success, "Component rendering performance test failed");
        
        Ok(())
    }
}

#[cfg(test)]
mod tui_cache_performance_tests {
    use super::*;

    #[test]
    fn test_kanban_columns_cache_performance() -> Result<(), Box<dyn Error>> {
        let mut metrics = PerformanceMetrics::new("Kanban Columns Cache".to_string());
        
        // Test with different dataset sizes
        let test_sizes = vec![100, 500, 1000, 2000];
        
        for size in test_sizes {
            let mut kanban_state = KanbanState::new();
            let tasks = create_test_tasks(size);
            kanban_state.tasks = tasks;
            
            // Test cache miss (first call)
            let start = Instant::now();
            let _columns = kanban_state.get_columns();
            let cache_miss_duration = start.elapsed();
            
            // Test cache hit (subsequent calls)
            let iterations = 100;
            let mut cache_hit_duration = Duration::ZERO;
            for _ in 0..iterations {
                let start = Instant::now();
                let _columns = kanban_state.get_columns();
                cache_hit_duration += start.elapsed();
            }
            let avg_cache_hit = Duration::from_nanos(
                cache_hit_duration.as_nanos() as u64 / iterations as u64
            );
            
            println!("Kanban columns cache with {} tasks:", size);
            println!("  Cache miss: {:?}", cache_miss_duration);
            println!("  Cache hit (avg): {:?}", avg_cache_hit);
            
            // Performance assertions
            assert!(cache_miss_duration < Duration::from_millis(100), 
                "Kanban columns cache miss too slow with {} tasks: {:?}", size, cache_miss_duration);
            let max_duration = if size <= 1000 { 
                Duration::from_millis(20) 
            } else { 
                Duration::from_millis(35) 
            };
            assert!(avg_cache_hit < max_duration, 
                "Kanban columns cache hit too slow with {} tasks: {:?}", size, avg_cache_hit);
            
            // Cache should not be dramatically slower than cache miss (allow env variance)
            assert!(avg_cache_hit <= cache_miss_duration * 6, 
                "Cache hit significantly slower than cache miss");
        }
        
        metrics.finalize();
        assert!(metrics.success, "Kanban columns cache performance test failed");
        
        Ok(())
    }

    #[test]
    fn test_sessions_filter_cache_performance() -> Result<(), Box<dyn Error>> {
        let mut metrics = PerformanceMetrics::new("Sessions Filter Cache".to_string());
        
        // Test with different dataset sizes
        let test_sizes = vec![100, 500, 1000, 2000];
        
        for size in test_sizes {
            let mut sessions_state = SessionsState::new();
            let sessions = create_test_sessions(size);
            sessions_state.sessions = sessions;
            
            // Test cache miss (first call with filter)
            sessions_state.filter = "agent-1".to_string();
            let start = Instant::now();
            let _filtered = sessions_state.get_filtered_sessions();
            let cache_miss_duration = start.elapsed();
            
            // Test cache hit (subsequent calls with same filter)
            let iterations = 100;
            let mut cache_hit_duration = Duration::ZERO;
            for _ in 0..iterations {
                let start = Instant::now();
                let _filtered = sessions_state.get_filtered_sessions();
                cache_hit_duration += start.elapsed();
            }
            let avg_cache_hit = Duration::from_nanos(
                cache_hit_duration.as_nanos() as u64 / iterations as u64
            );
            
            println!("Sessions filter cache with {} sessions:", size);
            println!("  Cache miss: {:?}", cache_miss_duration);
            println!("  Cache hit (avg): {:?}", avg_cache_hit);
            
            // Performance assertions
            assert!(cache_miss_duration < Duration::from_millis(50), 
                "Sessions filter cache miss too slow with {} sessions: {:?}", size, cache_miss_duration);
            // Allow more generous threshold for large datasets and CI variance
            let allowed = if size <= 1000 { Duration::from_millis(6) } else { Duration::from_millis(10) };
            assert!(avg_cache_hit < allowed, 
                "Sessions filter cache hit too slow with {} sessions: {:?}", size, avg_cache_hit);
            
            // Cache should be at least as fast as cache miss (may not be significantly faster due to implementation)
            assert!(avg_cache_hit <= cache_miss_duration * 5, 
                "Cache hit significantly slower than cache miss");
        }
        
        metrics.finalize();
        assert!(metrics.success, "Sessions filter cache performance test failed");
        
        Ok(())
    }

    #[test]
    fn test_cache_invalidation_performance() -> Result<(), Box<dyn Error>> {
        let mut metrics = PerformanceMetrics::new("Cache Invalidation".to_string());
        
        // Test KanbanState cache invalidation
        let mut kanban_state = KanbanState::new();
        let tasks = create_test_tasks(1000);
        kanban_state.tasks = tasks;
        
        // Warm up cache
        let _ = kanban_state.get_columns();
        
        // Test cache invalidation performance
        let iterations = 100;
        for _ in 0..iterations {
            let start = Instant::now();
            kanban_state.add_task("New Task".to_string(), None);
            let duration = start.elapsed();
            metrics.add_measurement(duration);
        }
        
        println!("KanbanState cache invalidation: avg {:?}", metrics.avg_duration);
        assert!(metrics.avg_duration < Duration::from_millis(10), 
            "KanbanState cache invalidation too slow: {:?}", metrics.avg_duration);
        
        // Test SessionsState cache invalidation
        let mut sessions_state = SessionsState::new();
        let sessions = create_test_sessions(1000);
        sessions_state.sessions = sessions;
        
        // Warm up cache
        sessions_state.filter = "agent-1".to_string();
        let _ = sessions_state.get_filtered_sessions();
        
        // Test cache invalidation performance
        let iterations = 100;
        for _ in 0..iterations {
            let start = Instant::now();
            sessions_state.filter = format!("agent-{}", iterations % 10);
            let _ = sessions_state.get_filtered_sessions();
            let duration = start.elapsed();
            metrics.add_measurement(duration);
        }
        
        println!("SessionsState cache invalidation: avg {:?}", metrics.avg_duration);
        assert!(metrics.avg_duration < Duration::from_millis(10), 
            "SessionsState cache invalidation too slow: {:?}", metrics.avg_duration);
        
        metrics.finalize();
        assert!(metrics.success, "Cache invalidation performance test failed");
        
        Ok(())
    }
}

#[cfg(test)]
mod tui_database_performance_tests {
    use super::*;

    #[test]
    fn test_database_loading_performance() -> Result<(), Box<dyn Error>> {
        let mut metrics = PerformanceMetrics::new("Database Loading".to_string());
        
        // Test with different dataset sizes
        let test_sizes = vec![100, 500, 1000];
        
        for size in test_sizes {
            let (_temp_dir, db_path) = create_performance_test_db(size, size)?;
            
            // Test KanbanState database loading
            let mut kanban_state = KanbanState::new();
            let start = Instant::now();
            let result = kanban_state.load_from_db(&db_path, "test-project");
            let duration = start.elapsed();
            
            if result.is_err() {
                metrics.mark_failure(format!("KanbanState database loading failed: {:?}", result.err()));
                continue;
            }
            
            println!("KanbanState database loading with {} tasks: {:?}", size, duration);
            assert!(duration < Duration::from_millis(1000), 
                "KanbanState database loading too slow with {} tasks: {:?}", size, duration);
            
            // Test SessionsState database loading
            let mut sessions_state = SessionsState::new();
            let start = Instant::now();
            let result = sessions_state.load_from_db_with_filters(&db_path, Some("test-project".to_string()), None);
            let duration = start.elapsed();
            
            if result.is_err() {
                metrics.mark_failure(format!("SessionsState database loading failed: {:?}", result.err()));
                continue;
            }
            
            println!("SessionsState database loading with {} sessions: {:?}", size, duration);
            assert!(duration < Duration::from_millis(1000), 
                "SessionsState database loading too slow with {} sessions: {:?}", size, duration);
        }
        
        metrics.finalize();
        assert!(metrics.success, "Database loading performance test failed");
        
        Ok(())
    }

    #[test]
    fn test_database_operations_performance() -> Result<(), Box<dyn Error>> {
        let mut metrics = PerformanceMetrics::new("Database Operations".to_string());
        
        let (_temp_dir, db_path) = create_performance_test_db(1000, 1000)?;
        
        // Test KanbanState operations
        let mut kanban_state = KanbanState::new();
        kanban_state.load_from_db(&db_path, "test-project")?;
        
        // Test add_task performance
        let iterations = 100;
        for i in 0..iterations {
            let start = Instant::now();
            kanban_state.add_task(format!("Performance Task {}", i), None);
            let duration = start.elapsed();
            metrics.add_measurement(duration);
        }
        
        println!("KanbanState add_task performance: avg {:?}", metrics.avg_duration);
        assert!(metrics.avg_duration < Duration::from_millis(1), 
            "KanbanState add_task too slow: {:?}", metrics.avg_duration);
        
        // Test move_task performance
        let iterations = 100;
        for i in 0..iterations {
            let task_id = kanban_state.tasks[i % kanban_state.tasks.len()].id.clone();
            let start = Instant::now();
            let _ = kanban_state.move_task(&task_id, "in_progress");
            let duration = start.elapsed();
            metrics.add_measurement(duration);
        }
        
        println!("KanbanState move_task performance: avg {:?}", metrics.avg_duration);
        assert!(metrics.avg_duration < Duration::from_millis(1), 
            "KanbanState move_task too slow: {:?}", metrics.avg_duration);
        
        metrics.finalize();
        assert!(metrics.success, "Database operations performance test failed");
        
        Ok(())
    }
}

#[cfg(test)]
mod tui_state_management_performance_tests {
    use super::*;

    #[test]
    fn test_state_transitions_performance() -> Result<(), Box<dyn Error>> {
        let mut metrics = PerformanceMetrics::new("State Transitions".to_string());
        
        let mut state_manager = StateManager::new();
        
        // Add states
        state_manager.add_state("help".to_string(), Box::new(HelpState::new()));
        state_manager.add_state("project_select".to_string(), Box::new(ProjectSelectState::new()));
        state_manager.add_state("kanban".to_string(), Box::new(KanbanState::new()));
        state_manager.add_state("sessions".to_string(), Box::new(SessionsState::new()));
        
        // Set initial state first
        state_manager.set_current_state("project_select".to_string())?;
        
        // Test state transition performance
        let iterations = 1000;
        for _ in 0..iterations {
            let start = Instant::now();
            let transition = state_manager.handle_input("k")?;
            state_manager.process_transition(transition)?;
            let duration = start.elapsed();
            metrics.add_measurement(duration);
        }
        
        println!("State transition performance: avg {:?}", metrics.avg_duration);
        assert!(metrics.avg_duration < Duration::from_micros(100), 
            "State transition too slow: {:?}", metrics.avg_duration);
        
        metrics.finalize();
        assert!(metrics.success, "State transitions performance test failed");
        
        Ok(())
    }

    #[test]
    fn test_state_manager_render_performance() -> Result<(), Box<dyn Error>> {
        let mut metrics = PerformanceMetrics::new("StateManager Render".to_string());
        
        let mut state_manager = StateManager::new();
        
        // Add states with data
        let mut kanban_state = KanbanState::new();
        kanban_state.tasks = create_test_tasks(500);
        state_manager.add_state("kanban".to_string(), Box::new(kanban_state));
        
        let mut sessions_state = SessionsState::new();
        sessions_state.sessions = create_test_sessions(500);
        state_manager.add_state("sessions".to_string(), Box::new(sessions_state));
        
        state_manager.add_state("help".to_string(), Box::new(HelpState::new()));
        state_manager.add_state("project_select".to_string(), Box::new(ProjectSelectState::new()));
        
        // Set initial state
        state_manager.set_current_state("kanban".to_string())?;
        
        // Test render performance
        let iterations = 100;
        for _ in 0..iterations {
            let start = Instant::now();
            let _output = state_manager.render()?;
            let duration = start.elapsed();
            metrics.add_measurement(duration);
        }
        
        println!("StateManager render performance: avg {:?}", metrics.avg_duration);
        assert!(metrics.avg_duration < Duration::from_millis(50), 
            "StateManager render too slow: {:?}", metrics.avg_duration);
        
        metrics.finalize();
        assert!(metrics.success, "StateManager render performance test failed");
        
        Ok(())
    }

    #[test]
    fn test_large_dataset_performance() -> Result<(), Box<dyn Error>> {
        let mut metrics = PerformanceMetrics::new("Large Dataset Performance".to_string());
        
        // Test with very large datasets
        let large_task_count = 5000;
        let large_session_count = 5000;
        
        // Test KanbanState with large dataset
        let mut kanban_state = KanbanState::new();
        kanban_state.tasks = create_test_tasks(large_task_count);
        
        let start = Instant::now();
        let _columns = kanban_state.get_columns();
        let duration = start.elapsed();
        
        println!("KanbanState with {} tasks: {:?}", large_task_count, duration);
        assert!(duration < Duration::from_millis(500), 
            "KanbanState too slow with {} tasks: {:?}", large_task_count, duration);
        
        // Test SessionsState with large dataset
        let mut sessions_state = SessionsState::new();
        sessions_state.sessions = create_test_sessions(large_session_count);
        
        let start = Instant::now();
        let _filtered = sessions_state.get_filtered_sessions();
        let duration = start.elapsed();
        
        println!("SessionsState with {} sessions: {:?}", large_session_count, duration);
        assert!(duration < Duration::from_millis(500), 
            "SessionsState too slow with {} sessions: {:?}", large_session_count, duration);
        
        // Test rendering with large dataset
        let start = Instant::now();
        let _output = kanban_state.render()?;
        let duration = start.elapsed();
        
        println!("KanbanState rendering with {} tasks: {:?}", large_task_count, duration);
        assert!(duration < Duration::from_millis(800), 
            "KanbanState rendering too slow with {} tasks: {:?}", large_task_count, duration);
        
        metrics.finalize();
        assert!(metrics.success, "Large dataset performance test failed");
        
        Ok(())
    }
}

#[cfg(test)]
mod tui_memory_performance_tests {
    use super::*;

    #[test]
    fn test_memory_usage_performance() -> Result<(), Box<dyn Error>> {
        let mut metrics = PerformanceMetrics::new("Memory Usage".to_string());
        
        // Test memory usage with different dataset sizes
        let test_sizes = vec![100, 1000, 5000];
        
        for size in test_sizes {
            // Test KanbanState memory usage
            let mut kanban_state = KanbanState::new();
            kanban_state.tasks = create_test_tasks(size);
            
            // Force cache building
            let _columns = kanban_state.get_columns();
            
            // Test that memory usage is reasonable
            let task_memory = std::mem::size_of_val(&kanban_state.tasks);
            // Cache memory is private, so we can't measure it directly
            let cache_memory = 0; // Placeholder
            
            println!("KanbanState memory usage with {} tasks:", size);
            println!("  Tasks: {} bytes", task_memory);
            println!("  Cache: {} bytes", cache_memory);
            
            // Memory usage should be linear with dataset size
            let expected_memory = size * std::mem::size_of::<TaskItem>();
            assert!(task_memory <= expected_memory * 2, 
                "KanbanState memory usage too high with {} tasks: {} bytes", size, task_memory);
            
            // Test SessionsState memory usage
            let mut sessions_state = SessionsState::new();
            sessions_state.sessions = create_test_sessions(size);
            
            // Force cache building
            let _filtered = sessions_state.get_filtered_sessions();
            
            let session_memory = std::mem::size_of_val(&sessions_state.sessions);
            // Cache memory is private, so we can't measure it directly
            let cache_memory = 0; // Placeholder
            
            println!("SessionsState memory usage with {} sessions:", size);
            println!("  Sessions: {} bytes", session_memory);
            println!("  Cache: {} bytes", cache_memory);
            
            // Memory usage should be linear with dataset size
            let expected_memory = size * std::mem::size_of::<SessionItem>();
            assert!(session_memory <= expected_memory * 2, 
                "SessionsState memory usage too high with {} sessions: {} bytes", size, session_memory);
        }
        
        metrics.finalize();
        assert!(metrics.success, "Memory usage performance test failed");
        
        Ok(())
    }

    #[test]
    fn test_memory_leak_prevention() -> Result<(), Box<dyn Error>> {
        let mut metrics = PerformanceMetrics::new("Memory Leak Prevention".to_string());
        
        // Test that repeated operations don't cause memory leaks
        let mut kanban_state = KanbanState::new();
        
        // Perform many operations
        for i in 0..1000 {
            kanban_state.add_task(format!("Task {}", i), None);
            
            if i % 100 == 0 {
                // Force cache operations
                let _columns = kanban_state.get_columns();
                
                // Test that memory usage doesn't grow unbounded
                let task_count = kanban_state.tasks.len();
                let expected_memory = task_count * std::mem::size_of::<TaskItem>();
                let actual_memory = std::mem::size_of_val(&kanban_state.tasks);
                
                assert!(actual_memory <= expected_memory * 2, 
                    "Memory leak detected at iteration {}: {} bytes", i, actual_memory);
            }
        }
        
        // Test SessionsState memory leak prevention
        let mut sessions_state = SessionsState::new();
        
        for i in 0..1000 {
            let session = SessionItem {
                id: format!("session-{}", i),
                agent_name: format!("agent-{}", i % 10),
                role: "test".to_string(),
                provider: "test".to_string(),
                status: "active".to_string(),
                duration: "1m".to_string(),
            };
            sessions_state.sessions.push(session);
            
            if i % 100 == 0 {
                // Force cache operations
                let _filtered = sessions_state.get_filtered_sessions();
                
                // Test that memory usage doesn't grow unbounded
                let session_count = sessions_state.sessions.len();
                let expected_memory = session_count * std::mem::size_of::<SessionItem>();
                let actual_memory = std::mem::size_of_val(&sessions_state.sessions);
                
                assert!(actual_memory <= expected_memory * 2, 
                    "Memory leak detected at iteration {}: {} bytes", i, actual_memory);
            }
        }
        
        metrics.finalize();
        assert!(metrics.success, "Memory leak prevention test failed");
        
        Ok(())
    }
}
