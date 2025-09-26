//! M7 Examples Validation Tests - Tests de validation des exemples d'utilisation M7
//! 
//! Ces tests valident que les exemples d'utilisation M7 sont fonctionnels:
//! - Configuration supervisor avec exemples
//! - Tutoriels de routing et supervision
//! - Cas d'usage avancés
//! - Scripts de démonstration
//! - Validation des exemples de configuration

use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Test de validation des exemples de configuration supervisor
#[test]
fn m7_examples_supervisor_config_validation() {
    // Vérifier que les exemples de configuration supervisor existent
    let supervisor_config_path = "../../examples/supervisor-config.yaml";
    assert!(Path::new(supervisor_config_path).exists(), "Supervisor config example should exist");
    
    let content = fs::read_to_string(supervisor_config_path).expect("Should be able to read supervisor config");
    
    // Vérifier que la configuration contient les éléments clés
    assert!(content.contains("supervisor"), "Should contain supervisor role");
    assert!(content.contains("routing"), "Should contain routing configuration");
    assert!(content.contains("monitoring"), "Should contain monitoring configuration");
    assert!(content.contains("metrics"), "Should contain metrics configuration");
    
    // Vérifier que la configuration est valide YAML
    let yaml: Result<serde_yaml::Value, _> = serde_yaml::from_str(&content);
    assert!(yaml.is_ok(), "Supervisor config should be valid YAML");
}

/// Test de validation des tutoriels de routing et supervision
#[test]
fn m7_examples_routing_tutorial_validation() {
    let tutorial_path = "../../docs/tutorials/routing-supervision.md";
    assert!(Path::new(tutorial_path).exists(), "Routing supervision tutorial should exist");
    
    let content = fs::read_to_string(tutorial_path).expect("Should be able to read tutorial");
    
    // Vérifier que le tutoriel contient les sections clés
    assert!(content.contains("# Tutoriel de Routing et Supervision"), "Should have main title");
    assert!(content.contains("## Configuration du Supervisor"), "Should have supervisor configuration section");
    assert!(content.contains("## Exemples de Routing"), "Should have routing examples section");
    assert!(content.contains("## Monitoring en Temps Réel"), "Should have real-time monitoring section");
    assert!(content.contains("## Cas d'Usage Avancés"), "Should have advanced use cases section");
    
    // Vérifier que le tutoriel contient des exemples de code
    assert!(content.contains("```bash"), "Should contain bash examples");
    assert!(content.contains("```yaml"), "Should contain YAML examples");
    assert!(content.contains("```rust"), "Should contain Rust examples");
    
    // Vérifier que le tutoriel contient des exemples pratiques
    assert!(content.contains("multi-agents send --to @backend"), "Should have backend routing example");
    assert!(content.contains("multi-agents send --to @all"), "Should have broadcast example");
    assert!(content.contains("supervisor"), "Should mention supervisor functionality");
}

/// Test de validation des cas d'usage avancés
#[test]
fn m7_examples_advanced_use_cases_validation() {
    let advanced_cases_path = "../../docs/tutorials/advanced-use-cases.md";
    assert!(Path::new(advanced_cases_path).exists(), "Advanced use cases should exist");
    
    let content = fs::read_to_string(advanced_cases_path).expect("Should be able to read advanced use cases");
    
    // Vérifier que les cas d'usage avancés sont documentés
    assert!(content.contains("# Cas d'Usage Avancés M7"), "Should have main title");
    assert!(content.contains("## Orchestration Multi-Agents"), "Should have multi-agent orchestration");
    assert!(content.contains("## Monitoring Distribué"), "Should have distributed monitoring");
    assert!(content.contains("## Gestion des Erreurs"), "Should have error handling");
    assert!(content.contains("## Optimisation des Performances"), "Should have performance optimization");
    
    // Vérifier que les cas d'usage contiennent des exemples pratiques
    assert!(content.contains("```bash"), "Should contain bash examples");
    assert!(content.contains("```yaml"), "Should contain YAML examples");
    assert!(content.contains("```rust"), "Should contain Rust examples");
    
    // Vérifier que les cas d'usage sont réalistes
    assert!(content.contains("production"), "Should mention production scenarios");
    assert!(content.contains("scaling"), "Should mention scaling scenarios");
    assert!(content.contains("monitoring"), "Should mention monitoring scenarios");
}

/// Test de validation des scripts de démonstration
#[test]
fn m7_examples_demo_scripts_validation() {
    let demo_script_path = "../../scripts/demo-m7-routing.sh";
    assert!(Path::new(demo_script_path).exists(), "M7 routing demo script should exist");
    
    let content = fs::read_to_string(demo_script_path).expect("Should be able to read demo script");
    
    // Vérifier que le script est exécutable
    assert!(content.starts_with("#!/bin/bash"), "Demo script should be a bash script");
    
    // Vérifier que le script contient les démonstrations clés
    assert!(content.contains("echo"), "Should contain echo statements for output");
    assert!(content.contains("multi-agents"), "Should use multi-agents CLI");
    assert!(content.contains("send --to"), "Should demonstrate send command");
    assert!(content.contains("supervisor"), "Should demonstrate supervisor functionality");
    
    // Vérifier que le script contient des commentaires explicatifs
    assert!(content.contains("#"), "Should contain comments");
    assert!(content.contains("Demo"), "Should mention demo purpose");
    assert!(content.contains("M7"), "Should mention M7 functionality");
}

/// Test de validation de la documentation des exemples
#[test]
fn m7_examples_documentation_validation() {
    let examples_readme_path = "../../examples/README.md";
    assert!(Path::new(examples_readme_path).exists(), "Examples README should exist");
    
    let content = fs::read_to_string(examples_readme_path).expect("Should be able to read examples README");
    
    // Vérifier que la documentation des exemples est complète
    assert!(content.contains("# Exemples d'Utilisation M7"), "Should have main title");
    assert!(content.contains("## Configuration Supervisor"), "Should have supervisor configuration section");
    assert!(content.contains("## Tutoriels"), "Should have tutorials section");
    assert!(content.contains("## Scripts de Démonstration"), "Should have demo scripts section");
    assert!(content.contains("## Cas d'Usage Avancés"), "Should have advanced use cases section");
    
    // Vérifier que la documentation contient des liens vers les exemples
    assert!(content.contains("supervisor-config.yaml"), "Should link to supervisor config");
    assert!(content.contains("routing-supervision.md"), "Should link to routing tutorial");
    assert!(content.contains("demo-m7-routing.sh"), "Should link to demo script");
    assert!(content.contains("advanced-use-cases.md"), "Should link to advanced use cases");
    
    // Vérifier que la documentation contient des instructions d'utilisation
    assert!(content.contains("## Comment Utiliser"), "Should have usage instructions");
    assert!(content.contains("## Prérequis"), "Should have prerequisites");
    assert!(content.contains("## Exemples Rapides"), "Should have quick examples");
}

/// Test de validation de la cohérence des exemples
#[test]
fn m7_examples_consistency_validation() {
    // Vérifier que tous les fichiers d'exemples existent
    let required_files = vec![
        "../../examples/supervisor-config.yaml",
        "../../docs/tutorials/routing-supervision.md",
        "../../docs/tutorials/advanced-use-cases.md",
        "../../scripts/demo-m7-routing.sh",
        "../../examples/README.md",
    ];
    
    for file_path in required_files {
        assert!(Path::new(file_path).exists(), "Required example file should exist: {}", file_path);
    }
    
    // Vérifier que les exemples sont cohérents entre eux
    let supervisor_config = fs::read_to_string("../../examples/supervisor-config.yaml").unwrap();
    let tutorial = fs::read_to_string("../../docs/tutorials/routing-supervision.md").unwrap();
    let demo_script = fs::read_to_string("../../scripts/demo-m7-routing.sh").unwrap();
    
    // Vérifier que les noms de projets sont cohérents
    assert!(supervisor_config.contains("demo") || supervisor_config.contains("example"), 
        "Supervisor config should use demo/example project names");
    
    // Vérifier que les commandes sont cohérentes
    assert!(tutorial.contains("multi-agents send --to @backend") && 
            demo_script.contains("multi-agents send --to @backend"), 
        "Tutorial and demo script should use consistent commands");
    
    // Vérifier que les rôles sont cohérents
    assert!(supervisor_config.contains("backend") && 
            tutorial.contains("backend") && 
            demo_script.contains("backend"), 
        "All examples should use consistent role names");
}

/// Test de validation de la fonctionnalité des exemples
#[test]
fn m7_examples_functionality_validation() {
    // Créer un répertoire temporaire pour tester les exemples
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Copier la configuration supervisor dans le répertoire temporaire
    let supervisor_config = fs::read_to_string("../../examples/supervisor-config.yaml").unwrap();
    let config_path = temp_path.join("supervisor-config.yaml");
    fs::write(&config_path, &supervisor_config).unwrap();
    
    // Vérifier que la configuration peut être lue par le CLI
    // (simulation - dans un vrai test, on appellerait le CLI)
    let yaml: Result<serde_yaml::Value, _> = serde_yaml::from_str(&supervisor_config);
    assert!(yaml.is_ok(), "Supervisor config should be parseable");
    
    // Vérifier que la configuration contient les champs requis
    if let Ok(config) = yaml {
        assert!(config.get("project").is_some(), "Config should have project field");
        assert!(config.get("agents").is_some(), "Config should have agents field");
        
        if let Some(agents) = config.get("agents").and_then(|a| a.as_sequence()) {
            assert!(!agents.is_empty(), "Config should have at least one agent");
            
            // Vérifier qu'il y a un agent supervisor
            let has_supervisor = agents.iter().any(|agent| {
                agent.get("role").and_then(|r| r.as_str()) == Some("supervisor")
            });
            assert!(has_supervisor, "Config should have a supervisor agent");
        }
    }
}

/// Test de validation des exemples de monitoring
#[test]
fn m7_examples_monitoring_validation() {
    let tutorial_path = "../../docs/tutorials/routing-supervision.md";
    let content = fs::read_to_string(tutorial_path).expect("Should be able to read tutorial");
    
    // Vérifier que les exemples de monitoring sont présents
    assert!(content.contains("monitoring"), "Should contain monitoring examples");
    assert!(content.contains("metrics"), "Should contain metrics examples");
    assert!(content.contains("logs"), "Should contain log examples");
    assert!(content.contains("supervisor"), "Should contain supervisor examples");
    
    // Vérifier que les exemples de monitoring sont pratiques
    assert!(content.contains("real-time"), "Should mention real-time monitoring");
    assert!(content.contains("aggregation"), "Should mention log aggregation");
    assert!(content.contains("analysis"), "Should mention log analysis");
    
    // Vérifier que les exemples contiennent des commandes utilisables
    assert!(content.contains("multi-agents"), "Should contain multi-agents commands");
    assert!(content.contains("send --to"), "Should contain send commands");
    assert!(content.contains("--message"), "Should contain message examples");
}

/// Test de validation des exemples de configuration avancée
#[test]
fn m7_examples_advanced_config_validation() {
    let advanced_cases_path = "../../docs/tutorials/advanced-use-cases.md";
    let content = fs::read_to_string(advanced_cases_path).expect("Should be able to read advanced use cases");
    
    // Vérifier que les exemples de configuration avancée sont présents
    assert!(content.contains("configuration"), "Should contain configuration examples");
    assert!(content.contains("advanced"), "Should contain advanced examples");
    assert!(content.contains("optimization"), "Should contain optimization examples");
    
    // Vérifier que les exemples sont réalistes et utilisables
    assert!(content.contains("production"), "Should mention production scenarios");
    assert!(content.contains("scaling"), "Should mention scaling scenarios");
    assert!(content.contains("performance"), "Should mention performance scenarios");
    
    // Vérifier que les exemples contiennent des configurations YAML
    assert!(content.contains("```yaml"), "Should contain YAML configuration examples");
    assert!(content.contains("project:"), "Should contain project configuration");
    assert!(content.contains("agents:"), "Should contain agents configuration");
    assert!(content.contains("providers:"), "Should contain providers configuration");
}
