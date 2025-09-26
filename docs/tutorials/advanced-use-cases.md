# Cas d'Usage Avancés M7

Ce guide présente des cas d'usage avancés pour le système de routing et supervision de Multi-Agents CLI, incluant l'orchestration multi-agents, le monitoring distribué, et l'optimisation des performances.

## Exemples Avancés

Les exemples suivants démontrent des scénarios d'usage avancés du système M7.

## Configuration Avancée

### Configuration des Providers

Configurez des providers multiples avec des paramètres spécifiques pour chaque environnement :

```yaml
# Configuration avancée des providers
providers:
  claude:
    cmd: "claude"
    oneshot_args: ["-p", "--print", "--output-format", "text", "{prompt}"]
    repl_args: ["repl"]
    timeout_ms: 30000
    retry_attempts: 3
  gemini:
    cmd: "gemini"
    oneshot_args: ["{prompt}"]
    repl_args: ["-i", "{system_prompt}"]
    timeout_ms: 20000
    retry_attempts: 2
```

## Orchestration Multi-Agents

### 1. Workflow de Développement Complexe

Orchestrez un workflow de développement complet avec coordination entre équipes :

```bash
# Phase 1: Planification et Architecture
multi-agents send --to supervisor --message "Planifie l'architecture de la nouvelle fonctionnalité de paiement"

# Phase 2: Développement Backend
multi-agents send --to @backend --message "Conçois l'API de paiement avec Stripe et PayPal"
multi-agents send --to @backend --message "Implémente les webhooks de paiement"
multi-agents send --to @backend --message "Ajoute les tests unitaires et d'intégration"

# Phase 3: Développement Frontend
multi-agents send --to @frontend --message "Conçois l'interface de paiement responsive"
multi-agents send --to @frontend --message "Implémente l'intégration Stripe Elements"
multi-agents send --to @frontend --message "Ajoute les tests E2E pour le flux de paiement"

# Phase 4: Infrastructure et Déploiement
multi-agents send --to @devops --message "Configure l'infrastructure pour les paiements"
multi-agents send --to @devops --message "Met en place le monitoring des transactions"
multi-agents send --to @devops --message "Prépare le déploiement en staging puis production"

# Phase 5: Validation et Surveillance
multi-agents send --to supervisor --message "Valide l'implémentation complète et génère un rapport"
```

### 2. Gestion de Projet Agile

Implémentez une méthodologie Agile avec coordination automatique :

```yaml
# Configuration pour gestion Agile
supervisor:
  agile:
    sprints:
      duration_days: 14
      planning_enabled: true
      retrospective_enabled: true
    
    roles:
      scrum_master: "supervisor"
      product_owner: "supervisor"
      developers: ["backend-dev", "frontend-dev"]
      qa: ["qa-engineer"]
      devops: ["devops-engineer"]
    
    ceremonies:
      daily_standup: "09:00"
      sprint_planning: "monday"
      sprint_review: "friday"
      retrospective: "friday"
```

```bash
# Sprint Planning
multi-agents send --to supervisor --message "Démarre le sprint planning pour le sprint 15"

# Daily Standup
multi-agents send --to @developers --message "Daily standup - partagez vos progrès et blocages"

# Sprint Review
multi-agents send --to @all --message "Sprint review - démonstration des fonctionnalités terminées"

# Retrospective
multi-agents send --to @all --message "Rétrospective - que s'est-il bien passé et que peut-on améliorer ?"
```

## Monitoring Distribué

### 1. Surveillance Multi-Environnements

Surveillez plusieurs environnements simultanément :

```rust
use crate::supervisor::subscription::SupervisorSubscription;

// Configuration pour monitoring multi-environnements
let environments = vec!["development", "staging", "production"];
let mut subscriptions = Vec::new();

for env in environments {
    let mut subscription = SupervisorSubscription::new(format!("m7-demo-{}", env));
    subscriptions.push(subscription);
}

// Surveillance en temps réel de tous les environnements
for (i, mut subscription) in subscriptions.iter_mut().enumerate() {
    let env = &environments[i];
    let events = subscription.aggregate_tail(
        vec!["backend".to_string(), "frontend".to_string(), "devops".to_string()],
        Some("routed".to_string()),
        100
    ).expect("Should aggregate events");
    
    println!("Environment {}: {} events", env, events.len());
}
```

### 2. Alerting Intelligent

Implémentez un système d'alerting basé sur les métriques :

```rust
use crate::supervisor::metrics;

// Configuration des seuils d'alerte
struct AlertThresholds {
    error_rate: f64,
    response_time_p95: u64,
    throughput_min: usize,
}

impl AlertThresholds {
    fn production() -> Self {
        Self {
            error_rate: 0.01,      // 1%
            response_time_p95: 1000, // 1 second
            throughput_min: 100,   // 100 events/min
        }
    }
    
    fn staging() -> Self {
        Self {
            error_rate: 0.05,      // 5%
            response_time_p95: 2000, // 2 seconds
            throughput_min: 50,    // 50 events/min
        }
    }
}

// Fonction d'alerte
fn check_alerts(metrics: &metrics::RoutedMetrics, thresholds: &AlertThresholds) {
    // Vérifier le taux d'erreur
    let error_rate = calculate_error_rate(metrics);
    if error_rate > thresholds.error_rate {
        println!("🚨 ALERT: Error rate {}% exceeds threshold {}%", 
                error_rate * 100.0, thresholds.error_rate * 100.0);
    }
    
    // Vérifier la latence P95
    for (broadcast_id, latency) in &metrics.p95_latency_per_broadcast {
        if *latency > thresholds.response_time_p95 {
            println!("🚨 ALERT: P95 latency {}ms exceeds threshold {}ms for broadcast {}", 
                    latency, thresholds.response_time_p95, broadcast_id);
        }
    }
    
    // Vérifier le débit
    if metrics.total < thresholds.throughput_min {
        println!("🚨 ALERT: Throughput {} events below minimum {}", 
                metrics.total, thresholds.throughput_min);
    }
}
```

### 3. Dashboard de Monitoring

Créez un dashboard de monitoring en temps réel :

```rust
use std::collections::HashMap;

struct MonitoringDashboard {
    metrics_history: Vec<metrics::RoutedMetrics>,
    alert_history: Vec<String>,
    environment_status: HashMap<String, String>,
}

impl MonitoringDashboard {
    fn new() -> Self {
        Self {
            metrics_history: Vec::new(),
            alert_history: Vec::new(),
            environment_status: HashMap::new(),
        }
    }
    
    fn update_metrics(&mut self, new_metrics: metrics::RoutedMetrics) {
        self.metrics_history.push(new_metrics);
        
        // Garder seulement les 100 dernières métriques
        if self.metrics_history.len() > 100 {
            self.metrics_history.remove(0);
        }
        
        // Vérifier les alertes
        self.check_and_alert();
    }
    
    fn check_and_alert(&mut self) {
        if let Some(latest) = self.metrics_history.last() {
            let thresholds = AlertThresholds::production();
            check_alerts(latest, &thresholds);
        }
    }
    
    fn generate_report(&self) -> String {
        let mut report = String::new();
        
        if let Some(latest) = self.metrics_history.last() {
            report.push_str(&format!("📊 Monitoring Report\n"));
            report.push_str(&format!("Total Events: {}\n", latest.total));
            report.push_str(&format!("Unique Broadcasts: {}\n", latest.unique_broadcasts));
            report.push_str(&format!("Top Roles: {:?}\n", latest.top_roles));
            
            // Calculer les tendances
            if self.metrics_history.len() >= 2 {
                let previous = &self.metrics_history[self.metrics_history.len() - 2];
                let trend = latest.total as i32 - previous.total as i32;
                report.push_str(&format!("Trend: {:+} events\n", trend));
            }
        }
        
        report
    }
}
```

## Gestion des Erreurs

### 1. Circuit Breaker Pattern

Implémentez un pattern circuit breaker pour la résilience :

```rust
use std::time::{Duration, Instant};

enum CircuitState {
    Closed,    // Normal operation
    Open,      // Circuit is open, failing fast
    HalfOpen,  // Testing if service is back
}

struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    failure_threshold: u32,
    timeout: Duration,
    last_failure_time: Option<Instant>,
}

impl CircuitBreaker {
    fn new(failure_threshold: u32, timeout: Duration) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold,
            timeout,
            last_failure_time: None,
        }
    }
    
    fn call<F, T>(&mut self, operation: F) -> Result<T, String>
    where
        F: FnOnce() -> Result<T, String>,
    {
        match self.state {
            CircuitState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.timeout {
                        self.state = CircuitState::HalfOpen;
                    } else {
                        return Err("Circuit breaker is open".to_string());
                    }
                }
            }
            CircuitState::HalfOpen => {
                // Allow one request to test if service is back
            }
            CircuitState::Closed => {
                // Normal operation
            }
        }
        
        match operation() {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(error) => {
                self.on_failure();
                Err(error)
            }
        }
    }
    
    fn on_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitState::Closed;
    }
    
    fn on_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());
        
        if self.failure_count >= self.failure_threshold {
            self.state = CircuitState::Open;
        }
    }
}
```

### 2. Retry avec Backoff Exponentiel

Implémentez un système de retry intelligent :

```rust
use std::thread;
use std::time::Duration;

struct RetryConfig {
    max_attempts: u32,
    base_delay: Duration,
    max_delay: Duration,
    backoff_multiplier: f64,
}

impl RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
        }
    }
}

fn retry_with_backoff<F, T>(
    operation: F,
    config: RetryConfig,
) -> Result<T, String>
where
    F: Fn() -> Result<T, String>,
{
    let mut delay = config.base_delay;
    
    for attempt in 1..=config.max_attempts {
        match operation() {
            Ok(result) => return Ok(result),
            Err(error) => {
                if attempt == config.max_attempts {
                    return Err(format!("Failed after {} attempts: {}", config.max_attempts, error));
                }
                
                println!("Attempt {} failed: {}. Retrying in {:?}...", attempt, error, delay);
                thread::sleep(delay);
                
                delay = Duration::from_millis(
                    (delay.as_millis() as f64 * config.backoff_multiplier) as u64
                ).min(config.max_delay);
            }
        }
    }
    
    Err("Unexpected error in retry logic".to_string())
}
```

## Optimisation des Performances

### 1. Cache Intelligent

Implémentez un cache pour optimiser les performances :

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

struct CacheEntry<T> {
    value: T,
    created_at: Instant,
    ttl: Duration,
}

impl<T> CacheEntry<T> {
    fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.ttl
    }
}

struct IntelligentCache<T> {
    entries: HashMap<String, CacheEntry<T>>,
    default_ttl: Duration,
}

impl<T: Clone> IntelligentCache<T> {
    fn new(default_ttl: Duration) -> Self {
        Self {
            entries: HashMap::new(),
            default_ttl,
        }
    }
    
    fn get(&mut self, key: &str) -> Option<T> {
        if let Some(entry) = self.entries.get(key) {
            if entry.is_expired() {
                self.entries.remove(key);
                None
            } else {
                Some(entry.value.clone())
            }
        } else {
            None
        }
    }
    
    fn set(&mut self, key: String, value: T) {
        self.set_with_ttl(key, value, self.default_ttl);
    }
    
    fn set_with_ttl(&mut self, key: String, value: T, ttl: Duration) {
        let entry = CacheEntry {
            value,
            created_at: Instant::now(),
            ttl,
        };
        self.entries.insert(key, entry);
    }
    
    fn cleanup_expired(&mut self) {
        self.entries.retain(|_, entry| !entry.is_expired());
    }
}
```

### 2. Load Balancing

Implémentez un load balancer pour distribuer la charge :

```rust
use std::collections::HashMap;

struct LoadBalancer {
    agents: Vec<String>,
    current_index: usize,
    agent_loads: HashMap<String, u32>,
}

impl LoadBalancer {
    fn new(agents: Vec<String>) -> Self {
        Self {
            agent_loads: agents.iter().map(|agent| (agent.clone(), 0)).collect(),
            agents,
            current_index: 0,
        }
    }
    
    fn get_next_agent(&mut self) -> String {
        // Round-robin avec considération de la charge
        let mut best_agent = &self.agents[0];
        let mut min_load = u32::MAX;
        
        for agent in &self.agents {
            let load = self.agent_loads.get(agent).unwrap_or(&0);
            if *load < min_load {
                min_load = *load;
                best_agent = agent;
            }
        }
        
        // Incrémenter la charge de l'agent sélectionné
        *self.agent_loads.get_mut(best_agent).unwrap() += 1;
        
        best_agent.clone()
    }
    
    fn complete_task(&mut self, agent: &str) {
        if let Some(load) = self.agent_loads.get_mut(agent) {
            *load = load.saturating_sub(1);
        }
    }
    
    fn get_agent_loads(&self) -> &HashMap<String, u32> {
        &self.agent_loads
    }
}
```

### 3. Optimisation des Métriques

Optimisez le calcul des métriques pour de gros volumes :

```rust
use std::collections::HashMap;

struct OptimizedMetricsCalculator {
    event_cache: HashMap<String, Vec<crate::logging::events::NdjsonEvent>>,
    metrics_cache: HashMap<String, metrics::RoutedMetrics>,
    cache_ttl: Duration,
}

impl OptimizedMetricsCalculator {
    fn new() -> Self {
        Self {
            event_cache: HashMap::new(),
            metrics_cache: HashMap::new(),
            cache_ttl: Duration::from_secs(60),
        }
    }
    
    fn calculate_metrics_cached(
        &mut self,
        project_id: &str,
        events: Vec<crate::logging::events::NdjsonEvent>,
    ) -> Result<metrics::RoutedMetrics, Box<dyn std::error::Error>> {
        // Vérifier le cache
        if let Some(cached_metrics) = self.metrics_cache.get(project_id) {
            return Ok(cached_metrics.clone());
        }
        
        // Calculer les métriques
        let metrics = metrics::compute_routed_metrics_from_events(events)?;
        
        // Mettre en cache
        self.metrics_cache.insert(project_id.to_string(), metrics.clone());
        
        Ok(metrics)
    }
    
    fn invalidate_cache(&mut self, project_id: &str) {
        self.metrics_cache.remove(project_id);
        self.event_cache.remove(project_id);
    }
    
    fn cleanup_expired_cache(&mut self) {
        // Implémentation simplifiée - dans un vrai système, 
        // vous utiliseriez des timestamps pour TTL
        if self.metrics_cache.len() > 100 {
            self.metrics_cache.clear();
            self.event_cache.clear();
        }
    }
}
```

## Exemples de Configuration Avancée

### 1. Configuration Multi-Environnements

```yaml
# Configuration pour environnements multiples
environments:
  development:
    project: m7-demo-dev
    supervisor:
      monitoring:
        log_level: "debug"
        metrics_collection: true
      routing:
        default_timeout: 10000
        retry_attempts: 1
      metrics:
        collection_interval: 1000
        retention_period: "1h"
        
  staging:
    project: m7-demo-staging
    supervisor:
      monitoring:
        log_level: "info"
        metrics_collection: true
      routing:
        default_timeout: 20000
        retry_attempts: 2
      metrics:
        collection_interval: 5000
        retention_period: "12h"
        
  production:
    project: m7-demo-prod
    supervisor:
      monitoring:
        log_level: "warn"
        metrics_collection: true
      routing:
        default_timeout: 30000
        retry_attempts: 3
      metrics:
        collection_interval: 10000
        retention_period: "24h"
        alert_thresholds:
          error_rate: 0.01
          response_time_p95: 1000
          throughput_min: 100
```

### 2. Configuration de Scaling

```yaml
# Configuration pour scaling automatique
scaling:
  enabled: true
  min_agents: 2
  max_agents: 10
  scale_up_threshold: 0.8  # 80% CPU ou mémoire
  scale_down_threshold: 0.3  # 30% CPU ou mémoire
  cooldown_period: 300  # 5 minutes
  
  metrics:
    - cpu_usage
    - memory_usage
    - response_time
    - throughput
    
  policies:
    - name: "high_load"
      condition: "cpu_usage > 0.8 OR response_time > 2000"
      action: "scale_up"
      target: 2
      
    - name: "low_load"
      condition: "cpu_usage < 0.3 AND response_time < 500"
      action: "scale_down"
      target: 1
```

### 3. Configuration de Sécurité

```yaml
# Configuration de sécurité avancée
security:
  authentication:
    enabled: true
    method: "jwt"
    token_expiry: 3600  # 1 hour
    
  authorization:
    enabled: true
    roles:
      admin: ["*"]
      supervisor: ["monitor", "route", "metrics"]
      developer: ["route", "view_metrics"]
      viewer: ["view_metrics"]
      
  encryption:
    enabled: true
    algorithm: "AES-256-GCM"
    key_rotation: 86400  # 24 hours
    
  audit:
    enabled: true
    log_all_actions: true
    retention_period: "90d"
    
  rate_limiting:
    enabled: true
    requests_per_minute: 100
    burst_size: 20
```

## Bonnes Pratiques Avancées

### 1. Monitoring Proactif

- **Surveillance continue** : Surveillez les métriques en temps réel
- **Alertes prédictives** : Utilisez des modèles ML pour prédire les problèmes
- **Dashboards personnalisés** : Créez des vues adaptées à vos besoins
- **Corrélation d'événements** : Corrélez les événements entre systèmes

### 2. Optimisation des Performances

- **Cache intelligent** : Utilisez des caches avec TTL appropriés
- **Load balancing** : Distribuez la charge efficacement
- **Compression** : Compressez les données de monitoring
- **Batch processing** : Traitez les métriques par lots

### 3. Gestion de la Résilience

- **Circuit breakers** : Protégez contre les cascades de pannes
- **Retry intelligent** : Implémentez des stratégies de retry adaptatives
- **Fallback mechanisms** : Prévoyez des mécanismes de secours
- **Health checks** : Surveillez la santé des composants

### 4. Sécurité et Conformité

- **Chiffrement** : Chiffrez les données sensibles
- **Authentification** : Implémentez une authentification robuste
- **Audit** : Enregistrez toutes les actions importantes
- **Conformité** : Respectez les réglementations (GDPR, SOX, etc.)

Ce guide vous donne les outils nécessaires pour implémenter des solutions avancées avec Multi-Agents CLI. Adaptez ces exemples à vos besoins spécifiques et n'hésitez pas à les étendre selon vos exigences.
