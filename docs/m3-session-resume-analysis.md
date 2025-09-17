# Milestone M3 - Session Resume - Analyse Architecturale

**Date** : 2025-01-17  
**Auteur** : Architecte Logiciel Expert  
**Statut** : Planification  
**Milestone Précédente** : M2-03 ✅ Terminée (Cursor headless stream-json)

## 🎯 Vue d'Ensemble

La milestone M3 implémente la gestion des sessions persistantes et leur reprise dans le système multi-agents CLI. Cette fonctionnalité est critique pour maintenir le contexte des conversations avec les agents AI sur plusieurs interactions.

### Exigences M3
- **Commands** : `session list`, `session resume`
- **Must** : Claude `--session-id` reuse; Cursor `create-chat`/`--resume`; Gemini internal ID
- **Timeout** : 5s per action
- **Concurrency** : Respecter la limite de 3 exécutions parallèles (FIFO)

## 🏗️ Architecture Actuelle

### État des Sessions
```sql
-- Table sessions existante
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    agent_id TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    provider_session_id TEXT,  -- Claude session_id, Cursor chat_id, Gemini internal
    created_at TEXT NOT NULL
);
```

### Commandes Existantes
- ✅ `session start` : Crée une session et retourne `conversation_id`
- ❌ `session list` : **À implémenter**
- ❌ `session resume` : **À implémenter**

## 📋 Tâches à Implémenter

### 1. Commande `session list` (Priorité 1)

```bash
# Structure cible
multi-agents session list --project <name> [--agent <name>] [--provider <prov>] [--format text|json]
```

**Fonctionnalités** :
- Lister toutes les sessions d'un projet
- Filtrage par agent et/ou provider
- Affichage formaté : ID, agent, provider, statut, date création
- Support format text/json
- Performance : < 5s pour 1000 sessions
- Pagination pour grandes volumétries

**Exemple de sortie** :
```
Sessions for project 'demo':
ID                                    Agent    Provider      Status    Created
a1b2c3d4-e5f6-7890-abcd-ef1234567890 backend  cursor-agent  active    2025-01-17T10:30:00Z
b2c3d4e5-f6g7-8901-bcde-f12345678901 frontend claude        active    2025-01-17T11:15:00Z
c3d4e5f6-g7h8-9012-cdef-123456789012 devops   gemini        expired   2025-01-17T09:45:00Z
```

### 2. Commande `session resume` (Priorité 1)

```bash
# Structure cible
multi-agents session resume --conversation-id <id> [--timeout-ms 5000]
```

**Fonctionnalités** :
- Reprendre une session existante par conversation_id
- Validation que la session existe et est active
- Réutilisation du provider_session_id selon le provider
- Gestion des erreurs (session expirée, provider indisponible)
- Timeout configurable (défaut 5s)

**Exemple d'usage** :
```bash
# Reprendre une session
multi-agents session resume --conversation-id a1b2c3d4-e5f6-7890-abcd-ef1234567890

# Envoyer un message à la session reprise
multi-agents send --conversation-id a1b2c3d4-e5f6-7890-abcd-ef1234567890 --message "Continue the discussion"
```

### 3. Logique de Réutilisation par Provider (Priorité 1)

#### Claude
- **Réutilisation** : `--session-id` existant
- **Validation** : Ping de santé avant réutilisation
- **Fallback** : Création nouvelle session si expirée

#### Cursor
- **Réutilisation** : `--resume` avec `chat_id` existant
- **Validation** : Vérification chat_id valide
- **Fallback** : `create-chat` si chat_id invalide

#### Gemini
- **Réutilisation** : ID interne généré (limité)
- **Validation** : Vérification contexte disponible
- **Fallback** : Nouvelle session (pas de persistance native)

### 4. Persistance des Sessions (Priorité 2)

- Sauvegarder les sessions lors des envois
- Mettre à jour les provider_session_id
- Gestion de la durée de vie des sessions
- Nettoyage automatique des sessions expirées

## 🏛️ Architecture Recommandée

### Couche de Persistance

```rust
// crates/db/src/lib.rs - Nouvelles fonctions
pub struct Session {
    pub id: String,
    pub project_id: String,
    pub agent_id: String,
    pub provider: String,
    pub provider_session_id: Option<String>,
    pub created_at: String,
    pub last_activity: Option<String>,
    pub status: SessionStatus,
}

pub enum SessionStatus {
    Active,
    Expired,
    Invalid,
}

// Fonctions CRUD
pub fn insert_session(conn: &Connection, project_id: &str, agent_id: &str, 
                     provider: &str, provider_session_id: Option<&str>) -> Result<Session, DbError>
pub fn find_session(conn: &Connection, session_id: &str) -> Result<Option<Session>, DbError>
pub fn list_sessions(conn: &Connection, project_id: &str, filters: SessionFilters) -> Result<Vec<Session>, DbError>
pub fn update_session(conn: &Connection, session_id: &str, provider_session_id: Option<&str>) -> Result<(), DbError>
pub fn validate_session(conn: &Connection, session_id: &str) -> Result<bool, DbError>
```

### Couche de Service

```rust
// Trait SessionManager
pub trait SessionManager {
    fn validate_session(&self, session_id: &str) -> Result<bool, SessionError>;
    fn resume_session(&self, session_id: &str) -> Result<SessionContext, SessionError>;
    fn create_session(&self, agent: &Agent) -> Result<String, SessionError>;
}

// Implémentations par provider
pub struct ClaudeSessionManager {
    client: ClaudeClient,
}

pub struct CursorSessionManager {
    client: CursorClient,
}

pub struct GeminiSessionManager {
    client: GeminiClient,
}

impl SessionManager for ClaudeSessionManager {
    fn validate_session(&self, session_id: &str) -> Result<bool, SessionError> {
        // Ping Claude avec session_id pour vérifier validité
    }
    
    fn resume_session(&self, session_id: &str) -> Result<SessionContext, SessionError> {
        // Retourner contexte Claude avec session_id
    }
}
```

### Couche CLI

```rust
// Nouvelles commandes dans main.rs
#[derive(Subcommand, Debug)]
enum SessionCmd {
    /// Start a provider session and print conversation_id
    Start {
        #[arg(long, value_name = "PATH")] project_file: Option<String>,
        #[arg(long, value_name = "PATH")] providers_file: Option<String>,
        #[arg(long)] agent: String,
    },
    /// List sessions for a project
    List {
        #[arg(long, value_name = "PATH")] project_file: Option<String>,
        #[arg(long)] project: String,
        #[arg(long)] agent: Option<String>,
        #[arg(long)] provider: Option<String>,
        #[arg(long, value_enum, default_value_t = Format::Text)] format: Format,
    },
    /// Resume an existing session
    Resume {
        #[arg(long)] conversation_id: String,
        #[arg(long, value_name = "MILLIS")] timeout_ms: Option<u64>,
    },
}
```

## ⚠️ Points de Blocage Identifiés

### 1. Gestion des provider_session_id
**Problème** : Les sessions peuvent expirer (Claude, Cursor)  
**Solution** : Validation ping + fallback vers création

### 2. Coordination Multi-Providers
**Problème** : Mécanismes de session différents par provider  
**Solution** : Interface unifiée avec adaptateurs par provider

### 3. Performance `session list`
**Problème** : Lenteur avec beaucoup de sessions  
**Solution** : Indexation + pagination + cache LRU

### 4. État des Sessions
**Problème** : Déterminer si une session est active  
**Solution** : Timestamp dernière activité + validation ping

### 5. Concurrence
**Problème** : Accès simultanés aux sessions  
**Solution** : Verrous optimistes + retry logic

## 🚀 Améliorations Potentielles

### Court Terme
1. **Système heartbeat** : Vérification périodique de la santé des sessions
2. **Cache sessions actives** : Éviter les requêtes DB répétées
3. **Nettoyage automatique** : Supprimer les sessions expirées
4. **Métriques de session** : Tracking de l'utilisation par provider

### Moyen Terme
1. **Sessions partagées** : Partage entre agents du même projet
2. **Backup/restore** : Sauvegarde des sessions critiques
3. **Sessions temporaires** : Auto-expiration après inactivité
4. **Monitoring avancé** : Dashboard des sessions actives

## 📅 Plan d'Implémentation

### Phase 1 : Base de données (1-2 jours)
- [ ] Ajouter fonctions CRUD sessions dans `crates/db/src/lib.rs`
- [ ] Ajouter champ `last_activity` et `status` à la table sessions
- [ ] Créer index sur `(project_id, status, created_at)`
- [ ] Tests unitaires pour la persistance
- [ ] Migration de base de données

### Phase 2 : Gestion par provider (2-3 jours)
- [ ] Créer trait `SessionManager`
- [ ] Implémenter `ClaudeSessionManager` (réutilisation session_id)
- [ ] Implémenter `CursorSessionManager` (réutilisation chat_id)
- [ ] Implémenter `GeminiSessionManager` (ID interne)
- [ ] Tests unitaires pour chaque provider

### Phase 3 : Commandes CLI (1-2 jours)
- [ ] Implémenter `session list` avec filtres et pagination
- [ ] Implémenter `session resume` avec validation
- [ ] Gestion des erreurs et timeouts (5s)
- [ ] Tests d'intégration CLI

### Phase 4 : Intégration (1-2 jours)
- [ ] Modifier `run_send` pour utiliser sessions existantes
- [ ] Ajouter support `--conversation-id` dans `send`
- [ ] Tests d'intégration end-to-end
- [ ] Documentation utilisateur

### Phase 5 : Tests et validation (1 jour)
- [ ] Tests de régression
- [ ] Validation des critères d'acceptation M3
- [ ] Performance testing (1000+ sessions)
- [ ] Tests de charge et concurrence

## 🎯 Métriques de Succès M3

### Performance
- ✅ `session list` répond en < 5s pour 1000 sessions
- ✅ `session resume` réussit > 95% du temps
- ✅ Timeout 5s respecté pour toutes les actions

### Fonctionnalité
- ✅ Réutilisation effective des provider_session_id
- ✅ Pas de régression sur M2
- ✅ Gestion robuste des erreurs de session

### Qualité
- ✅ Tests de non-régression passants
- ✅ Documentation utilisateur complète
- ✅ Code review et validation

## 🚨 Risques et Mitigations

| Risque | Impact | Probabilité | Mitigation |
|--------|--------|-------------|------------|
| Sessions expirées | Élevé | Moyen | Validation ping + fallback automatique |
| Performance lente | Moyen | Faible | Indexation + pagination + cache |
| Concurrence | Moyen | Faible | Verrous optimistes + retry logic |
| Complexité utilisateur | Faible | Moyen | Documentation claire + exemples |
| Rétrocompatibilité | Élevé | Faible | Tests de régression complets |

## 📚 Références Multi-Agents

Basé sur l'analyse de l'article [Anthropic Multi-Agent Research System](https://www.anthropic.com/engineering/multi-agent-research-system), les principes clés appliqués :

1. **Orchestrateur-Worker** : CLI comme orchestrateur, agents comme workers
2. **Séparation des préoccupations** : Chaque provider gère ses propres sessions
3. **Gestion d'état robuste** : Persistance et validation des sessions
4. **Coordination claire** : Interface unifiée pour tous les providers
5. **Gestion d'erreurs** : Fallback et récupération automatique

## 🔗 Liens Utiles

- [Roadmap complet](docs/roadmap.md)
- [Modèle de données](docs/data-model.md)
- [Référence CLI](docs/cli-reference.md)
- [Sessions et Broadcasts](.cursor/rules/sessions-broadcasts.mdc)
- [Concurrence et Timeouts](.cursor/rules/concurrency-timeouts.mdc)

---

**Prochaine étape** : Valider ce plan avec l'équipe et commencer l'implémentation par la Phase 1 (Base de données).
