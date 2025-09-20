# Commandes Cursor - Multi-Agents Project

## Description
Collection de commandes Cursor pour l'analyse experte et la gestion du projet multi-agents.

## Commandes Disponibles

### 🔍 Analyse Expert Complète
- **Fichier**: `expert-analysis.md`
- **Description**: Commande principale d'analyse approfondie par des experts spécialisés
- **Utilisation**: `/expert-analysis --role <role> [--project <name>] [--priority <level>]`
- **Rôles**: architecte, dev-expert, backend-expert, frontend-expert, ui-ux-expert

### 🛠️ Implémentation Détaillée
- **Fichier**: `expert-analysis-implementation.md`
- **Description**: Script d'implémentation automatisée utilisant tous les outils MCP
- **Workflow**: explore > plan > analyse > génère > itère > finalise (rapports dans .reports/)

### 📋 Exemple d'Utilisation
- **Fichier**: `expert-analysis-example.md`
- **Description**: Exemple complet d'utilisation avec tous les outils MCP
- **Inclut**: Exemple d'analyse architecturale complète

### ⚡ Commandes Rapides
- **Fichier**: `quick-analysis.md`
- **Description**: Commandes prêtes à l'emploi pour différents scénarios
- **Inclut**: Alias, scripts, et workflows recommandés

### 🏗️ Rôles Experts Spécialisés

#### Architecte Expert
- **Fichier**: `architecte-expert.md`
- **Focus**: Architecture globale, scalabilité, maintenabilité, patterns
- **Outils**: Context, Perplexity, Memory, Sequential Thinking

#### Dev Expert
- **Fichier**: `dev-expert.md`
- **Focus**: Qualité du code, tests, documentation, bonnes pratiques
- **Outils**: Context, Memory, Sequential Thinking, Git

#### Backend Expert
- **Fichier**: `backend-expert.md`
- **Focus**: CLI Rust, SQLite, providers, performance
- **Outils**: Context, Sequential Thinking, Memory, Git

#### Frontend Expert
- **Fichier**: `frontend-expert.md`
- **Focus**: Interface TUI, navigation, UX, performance
- **Outils**: Context, Playwright, Memory, Sequential Thinking

#### UI/UX Expert
- **Fichier**: `ui-ux-expert.md`
- **Focus**: Ergonomie, accessibilité, design, expérience utilisateur
- **Outils**: Context, Playwright, Memory, Sequential Thinking

## Outils MCP Utilisés

### 🔧 Outils de Base
- **Git**: Analyse du dépôt, branches, commits, diff, historique
- **Memory**: Stockage des observations, décisions, entités, relations
- **Sequential Thinking**: Planification structurée, analyse logique
- **Time**: Horodatage, gestion des délais, planification

### 🔍 Outils d'Exploration
- **Context**: Exploration du codebase, documentation, recherche sémantique
- **Perplexity**: Recherche d'informations externes, bonnes pratiques
- **Playwright**: Tests d'interface, validation UX, tests d'accessibilité

### 📊 Outils d'Analyse
- **Codebase Search**: Recherche sémantique dans le code
- **Grep**: Recherche exacte de patterns et symboles
- **File Operations**: Lecture, écriture, modification de fichiers

## Workflow d'Implémentation

### Phase 1: Initialisation
1. **Créer branche**: `git checkout -b feature/analysis-<role>-<date>`
2. **Horodatage**: Obtenir l'heure actuelle
3. **Vérification Git**: Analyser l'état du dépôt

### Phase 2: Exploration
1. **Analyse Git**: Historique, branches, différences
2. **Exploration Codebase**: Structure, composants, documentation
3. **Stockage Mémoire**: Créer entités et relations

### Phase 3: Planification
1. **Sequential Thinking**: Planification structurée
2. **Analyse par Rôle**: Focus spécialisé selon le rôle
3. **Détection Problèmes**: Identification des issues

### Phase 4: Planification des Tâches
1. **Priorisation**: P0 (critique) → P1 (important) → P2 (moyen) → P3 (mineur)
2. **Dépendances**: Cartographier les dépendances entre tâches
3. **Ressources**: Estimer les ressources nécessaires

### Phase 5: Génération Rapport
1. **Template Markdown**: Rapport structuré avec checkboxes
2. **Métriques**: KPIs et objectifs de performance
3. **Recommandations**: Actions techniques et stratégiques

### Phase 6: Tests (Si Applicable)
1. **Playwright**: Tests d'interface et accessibilité
2. **Validation**: Vérification des recommandations

### Phase 7: Workflow Git
1. **Commit**: Message conventionnel avec détails
2. **Push**: Pousser la branche
3. **MR**: Créer la merge request avec template

## Format de Rapport

### Structure Standard
```markdown
# Rapport d'Analyse Expert - [ROLE] - [DATE]

## 📋 Résumé Exécutif
- [ ] Vue d'ensemble des observations clés
- [ ] Points critiques identifiés
- [ ] Recommandations prioritaires
- [ ] Impact estimé des changements

## 🔍 Analyse Détaillée
### Architecture & Structure
### Qualité du Code
### Tests & Documentation
### Performance & Sécurité

## 🚨 Problèmes Identifiés
### Critiques (P0) - À traiter immédiatement
### Importants (P1) - À traiter cette semaine
### Moyens (P2) - À traiter ce mois
### Mineurs (P3) - À traiter plus tard

## 📅 Plan d'Action (Par Priorité et Dépendance)
### Phase 1: Critique (1-3 jours)
### Phase 2: Important (1-2 semaines)
### Phase 3: Moyen (2-4 semaines)
### Phase 4: Mineur (1-3 mois)

## 🛠️ Recommandations Techniques
### Optimisations de Performance
### Améliorations de Sécurité
### Refactoring Architectural
### Outils et Technologies

## 📊 Métriques et KPIs
### Métriques Actuelles
### Objectifs de Performance

## 🎯 Prochaines Étapes
### Actions Immédiates (Cette semaine)
### Actions Court Terme (Ce mois)
### Actions Long Terme (3+ mois)

## 📚 Ressources Nécessaires
### Ressources Humaines
### Ressources Techniques
### Formation

## ✅ Conclusion
### Synthèse des Points Clés
### Impact Estimé
### Risques et Mitigation
```

## Exemples d'Utilisation

### Analyse Architecturale
```bash
/expert-analysis --role architecte --project multi-agents --priority critical
# Génère: .reports/analysis-architecte-2024-01-15.md
```

### Analyse Backend
```bash
/expert-analysis --role backend-expert --project multi-agents --priority high
# Génère: .reports/analysis-backend-expert-2024-01-15.md
```

### Analyse Frontend
```bash
/expert-analysis --role frontend-expert --project multi-agents --priority medium
# Génère: .reports/analysis-frontend-expert-2024-01-15.md
```

### Analyse UI/UX
```bash
/expert-analysis --role ui-ux-expert --project multi-agents --priority medium
# Génère: .reports/analysis-ui-ux-expert-2024-01-15.md
```

### Analyse Dev
```bash
/expert-analysis --role dev-expert --project multi-agents --priority high
# Génère: .reports/analysis-dev-expert-2024-01-15.md
```

## Conventions Respectées

### Rapports
- **Localisation** : `.reports/analysis-<role>-<date>.md`
- **Format** : Markdown avec checkboxes et priorités
- **Stockage** : Local uniquement (non commités)

### Workflow Git (Indicatif pour Features/Bugfix/Hotfix)
- **Branches** : `feature/`, `bugfix/`, `hotfix/`, `refactor/`
- **Commits** : Conventional commits
- **MR** : Avec template et labels appropriés

## Intégration avec le Projet

### Outils du Projet
- **CLI orchestrateur Rust**: `crates/cli/src/main.rs`
- **Base de données SQLite**: `./data/multi-agents.sqlite3`
- **Configuration YAML**: `config/`
- **Logging NDJSON**: `logs/`
- **Gestionnaire tmux**: Intégration pour sessions agents

### Documentation
- **Overview**: `docs/overview.md`
- **Product Brief**: `docs/product-brief.md`
- **Data Model**: `docs/data-model.md`
- **CLI Reference**: `docs/cli-reference.md`
- **Roadmap**: `docs/roadmap.md`

## Notes Importantes

- **Toujours créer une branche avant l'analyse**
- **Utiliser tous les outils MCP disponibles**
- **Respecter l'ordre de priorité et de dépendance des tâches**
- **Générer un rapport actionnable avec des checkboxes**
- **Persister les observations en mémoire pour référence future**
- **Suivre le workflow d'implémentation strict**
- **Respecter les conventions de branches et commits du projet**

## Support et Maintenance

### Mise à Jour
Les commandes sont maintenues et mises à jour selon l'évolution du projet.

### Personnalisation
Chaque commande peut être personnalisée selon les besoins spécifiques du projet.

### Feedback
Les retours d'expérience sont intégrés pour améliorer continuellement les commandes.

## Références

- [Documentation Cursor Commands](https://cursor.com/fr/docs/agent/chat/commands)
- [Conventions du Projet](.cursor/rules/)
- [Workflow de Développement](docs/workflows.md)
- [Architecture du Projet](docs/overview.md)
