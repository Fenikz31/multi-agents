#!/bin/bash

# Demo M7 Routing and Supervisor
# Ce script démontre les fonctionnalités M7 de routing et supervision

set -e

# Couleurs pour l'affichage
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
PROJECT_NAME="m7-demo"
CONFIG_FILE="examples/supervisor-config.yaml"
PROVIDERS_FILE="examples/providers-complete.yaml"

# Fonctions utilitaires
print_header() {
    echo -e "${BLUE}================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}================================${NC}"
}

print_step() {
    echo -e "${GREEN}➤ $1${NC}"
}

print_info() {
    echo -e "${CYAN}ℹ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

# Vérification des prérequis
check_prerequisites() {
    print_header "Vérification des prérequis"
    
    # Vérifier que $CLI_CMD CLI est installé
    if ! command -v $CLI_CMD &> /dev/null; then
        print_error "$CLI_CMD CLI n'est pas installé"
        exit 1
    fi
    print_success "$CLI_CMD CLI trouvé"
    
    # Vérifier que les fichiers de configuration existent
    if [ ! -f "$CONFIG_FILE" ]; then
        print_error "Fichier de configuration non trouvé: $CONFIG_FILE"
        exit 1
    fi
    print_success "Fichier de configuration trouvé: $CONFIG_FILE"
    
    if [ ! -f "$PROVIDERS_FILE" ]; then
        print_error "Fichier de providers non trouvé: $PROVIDERS_FILE"
        exit 1
    fi
    print_success "Fichier de providers trouvé: $PROVIDERS_FILE"
    
    # Vérifier que la base de données est initialisée
    if [ ! -f "./data/multi-agents.sqlite3" ]; then
        print_info "Initialisation de la base de données..."
        $CLI_CMD db init
        print_success "Base de données initialisée"
    else
        print_success "Base de données trouvée"
    fi
}

# Configuration du projet
setup_project() {
    print_header "Configuration du projet M7 Demo"
    
    # Ajouter le projet
    print_step "Ajout du projet $PROJECT_NAME"
    $CLI_CMD project add --name "$PROJECT_NAME" || print_warning "Projet peut déjà exister"
    
    # Ajouter les agents
    print_step "Ajout des agents"
    
    # Supervisor
    $CLI_CMD agent add \
        --project "$PROJECT_NAME" \
        --name "supervisor" \
        --role "supervisor" \
        --provider "claude" \
        --model "sonnet-4" \
        --system-prompt "You are a supervisor agent responsible for coordinating other agents and monitoring system performance." || print_warning "Agent supervisor peut déjà exister"
    
    # Backend Developer
    $CLI_CMD agent add \
        --project "$PROJECT_NAME" \
        --name "backend-dev" \
        --role "backend" \
        --provider "gemini" \
        --model "gemini-1.5-flash" \
        --system-prompt "You are a backend developer specializing in API development and system architecture." || print_warning "Agent backend-dev peut déjà exister"
    
    # Frontend Developer
    $CLI_CMD agent add \
        --project "$PROJECT_NAME" \
        --name "frontend-dev" \
        --role "frontend" \
        --provider "cursor-agent" \
        --model "gpt-5" \
        --system-prompt "You are a frontend developer focused on user experience and modern web technologies." || print_warning "Agent frontend-dev peut déjà exister"
    
    # DevOps Engineer
    $CLI_CMD agent add \
        --project "$PROJECT_NAME" \
        --name "devops-engineer" \
        --role "devops" \
        --provider "claude" \
        --model "sonnet-4" \
        --system-prompt "You are a DevOps engineer responsible for infrastructure and deployment." || print_warning "Agent devops-engineer peut déjà exister"
    
    print_success "Configuration du projet terminée"
}

# Démonstration du routing par rôle
demo_role_routing() {
    print_header "Démonstration du Routing par Rôle"
    
    # Routing vers backend
    print_step "Routing vers les développeurs backend"
    $CLI_CMD send \
        --project-file "$CONFIG_FILE" \
        --providers-file "$PROVIDERS_FILE" \
        --to "@backend" \
        --message "Veuillez revoir les spécifications de l'API utilisateur et implémenter les endpoints manquants." \
        --timeout 10000 || print_warning "Erreur lors du routing backend (normal si providers non disponibles)"
    
    sleep 2
    
    # Routing vers frontend
    print_step "Routing vers les développeurs frontend"
    $CLI_CMD send \
        --project-file "$CONFIG_FILE" \
        --providers-file "$PROVIDERS_FILE" \
        --to "@frontend" \
        --message "Implémentez la nouvelle interface de connexion avec validation en temps réel." \
        --timeout 10000 || print_warning "Erreur lors du routing frontend (normal si providers non disponibles)"
    
    sleep 2
    
    # Routing vers devops
    print_step "Routing vers les ingénieurs DevOps"
    $CLI_CMD send \
        --project-file "$CONFIG_FILE" \
        --providers-file "$PROVIDERS_FILE" \
        --to "@devops" \
        --message "Préparez le déploiement en production avec monitoring et alertes." \
        --timeout 10000 || print_warning "Erreur lors du routing devops (normal si providers non disponibles)"
    
    print_success "Démonstration du routing par rôle terminée"
}

# Démonstration du broadcast global
demo_broadcast() {
    print_header "Démonstration du Broadcast Global"
    
    print_step "Broadcast à toute l'équipe"
    $CLI_CMD send \
        --project-file "$CONFIG_FILE" \
        --providers-file "$PROVIDERS_FILE" \
        --to "@all" \
        --message "URGENT: Mise à jour de sécurité requise - arrêtez tous les déploiements et appliquez le patch critique." \
        --timeout 15000 || print_warning "Erreur lors du broadcast (normal si providers non disponibles)"
    
    sleep 3
    
    print_step "Broadcast d'information générale"
    $CLI_CMD send \
        --project-file "$CONFIG_FILE" \
        --providers-file "$PROVIDERS_FILE" \
        --to "@all" \
        --message "Réunion d'équipe demain à 14h - préparez vos rapports de sprint et vos questions techniques." \
        --timeout 15000 || print_warning "Erreur lors du broadcast (normal si providers non disponibles)"
    
    print_success "Démonstration du broadcast global terminée"
}

# Démonstration du routing spécifique
demo_specific_routing() {
    print_header "Démonstration du Routing Spécifique"
    
    # Message au supervisor
    print_step "Message au supervisor"
    $CLI_CMD send \
        --project-file "$CONFIG_FILE" \
        --providers-file "$PROVIDERS_FILE" \
        --to "supervisor" \
        --message "Génère un rapport de performance du système et identifie les goulots d'étranglement." \
        --timeout 10000 || print_warning "Erreur lors du routing supervisor (normal si providers non disponibles)"
    
    sleep 2
    
    # Message à un agent spécifique
    print_step "Message à un agent spécifique"
    $CLI_CMD send \
        --project-file "$CONFIG_FILE" \
        --providers-file "$PROVIDERS_FILE" \
        --to "backend-dev" \
        --message "Corrige le bug critique dans l'API d'authentification - les tokens JWT expirent prématurément." \
        --timeout 10000 || print_warning "Erreur lors du routing spécifique (normal si providers non disponibles)"
    
    print_success "Démonstration du routing spécifique terminée"
}

# Démonstration du monitoring
demo_monitoring() {
    print_header "Démonstration du Monitoring"
    
    print_step "Vérification des logs générés"
    if [ -d "logs/$PROJECT_NAME" ]; then
        print_info "Logs trouvés dans logs/$PROJECT_NAME/"
        ls -la "logs/$PROJECT_NAME/" || print_warning "Impossible de lister les logs"
        
        # Afficher quelques lignes de log
        for log_file in "logs/$PROJECT_NAME"/*.ndjson; do
            if [ -f "$log_file" ]; then
                print_info "Contenu de $(basename "$log_file"):"
                head -n 3 "$log_file" || print_warning "Impossible de lire le fichier de log"
                echo ""
            fi
        done
    else
        print_warning "Aucun log trouvé - les agents n'ont peut-être pas été exécutés"
    fi
    
    print_step "Vérification des métriques"
    print_info "Les métriques sont calculées automatiquement par le supervisor"
    print_info "Consultez la documentation pour plus de détails sur l'analyse des métriques"
    
    print_success "Démonstration du monitoring terminée"
}

# Démonstration des cas d'usage avancés
demo_advanced_use_cases() {
    print_header "Démonstration des Cas d'Usage Avancés"
    
    print_step "Orchestration de tâches complexes"
    $CLI_CMD send \
        --project-file "$CONFIG_FILE" \
        --providers-file "$PROVIDERS_FILE" \
        --to "supervisor" \
        --message "Planifie le déploiement de la nouvelle fonctionnalité de paiement avec coordination entre équipes." \
        --timeout 10000 || print_warning "Erreur lors de l'orchestration (normal si providers non disponibles)"
    
    sleep 2
    
    print_step "Gestion d'incident"
    $CLI_CMD send \
        --project-file "$CONFIG_FILE" \
        --providers-file "$PROVIDERS_FILE" \
        --to "supervisor" \
        --message "INCIDENT: Service de paiement indisponible - coordonne la résolution avec les équipes." \
        --timeout 10000 || print_warning "Erreur lors de la gestion d'incident (normal si providers non disponibles)"
    
    sleep 2
    
    print_step "Code review et qualité"
    $CLI_CMD send \
        --project-file "$CONFIG_FILE" \
        --providers-file "$PROVIDERS_FILE" \
        --to "@developers" \
        --message "Review du code de la fonctionnalité d'authentification - focus sur la sécurité et les performances." \
        --timeout 10000 || print_warning "Erreur lors du code review (normal si providers non disponibles)"
    
    print_success "Démonstration des cas d'usage avancés terminée"
}

# Nettoyage
cleanup() {
    print_header "Nettoyage"
    
    print_step "Suppression des logs de démonstration"
    if [ -d "logs/$PROJECT_NAME" ]; then
        rm -rf "logs/$PROJECT_NAME"
        print_success "Logs de démonstration supprimés"
    else
        print_info "Aucun log à supprimer"
    fi
    
    print_step "Nettoyage de la base de données (optionnel)"
    print_warning "Pour supprimer le projet de la base de données, exécutez:"
    print_info "$CLI_CMD project remove --name $PROJECT_NAME"
    
    print_success "Nettoyage terminé"
}

# Fonction principale
main() {
    print_header "Demo M7 Routing and Supervisor"
    print_info "Ce script démontre les fonctionnalités M7 de routing et supervision"
    print_info "Les erreurs de providers sont normales si les CLIs externes ne sont pas configurés"
    echo ""
    
    # Vérification des prérequis
    check_prerequisites
    echo ""
    
    # Configuration du projet
    setup_project
    echo ""
    
    # Démonstrations
    demo_role_routing
    echo ""
    
    demo_broadcast
    echo ""
    
    demo_specific_routing
    echo ""
    
    demo_monitoring
    echo ""
    
    demo_advanced_use_cases
    echo ""
    
    # Nettoyage
    cleanup
    echo ""
    
    print_header "Demo M7 Terminée"
    print_success "Toutes les démonstrations ont été effectuées"
    print_info "Consultez la documentation pour plus de détails:"
    print_info "- docs/tutorials/routing-supervision.md"
    print_info "- docs/tutorials/advanced-use-cases.md"
    print_info "- docs/supervisor-guide.md"
    echo ""
    print_info "Pour des tests plus approfondis, configurez les providers externes:"
    print_info "- gemini CLI"
    print_info "- claude CLI"
    print_info "- cursor-agent CLI"
}

# Gestion des signaux
trap cleanup EXIT

# Exécution
main "$@"
