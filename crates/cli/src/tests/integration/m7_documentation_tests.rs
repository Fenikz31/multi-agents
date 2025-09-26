//! M7 Documentation Tests - Validation de la documentation des nouvelles fonctionnalités M7
//! 
//! Ces tests valident que la documentation M7 est complète et correcte:
//! - Documentation des commandes send --to @role|@all
//! - Guide d'utilisation du supervisor
//! - Exemples d'utilisation avec routing
//! - Mise à jour des workflows
//! - Documentation des événements routed

use std::fs;
use std::path::Path;

/// Test de validation de la documentation CLI reference
#[test]
fn m7_documentation_cli_reference_validation() {
    let cli_ref_path = "docs/cli-reference.md";
    assert!(Path::new(cli_ref_path).exists(), "CLI reference documentation should exist");
    
    let content = fs::read_to_string(cli_ref_path).expect("Should be able to read CLI reference");
    
    // Vérifier que la documentation contient les commandes M7
    assert!(content.contains("send --to @role"), "Should document send --to @role command");
    assert!(content.contains("send --to @all"), "Should document send --to @all command");
    
    // Vérifier que les exemples sont présents
    assert!(content.contains("Send to all agents in a role"), "Should have role-based routing example");
    assert!(content.contains("Send to all agents"), "Should have broadcast to all example");
    
    // Vérifier que les exit codes sont documentés
    assert!(content.contains("Exit Codes:"), "Should document exit codes");
    assert!(content.contains("0: Message sent successfully"), "Should document success exit code");
    assert!(content.contains("2: Invalid input"), "Should document invalid input exit code");
}

/// Test de validation de la documentation des workflows
#[test]
fn m7_documentation_workflows_validation() {
    let workflows_path = "docs/workflows.md";
    assert!(Path::new(workflows_path).exists(), "Workflows documentation should exist");
    
    let content = fs::read_to_string(workflows_path).expect("Should be able to read workflows");
    
    // Vérifier que les workflows M7 sont documentés
    assert!(content.contains("Routing and Supervisor"), "Should document routing and supervisor workflow");
    assert!(content.contains("@role"), "Should document @role routing");
    assert!(content.contains("@all"), "Should document @all routing");
    
    // Vérifier que les exemples de workflow sont présents
    assert!(content.contains("multi-agents send --to @backend"), "Should have backend routing example");
    assert!(content.contains("multi-agents send --to @all"), "Should have broadcast example");
}

/// Test de validation du guide d'utilisation du supervisor
#[test]
fn m7_documentation_supervisor_guide_validation() {
    let supervisor_guide_path = "docs/supervisor-guide.md";
    
    // Le guide du supervisor doit exister
    assert!(Path::new(supervisor_guide_path).exists(), "Supervisor guide should exist");
    
    let content = fs::read_to_string(supervisor_guide_path).expect("Should be able to read supervisor guide");
    
    // Vérifier que le guide contient les sections essentielles
    assert!(content.contains("# Supervisor Guide"), "Should have supervisor guide title");
    assert!(content.contains("SupervisorManager"), "Should document SupervisorManager");
    assert!(content.contains("SupervisorSubscription"), "Should document SupervisorSubscription");
    assert!(content.contains("RoutedMetrics"), "Should document RoutedMetrics");
    
    // Vérifier que les fonctionnalités sont documentées
    assert!(content.contains("subscribe"), "Should document subscription functionality");
    assert!(content.contains("tail_and_filter"), "Should document log filtering");
    assert!(content.contains("aggregate_tail"), "Should document log aggregation");
    assert!(content.contains("routed_summary"), "Should document metrics computation");
    
    // Vérifier que les exemples d'utilisation sont présents
    assert!(content.contains("```rust"), "Should have code examples");
    assert!(content.contains("```bash"), "Should have command examples");
}

/// Test de validation des exemples d'utilisation avec routing
#[test]
fn m7_documentation_routing_examples_validation() {
    let supervisor_guide_path = "docs/supervisor-guide.md";
    assert!(Path::new(supervisor_guide_path).exists(), "Supervisor guide should exist");
    
    let content = fs::read_to_string(supervisor_guide_path).expect("Should be able to read supervisor guide");
    
    // Vérifier que les exemples de routing sont présents
    assert!(content.contains("send --to @backend"), "Should have backend routing example");
    assert!(content.contains("send --to @all"), "Should have broadcast example");
    assert!(content.contains("routed events"), "Should document routed events");
    
    // Vérifier que les exemples de métriques sont présents
    assert!(content.contains("RoutedMetrics"), "Should have metrics examples");
    assert!(content.contains("total"), "Should document total routed events");
    assert!(content.contains("per_role"), "Should document per-role metrics");
    assert!(content.contains("unique_broadcasts"), "Should document unique broadcasts");
    assert!(content.contains("p95_latency"), "Should document latency metrics");
}

/// Test de validation de la documentation des événements routed
#[test]
fn m7_documentation_routed_events_validation() {
    let supervisor_guide_path = "docs/supervisor-guide.md";
    assert!(Path::new(supervisor_guide_path).exists(), "Supervisor guide should exist");
    
    let content = fs::read_to_string(supervisor_guide_path).expect("Should be able to read supervisor guide");
    
    // Vérifier que les événements routed sont documentés
    assert!(content.contains("routed"), "Should document routed events");
    assert!(content.contains("broadcast_id"), "Should document broadcast_id field");
    assert!(content.contains("message_id"), "Should document message_id field");
    assert!(content.contains("NDJSON"), "Should document NDJSON format");
    
    // Vérifier que la structure des événements est documentée
    assert!(content.contains("ts"), "Should document timestamp field");
    assert!(content.contains("agent_role"), "Should document agent_role field");
    assert!(content.contains("event"), "Should document event field");
}

/// Test de validation de la cohérence de la documentation
#[test]
fn m7_documentation_consistency_validation() {
    let cli_ref_path = "docs/cli-reference.md";
    let workflows_path = "docs/workflows.md";
    let supervisor_guide_path = "docs/supervisor-guide.md";
    
    // Tous les fichiers de documentation doivent exister
    assert!(Path::new(cli_ref_path).exists(), "CLI reference should exist");
    assert!(Path::new(workflows_path).exists(), "Workflows should exist");
    assert!(Path::new(supervisor_guide_path).exists(), "Supervisor guide should exist");
    
    let cli_content = fs::read_to_string(cli_ref_path).expect("Should read CLI reference");
    let workflows_content = fs::read_to_string(workflows_path).expect("Should read workflows");
    let supervisor_content = fs::read_to_string(supervisor_guide_path).expect("Should read supervisor guide");
    
    // Vérifier la cohérence des commandes documentées
    assert!(cli_content.contains("@role") && workflows_content.contains("@role"), 
        "CLI reference and workflows should both document @role");
    assert!(cli_content.contains("@all") && workflows_content.contains("@all"), 
        "CLI reference and workflows should both document @all");
    
    // Vérifier que les exemples sont cohérents
    assert!(cli_content.contains("send --to @backend") && workflows_content.contains("send --to @backend"), 
        "CLI reference and workflows should have consistent examples");
}

/// Test de validation de la complétude de la documentation
#[test]
fn m7_documentation_completeness_validation() {
    let supervisor_guide_path = "docs/supervisor-guide.md";
    assert!(Path::new(supervisor_guide_path).exists(), "Supervisor guide should exist");
    
    let content = fs::read_to_string(supervisor_guide_path).expect("Should be able to read supervisor guide");
    
    // Vérifier que toutes les fonctionnalités M7 sont documentées
    let required_sections = [
        "SupervisorManager",
        "SupervisorSubscription", 
        "RoutedMetrics",
        "subscribe",
        "unsubscribe",
        "tail_and_filter",
        "aggregate_tail",
        "routed_summary",
        "compute_routed_metrics",
        "routed events",
        "broadcast_id",
        "message_id",
        "NDJSON",
        "examples",
        "usage"
    ];
    
    for section in &required_sections {
        assert!(content.contains(section), "Should document {} functionality", section);
    }
    
    // Vérifier que la documentation contient des exemples pratiques
    assert!(content.contains("```"), "Should contain code examples");
    assert!(content.len() > 1000, "Documentation should be comprehensive (at least 1000 characters)");
}

/// Test de validation des exemples fonctionnels
#[test]
fn m7_documentation_functional_examples_validation() {
    let supervisor_guide_path = "docs/supervisor-guide.md";
    assert!(Path::new(supervisor_guide_path).exists(), "Supervisor guide should exist");
    
    let content = fs::read_to_string(supervisor_guide_path).expect("Should be able to read supervisor guide");
    
    // Vérifier que les exemples sont fonctionnels (contiennent du code valide)
    assert!(content.contains("use crate::supervisor"), "Should have import examples");
    assert!(content.contains("SupervisorManager::new()"), "Should have manager creation example");
    assert!(content.contains("SupervisorSubscription::new"), "Should have subscription creation example");
    
    // Vérifier que les exemples de commandes sont présents
    assert!(content.contains("multi-agents send"), "Should have CLI command examples");
    assert!(content.contains("--to @"), "Should have routing command examples");
}

/// Test de validation de la structure de la documentation
#[test]
fn m7_documentation_structure_validation() {
    let supervisor_guide_path = "docs/supervisor-guide.md";
    assert!(Path::new(supervisor_guide_path).exists(), "Supervisor guide should exist");
    
    let content = fs::read_to_string(supervisor_guide_path).expect("Should be able to read supervisor guide");
    
    // Vérifier que la documentation a une structure claire
    assert!(content.contains("# "), "Should have main headings");
    assert!(content.contains("## "), "Should have subheadings");
    assert!(content.contains("### "), "Should have detailed sections");
    
    // Vérifier que les sections importantes sont présentes
    let required_headings = [
        "Overview",
        "SupervisorManager",
        "SupervisorSubscription",
        "RoutedMetrics",
        "Examples",
        "Usage"
    ];
    
    for heading in &required_headings {
        assert!(content.contains(&format!("## {}", heading)) || content.contains(&format!("# {}", heading)), 
            "Should have {} section", heading);
    }
}
