# Analyse Rapide Expert - Commandes Prêtes à l'Emploi

## Description
Commandes rapides pour lancer des analyses expertes avec tous les outils MCP disponibles.

## Commandes Prêtes à l'Emploi

### 🏗️ Analyse Architecturale Complète
```
/expert-analysis --role architecte --project multi-agents --priority critical
# Génère: .reports/analysis-architecte-$(date +%Y-%m-%d).md
```

### 💻 Analyse Développement Générale
```
/expert-analysis --role dev-expert --project multi-agents --priority high
# Génère: .reports/analysis-dev-expert-$(date +%Y-%m-%d).md
```

### 🔧 Analyse Backend Rust
```
/expert-analysis --role backend-expert --project multi-agents --priority high
# Génère: .reports/analysis-backend-expert-$(date +%Y-%m-%d).md
```

### 🎨 Analyse Frontend TUI
```
/expert-analysis --role frontend-expert --project multi-agents --priority medium
# Génère: .reports/analysis-frontend-expert-$(date +%Y-%m-%d).md
```

### 🎯 Analyse UI/UX
```
/expert-analysis --role ui-ux-expert --project multi-agents --priority medium
# Génère: .reports/analysis-ui-ux-expert-$(date +%Y-%m-%d).md
```

## Analyse Multi-Rôles (Tous les Experts)

### Lancement Séquentiel
```bash
# Analyser avec tous les rôles
for role in architecte dev-expert backend-expert frontend-expert ui-ux-expert; do
  echo "Lancement de l'analyse $role..."
  /expert-analysis --role $role --project multi-agents --priority high
  # Génère: .reports/analysis-${role}-$(date +%Y-%m-%d).md
done
```

### Lancement Parallèle (Si Supporté)
```bash
# Analyser avec tous les rôles en parallèle
/expert-analysis --role architecte --project multi-agents --priority critical &
/expert-analysis --role dev-expert --project multi-agents --priority high &
/expert-analysis --role backend-expert --project multi-agents --priority high &
/expert-analysis --role frontend-expert --project multi-agents --priority medium &
/expert-analysis --role ui-ux-expert --project multi-agents --priority medium &
wait
# Tous les rapports sont générés dans .reports/
```

## Analyse par Priorité

### Analyse Critique (P0)
```
/expert-analysis --role architecte --project multi-agents --priority critical --output critical-analysis-$(date +%Y-%m-%d).md
```

### Analyse Haute Priorité (P1)
```
/expert-analysis --role backend-expert --project multi-agents --priority high --output high-priority-analysis-$(date +%Y-%m-%d).md
```

### Analyse Moyenne Priorité (P2)
```
/expert-analysis --role frontend-expert --project multi-agents --priority medium --output medium-priority-analysis-$(date +%Y-%m-%d).md
```

### Analyse Basse Priorité (P3)
```
/expert-analysis --role ui-ux-expert --project multi-agents --priority low --output low-priority-analysis-$(date +%Y-%m-%d).md
```

## Analyse par Composant

### Analyse CLI Rust
```
/expert-analysis --role backend-expert --project multi-agents --priority high --output cli-analysis-$(date +%Y-%m-%d).md
```

### Analyse Base de Données
```
/expert-analysis --role backend-expert --project multi-agents --priority high --output database-analysis-$(date +%Y-%m-%d).md
```

### Analyse Interface TUI
```
/expert-analysis --role frontend-expert --project multi-agents --priority medium --output tui-analysis-$(date +%Y-%m-%d).md
```

### Analyse Intégration tmux
```
/expert-analysis --role backend-expert --project multi-agents --priority medium --output tmux-analysis-$(date +%Y-%m-%d).md
```

## Analyse par Type de Problème

### Analyse des Problèmes Critiques
```
/expert-analysis --role architecte --project multi-agents --priority critical --output critical-issues-$(date +%Y-%m-%d).md
```

### Analyse des Performances
```
/expert-analysis --role backend-expert --project multi-agents --priority high --output performance-analysis-$(date +%Y-%m-%d).md
```

### Analyse de la Sécurité
```
/expert-analysis --role architecte --project multi-agents --priority high --output security-analysis-$(date +%Y-%m-%d).md
```

### Analyse de la Maintenabilité
```
/expert-analysis --role dev-expert --project multi-agents --priority medium --output maintainability-analysis-$(date +%Y-%m-%d).md
```

## Analyse par Milestone

### Analyse M4 (Milestone 4)
```
/expert-analysis --role architecte --project multi-agents --priority high --output m4-analysis-$(date +%Y-%m-%d).md
```

### Analyse Pré-Release
```
/expert-analysis --role dev-expert --project multi-agents --priority critical --output pre-release-analysis-$(date +%Y-%m-%d).md
```

### Analyse Post-Release
```
/expert-analysis --role ui-ux-expert --project multi-agents --priority medium --output post-release-analysis-$(date +%Y-%m-%d).md
```

## Analyse par Équipe

### Analyse Équipe Backend
```
/expert-analysis --role backend-expert --project multi-agents --priority high --output backend-team-analysis-$(date +%Y-%m-%d).md
```

### Analyse Équipe Frontend
```
/expert-analysis --role frontend-expert --project multi-agents --priority medium --output frontend-team-analysis-$(date +%Y-%m-%d).md
```

### Analyse Équipe DevOps
```
/expert-analysis --role architecte --project multi-agents --priority high --output devops-team-analysis-$(date +%Y-%m-%d).md
```

## Analyse par Type de Tâche

### Analyse des Tâches de Développement
```
/expert-analysis --role dev-expert --project multi-agents --priority high --output development-tasks-$(date +%Y-%m-%d).md
```

### Analyse des Tâches de Refactoring
```
/expert-analysis --role architecte --project multi-agents --priority medium --output refactoring-tasks-$(date +%Y-%m-%d).md
```

### Analyse des Tâches de Tests
```
/expert-analysis --role dev-expert --project multi-agents --priority medium --output testing-tasks-$(date +%Y-%m-%d).md
```

### Analyse des Tâches de Documentation
```
/expert-analysis --role dev-expert --project multi-agents --priority low --output documentation-tasks-$(date +%Y-%m-%d).md
```

## Analyse par Métrique

### Analyse de la Qualité du Code
```
/expert-analysis --role dev-expert --project multi-agents --priority high --output code-quality-$(date +%Y-%m-%d).md
```

### Analyse de la Couverture de Tests
```
/expert-analysis --role dev-expert --project multi-agents --priority medium --output test-coverage-$(date +%Y-%m-%d).md
```

### Analyse de la Performance
```
/expert-analysis --role backend-expert --project multi-agents --priority high --output performance-metrics-$(date +%Y-%m-%d).md
```

### Analyse de la Sécurité
```
/expert-analysis --role architecte --project multi-agents --priority high --output security-metrics-$(date +%Y-%m-%d).md
```

## Analyse par Période

### Analyse Quotidienne
```
/expert-analysis --role dev-expert --project multi-agents --priority medium --output daily-analysis-$(date +%Y-%m-%d).md
```

### Analyse Hebdomadaire
```
/expert-analysis --role architecte --project multi-agents --priority high --output weekly-analysis-$(date +%Y-%m-%d).md
```

### Analyse Mensuelle
```
/expert-analysis --role architecte --project multi-agents --priority critical --output monthly-analysis-$(date +%Y-%m-%d).md
```

### Analyse Trimestrielle
```
/expert-analysis --role architecte --project multi-agents --priority critical --output quarterly-analysis-$(date +%Y-%m-%d).md
```

## Analyse par Contexte

### Analyse Pré-Développement
```
/expert-analysis --role architecte --project multi-agents --priority critical --output pre-development-$(date +%Y-%m-%d).md
```

### Analyse Pendant Développement
```
/expert-analysis --role dev-expert --project multi-agents --priority high --output during-development-$(date +%Y-%m-%d).md
```

### Analyse Post-Développement
```
/expert-analysis --role ui-ux-expert --project multi-agents --priority medium --output post-development-$(date +%Y-%m-%d).md
```

### Analyse de Maintenance
```
/expert-analysis --role dev-expert --project multi-agents --priority medium --output maintenance-$(date +%Y-%m-%d).md
```

## Notes d'Utilisation

### Variables d'Environnement
```bash
# Définir le projet par défaut
export DEFAULT_PROJECT="multi-agents"

# Définir la priorité par défaut
export DEFAULT_PRIORITY="high"

# Définir le répertoire de sortie
export OUTPUT_DIR="./analysis-reports"
```

### Alias Utiles
```bash
# Alias pour l'analyse architecturale
alias analyze-arch="expert-analysis --role architecte --project multi-agents --priority critical"

# Alias pour l'analyse backend
alias analyze-backend="expert-analysis --role backend-expert --project multi-agents --priority high"

# Alias pour l'analyse frontend
alias analyze-frontend="expert-analysis --role frontend-expert --project multi-agents --priority medium"

# Alias pour l'analyse UI/UX
alias analyze-ux="expert-analysis --role ui-ux-expert --project multi-agents --priority medium"

# Alias pour l'analyse dev
alias analyze-dev="expert-analysis --role dev-expert --project multi-agents --priority high"
```

### Script d'Analyse Complète
```bash
#!/bin/bash
# Script d'analyse complète du projet multi-agents

PROJECT="multi-agents"
DATE=$(date +%Y-%m-%d)
OUTPUT_DIR="./analysis-reports"

# Créer le répertoire de sortie
mkdir -p "$OUTPUT_DIR"

echo "🚀 Lancement de l'analyse complète du projet $PROJECT"
echo "📅 Date: $DATE"
echo "📁 Répertoire de sortie: $OUTPUT_DIR"
echo ""

# Analyser avec tous les rôles
for role in architecte dev-expert backend-expert frontend-expert ui-ux-expert; do
  echo "🔍 Analyse en cours: $role"
  /expert-analysis --role "$role" --project "$PROJECT" --priority high --output "$OUTPUT_DIR/${role}-analysis-${DATE}.md"
  echo "✅ Analyse terminée: $role"
  echo ""
done

echo "🎉 Analyse complète terminée !"
echo "📊 Rapports générés dans: $OUTPUT_DIR"
```

## Workflow Recommandé

### 1. Analyse Initiale
```bash
# Commencer par l'analyse architecturale
/expert-analysis --role architecte --project multi-agents --priority critical
```

### 2. Analyse Spécialisée
```bash
# Analyser les composants critiques
/expert-analysis --role backend-expert --project multi-agents --priority high
/expert-analysis --role dev-expert --project multi-agents --priority high
```

### 3. Analyse UX/UI
```bash
# Analyser l'expérience utilisateur
/expert-analysis --role frontend-expert --project multi-agents --priority medium
/expert-analysis --role ui-ux-expert --project multi-agents --priority medium
```

### 4. Analyse de Synthèse
```bash
# Analyser l'ensemble du projet
/expert-analysis --role architecte --project multi-agents --priority critical --output synthesis-analysis-$(date +%Y-%m-%d).md
```

## Intégration avec le Projet

### Conventions Respectées
- [x] Conventions de branches (feature/, bugfix/, hotfix/, refactor/)
- [x] Conventions de commits (conventional commits)
- [x] Architecture hexagonale (Ports & Adapters)
- [x] Approche API-First avec OpenAPI
- [x] Workflow TDD (Red-Green-Refactor)

### Outils du Projet
- [x] CLI orchestrateur Rust
- [x] Base de données SQLite
- [x] Configuration YAML
- [x] Logging NDJSON
- [x] Gestionnaire tmux

## Notes Importantes

- **Toujours créer une branche avant l'analyse**
- **Utiliser tous les outils MCP disponibles**
- **Respecter l'ordre de priorité et de dépendance des tâches**
- **Générer un rapport actionnable avec des checkboxes**
- **Persister les observations en mémoire pour référence future**
- **Suivre le workflow d'implémentation strict**
