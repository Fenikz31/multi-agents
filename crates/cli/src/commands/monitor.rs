//! Monitor command for broadcast operations

use crate::monitoring::*;
use crate::logging::ndjson::emit_metrics_event;

/// Monitor broadcast operations and display metrics
pub fn run_monitor(
    project: &str,
    duration_seconds: Option<u64>,
    format: &str,
    output_file: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let duration = duration_seconds.unwrap_or(60); // Default 60 seconds
    
    // Initialize monitoring components
    let performance_monitor = PerformanceMonitor::new(project.to_string());
    let error_tracker = ErrorTracker::new(project.to_string());
    let mut resource_monitor = ResourceMonitor::new(project.to_string());
    let mut alert_manager = AlertManager::new(project.to_string());
    let mut dashboard = BroadcastDashboard::new(project.to_string());
    
    // Set up default alert rules and channels
    alert_manager.create_default_rules();
    alert_manager.create_default_channels();
    
    println!("Starting broadcast monitoring for project: {}", project);
    println!("Duration: {} seconds", duration);
    println!("Format: {}", format);
    
    let start_time = std::time::Instant::now();
    let mut iteration = 0;
    
    while start_time.elapsed().as_secs() < duration {
        iteration += 1;
        
        // Update resource metrics
        if let Err(e) = resource_monitor.update_metrics() {
            eprintln!("Warning: Failed to update resource metrics: {}", e);
        }
        
        // Get current performance status
        let performance_status = performance_monitor.get_current_status();
        
        // Generate error analysis (simplified)
        let error_analysis = error_tracker.generate_analysis_report();
        
        // Evaluate alerts
        let alert_result = alert_manager.evaluate_alerts(
            &performance_status,
            &error_analysis,
            &[]
        );
        
        // Update dashboard
        dashboard.update_performance_data(
            performance_status.clone(),
            Vec::new(), // Recent broadcasts would be passed here
            PerformanceMetrics::default()
        );
        
        // Display current status
        if iteration % 10 == 0 { // Every 10 iterations
            display_monitoring_status(
                &performance_status,
                &resource_monitor,
                &alert_result,
                iteration
            );
        }
        
        // Emit monitoring metrics
        let _ = emit_metrics_event(
            project,
            "monitor",
            "system",
            "system",
            "monitoring_iteration",
            0,
            "completed",
            Some(&format!("Iteration: {}, Active broadcasts: {}", iteration, performance_status.active_broadcasts))
        );
        
        // Sleep for 1 second
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    
    // Generate final report
    let final_report = dashboard.generate_dashboard_report();
    
    // Output results
    match format {
        "json" => {
            let json_output = serde_json::to_string_pretty(&final_report)?;
            if let Some(file_path) = output_file {
                std::fs::write(file_path, json_output)?;
                println!("Report saved to: {}", file_path);
            } else {
                println!("{}", json_output);
            }
        }
        "text" => {
            display_final_report(&final_report);
        }
        _ => {
            return Err("Unsupported format. Use 'json' or 'text'".into());
        }
    }
    
    Ok(())
}

/// Display current monitoring status
fn display_monitoring_status(
    performance_status: &PerformanceStatus,
    resource_monitor: &ResourceMonitor,
    alert_result: &crate::monitoring::alerting::AlertEvaluationResult,
    iteration: usize,
) {
    println!("\n=== Monitoring Status (Iteration {}) ===", iteration);
    println!("Active Broadcasts: {}", performance_status.active_broadcasts);
    println!("Total Throughput: {:.2} ops/sec", performance_status.total_throughput);
    println!("Average Response Time: {:.2} ms", performance_status.average_response_time_ms);
    println!("Performance Health: {:.1}%", performance_status.performance_health);
    
    let resource_metrics = &resource_monitor.resource_metrics;
    println!("Memory Usage: {:.1} MB ({:.1}%)", 
        resource_metrics.memory.used_mb, 
        resource_metrics.memory.usage_percentage
    );
    println!("CPU Usage: {:.1}%", resource_metrics.cpu.usage_percentage);
    println!("Disk Usage: {:.1}%", resource_metrics.disk.usage_percentage);
    
    if !alert_result.triggered_alerts.is_empty() {
        println!("🚨 Active Alerts: {}", alert_result.triggered_alerts.len());
        for alert in &alert_result.triggered_alerts {
            println!("  - {}: {}", alert.title, alert.message);
        }
    }
    
    if !alert_result.resolved_alerts.is_empty() {
        println!("✅ Resolved Alerts: {}", alert_result.resolved_alerts.len());
    }
    
    println!("==========================================\n");
}

/// Display final monitoring report
fn display_final_report(report: &DashboardReport) {
    println!("\n=== Final Monitoring Report ===");
    println!("Project: {}", report.project_id);
    println!("Generated: {}", report.generated_at);
    println!();
    
    println!("=== Summary ===");
    println!("Overall Health: {:.1}%", report.summary.overall_health);
    println!("Status: {:?}", report.summary.status);
    println!("Active Broadcasts: {}", report.summary.active_broadcasts);
    println!("Total Broadcasts Today: {}", report.summary.total_broadcasts_today);
    println!("Success Rate: {:.2}%", report.summary.success_rate * 100.0);
    println!("Average Response Time: {:.2} ms", report.summary.average_response_time_ms);
    println!("Error Rate: {:.2}%", report.summary.error_rate * 100.0);
    println!();
    
    if !report.alerts.is_empty() {
        println!("=== Alerts ===");
        for alert in &report.alerts {
            println!("[{}] {}: {}", 
                format!("{:?}", alert.severity), 
                alert.category, 
                alert.message
            );
        }
        println!();
    }
    
    if !report.recommendations.is_empty() {
        println!("=== Recommendations ===");
        for (i, recommendation) in report.recommendations.iter().enumerate() {
            println!("{}. {}", i + 1, recommendation);
        }
        println!();
    }
    
    println!("=== Resource Usage ===");
    let metrics = &report.dashboard_data.resource_data;
    println!("Memory: {:.1} MB ({:.1}%)", 
        metrics.memory_usage.current, 
        metrics.memory_usage.current
    );
    println!("CPU: {:.1}%", metrics.cpu_usage.current);
    println!("Disk: {:.1}%", metrics.disk_usage.current);
    println!("Network: {:.1} MB/s", metrics.network_usage.current);
    println!();
    
    println!("================================");
}
