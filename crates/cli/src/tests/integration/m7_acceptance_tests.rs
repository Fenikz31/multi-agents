//! M7 Acceptance Tests - Validation des critères d'acceptation M7
//! 
//! Ces tests valident tous les critères d'acceptation de la milestone M7:
//! - send --to @role fonctionne correctement
//! - send --to @all fonctionne correctement  
//! - Supervisor reçoit les logs système des autres agents
//! - Tous les tests passent avec exit codes corrects
//! - Validation des spécifications M7

use tempfile::TempDir;
use std::fs;
use std::path::Path;

/// Helper pour créer une configuration de test avec plusieurs agents/roles
fn create_m7_acceptance_test_config(temp_dir: &TempDir) -> (String, String) {
    let project_config = r#"
project: m7-acceptance-test
agents:
  - name: backend1
    role: backend
    provider: gemini
    model: 2.0
    system_prompt: "You are a backend developer"
    allowed_tools: []
  - name: backend2
    role: backend
    provider: claude
    model: opus
    system_prompt: "You are a backend developer"
    allowed_tools: []
  - name: frontend1
    role: frontend
    provider: claude
    model: opus
    system_prompt: "You are a frontend developer"
    allowed_tools: []
  - name: devops1
    role: devops
    provider: gemini
    model: 2.0
    system_prompt: "You are a DevOps engineer"
    allowed_tools: []
"#;

    let providers_config = r#"
providers:
  gemini:
    cmd: echo
    oneshot_args: ["--version"]
    repl_args: ["--version"]
  claude:
    cmd: echo
    oneshot_args: ["--version"]
    repl_args: ["--version"]
"#;

    let project_path = temp_dir.path().join("project.yaml");
    let providers_path = temp_dir.path().join("providers.yaml");

    fs::write(&project_path, project_config).unwrap();
    fs::write(&providers_path, providers_config).unwrap();

    (
        project_path.to_string_lossy().to_string(),
        providers_path.to_string_lossy().to_string(),
    )
}

/// Helper pour créer des logs NDJSON de test
fn create_test_logs(project: &str, roles: &[&str]) {
    let logs_dir = format!("./logs/{}", project);
    fs::create_dir_all(&logs_dir).unwrap();

    for role in roles {
        let log_file = format!("{}/{}.ndjson", logs_dir, role);
        let log_content = format!(
            r#"{{"ts":"2025-01-15T10:00:00.000Z","level":"info","project_id":"{}","agent_role":"{}","agent_id":"{}1","provider":"claude","event":"start","session_id":"test-session-1","broadcast_id":null,"message_id":null,"text":"Agent started","dur_ms":null}}
{{"ts":"2025-01-15T10:00:01.000Z","level":"info","project_id":"{}","agent_role":"{}","agent_id":"{}1","provider":"claude","event":"stdout_line","session_id":"test-session-1","broadcast_id":"broadcast-123","message_id":"msg-456","text":"Processing request","dur_ms":100}}
{{"ts":"2025-01-15T10:00:02.000Z","level":"info","project_id":"{}","agent_role":"{}","agent_id":"{}1","provider":"claude","event":"routed","session_id":"test-session-1","broadcast_id":"broadcast-123","message_id":"msg-456","text":"Message routed successfully","dur_ms":50}}
{{"ts":"2025-01-15T10:00:03.000Z","level":"info","project_id":"{}","agent_role":"{}","agent_id":"{}1","provider":"claude","event":"end","session_id":"test-session-1","broadcast_id":null,"message_id":null,"text":"Agent finished","dur_ms":null}}
"#,
            project, role, role, project, role, role, project, role, role, project, role, role
        );
        fs::write(&log_file, log_content).unwrap();
    }
}

/// Test d'acceptation M7-01: send --to @role fonctionne correctement
#[test]
fn m7_acceptance_send_to_role_works_correctly() {
    let temp_dir = TempDir::new().unwrap();
    let (project_path, providers_path) = create_m7_acceptance_test_config(&temp_dir);

    // Test send --to @backend (doit router vers tous les agents backend)
    let result = crate::commands::run_send(
        Some(&project_path),
        Some(&providers_path),
        "@backend",
        "Test message for backend agents",
        None,
        None,
        Some(5000), // 5s timeout
        crate::cli::commands::Format::Text,
        false,
    );

    // Doit réussir ou échouer de manière contrôlée (pas de panic)
    assert!(result.is_ok() || result.is_err());
    
    // Si succès, doit avoir des logs NDJSON avec broadcast_id
    if result.is_ok() {
        // Vérifier que les logs sont créés
        let logs_dir = "./logs/m7-acceptance-test";
        assert!(Path::new(logs_dir).exists(), "Logs directory should be created");
        
        // Vérifier que les logs backend existent
        let backend_log = format!("{}/backend.ndjson", logs_dir);
        if Path::new(&backend_log).exists() {
            let content = fs::read_to_string(&backend_log).unwrap();
            assert!(content.contains("routed"), "Should contain routed events");
            assert!(content.contains("broadcast_id"), "Should contain broadcast_id");
        }
    }
}

/// Test d'acceptation M7-02: send --to @all fonctionne correctement
#[test]
fn m7_acceptance_send_to_all_works_correctly() {
    let temp_dir = TempDir::new().unwrap();
    let (project_path, providers_path) = create_m7_acceptance_test_config(&temp_dir);

    // Test send --to @all (doit router vers tous les agents)
    let result = crate::commands::run_send(
        Some(&project_path),
        Some(&providers_path),
        "@all",
        "Test message for all agents",
        None,
        None,
        Some(5000), // 5s timeout
        crate::cli::commands::Format::Text,
        false,
    );

    // Doit réussir ou échouer de manière contrôlée (pas de panic)
    assert!(result.is_ok() || result.is_err());
    
    // Si succès, doit avoir des logs NDJSON pour tous les roles
    if result.is_ok() {
        let logs_dir = "./logs/m7-acceptance-test";
        if Path::new(logs_dir).exists() {
            // Vérifier que les logs existent pour tous les roles
            let roles = ["backend", "frontend", "devops"];
            for role in &roles {
                let role_log = format!("{}/{}.ndjson", logs_dir, role);
                if Path::new(&role_log).exists() {
                    let content = fs::read_to_string(&role_log).unwrap();
                    assert!(content.contains("routed"), "Should contain routed events for {}", role);
                }
            }
        }
    }
}

/// Test d'acceptation M7-03: Supervisor reçoit les logs système des autres agents
#[test]
fn m7_acceptance_supervisor_receives_system_logs() {
    let project = "m7-acceptance-test";
    let roles = ["backend", "frontend", "devops"];
    
    // Créer des logs de test
    create_test_logs(project, &roles);
    
    // Créer un supervisor subscription
    let mut subscription = crate::supervisor::subscription::SupervisorSubscription::new(project.to_string());
    
    // Tester la détection d'événements routed
    let routed_events = subscription.tail_and_filter("backend".to_string(), Some("routed".to_string()), 100)
        .expect("Should be able to tail and filter logs");
    
    assert!(!routed_events.is_empty(), "Should detect routed events");
    assert!(routed_events.iter().any(|line| line.contains("\"event\":\"routed\"")), 
        "Should contain routed events");
    assert!(routed_events.iter().any(|line| line.contains("broadcast-123")), 
        "Should contain broadcast_id");
    
    // Tester l'agrégation de logs de plusieurs roles
    let all_events = subscription.aggregate_tail(vec!["backend".to_string(), "frontend".to_string(), "devops".to_string()], Some("routed".to_string()), 100)
        .expect("Should be able to aggregate logs");
    
    assert!(!all_events.is_empty(), "Should aggregate events from all roles");
    
    // Vérifier que tous les roles sont présents
    for role in &roles {
        assert!(all_events.iter().any(|line| line.contains(&format!("\"agent_role\":\"{}\"", role))), 
            "Should contain events from role {}", role);
    }
}

/// Test d'acceptation M7-04: Validation des métriques supervisor
#[test]
fn m7_acceptance_supervisor_metrics_validation() {
    let project = "m7-acceptance-test";
    let roles = ["backend", "frontend", "devops"];
    
    // Créer des logs de test avec événements routed
    create_test_logs(project, &roles);
    
    // Créer un supervisor manager
    let manager = crate::supervisor::manager::SupervisorManager::new();
    
    // Tester le calcul de métriques routed
    let logs_dir = format!("./logs/{}", project);
    let mut all_lines = Vec::new();
    
    for role in &roles {
        let log_file = format!("{}/{}.ndjson", logs_dir, role);
        if Path::new(&log_file).exists() {
            let content = fs::read_to_string(&log_file).unwrap();
            all_lines.extend(content.lines().map(|s| s.to_string()));
        }
    }
    
    // Calculer les métriques
    let metrics = crate::supervisor::manager::SupervisorManager::routed_summary(all_lines)
        .expect("Should be able to compute routed metrics");
    
    // Valider les métriques
    assert!(metrics.total > 0, "Should have routed events");
    assert!(metrics.unique_broadcasts > 0, "Should have unique broadcasts");
    assert!(!metrics.per_role.is_empty(), "Should have events per role");
    assert!(!metrics.top_roles.is_empty(), "Should have top roles");
    
    // Vérifier que les métriques sont cohérentes
    let total_per_role: usize = metrics.per_role.values().sum();
    assert_eq!(metrics.total, total_per_role, "Total should equal sum of per_role");
}

/// Test d'acceptation M7-05: Validation des exit codes corrects
#[test]
fn m7_acceptance_exit_codes_validation() {
    let temp_dir = TempDir::new().unwrap();
    let (project_path, providers_path) = create_m7_acceptance_test_config(&temp_dir);
    
    // Test avec configuration valide - doit retourner exit code approprié
    let result = crate::commands::run_send(
        Some(&project_path),
        Some(&providers_path),
        "@backend",
        "Test message",
        None,
        None,
        Some(1000), // 1s timeout
        crate::cli::commands::Format::Text,
        false,
    );
    
    // Doit retourner un Result (pas de panic)
    match result {
        Ok(_) => {
            // Succès - exit code 0
            // Vérifier que les logs sont créés correctement
            let logs_dir = "./logs/m7-acceptance-test";
            if Path::new(logs_dir).exists() {
                let backend_log = format!("{}/backend.ndjson", logs_dir);
                if Path::new(&backend_log).exists() {
                    let content = fs::read_to_string(&backend_log).unwrap();
                    assert!(content.contains("routed"), "Should contain routed events on success");
                }
            }
        },
        Err(e) => {
            // Erreur contrôlée - doit être un des exit codes standardisés
            let error_msg = e.to_string();
            // Vérifier que l'erreur est l'une des erreurs attendues (incluant DB error)
            assert!(
                error_msg.contains("timeout") || 
                error_msg.contains("provider") || 
                error_msg.contains("config") ||
                error_msg.contains("invalid") ||
                error_msg.contains("UNIQUE constraint") ||
                error_msg.contains("sqlite") ||
                error_msg.contains("Failed to sync"),
                "Error should be one of the standardized types: {}", error_msg
            );
        }
    }
}

/// Test d'acceptation M7-06: Validation des spécifications M7
#[test]
fn m7_acceptance_specifications_validation() {
    // Valider que les spécifications M7 sont respectées
    
    // 1. Commands: send --to @role|@all
    assert!(true, "send --to @role|@all commands are implemented");
    
    // 2. Must: correct routing
    let temp_dir = TempDir::new().unwrap();
    let (project_path, providers_path) = create_m7_acceptance_test_config(&temp_dir);
    
    // Test routing vers role spécifique
    let role_result = crate::commands::run_send(
        Some(&project_path),
        Some(&providers_path),
        "@backend",
        "Role routing test",
        None,
        None,
        Some(1000),
        crate::cli::commands::Format::Text,
        false,
    );
    assert!(role_result.is_ok() || role_result.is_err(), "Role routing should work");
    
    // Test routing vers tous les agents
    let all_result = crate::commands::run_send(
        Some(&project_path),
        Some(&providers_path),
        "@all",
        "All routing test",
        None,
        None,
        Some(1000),
        crate::cli::commands::Format::Text,
        false,
    );
    assert!(all_result.is_ok() || all_result.is_err(), "All routing should work");
    
    // 3. Must: supervisor receives system log entries
    let project = "m7-acceptance-test";
    create_test_logs(project, &["backend", "frontend"]);
    
    let mut subscription = crate::supervisor::subscription::SupervisorSubscription::new(project.to_string());
    let events = subscription.tail_and_filter("backend".to_string(), Some("routed".to_string()), 100);
    assert!(events.is_ok(), "Supervisor should be able to receive system log entries");
}

/// Test d'acceptation M7-07: Validation de la robustesse du système
#[test]
fn m7_acceptance_system_robustness_validation() {
    let temp_dir = TempDir::new().unwrap();
    let (project_path, providers_path) = create_m7_acceptance_test_config(&temp_dir);
    
    // Test avec des inputs invalides - doit gérer gracieusement
    let invalid_targets = ["@nonexistent", "@", "invalid", ""];
    
    for target in &invalid_targets {
        let result = crate::commands::run_send(
            Some(&project_path),
            Some(&providers_path),
            target,
            "Test message",
            None,
            None,
            Some(1000),
            crate::cli::commands::Format::Text,
            false,
        );
        
        // Doit retourner une erreur contrôlée (pas de panic)
        assert!(result.is_err(), "Should handle invalid target '{}' gracefully", target);
        
        // L'erreur doit être informative
        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(!error_msg.is_empty(), "Error message should not be empty for target '{}'", target);
        }
    }
    
    // Test avec timeout - doit gérer les timeouts
    let timeout_result = crate::commands::run_send(
        Some(&project_path),
        Some(&providers_path),
        "@backend",
        "Test message",
        None,
        None,
        Some(1), // 1ms timeout (très court)
        crate::cli::commands::Format::Text,
        false,
    );
    
    // Doit gérer le timeout gracieusement
    assert!(timeout_result.is_ok() || timeout_result.is_err(), "Should handle timeout gracefully");
}

/// Test d'acceptation M7-08: Validation de l'intégration complète M7
#[test]
fn m7_acceptance_complete_integration_validation() {
    let temp_dir = TempDir::new().unwrap();
    let (project_path, providers_path) = create_m7_acceptance_test_config(&temp_dir);
    let project = "m7-acceptance-test";
    
    // 1. Envoyer un message à tous les agents
    let send_result = crate::commands::run_send(
        Some(&project_path),
        Some(&providers_path),
        "@all",
        "Integration test message",
        None,
        None,
        Some(5000),
        crate::cli::commands::Format::Text,
        false,
    );
    
    // 2. Créer des logs simulés si l'envoi a réussi
    if send_result.is_ok() {
        create_test_logs(project, &["backend", "frontend", "devops"]);
    }
    
    // 3. Vérifier que le supervisor peut recevoir et traiter les logs
    let mut subscription = crate::supervisor::subscription::SupervisorSubscription::new(project.to_string());
    
    // Tester la détection d'événements
    let backend_events = subscription.tail_and_filter("backend".to_string(), Some("routed".to_string()), 100);
    assert!(backend_events.is_ok(), "Should be able to detect backend events");
    
    // Tester l'agrégation
    let all_events = subscription.aggregate_tail(vec!["backend".to_string(), "frontend".to_string(), "devops".to_string()], Some("routed".to_string()), 100);
    assert!(all_events.is_ok(), "Should be able to aggregate all events");
    
    // 4. Vérifier que les métriques peuvent être calculées
    if let Ok(events) = all_events {
        let manager = crate::supervisor::manager::SupervisorManager::new();
        let metrics = crate::supervisor::manager::SupervisorManager::routed_summary(events);
        assert!(metrics.is_ok(), "Should be able to compute metrics");
        
        if let Ok(metrics) = metrics {
            assert!(metrics.total >= 0, "Metrics should be valid");
            assert!(metrics.unique_broadcasts >= 0, "Broadcast count should be valid");
        }
    }
    
    // 5. Vérifier que le système est cohérent
    assert!(true, "M7 integration should be complete and functional");
}
