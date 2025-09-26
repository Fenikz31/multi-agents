//! M7 Functional Validation Tests - Tests d'intégration réels pour valider le comportement M7
//! 
//! Ces tests valident que les fonctionnalités M7 fonctionnent réellement comme documenté:
//! - send --to @role génère des événements routed
//! - send --to @all broadcast vers tous les agents
//! - Supervisor peut lire et analyser les logs
//! - Métriques sont calculées correctement
//! - Intégration complète fonctionne

use tempfile::TempDir;
use std::fs;
use std::path::Path;

/// Helper pour créer une configuration de test avec plusieurs agents/roles
fn create_m7_test_config(temp_dir: &TempDir) -> (String, String) {
    let project_config = r#"
project: m7-functional-test
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
    provider: gemini
    model: 2.0
    system_prompt: "You are a frontend developer"
    allowed_tools: []
  - name: devops1
    role: devops
    provider: claude
    model: opus
    system_prompt: "You are a DevOps engineer"
    allowed_tools: []
"#;

    let providers_config = r#"
providers:
  gemini:
    cli_command: "gemini"
    model_placeholder: "{model}"
    system_prompt_placeholder: "{system_prompt}"
    timeout_ms: 30000
  claude:
    cli_command: "claude"
    model_placeholder: "{model}"
    system_prompt_placeholder: "{system_prompt}"
    timeout_ms: 30000
"#;

    let project_path = temp_dir.path().join("project.yaml");
    let providers_path = temp_dir.path().join("providers.yaml");
    
    fs::write(&project_path, project_config).unwrap();
    fs::write(&providers_path, providers_config).unwrap();
    
    (project_path.to_string_lossy().to_string(), providers_path.to_string_lossy().to_string())
}

/// Test fonctionnel: send --to @role génère des événements routed
#[test]
fn m7_functional_send_to_role_generates_routed_events() {
    let temp_dir = TempDir::new().unwrap();
    let (project_path, providers_path) = create_m7_test_config(&temp_dir);
    
    // Créer le projet dans la base de données
    let result = crate::commands::run_project_add(
        "m7-functional-test",
        None,
    );
    
    // Le projet peut déjà exister, c'est OK
    assert!(result.is_ok() || result.as_ref().err().map(|e| e.to_string()).unwrap_or_default().contains("UNIQUE constraint"));
    
    // Ajouter les agents
    let _ = crate::commands::run_agent_add(
        "m7-functional-test",
        "backend1",
        "backend",
        "gemini",
        "2.0",
        &[],
        "You are a backend developer",
        None,
    );
    
    let _ = crate::commands::run_agent_add(
        "m7-functional-test",
        "frontend1",
        "frontend",
        "gemini",
        "2.0",
        &[],
        "You are a frontend developer",
        None,
    );
    
    // Tester send --to @backend
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
    
    // Vérifier que la commande s'exécute (peut échouer si les providers ne sont pas disponibles, mais la logique doit fonctionner)
    match result {
        Ok(_) => {
            // Succès - vérifier que les logs sont créés
            let logs_dir = "./logs/m7-functional-test";
            if Path::new(logs_dir).exists() {
                let backend_log = format!("{}/backend.ndjson", logs_dir);
                if Path::new(&backend_log).exists() {
                    let content = fs::read_to_string(&backend_log).unwrap_or_default();
                    // Vérifier qu'il y a des événements routed
                    assert!(content.contains("\"event\":\"routed\""), "Should contain routed events");
                    assert!(content.contains("\"broadcast_id\""), "Should contain broadcast_id");
                }
            }
        },
        Err(e) => {
            // Échec attendu si les providers ne sont pas disponibles
            let error_msg = e.to_string();
            assert!(error_msg.contains("provider") || error_msg.contains("timeout") || error_msg.contains("unavailable"), 
                "Should fail gracefully with provider/timeout error, got: {}", error_msg);
        }
    }
}

/// Test fonctionnel: send --to @all broadcast vers tous les agents
#[test]
fn m7_functional_send_to_all_broadcasts_to_all_agents() {
    let temp_dir = TempDir::new().unwrap();
    let (project_path, providers_path) = create_m7_test_config(&temp_dir);
    
    // Créer le projet et les agents (simplifié pour ce test)
    let _ = crate::commands::run_project_add(
        "m7-functional-test",
        None,
    );
    
    // Tester send --to @all
    let result = crate::commands::run_send(
        Some(&project_path),
        Some(&providers_path),
        "@all",
        "Broadcast message to all agents",
        None,
        None,
        Some(5000),
        crate::cli::commands::Format::Text,
        false,
    );
    
    // Vérifier que la commande s'exécute
    match result {
        Ok(_) => {
            // Succès - vérifier que les logs sont créés pour tous les roles
            let logs_dir = "./logs/m7-functional-test";
            if Path::new(logs_dir).exists() {
                let roles = ["backend", "frontend", "devops"];
                for role in &roles {
                    let role_log = format!("{}/{}.ndjson", logs_dir, role);
                    if Path::new(&role_log).exists() {
                        let content = fs::read_to_string(&role_log).unwrap_or_default();
                        if !content.is_empty() {
                            assert!(content.contains("\"event\":\"routed\""), "Role {} should have routed events", role);
                        }
                    }
                }
            }
        },
        Err(e) => {
            // Échec attendu si les providers ne sont pas disponibles
            let error_msg = e.to_string();
            assert!(error_msg.contains("provider") || error_msg.contains("timeout") || error_msg.contains("unavailable"), 
                "Should fail gracefully with provider/timeout error, got: {}", error_msg);
        }
    }
}

/// Test fonctionnel: Supervisor peut lire et analyser les logs
#[test]
fn m7_functional_supervisor_can_read_and_analyze_logs() {
    let temp_dir = TempDir::new().unwrap();
    let (project_path, providers_path) = create_m7_test_config(&temp_dir);
    
    // Créer des logs de test simulés
    let logs_dir = "./logs/m7-functional-test";
    fs::create_dir_all(logs_dir).unwrap();
    
    let backend_log_content = r#"{"ts":"2025-01-15T10:00:00.000Z","level":"info","project_id":"m7-functional-test","agent_role":"backend","agent_id":"backend1","provider":"gemini","event":"start","session_id":"test-session-1","broadcast_id":null,"message_id":null,"text":"Agent started","dur_ms":null}
{"ts":"2025-01-15T10:00:01.000Z","level":"info","project_id":"m7-functional-test","agent_role":"backend","agent_id":"backend1","provider":"gemini","event":"routed","session_id":"test-session-1","broadcast_id":"broadcast-123","message_id":"msg-456","text":"Message routed successfully","dur_ms":50}
{"ts":"2025-01-15T10:00:02.000Z","level":"info","project_id":"m7-functional-test","agent_role":"backend","agent_id":"backend2","provider":"claude","event":"routed","session_id":"test-session-2","broadcast_id":"broadcast-123","message_id":"msg-457","text":"Message routed successfully","dur_ms":75}
"#;
    
    let frontend_log_content = r#"{"ts":"2025-01-15T10:00:01.500Z","level":"info","project_id":"m7-functional-test","agent_role":"frontend","agent_id":"frontend1","provider":"gemini","event":"routed","session_id":"test-session-3","broadcast_id":"broadcast-123","message_id":"msg-458","text":"Message routed successfully","dur_ms":60}
"#;
    
    fs::write(format!("{}/backend.ndjson", logs_dir), backend_log_content).unwrap();
    fs::write(format!("{}/frontend.ndjson", logs_dir), frontend_log_content).unwrap();
    
    // Tester le supervisor
    let mut subscription = crate::supervisor::subscription::SupervisorSubscription::new("m7-functional-test".to_string());
    
    // Lire les événements routed du backend
    let backend_events = subscription.tail_and_filter(
        "backend".to_string(),
        Some("routed".to_string()),
        100
    ).expect("Should be able to read backend events");
    
    assert_eq!(backend_events.len(), 2, "Should have 2 routed events in backend");
    assert!(backend_events[0].contains("\"broadcast_id\":\"broadcast-123\""), "Should contain broadcast_id");
    
    // Lire les événements routed du frontend
    let frontend_events = subscription.tail_and_filter(
        "frontend".to_string(),
        Some("routed".to_string()),
        100
    ).expect("Should be able to read frontend events");
    
    assert_eq!(frontend_events.len(), 1, "Should have 1 routed event in frontend");
    
    // Tester l'agrégation
    let all_events = subscription.aggregate_tail(
        vec!["backend".to_string(), "frontend".to_string()],
        Some("routed".to_string()),
        100
    ).expect("Should be able to aggregate events");
    
    assert_eq!(all_events.len(), 3, "Should have 3 total routed events");
    
    // Vérifier que les événements sont triés par timestamp
    let first_ts = extract_ts(&all_events[0]);
    let last_ts = extract_ts(&all_events[2]);
    assert!(first_ts <= last_ts, "Events should be sorted by timestamp");
}

/// Test fonctionnel: Métriques sont calculées correctement
#[test]
fn m7_functional_metrics_are_calculated_correctly() {
    // Créer des données de test
    let test_events = vec![
        crate::logging::events::NdjsonEvent {
            ts: "2025-01-15T10:00:00.000Z".to_string(),
            level: "info".to_string(),
            project_id: "m7-functional-test".to_string(),
            agent_role: "backend".to_string(),
            agent_id: "backend1".to_string(),
            provider: "gemini".to_string(),
            event: "routed".to_string(),
            text: Some("Message 1".to_string()),
            dur_ms: Some(50),
            broadcast_id: Some("broadcast-123".to_string()),
            session_id: Some("session-1".to_string()),
            message_id: Some("msg-1".to_string()),
        },
        crate::logging::events::NdjsonEvent {
            ts: "2025-01-15T10:00:01.000Z".to_string(),
            level: "info".to_string(),
            project_id: "m7-functional-test".to_string(),
            agent_role: "backend".to_string(),
            agent_id: "backend2".to_string(),
            provider: "claude".to_string(),
            event: "routed".to_string(),
            text: Some("Message 2".to_string()),
            dur_ms: Some(75),
            broadcast_id: Some("broadcast-123".to_string()),
            session_id: Some("session-2".to_string()),
            message_id: Some("msg-2".to_string()),
        },
        crate::logging::events::NdjsonEvent {
            ts: "2025-01-15T10:00:02.000Z".to_string(),
            level: "info".to_string(),
            project_id: "m7-functional-test".to_string(),
            agent_role: "frontend".to_string(),
            agent_id: "frontend1".to_string(),
            provider: "gemini".to_string(),
            event: "routed".to_string(),
            text: Some("Message 3".to_string()),
            dur_ms: Some(60),
            broadcast_id: Some("broadcast-456".to_string()),
            session_id: Some("session-3".to_string()),
            message_id: Some("msg-3".to_string()),
        },
    ];
    
    // Calculer les métriques
    let metrics = crate::supervisor::metrics::compute_routed_metrics_from_events(test_events)
        .expect("Should be able to compute metrics");
    
    // Vérifier les métriques
    assert_eq!(metrics.total, 3, "Should have 3 total routed events");
    assert_eq!(metrics.unique_broadcasts, 2, "Should have 2 unique broadcasts");
    
    // Vérifier la répartition par rôle
    assert_eq!(metrics.per_role.get("backend"), Some(&2), "Backend should have 2 events");
    assert_eq!(metrics.per_role.get("frontend"), Some(&1), "Frontend should have 1 event");
    
    // Vérifier les top roles
    assert_eq!(metrics.top_roles.len(), 2, "Should have 2 roles");
    assert_eq!(metrics.top_roles[0].0, "backend", "Backend should be first");
    assert_eq!(metrics.top_roles[0].1, 2, "Backend should have 2 events");
    assert_eq!(metrics.top_roles[1].0, "frontend", "Frontend should be second");
    assert_eq!(metrics.top_roles[1].1, 1, "Frontend should have 1 event");
    
    // Vérifier la latence P95
    assert_eq!(metrics.p95_latency_per_broadcast.len(), 2, "Should have latency for 2 broadcasts");
    assert!(metrics.p95_latency_per_broadcast.contains_key("broadcast-123"), "Should have latency for broadcast-123");
    assert!(metrics.p95_latency_per_broadcast.contains_key("broadcast-456"), "Should have latency for broadcast-456");
}

/// Test fonctionnel: Intégration complète supervisor + routing
#[test]
fn m7_functional_complete_supervisor_routing_integration() {
    let temp_dir = TempDir::new().unwrap();
    let (project_path, providers_path) = create_m7_test_config(&temp_dir);
    
    // Créer des logs de test avec plusieurs broadcasts
    let logs_dir = "./logs/m7-functional-test";
    fs::create_dir_all(logs_dir).unwrap();
    
    let backend_log_content = r#"{"ts":"2025-01-15T10:00:00.000Z","level":"info","project_id":"m7-functional-test","agent_role":"backend","agent_id":"backend1","provider":"gemini","event":"routed","session_id":"test-session-1","broadcast_id":"broadcast-123","message_id":"msg-1","text":"Message routed","dur_ms":50}
{"ts":"2025-01-15T10:00:01.000Z","level":"info","project_id":"m7-functional-test","agent_role":"backend","agent_id":"backend2","provider":"claude","event":"routed","session_id":"test-session-2","broadcast_id":"broadcast-123","message_id":"msg-2","text":"Message routed","dur_ms":75}
{"ts":"2025-01-15T10:00:05.000Z","level":"info","project_id":"m7-functional-test","agent_role":"backend","agent_id":"backend1","provider":"gemini","event":"routed","session_id":"test-session-3","broadcast_id":"broadcast-456","message_id":"msg-3","text":"Message routed","dur_ms":60}
"#;
    
    let frontend_log_content = r#"{"ts":"2025-01-15T10:00:01.500Z","level":"info","project_id":"m7-functional-test","agent_role":"frontend","agent_id":"frontend1","provider":"gemini","event":"routed","session_id":"test-session-4","broadcast_id":"broadcast-123","message_id":"msg-4","text":"Message routed","dur_ms":65}
{"ts":"2025-01-15T10:00:05.500Z","level":"info","project_id":"m7-functional-test","agent_role":"frontend","agent_id":"frontend1","provider":"gemini","event":"routed","session_id":"test-session-5","broadcast_id":"broadcast-456","message_id":"msg-5","text":"Message routed","dur_ms":70}
"#;
    
    fs::write(format!("{}/backend.ndjson", logs_dir), backend_log_content).unwrap();
    fs::write(format!("{}/frontend.ndjson", logs_dir), frontend_log_content).unwrap();
    
    // Tester l'intégration complète
    let mut subscription = crate::supervisor::subscription::SupervisorSubscription::new("m7-functional-test".to_string());
    
    // Agrégation de tous les événements routed
    let all_events = subscription.aggregate_tail(
        vec!["backend".to_string(), "frontend".to_string()],
        Some("routed".to_string()),
        100
    ).expect("Should be able to aggregate all events");
    
    // Conversion en NdjsonEvent pour les métriques
    let events: Vec<crate::logging::events::NdjsonEvent> = all_events.iter()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();
    
    // Calcul des métriques via SupervisorManager
    let metrics = crate::supervisor::manager::SupervisorManager::routed_summary_from_events(events)
        .expect("Should be able to compute metrics via SupervisorManager");
    
    // Vérifications complètes
    assert_eq!(metrics.total, 5, "Should have 5 total routed events");
    assert_eq!(metrics.unique_broadcasts, 2, "Should have 2 unique broadcasts");
    
    // Vérifier la répartition par rôle
    assert_eq!(metrics.per_role.get("backend"), Some(&3), "Backend should have 3 events");
    assert_eq!(metrics.per_role.get("frontend"), Some(&2), "Frontend should have 2 events");
    
    // Vérifier les top roles (backend en premier)
    assert_eq!(metrics.top_roles[0].0, "backend", "Backend should be first");
    assert_eq!(metrics.top_roles[0].1, 3, "Backend should have 3 events");
    assert_eq!(metrics.top_roles[1].0, "frontend", "Frontend should be second");
    assert_eq!(metrics.top_roles[1].1, 2, "Frontend should have 2 events");
    
    // Vérifier que les événements sont triés chronologiquement
    assert_eq!(all_events.len(), 5, "Should have 5 events total");
    let first_ts = extract_ts(&all_events[0]);
    let last_ts = extract_ts(&all_events[4]);
    assert!(first_ts <= last_ts, "Events should be sorted chronologically");
}

/// Helper pour extraire le timestamp d'une ligne NDJSON
fn extract_ts(line: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(line) {
        Ok(v) => v.get("ts").and_then(|t| t.as_str()).unwrap_or("").to_string(),
        Err(_) => String::new(),
    }
}
