# Exemples d'Utilisation M7

Ce répertoire contient des exemples complets pour utiliser les fonctionnalités M7 de Multi-Agents CLI, incluant le routing avancé et le système de supervision.

## Configuration Supervisor

### Fichiers de Configuration

- **`supervisor-config.yaml`** : Configuration complète avec agent supervisor et monitoring
- **`providers-complete.yaml`** : Configuration des providers pour tous les agents
- **`project-complete.yaml`** : Exemple de projet complet avec tous les types d'agents

### Utilisation Rapide

```bash
# Utiliser la configuration supervisor
multi-agents send --project-file examples/supervisor-config.yaml --to @backend --message "Test message"

# Utiliser avec providers complets
multi-agents send --project-file examples/supervisor-config.yaml --providers-file examples/providers-complete.yaml --to @all --message "Broadcast message"
```

## Tutoriels

### Documentation Complète

- **`docs/tutorials/routing-supervision.md`** : Tutoriel complet de routing et supervision
- **`docs/tutorials/advanced-use-cases.md`** : Cas d'usage avancés et optimisations

### Exemples de Routing

#### Routing par Rôle
```bash
# Envoyer à tous les développeurs backend
multi-agents send --to @backend --message "Veuillez revoir les spécifications de l'API utilisateur"

# Envoyer à tous les développeurs frontend
multi-agents send --to @frontend --message "Implémentez la nouvelle interface de connexion"

# Envoyer à tous les agents DevOps
multi-agents send --to @devops --message "Préparez le déploiement en production"
```

#### Broadcast Global
```bash
# Message d'urgence à toute l'équipe
multi-agents send --to @all --message "URGENT: Mise à jour de sécurité requise"

# Message d'information générale
multi-agents send --to @all --message "Réunion d'équipe demain à 14h"
```

#### Routing Spécifique
```bash
# Message au supervisor
multi-agents send --to supervisor --message "Génère un rapport de performance du système"

# Message à un agent spécifique
multi-agents send --to backend-dev --message "Corrige le bug critique dans l'API d'authentification"
```

## Scripts de Démonstration

### Script Principal

- **`scripts/demo-m7-routing.sh`** : Script de démonstration complet des fonctionnalités M7

#### Utilisation du Script

```bash
# Exécuter la démonstration complète
./scripts/demo-m7-routing.sh

# Le script effectue automatiquement :
# 1. Vérification des prérequis
# 2. Configuration du projet
# 3. Démonstration du routing par rôle
# 4. Démonstration du broadcast global
# 5. Démonstration du routing spécifique
# 6. Démonstration du monitoring
# 7. Démonstration des cas d'usage avancés
# 8. Nettoyage
```

#### Fonctionnalités Démonstrées

- ✅ **Routing par rôle** : `@backend`, `@frontend`, `@devops`
- ✅ **Broadcast global** : `@all`
- ✅ **Routing spécifique** : agents individuels
- ✅ **Monitoring** : logs et métriques
- ✅ **Cas d'usage avancés** : orchestration, gestion d'incidents

## Cas d'Usage Avancés

### Orchestration Multi-Agents

```bash
# Planification de tâches complexes
multi-agents send --to supervisor --message "Planifie le déploiement de la nouvelle fonctionnalité"

# Coordination entre équipes
multi-agents send --to @backend --message "Préparez l'API pour la nouvelle fonctionnalité"
multi-agents send --to @frontend --message "Implémentez l'interface utilisateur"
multi-agents send --to @devops --message "Préparez l'infrastructure de déploiement"
```

### Gestion d'Incidents

```bash
# Détection d'incident
multi-agents send --to supervisor --message "INCIDENT: Service de paiement indisponible"

# Coordination de la résolution
multi-agents send --to @backend --message "URGENT: Vérifiez le service de paiement"
multi-agents send --to @devops --message "URGENT: Vérifiez l'infrastructure de paiement"
```

### Code Review et Qualité

```bash
# Demande de review
multi-agents send --to @developers --message "Review du code de la fonctionnalité d'authentification"

# Coordination
multi-agents send --to @backend --message "Review l'API d'authentification"
multi-agents send --to @frontend --message "Review l'interface d'authentification"
```

## Comment Utiliser

### Prérequis

1. **Multi-Agents CLI installé** : `multi-agents` doit être disponible dans le PATH
2. **Base de données initialisée** : `multi-agents db init`
3. **Providers configurés** : gemini, claude, cursor-agent (optionnel pour les tests)

### Étapes de Démarrage

1. **Cloner le projet** et naviguer vers le répertoire
2. **Initialiser la base de données** : `multi-agents db init`
3. **Exécuter la démonstration** : `./scripts/demo-m7-routing.sh`
4. **Consulter la documentation** : `docs/tutorials/`

### Configuration Personnalisée

1. **Copier les fichiers d'exemple** : `cp examples/supervisor-config.yaml my-config.yaml`
2. **Modifier la configuration** selon vos besoins
3. **Utiliser votre configuration** : `multi-agents send --project-file my-config.yaml --to @backend --message "Test"`

## Exemples Rapides

### Test de Base

```bash
# Test simple de routing
multi-agents send --to @backend --message "Hello from M7 routing!"

# Test de broadcast
multi-agents send --to @all --message "Broadcast test message"
```

### Test de Monitoring

```bash
# Vérifier les logs générés
ls -la logs/

# Consulter les logs d'un agent
cat logs/m7-demo/backend.ndjson
```

### Test de Métriques

```bash
# Les métriques sont calculées automatiquement
# Consultez la documentation pour l'analyse programmatique
```

## Dépannage

### Problèmes Courants

1. **"multi-agents command not found"**
   - Solution : Vérifiez que le CLI est installé et dans le PATH

2. **"Database not initialized"**
   - Solution : Exécutez `multi-agents db init`

3. **"Provider not available"**
   - Solution : Les erreurs de providers sont normales si les CLIs externes ne sont pas configurés

4. **"No agents found for target"**
   - Solution : Vérifiez que les agents sont configurés avec les bons rôles

### Commandes de Diagnostic

```bash
# Vérifier la configuration
multi-agents config validate --project-file examples/supervisor-config.yaml

# Vérifier l'état des agents
multi-agents agent list --project m7-demo

# Vérifier les logs
ls -la logs/m7-demo/

# Tester le routing
multi-agents send --to @all --message "Test de routing"
```

## Ressources Supplémentaires

### Documentation

- **Guide du Supervisor** : `docs/supervisor-guide.md`
- **Référence CLI** : `docs/cli-reference.md`
- **Workflows** : `docs/workflows.md`

### Exemples de Code

- **Configuration YAML** : `examples/supervisor-config.yaml`
- **Script de démonstration consolidé** : `scripts/demo-m7-routing.sh`
- **Tutoriels détaillés** : `docs/tutorials/`

### Support

- **Issues** : Signalez les problèmes via GitLab Issues
- **Documentation** : Consultez la documentation complète
- **Exemples** : Utilisez les exemples fournis comme base

## Contribution

Pour contribuer aux exemples :

1. **Forkez le projet**
2. **Créez une branche** : `git checkout -b feature/new-example`
3. **Ajoutez vos exemples** dans le répertoire approprié
4. **Testez vos exemples** avec le script de démonstration
5. **Créez une MR** avec vos modifications

Les exemples doivent être :
- ✅ **Fonctionnels** : Testés et validés
- ✅ **Documentés** : Avec des commentaires clairs
- ✅ **Cohérents** : Suivant les conventions du projet
- ✅ **Utiles** : Apportant une valeur réelle aux utilisateurs
