# 📋 Milestone M3 - Session Resume - Récapitulatif Détaillé

**Date** : 2025-01-17  
**Auteur** : Développeur Expert  
**Statut** : Planification Complète  
**Basé sur** : [Anthropic Multi-Agent Research System](https://www.anthropic.com/engineering/multi-agent-research-system)

## 🎯 Résumé Exécutif

La milestone M3 implémente la gestion des sessions persistantes dans le CLI multi-agents, en appliquant les principes architecturaux d'Anthropic pour les systèmes multi-agents. Cette implémentation suit le pattern **orchestrateur-worker** où le CLI coordonne les sessions avec les providers spécialisés (Claude, Cursor, Gemini).

### Objectifs Principaux
- ✅ **Commandes** : `session list`, `session resume`
- ✅ **Providers** : Claude `--session-id`, Cursor `--resume`, Gemini ID interne
- ✅ **Performance** : Timeout 5s, concurrency limit 3
- ✅ **Architecture** : Pattern orchestrateur-worker d'Anthropic

## 🏗️ Architecture Multi-Agents

### Principe Anthropic Appliqué
Basé sur l'[article Anthropic](https://www.anthropic.com/engineering/multi-agent-research-system), notre architecture respecte :

1. **Orchestrateur-Worker** : CLI comme orchestrateur, providers comme workers
2. **Séparation des préoccupations** : Chaque provider gère ses sessions
3. **Gestion d'état robuste** : Persistance et validation des sessions
4. **Coordination claire** : Interface unifiée pour tous les providers
5. **Gestion d'erreurs** : Fallback et récupération automatique

### Modèle de Données Étendu
```sql
-- Table sessions étendue
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    agent_id TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    provider_session_id TEXT,  -- Claude session_id, Cursor chat_id, Gemini internal
    created_at TEXT NOT NULL,
    last_activity TEXT,        -- Nouveau: timestamp dernière activité
    status TEXT,               -- Nouveau: active, expired, invalid
    metadata TEXT,             -- Nouveau: JSON pour données provider-spécifiques
    expires_at TEXT            -- Nouveau: timestamp d'expiration
);
```

## 📋 Checklist Détaillée des Tâches

### Phase 1 : Architecture et Base de Données (3-4 jours)

#### 1.1 Extension du Modèle de Données
- [ ] **Ajouter champs à la table `sessions`**
  - [ ] `last_activity: TEXT` (timestamp dernière activité)
  - [ ] `status: TEXT` (active, expired, invalid)
  - [ ] `metadata: TEXT` (JSON pour données provider-spécifiques)
  - [ ] `expires_at: TEXT` (timestamp d'expiration)

- [ ] **Créer index de performance**
  - [ ] Index composite `(project_id, status, created_at)`
  - [ ] Index sur `provider_session_id` pour lookups rapides
  - [ ] Index sur `last_activity` pour nettoyage automatique

- [ ] **Migration de base de données**
  - [ ] Script de migration SQLite
  - [ ] Validation des données existantes
  - [ ] Tests de rollback

#### 1.2 Trait SessionManager
- [ ] **Définir l'interface unifiée**
  ```rust
  pub trait SessionManager {
      fn validate_session(&self, session_id: &str) -> Result<bool, SessionError>;
      fn resume_session(&self, session_id: &str) -> Result<SessionContext, SessionError>;
      fn create_session(&self, agent: &Agent) -> Result<String, SessionError>;
      fn cleanup_expired_sessions(&self) -> Result<u32, SessionError>;
  }
  ```

- [ ] **Types de données associés**
  - [ ] `SessionContext` (contexte de session)
  - [ ] `SessionError` (erreurs spécialisées)
  - [ ] `SessionFilters` (filtres pour list)

### Phase 2 : Implémentation par Provider (4-5 jours)

#### 2.1 ClaudeSessionManager
- [ ] **Réutilisation session_id**
  - [ ] Validation ping avec `--session-id`
  - [ ] Fallback vers création si expirée
  - [ ] Gestion des erreurs de session

- [ ] **Tests spécifiques Claude**
  - [ ] Test validation session valide
  - [ ] Test fallback session expirée
  - [ ] Test création nouvelle session

#### 2.2 CursorSessionManager
- [ ] **Réutilisation chat_id**
  - [ ] Validation avec `--resume <chat_id>`
  - [ ] Fallback vers `create-chat` si invalide
  - [ ] Gestion des erreurs de chat

- [ ] **Tests spécifiques Cursor**
  - [ ] Test validation chat_id valide
  - [ ] Test fallback chat_id invalide
  - [ ] Test création nouveau chat

#### 2.3 GeminiSessionManager
- [ ] **Gestion ID interne**
  - [ ] Génération ID interne unique
  - [ ] Validation contexte disponible
  - [ ] Fallback vers nouvelle session

- [ ] **Tests spécifiques Gemini**
  - [ ] Test génération ID interne
  - [ ] Test validation contexte
  - [ ] Test fallback nouvelle session

### Phase 3 : Commandes CLI (3-4 jours)

#### 3.1 Commande `session list`
- [ ] **Implémentation de base**
  - [ ] Parsing des arguments (project, agent, provider, format)
  - [ ] Récupération des sessions depuis la DB
  - [ ] Formatage de sortie (text/json)

- [ ] **Fonctionnalités avancées**
  - [ ] Pagination (limite 50 par page)
  - [ ] Filtrage par statut
  - [ ] Tri par date de création/activité
  - [ ] Performance < 5s pour 1000 sessions

- [ ] **Tests CLI**
  - [ ] Test list basique
  - [ ] Test filtres multiples
  - [ ] Test pagination
  - [ ] Test performance

#### 3.2 Commande `session resume`
- [ ] **Implémentation de base**
  - [ ] Validation conversation_id
  - [ ] Récupération session depuis DB
  - [ ] Validation session active
  - [ ] Retour contexte de session

- [ ] **Gestion d'erreurs**
  - [ ] Session inexistante
  - [ ] Session expirée
  - [ ] Provider indisponible
  - [ ] Timeout 5s

- [ ] **Tests CLI**
  - [ ] Test resume session valide
  - [ ] Test erreurs session invalide
  - [ ] Test timeout

### Phase 4 : Intégration et Persistance (2-3 jours)

#### 4.1 Modification `run_send`
- [ ] **Support conversation_id**
  - [ ] Détection session existante
  - [ ] Réutilisation provider_session_id
  - [ ] Mise à jour last_activity

- [ ] **Gestion des sessions**
  - [ ] Création automatique si nécessaire
  - [ ] Sauvegarde provider_session_id
  - [ ] Gestion des erreurs de session

#### 4.2 Système de Nettoyage
- [ ] **Nettoyage automatique**
  - [ ] Tâche périodique (cron job)
  - [ ] Suppression sessions expirées
  - [ ] Logs de nettoyage

- [ ] **Métriques de session**
  - [ ] Compteurs par provider
  - [ ] Durée de vie moyenne
  - [ ] Taux d'utilisation

### Phase 5 : Tests et Validation (2-3 jours)

#### 5.1 Tests de Régression
- [ ] **Validation M2**
  - [ ] Tests M2-03 passants
  - [ ] Pas de régression performance
  - [ ] Compatibilité backward

#### 5.2 Tests de Performance
- [ ] **Charge et concurrence**
  - [ ] 1000+ sessions simultanées
  - [ ] Concurrency limit 3 respectée
  - [ ] Timeout 5s respecté

#### 5.3 Tests d'Intégration
- [ ] **End-to-end**
  - [ ] Workflow complet session
  - [ ] Multi-providers simultanés
  - [ ] Gestion d'erreurs robuste

## ⚠️ Points de Blocage Identifiés

### 1. **Gestion des Sessions Expirées** (Critique)
**Problème** : Les sessions Claude/Cursor peuvent expirer sans notification  
**Impact** : Échec des `session resume`  
**Solution** : 
- Validation ping avant réutilisation
- Fallback automatique vers création
- Cache des sessions actives avec TTL

### 2. **Coordination Multi-Providers** (Élevé)
**Problème** : Mécanismes de session différents par provider  
**Impact** : Complexité d'implémentation  
**Solution** :
- Interface unifiée `SessionManager`
- Adaptateurs spécialisés par provider
- Configuration centralisée

### 3. **Performance `session list`** (Moyen)
**Problème** : Lenteur avec beaucoup de sessions  
**Impact** : UX dégradée  
**Solution** :
- Indexation optimisée
- Pagination intelligente
- Cache LRU des sessions actives

### 4. **État des Sessions** (Moyen)
**Problème** : Déterminer si une session est active  
**Impact** : Erreurs de validation  
**Solution** :
- Timestamp `last_activity`
- Validation ping périodique
- Statut explicite (active/expired/invalid)

### 5. **Concurrence et Verrous** (Faible)
**Problème** : Accès simultanés aux sessions  
**Impact** : Corruption de données  
**Solution** :
- Verrous optimistes
- Retry logic
- Transactions atomiques

## 🚀 Améliorations Potentielles

### Court Terme (M3+)
1. **Système Heartbeat**
   - Validation périodique des sessions actives
   - Détection précoce des sessions expirées
   - Métriques de santé en temps réel

2. **Cache Intelligent**
   - Cache LRU des sessions actives
   - Pré-chargement des sessions fréquentes
   - Invalidation automatique

3. **Métriques Avancées**
   - Dashboard des sessions actives
   - Analytics d'utilisation par provider
   - Alertes de performance

### Moyen Terme (M4+)
1. **Sessions Partagées**
   - Partage entre agents du même projet
   - Collaboration multi-agents
   - Gestion des conflits

2. **Backup/Restore**
   - Sauvegarde des sessions critiques
   - Restauration point-in-time
   - Migration entre environnements

3. **Sessions Temporaires**
   - Auto-expiration après inactivité
   - Sessions à durée limitée
   - Nettoyage intelligent

### Long Terme (M5+)
1. **Orchestration Avancée**
   - Délégation dynamique entre agents
   - Load balancing des sessions
   - Failover automatique

2. **Analytics Prédictifs**
   - Prédiction des sessions expirées
   - Optimisation des ressources
   - Recommandations d'usage

## 🎯 Métriques de Succès M3

### Performance
- ✅ `session list` < 5s pour 1000 sessions
- ✅ `session resume` > 95% de succès
- ✅ Timeout 5s respecté
- ✅ Concurrency limit 3 maintenue

### Fonctionnalité
- ✅ Réutilisation effective provider_session_id
- ✅ Pas de régression M2
- ✅ Gestion d'erreurs robuste
- ✅ Support multi-providers complet

### Qualité
- ✅ Tests de non-régression passants
- ✅ Documentation utilisateur complète
- ✅ Code review validé
- ✅ Performance benchmarks respectés

## 📅 Plan d'Exécution Recommandé

### Semaine 1 : Architecture (Phase 1)
- Extension modèle de données
- Trait SessionManager
- Migration et tests

### Semaine 2 : Providers (Phase 2)
- Implémentation ClaudeSessionManager
- Implémentation CursorSessionManager
- Implémentation GeminiSessionManager

### Semaine 3 : CLI (Phase 3)
- Commande `session list`
- Commande `session resume`
- Tests CLI complets

### Semaine 4 : Intégration (Phase 4-5)
- Modification `run_send`
- Tests d'intégration
- Validation finale

## 🔧 Intégration avec les Providers

### Claude Code SDK
Basé sur la [documentation Claude Code](https://docs.claude.com/en/docs/claude-code/sdk/sdk-sessions) :
- **Session ID** : Généré automatiquement et fourni dans le message système
- **Résumption** : Utilisation du paramètre `resume` avec session_id
- **Persistance** : Fichiers JSONL dans `~/.config/claude/projects/{hash}/{session-id}.jsonl`

### Cursor CLI
Basé sur la [documentation Cursor CLI](https://docs.cursor.com/en/cli/overview) :
- **Chat ID** : Généré pour chaque conversation
- **Résumption** : Utilisation de `--resume="chat-id-here"`
- **Liste** : Commande `cursor-agent ls` pour lister les conversations

### Gemini
- **ID Interne** : Génération d'ID unique par le système
- **Contexte** : Validation de la disponibilité du contexte
- **Fallback** : Création de nouvelle session si nécessaire

## 🚨 Risques et Mitigations

| Risque | Impact | Probabilité | Mitigation |
|--------|--------|-------------|------------|
| Sessions expirées | Élevé | Moyen | Validation ping + fallback automatique |
| Performance lente | Moyen | Faible | Indexation + pagination + cache |
| Concurrence | Moyen | Faible | Verrous optimistes + retry logic |
| Complexité utilisateur | Faible | Moyen | Documentation claire + exemples |
| Rétrocompatibilité | Élevé | Faible | Tests de régression complets |

## 📚 Références Multi-Agents

Basé sur l'analyse de l'[article Anthropic](https://www.anthropic.com/engineering/multi-agent-research-system), les principes clés appliqués :

1. **Orchestrateur-Worker** : CLI comme orchestrateur, providers comme workers
2. **Séparation des préoccupations** : Chaque provider gère ses sessions
3. **Gestion d'état robuste** : Persistance et validation des sessions
4. **Coordination claire** : Interface unifiée pour tous les providers
5. **Gestion d'erreurs** : Fallback et récupération automatique
6. **Observabilité** : Logging et métriques détaillés
7. **Résilience** : Gestion des erreurs composées et stateful

## 🔗 Liens Utiles

- [Roadmap complet](docs/roadmap.md)
- [Modèle de données](docs/data-model.md)
- [Référence CLI](docs/cli-reference.md)
- [Sessions et Broadcasts](.cursor/rules/sessions-broadcasts.mdc)
- [Concurrence et Timeouts](.cursor/rules/concurrency-timeouts.mdc)
- [Analyse détaillée M3](docs/m3-session-resume-analysis.md)

---

**Prochaine étape** : Valider ce plan avec l'équipe et commencer l'implémentation par la Phase 1 (Architecture et Base de Données).

## 📊 Résumé des Tâches

| Phase | Tâches | Durée | Priorité |
|-------|--------|-------|----------|
| 1. Architecture | 15 tâches | 3-4 jours | Critique |
| 2. Providers | 18 tâches | 4-5 jours | Critique |
| 3. CLI | 12 tâches | 3-4 jours | Critique |
| 4. Intégration | 8 tâches | 2-3 jours | Élevé |
| 5. Tests | 9 tâches | 2-3 jours | Élevé |
| **Total** | **62 tâches** | **14-19 jours** | |

## 🎯 Points Clés de Succès

1. **Architecture Robuste** : Pattern orchestrateur-worker d'Anthropic
2. **Gestion d'Erreurs** : Fallback automatique et récupération
3. **Performance** : Timeout 5s, concurrency limit 3
4. **Observabilité** : Logging et métriques détaillés
5. **Tests Complets** : Régression, performance, intégration
6. **Documentation** : Utilisateur et technique complète

---

**Statut** : ✅ Planification terminée, prêt pour l'implémentation
