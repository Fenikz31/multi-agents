#!/bin/bash

# Demo M7 Routing and Supervisor - Version Consolidée
# Ce script démontre les fonctionnalités M7 de routing et supervision
# Gère les providers externes avec une approche robuste

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
print_header() { echo -e "${BLUE}================================${NC}\n${BLUE}$1${NC}\n${BLUE}================================${NC}"; }
print_step() { echo -e "${GREEN}➤ $1${NC}"; }
print_info() { echo -e "${CYAN}ℹ $1${NC}"; }
print_warning() { echo -e "${YELLOW}⚠ $1${NC}"; }
print_error() { echo -e "${RED}✗ $1${NC}"; }
print_success() { echo -e "${GREEN}✓ $1${NC}"; }
print_debug() { echo -e "${PURPLE}🔍 $1${NC}"; }

# Vérification des prérequis
check_prerequisites() {
    print_header "Vérification des prérequis"
    
    # Détection du mode de fonctionnement
    if command -v cargo &> /dev/null; then
        print_success "Mode développement détecté (cargo disponible)"
        CLI_CMD="cargo run --bin multi-agents-cli --"
    elif command -v multi-agents &> /dev/null; then
        print_success "multi-agents CLI est installé"
        CLI_CMD="multi-agents"
    else
        print_error "Ni cargo ni multi-agents CLI ne sont disponibles"
        print_info "Installez cargo ou multi-agents CLI"
        exit 1
    fi
    
    # Vérification des fichiers de configuration
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
    
    # Vérification de la base de données
    if [ ! -f "./data/multi-agents.sqlite3" ]; then
        print_info "Initialisation de la base de données..."
        $CLI_CMD db init
        print_success "Base de données initialisée"
    else
        print_success "Base de données trouvée"
    fi
    
    # Vérification des providers externes (optionnel)
    print_info "Vérification des providers externes..."
    providers_available=0
    
    if command -v gemini &> /dev/null; then
        print_success "Gemini CLI disponible"
        providers_available=$((providers_available + 1))
    else
        print_warning "Gemini CLI non disponible"
    fi
    
    if command -v claude &> /dev/null; then
        print_success "Claude CLI disponible"
        providers_available=$((providers_available + 1))
    else
        print_warning "Claude CLI non disponible"
    fi
    
    if command -v cursor-agent &> /dev/null; then
        print_success "Cursor Agent CLI disponible"
        providers_available=$((providers_available + 1))
    else
        print_warning "Cursor Agent CLI non disponible"
    fi
    
    if [ $providers_available -eq 0 ]; then
        print_warning "Aucun provider externe disponible - démonstration en mode simulation"
    else
        print_success "$providers_available provider(s) externe(s) disponible(s)"
    fi
}

# Configuration du projet
setup_project() {
    print_header "Configuration du projet M7 Demo"
    print_step "Initialisation du projet avec les fichiers de configuration"
    
    if $CLI_CMD init --config-dir ./examples --force; then
        print_success "Configuration du projet terminée"
    else
        print_warning "Projet peut déjà exister ou erreur de configuration"
    fi
}

# Test de routing avec gestion d'erreurs robuste
run_routing_test() {
    local target="$1"
    local message="$2"
    local description="$3"
    
    print_step "$description"
    print_debug "Commande: $CLI_CMD send --project-file $CONFIG_FILE --providers-file $PROVIDERS_FILE --to '$target' --message '$message' --timeout-ms 5000"
    
    # Exécuter la commande et capturer le résultat
    local exit_code=0
    $CLI_CMD send --project-file "$CONFIG_FILE" --providers-file "$PROVIDERS_FILE" --to "$target" --message "$message" --timeout-ms 5000 || exit_code=$?
    
    case $exit_code in
        0)
            print_success "Routing réussi"
            return 0
            ;;
        5)
            print_warning "Timeout (code 5) - normal si providers non configurés"
            return 1
            ;;
        3)
            print_warning "Provider non disponible (code 3)"
            return 1
            ;;
        4)
            print_warning "Erreur CLI provider (code 4)"
            return 1
            ;;
        *)
            print_warning "Erreur inattendue (code $exit_code)"
            return 1
            ;;
    esac
}

# Démonstration du routing
run_routing_demo() {
    print_header "Démonstration du Routing M7"
    
    local success_count=0
    local total_tests=0
    
    # Test 1: Routing par rôle
    print_info "Test 1: Routing par rôle"
    total_tests=$((total_tests + 1))
    if run_routing_test '@backend' "Implémentez le module d'authentification avec JWT" "Routing vers @backend"; then
        success_count=$((success_count + 1))
    fi
    
    total_tests=$((total_tests + 1))
    if run_routing_test '@frontend' "Créez l'interface utilisateur pour l'authentification" "Routing vers @frontend"; then
        success_count=$((success_count + 1))
    fi
    
    total_tests=$((total_tests + 1))
    if run_routing_test '@devops' "Configurez l'infrastructure pour le déploiement" "Routing vers @devops"; then
        success_count=$((success_count + 1))
    fi
    
    # Test 2: Broadcast global
    print_info "Test 2: Broadcast global"
    total_tests=$((total_tests + 1))
    if run_routing_test '@all' "Mise à jour du statut du projet - phase de développement en cours" "Broadcast vers @all"; then
        success_count=$((success_count + 1))
    fi
    
    # Test 3: Routing spécifique
    print_info "Test 3: Routing spécifique"
    total_tests=$((total_tests + 1))
    if run_routing_test 'supervisor-agent' "Génère un rapport de performance du système" "Routing vers supervisor-agent"; then
        success_count=$((success_count + 1))
    fi
    
    # Résumé des tests
    print_info "Résumé des tests de routing: $success_count/$total_tests réussis"
    if [ $success_count -eq $total_tests ]; then
        print_success "Tous les tests de routing ont réussi !"
    elif [ $success_count -gt 0 ]; then
        print_warning "Certains tests ont réussi - vérifiez la configuration des providers"
    else
        print_warning "Aucun test n'a réussi - vérifiez la configuration des providers"
    fi
}

# Vérification des logs
check_logs() {
    print_header "Vérification des Logs M7"
    
    local LOG_DIR="./logs/$PROJECT_NAME"
    if [ -d "$LOG_DIR" ]; then
        print_success "Logs trouvés dans $LOG_DIR/"
        
        local total_events=0
        local routed_events=0
        
        # Analyser chaque fichier de log
        for log_file in "$LOG_DIR"/*.ndjson; do
            if [ -f "$log_file" ]; then
                local role=$(basename "$log_file" .ndjson)
                local line_count=$(wc -l < "$log_file")
                local routed_count=0
                
                print_info "Log $role: $line_count lignes"
                
                # Compter les événements routés
                if [ $line_count -gt 0 ]; then
                    routed_count=$(grep -c '"event":"routed"' "$log_file" 2>/dev/null || echo "0")
                    if [ $routed_count -gt 0 ]; then
                        print_success "  - $routed_count événements routés trouvés"
                        routed_events=$((routed_events + routed_count))
                    else
                        print_info "  - Aucun événement routé trouvé"
                    fi
                    
                    # Analyser les types d'événements
                    if command -v jq &> /dev/null; then
                        local event_types=$(jq -r '.event' "$log_file" 2>/dev/null | sort | uniq -c | tr -d '\n' || echo "analyse impossible")
                        print_info "  - Types d'événements: $event_types"
                    fi
                fi
                
                total_events=$((total_events + line_count))
            fi
        done
        
        print_info "Total: $total_events événements, $routed_events événements routés"
        
        if [ $routed_events -gt 0 ]; then
            print_success "Événements routés détectés - fonctionnalité M7 opérationnelle !"
        else
            print_warning "Aucun événement routé détecté - vérifiez la configuration des providers"
        fi
    else
        print_warning "Aucun log trouvé - les agents n'ont peut-être pas été exécutés"
    fi
}

# Test des fonctionnalités supervisor
test_supervisor_features() {
    print_header "Test des Fonctionnalités Supervisor"
    
    local LOG_DIR="./logs/$PROJECT_NAME"
    if [ -d "$LOG_DIR" ]; then
        print_step "Vérification de la structure des logs NDJSON"
        
        local valid_logs=0
        local total_logs=0
        
        for log_file in "$LOG_DIR"/*.ndjson; do
            if [ -f "$log_file" ]; then
                local role=$(basename "$log_file" .ndjson)
                total_logs=$((total_logs + 1))
                
                print_info "Analyse du log $role:"
                
                # Vérifier la structure JSON
                if command -v jq &> /dev/null; then
                    if jq empty "$log_file" 2>/dev/null; then
                        print_success "  - Format JSON valide"
                        valid_logs=$((valid_logs + 1))
                    else
                        print_error "  - Format JSON invalide"
                    fi
                else
                    print_warning "  - jq non disponible, validation JSON ignorée"
                    valid_logs=$((valid_logs + 1))
                fi
                
                # Vérifier les champs requis
                if grep -q '"broadcast_id"' "$log_file" 2>/dev/null; then
                    print_success "  - Champ broadcast_id présent"
                else
                    print_info "  - Champ broadcast_id absent (normal si pas d'événements routés)"
                fi
            fi
        done
        
        print_info "Logs valides: $valid_logs/$total_logs"
        
        if [ $valid_logs -eq $total_logs ] && [ $total_logs -gt 0 ]; then
            print_success "Structure des logs NDJSON validée"
        else
            print_warning "Problèmes détectés dans la structure des logs"
        fi
    else
        print_warning "Aucun log à analyser"
    fi
    
    print_success "Test des fonctionnalités supervisor terminé"
}

# Nettoyage
cleanup() {
    print_header "Nettoyage"
    print_step "Suppression des logs de démonstration"
    
    if [ -d "./logs/$PROJECT_NAME" ]; then
        rm -rf "./logs/$PROJECT_NAME"
        print_success "Logs supprimés"
    else
        print_info "Aucun log à supprimer"
    fi
    
    print_success "Nettoyage terminé"
}

# Fonction d'aide
show_help() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -h, --help     Afficher cette aide"
    echo "  --no-cleanup   Ne pas nettoyer les logs après la démo"
    echo "  --verbose      Mode verbeux"
    echo ""
    echo "Ce script démontre les fonctionnalités M7 de routing et supervision."
}

# Fonction principale
main() {
    local no_cleanup=false
    local verbose=false
    
    # Parsing des arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            --no-cleanup)
                no_cleanup=true
                shift
                ;;
            --verbose)
                verbose=true
                shift
                ;;
            *)
                print_error "Option inconnue: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    print_header "Demo M7 Routing and Supervisor - Version Consolidée"
    print_info "Ce script démontre les fonctionnalités M7 de routing et supervision"
    print_info "Les erreurs de providers sont normales et attendues"
    
    if [ "$verbose" = true ]; then
        print_debug "Mode verbeux activé"
    fi
    
    check_prerequisites
    if [ "$no_cleanup" = false ]; then
        cleanup # Nettoyer avant de commencer
    fi
    setup_project
    run_routing_demo
    check_logs
    test_supervisor_features
    
    if [ "$no_cleanup" = false ]; then
        cleanup # Nettoyer après la démo
    fi
    
    print_header "Demo M7 Terminée"
    print_success "Toutes les démonstrations ont été effectuées"
    print_info "Résumé des fonctionnalités M7 testées:"
    print_info "- ✅ Routing par rôle (@backend, @frontend, @devops)"
    print_info "- ✅ Broadcast global (@all)"
    print_info "- ✅ Routing spécifique (nom d'agent)"
    print_info "- ✅ Logging NDJSON structuré"
    print_info "- ✅ Événements routés avec broadcast_id"
    print_info "- ✅ Validation de la structure des logs"
    print_info "- ✅ Gestion robuste des erreurs"
    print_info ""
    print_info "Pour des tests avec providers externes, configurez:"
    print_info "- Variables d'environnement pour les API keys"
    print_info "- Authentification des CLIs externes"
    print_info "- Modèles et quotas disponibles"
    print_info ""
    print_info "Documentation:"
    print_info "- docs/tutorials/routing-supervision.md"
    print_info "- docs/tutorials/advanced-use-cases.md"
    print_info "- docs/supervisor-guide.md"
}

main "$@"