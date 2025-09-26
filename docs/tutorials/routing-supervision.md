# Tutoriel de Routing et Supervision

Ce tutoriel vous guide à travers l'utilisation des fonctionnalités M7 de Multi-Agents CLI, incluant le routing avancé et le système de supervision.

## Configuration du Supervisor

### 1. Configuration de Base

Commencez par créer une configuration de projet avec un agent supervisor :

```yaml
# examples/supervisor-config.yaml
schema_version: 1
project: m7-supervisor-demo
agents:
  - name: supervisor
    role: supervisor
    provider: claude
    model: sonnet-4
    allowed_tools: ["Search", "Edit", "Bash(git:status)"]
    system_prompt: >
      You are a supervisor agent responsible for coordinating other agents,
      monitoring system performance, and routing tasks efficiently.
```

### 2. Configuration du Monitoring

Configurez le monitoring et les métriques dans votre fichier de projet :

```yaml
supervisor:
  monitoring:
    enabled: true
    log_level: "info"
    metrics_collection: true
    
  routing:
    default_timeout: 30000  # 30 seconds
    retry_attempts: 3
    broadcast_mode: "oneshot"
    
  metrics:
    collection_interval: 5000  # 5 seconds
    retention_period: "24h"
    alert_thresholds:
      error_rate: 0.05  # 5%
      response_time_p95: 2000  # 2 seconds
```

## Exemples de Routing

### 1. Routing par Rôle

Envoyez un message à tous les agents d'un rôle spécifique :

```bash
# Envoyer à tous les développeurs backend
multi-agents send --to @backend --message "Veuillez revoir les spécifications de l'API utilisateur"

# Envoyer à tous les développeurs frontend
multi-agents send --to @frontend --message "Implémentez la nouvelle interface de connexion"

# Envoyer à tous les agents DevOps
multi-agents send --to @devops --message "Préparez le déploiement en production"
```

### 2. Broadcast Global

Envoyez un message à tous les agents du projet :

```bash
# Message d'urgence à toute l'équipe
multi-agents send --to @all --message "URGENT: Mise à jour de sécurité requise - arrêtez tous les déploiements"

# Message d'information générale
multi-agents send --to @all --message "Réunion d'équipe demain à 14h - préparez vos rapports de sprint"
```

### 3. Routing Spécifique

Envoyez un message à un agent spécifique :

```bash
# Message au supervisor
multi-agents send --to supervisor --message "Génère un rapport de performance du système"

# Message à un agent spécifique
multi-agents send --to backend-dev --message "Corrige le bug critique dans l'API d'authentification"
```

## Monitoring en Temps Réel

### 1. Surveillance des Événements Routés

Le supervisor surveille automatiquement tous les événements de routing :

```rust
use crate::supervisor::subscription::SupervisorSubscription;

// Créer une subscription pour surveiller les événements
let mut subscription = SupervisorSubscription::new("m7-supervisor-demo".to_string());

// Lire les événements routed du backend
let backend_events = subscription.tail_and_filter(
    "backend".to_string(),
    Some("routed".to_string()),
    100
).expect("Should be able to read backend events");

println!("Backend routed events: {}", backend_events.len());
```

### 2. Agrégation des Logs

Agrégez les logs de plusieurs rôles pour une vue d'ensemble :

```rust
// Agrégation de tous les événements routed
let all_events = subscription.aggregate_tail(
    vec!["backend".to_string(), "frontend".to_string(), "devops".to_string()],
    Some("routed".to_string()),
    100
).expect("Should be able to aggregate all events");

// Les événements sont automatiquement triés par timestamp
for event_line in &all_events {
    println!("Event: {}", event_line);
}
```

### 3. Calcul des Métriques

Calculez des métriques détaillées sur les événements routés :

```rust
use crate::supervisor::metrics;

// Convertir les lignes en objets NdjsonEvent
let events: Vec<crate::logging::events::NdjsonEvent> = all_events.iter()
    .filter_map(|line| serde_json::from_str(line).ok())
    .collect();

// Calculer les métriques
let metrics = metrics::compute_routed_metrics_from_events(events)
    .expect("Should be able to compute routed metrics");

println!("Total routed events: {}", metrics.total);
println!("Unique broadcasts: {}", metrics.unique_broadcasts);
println!("Events per role: {:?}", metrics.per_role);
println!("P95 latency per broadcast: {:?}", metrics.p95_latency_per_broadcast);
println!("Top roles: {:?}", metrics.top_roles);
```

## Cas d'Usage Avancés

### 1. Orchestration de Tâches Complexes

Utilisez le supervisor pour orchestrer des tâches complexes :

```bash
# Étape 1: Demander au supervisor de planifier
multi-agents send --to supervisor --message "Planifie le déploiement de la nouvelle fonctionnalité"

# Étape 2: Le supervisor coordonne avec les équipes
multi-agents send --to @backend --message "Préparez l'API pour la nouvelle fonctionnalité"
multi-agents send --to @frontend --message "Implémentez l'interface utilisateur"
multi-agents send --to @devops --message "Préparez l'infrastructure de déploiement"

# Étape 3: Surveillance et rapport
multi-agents send --to supervisor --message "Génère un rapport de progression"
```

### 2. Gestion des Erreurs

Le supervisor peut détecter et gérer les erreurs :

```bash
# En cas d'erreur, le supervisor peut être alerté
multi-agents send --to supervisor --message "ERREUR: L'API backend ne répond plus"

# Le supervisor coordonne la résolution
multi-agents send --to @backend --message "URGENT: Vérifiez l'état de l'API"
multi-agents send --to @devops --message "Vérifiez les logs du serveur"
```

### 3. Optimisation des Performances

Surveillez les performances avec les métriques du supervisor :

```rust
// Analyser les performances de routing
let metrics = supervisor_manager.routed_summary_from_events(events)?;

// Identifier les goulots d'étranglement
if metrics.p95_latency_per_broadcast.values().any(|&latency| latency > 5000) {
    println!("⚠️  Latence élevée détectée - optimisation requise");
}

// Analyser la répartition des charges
for (role, count) in &metrics.top_roles {
    println!("Rôle {}: {} événements routés", role, count);
}
```

## Exemples Pratiques

### 1. Déploiement d'Application

```bash
# 1. Planification
multi-agents send --to supervisor --message "Planifie le déploiement de l'application v2.1.0"

# 2. Préparation backend
multi-agents send --to @backend --message "Finalisez les tests de l'API v2.1.0"

# 3. Préparation frontend
multi-agents send --to @frontend --message "Finalisez les tests de l'interface v2.1.0"

# 4. Déploiement
multi-agents send --to @devops --message "Déployez l'application v2.1.0 en production"

# 5. Surveillance
multi-agents send --to supervisor --message "Surveille le déploiement et génère un rapport"
```

### 2. Gestion d'Incident

```bash
# 1. Détection d'incident
multi-agents send --to supervisor --message "INCIDENT: Service de paiement indisponible"

# 2. Coordination de la résolution
multi-agents send --to @backend --message "URGENT: Vérifiez le service de paiement"
multi-agents send --to @devops --message "URGENT: Vérifiez l'infrastructure de paiement"

# 3. Communication
multi-agents send --to @all --message "INCIDENT: Service de paiement en cours de résolution"

# 4. Résolution et rapport
multi-agents send --to supervisor --message "Génère un rapport d'incident"
```

### 3. Code Review et Qualité

```bash
# 1. Demande de review
multi-agents send --to @developers --message "Review du code de la fonctionnalité d'authentification"

# 2. Coordination
multi-agents send --to @backend --message "Review l'API d'authentification"
multi-agents send --to @frontend --message "Review l'interface d'authentification"

# 3. Validation
multi-agents send --to supervisor --message "Valide la qualité du code d'authentification"
```

## Bonnes Pratiques

### 1. Messages Efficaces

- **Soyez concis** : Messages courts et clairs
- **Utilisez des actions** : "Implémentez", "Vérifiez", "Génèrez"
- **Spécifiez le contexte** : Incluez les détails nécessaires
- **Priorisez** : Utilisez "URGENT" pour les tâches critiques

### 2. Monitoring Proactif

- **Surveillez régulièrement** : Vérifiez les métriques du supervisor
- **Définissez des alertes** : Configurez des seuils appropriés
- **Analysez les tendances** : Identifiez les patterns de performance
- **Optimisez continuellement** : Améliorez basé sur les métriques

### 3. Coordination d'Équipe

- **Utilisez les rôles** : `@backend`, `@frontend`, `@devops`
- **Leveragez les groupes** : `@developers`, `@all`
- **Communiquez clairement** : Messages structurés et informatifs
- **Suivez les progrès** : Demandez des rapports réguliers

## Dépannage

### Problèmes Courants

1. **Messages non reçus** : Vérifiez la configuration des agents
2. **Latence élevée** : Analysez les métriques de performance
3. **Erreurs de routing** : Vérifiez les noms de rôles et agents
4. **Métriques manquantes** : Vérifiez la configuration du monitoring

### Commandes de Diagnostic

```bash
# Vérifier la configuration
multi-agents config validate --project-file examples/supervisor-config.yaml

# Vérifier l'état des agents
multi-agents agent list --project m7-supervisor-demo

# Vérifier les logs
ls -la logs/m7-supervisor-demo/

# Tester le routing
multi-agents send --to @all --message "Test de routing"
```

Ce tutoriel vous donne les bases pour utiliser efficacement le système de routing et supervision de Multi-Agents CLI. Pour des cas d'usage plus avancés, consultez le guide des cas d'usage avancés.
