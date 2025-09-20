# Analyse Expert Multi-Agents - Analyse Approfondie et Planification

## Description
Commande d'analyse approfondie du projet multi-agents par des experts spécialisés. Cette commande permet à différents rôles d'experts d'analyser le projet, planifier les tâches, détecter les blocages et proposer des améliorations en utilisant tous les outils MCP disponibles.

## Rôles Disponibles
- **architecte** : Analyse architecturale globale et stratégique
- **dev-expert** : Analyse générale du développement
- **backend-expert** : Analyse spécialisée backend (Rust, CLI, SQLite)
- **frontend-expert** : Analyse spécialisée frontend (TUI, ratatui)
- **ui-ux-expert** : Analyse spécialisée UI/UX et expérience utilisateur

## Utilisation
```
/expert-analysis --role <role> [--project <name>] [--priority <level>]
```

## Paramètres
- `--role` : Rôle expert (architecte|dev-expert|backend-expert|frontend-expert|ui-ux-expert)
- `--project` : Nom du projet à analyser (optionnel, défaut: projet courant)
- `--priority` : Niveau de priorité (critical|high|medium|low, défaut: high)

## Sortie
- **Rapport** : Généré automatiquement dans `.reports/analysis-<role>-<date>.md`
- **Console** : Affichage du résumé de l'analyse
- **Logs** : Stockage des observations en mémoire MCP

## Workflow d'Implémentation (Indicatif pour Features/Bugfix/Hotfix)
1. **Explorer** : Analyser le code existant et comprendre les fonctionnalités
2. **Planifier** : Définir les tâches à accomplir et établir un plan d'action
3. **Analyser** : Effectuer l'analyse experte avec tous les outils MCP
4. **Générer** : Créer le rapport d'analyse dans le dossier `.reports/`
5. **Itérer** : Améliorer l'analyse et affiner les recommandations
6. **Finaliser** : Sauvegarder le rapport final localement

**Note** : Les rapports d'analyse sont stockés localement dans `.reports/` et ne sont pas commités. Le workflow Git ci-dessus est indicatif pour les features/bugfix/hotfix du projet.

## Processus d'Analyse Automatisé

### Phase 1: Exploration Initiale (Obligatoire)
- [ ] **Git Analysis** : Vérifier l'état du dépôt, branches, commits récents
- [ ] **Structure Analysis** : Analyser l'architecture et les composants clés
- [ ] **Configuration Review** : Examiner les fichiers YAML, SQLite, config
- [ ] **Dependencies Check** : Vérifier les dépendances et versions
- [ ] **Documentation Scan** : Analyser la documentation existante

### Phase 2: Analyse Technique Spécialisée (Par Rôle)

#### 🏗️ Architecte Expert
- [ ] **Architecture Assessment** : Évaluer la scalabilité et maintenabilité
- [ ] **Pattern Analysis** : Analyser les patterns architecturaux utilisés
- [ ] **Scalability Review** : Identifier les goulots d'étranglement
- [ ] **Security Architecture** : Évaluer la sécurité architecturale
- [ ] **Technology Stack** : Analyser les choix technologiques

#### 💻 Dev Expert
- [ ] **Code Quality** : Analyser la qualité et la lisibilité du code
- [ ] **Testing Coverage** : Évaluer la couverture et qualité des tests
- [ ] **Documentation Quality** : Analyser la documentation technique
- [ ] **Best Practices** : Vérifier l'application des bonnes pratiques
- [ ] **Project Management** : Évaluer les processus de développement

#### 🔧 Backend Expert
- [ ] **CLI Performance** : Analyser les performances du CLI Rust
- [ ] **Database Optimization** : Optimiser les requêtes SQLite
- [ ] **Provider Integration** : Examiner l'intégration des providers
- [ ] **Error Handling** : Analyser la gestion des erreurs
- [ ] **Memory Management** : Évaluer la gestion mémoire

#### 🎨 Frontend Expert
- [ ] **TUI Interface** : Analyser l'interface terminal (ratatui)
- [ ] **Navigation UX** : Évaluer la navigation et l'ergonomie
- [ ] **Performance Rendering** : Analyser les performances de rendu
- [ ] **State Management** : Examiner la gestion d'état
- [ ] **User Experience** : Évaluer l'expérience utilisateur globale

#### 🎯 UI/UX Expert
- [ ] **Usability Analysis** : Analyser la facilité d'utilisation
- [ ] **Accessibility Review** : Évaluer l'accessibilité
- [ ] **Visual Design** : Analyser le design visuel et la cohérence
- [ ] **User Journey** : Examiner les parcours utilisateur
- [ ] **Interaction Design** : Évaluer les patterns d'interaction

### Phase 3: Détection des Problèmes et Blocages
- [ ] **Critical Issues** : Identifier les problèmes critiques (P0)
- [ ] **High Priority Issues** : Détecter les problèmes importants (P1)
- [ ] **Medium Priority Issues** : Identifier les problèmes moyens (P2)
- [ ] **Low Priority Issues** : Détecter les problèmes mineurs (P3)
- [ ] **Technical Debt** : Évaluer la dette technique
- [ ] **Performance Bottlenecks** : Identifier les goulots d'étranglement
- [ ] **Security Vulnerabilities** : Détecter les vulnérabilités

### Phase 4: Planification des Tâches (Par Priorité et Dépendance)
- [ ] **P0 Tasks** : Tâches critiques à traiter immédiatement
- [ ] **P1 Tasks** : Tâches importantes avec dépendances P0
- [ ] **P2 Tasks** : Tâches moyennes avec dépendances P1
- [ ] **P3 Tasks** : Tâches mineures avec dépendances P2
- [ ] **Dependency Mapping** : Cartographier les dépendances entre tâches
- [ ] **Resource Estimation** : Estimer les ressources nécessaires
- [ ] **Timeline Planning** : Planifier les échéances

### Phase 5: Recommandations et Optimisations
- [ ] **Performance Optimizations** : Recommandations de performance
- [ ] **Security Improvements** : Améliorations de sécurité
- [ ] **Code Quality** : Améliorations de qualité de code
- [ ] **Architecture Refactoring** : Refactoring architectural
- [ ] **Tool Recommendations** : Recommandations d'outils
- [ ] **Best Practices** : Bonnes pratiques à implémenter

## Outils MCP Utilisés (Tous Disponibles)

### 🔧 Outils de Base
- **Git** : Analyse du dépôt, branches, commits, diff, historique
- **Memory** : Stockage des observations, décisions, entités, relations
- **Sequential Thinking** : Planification structurée, analyse logique
- **Time** : Horodatage, gestion des délais, planification

### 🔍 Outils d'Exploration
- **Context** : Exploration du codebase, documentation, recherche sémantique
- **Perplexity** : Recherche d'informations externes, bonnes pratiques
- **Playwright** : Tests d'interface, validation UX, tests d'accessibilité

### 📊 Outils d'Analyse
- **Codebase Search** : Recherche sémantique dans le code
- **Grep** : Recherche exacte de patterns et symboles
- **File Operations** : Lecture, écriture, modification de fichiers

## Format de Rapport Markdown

### Structure du Rapport
```markdown
# Rapport d'Analyse Expert - [ROLE] - [DATE]

## 📋 Résumé Exécutif
- [ ] Vue d'ensemble des observations clés
- [ ] Points critiques identifiés
- [ ] Recommandations prioritaires
- [ ] Impact estimé des changements

## 🔍 Analyse Détaillée

### Architecture & Structure
- [ ] Analyse de l'architecture actuelle
- [ ] Patterns identifiés et évaluation
- [ ] Séparation des responsabilités
- [ ] Couplage et cohésion des modules

### Qualité du Code
- [ ] Lisibilité et maintenabilité
- [ ] Conventions et standards
- [ ] Gestion des erreurs
- [ ] Documentation du code

### Tests & Documentation
- [ ] Couverture de tests actuelle
- [ ] Qualité des tests existants
- [ ] Documentation technique
- [ ] Guides et exemples

### Performance & Sécurité
- [ ] Métriques de performance
- [ ] Vulnérabilités identifiées
- [ ] Optimisations possibles
- [ ] Gestion des ressources

## 🚨 Problèmes Identifiés

### Critiques (P0) - À traiter immédiatement
- [ ] Problème critique 1
- [ ] Problème critique 2

### Importants (P1) - À traiter cette semaine
- [ ] Problème important 1
- [ ] Problème important 2

### Moyens (P2) - À traiter ce mois
- [ ] Problème moyen 1
- [ ] Problème moyen 2

### Mineurs (P3) - À traiter plus tard
- [ ] Problème mineur 1
- [ ] Problème mineur 2

## 📅 Plan d'Action (Par Priorité et Dépendance)

### Phase 1: Critique (1-3 jours)
- [ ] Tâche critique 1 (dépend de: aucune)
- [ ] Tâche critique 2 (dépend de: tâche 1)

### Phase 2: Important (1-2 semaines)
- [ ] Tâche importante 1 (dépend de: tâche critique 2)
- [ ] Tâche importante 2 (dépend de: tâche critique 1)

### Phase 3: Moyen (2-4 semaines)
- [ ] Tâche moyenne 1 (dépend de: tâche importante 1)
- [ ] Tâche moyenne 2 (dépend de: tâche importante 2)

### Phase 4: Mineur (1-3 mois)
- [ ] Tâche mineure 1 (dépend de: tâche moyenne 1)
- [ ] Tâche mineure 2 (dépend de: tâche moyenne 2)

## 🛠️ Recommandations Techniques

### Optimisations de Performance
- [ ] Recommandation 1
- [ ] Recommandation 2

### Améliorations de Sécurité
- [ ] Recommandation 1
- [ ] Recommandation 2

### Refactoring Architectural
- [ ] Recommandation 1
- [ ] Recommandation 2

### Outils et Technologies
- [ ] Recommandation 1
- [ ] Recommandation 2

## 📊 Métriques et KPIs

### Métriques Actuelles
- [ ] Métrique 1: Valeur actuelle / Cible
- [ ] Métrique 2: Valeur actuelle / Cible
- [ ] Métrique 3: Valeur actuelle / Cible

### Objectifs de Performance
- [ ] Objectif 1: Amélioration de X%
- [ ] Objectif 2: Réduction de Y%
- [ ] Objectif 3: Augmentation de Z%

## 🎯 Prochaines Étapes

### Actions Immédiates (Cette semaine)
- [ ] Action 1
- [ ] Action 2

### Actions Court Terme (Ce mois)
- [ ] Action 1
- [ ] Action 2

### Actions Long Terme (3+ mois)
- [ ] Action 1
- [ ] Action 2

## 📚 Ressources Nécessaires

### Ressources Humaines
- [ ] Rôle 1: X heures
- [ ] Rôle 2: Y heures

### Ressources Techniques
- [ ] Outil 1: Description
- [ ] Outil 2: Description

### Formation
- [ ] Formation 1: Description
- [ ] Formation 2: Description

## ✅ Conclusion

### Synthèse des Points Clés
- [ ] Point clé 1
- [ ] Point clé 2

### Impact Estimé
- [ ] Impact technique: Description
- [ ] Impact business: Description

### Risques et Mitigation
- [ ] Risque 1: Mitigation
- [ ] Risque 2: Mitigation
```

## Exemples d'Utilisation

### Analyse Architecturale Complète
```
/expert-analysis --role architecte --project multi-agents --priority critical
# Génère: .reports/analysis-architecte-2024-01-15.md
```

### Analyse Backend
```
/expert-analysis --role backend-expert --priority high
# Génère: .reports/analysis-backend-expert-2024-01-15.md
```

### Analyse UI/UX
```
/expert-analysis --role ui-ux-expert --project multi-agents --priority medium
# Génère: .reports/analysis-ui-ux-expert-2024-01-15.md
```

## Intégration avec le Projet

### Conventions Respectées
- [ ] Conventions de branches (feature/, bugfix/, hotfix/, refactor/)
- [ ] Conventions de commits (conventional commits)
- [ ] Architecture hexagonale (Ports & Adapters)
- [ ] Approche API-First avec OpenAPI
- [ ] Workflow TDD (Red-Green-Refactor)

### Outils du Projet
- [ ] CLI orchestrateur Rust
- [ ] Base de données SQLite
- [ ] Configuration YAML
- [ ] Logging NDJSON
- [ ] Gestionnaire tmux

## Notes Importantes
- **Les rapports sont stockés localement dans `.reports/`**
- **Utiliser tous les outils MCP disponibles**
- **Respecter l'ordre de priorité et de dépendance des tâches**
- **Générer un rapport actionnable avec des checkboxes**
- **Persister les observations en mémoire pour référence future**
- **Le workflow Git est indicatif pour les features/bugfix/hotfix du projet**
