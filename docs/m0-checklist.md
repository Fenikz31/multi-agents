# Checklist M0 — Config & Doctor

Objectif: verrouiller l’environnement (CLIs, tmux), les schémas YAML, la matrice de compatibilité, les conventions, les timeouts/codes d’erreur et la spéc NDJSON. Aucun run d’agent ni DB à ce stade.

## Tâches (priorisées et dépendantes)

- [ ] 1) Définir les codes d’erreur normalisés et les timeouts par défaut
  - [ ] Énumérer les codes: 0 OK, 1 générique, 2 entrée invalide, 3 provider indisponible, 4 erreur CLI provider, 5 timeout, 6 config manquante, 7 erreur DB (réservé M1+), 8 erreur tmux
  - [ ] Définir les timeouts par défaut: sends 120s; doctor (provider) 2s; doctor (global) 10s; tmux 5s
  - [ ] Documenter dans `docs/roadmap.md` et `docs/cli-reference.md`

- [ ] 2) Spécifier les schémas YAML (`project.yaml`, `providers.yaml`) avec `schema_version`
  - [ ] Ajouter `schema_version: 1` dans chaque fichier
  - [ ] Modéliser via Serde avec `deny_unknown_fields`
  - [ ] Générer JSON Schema via `schemars` (pour validation externe)
  - [ ] Documenter les champs obligatoires et formats

- [ ] 3) Poser les règles de validation métier et placeholders requis
  - [ ] `groups[*].members` référencent des `agents[*].name` existants
  - [ ] `allowed_tools` non vide pour les rôles concernés
  - [ ] Placeholders requis par provider: 
    - [ ] Claude: `{prompt}`, `{session_id}`, `{allowed_tools}` (si whitelist)
    - [ ] Cursor: `{prompt}`, `{chat_id}`; interdire `--force`
    - [ ] Gemini: `{prompt}` (one-shot), `{system_prompt}` (REPL), `{allowed_tools}` (si supporté)
  - [ ] Messages d’erreur avec chemin YAML précis

- [ ] 4) Implémenter la commande `config validate` (sortie `text` | `json`)
  - [ ] Valider schémas + règles métier + placeholders
  - [ ] Afficher un résumé clair en `text` et un rapport structuré en `json`

- [ ] 5) Formaliser les conventions tmux
  - [ ] Nommage: session `proj:{project}`, fenêtre `{role}:{agent}`
  - [ ] Préparer l’usage futur de `pipe-pane -o` (chemins de logs)
  - [ ] Documenter dans `docs/overview.md` et `docs/workflows.md`

- [ ] 6) Finaliser la spéc NDJSON + self-check parser
  - [ ] Événements: `start`, `stdout_line`, `stderr_line`, `end`, `routed`
  - [ ] Champs requis: `ts`, `project_id`, `agent_role`, `provider`, `session_id`, `direction`, `event`
  - [ ] Champs optionnels: `agent_id`, `broadcast_id`, `message_id`, `text`, `dur_ms`, `exit_code`
  - [ ] Règles: 1 JSON/ligne, UTF‑8 strict, pas d’ANSI
  - [ ] Self-check de parsing (échantillon) dans `doctor`

- [ ] 7) Concevoir les détecteurs de compatibilité CLIs (capabilities snapshot)
  - [ ] Détecter versions et flags clés (timeouts courts)
  - [ ] Claude: `--output-format`, `--session-id`, `-r/--resume`, `--allowed-tools`, `--permission-mode`
  - [ ] Cursor: `-p/--print`, `--output-format`, `create-chat`, `--resume` (et vérifier l’absence d’usage de `--force`)
  - [ ] Gemini: `-i/--prompt-interactive`, (optionnel) `--allowed-tools`
  - [ ] Sortie: `supports.{feature}=true|false|unknown`

- [ ] 8) Définir les remédiations et la table de compatibilité attendue
  - [ ] Mapper issue → action (maj CLI, ajustement YAML, désactivation feature)
  - [ ] Marquer `KO (dégradé)` si une feature demandée n’est pas supportée
  - [ ] Documenter dans `docs/roadmap.md`

- [ ] 9) Implémenter la commande `doctor` (sortie `text` | `json`)
  - [ ] Vérifier CLIs (`gemini`, `claude`, `cursor-agent`), `tmux`, `git`
  - [ ] Afficher versions + features + statut OK/KO/DEGRADE
  - [ ] Respecter les timeouts; mapper les échecs aux codes d’erreur

- [ ] 10) Préparer des exemples de `project.yaml`/`providers.yaml`
  - [ ] Cas minimal fonctionnel
  - [ ] Cas complet (whitelists, groupes, placeholders)
  - [ ] Cas piégeux pour tests (champs inconnus, placeholders manquants)

- [ ] 11) Mettre à jour la documentation M0
  - [ ] `configuration.md` (schémas + JSON Schema)
  - [ ] `roadmap.md` (DoD M0 et timeouts/codes)
  - [ ] `security.md` (whitelists et politiques)
  - [ ] `logging.md` (schéma NDJSON stabilisé)

- [ ] 12) Tests M0 (unitaires + fumée)
  - [ ] YAML valides/invalides (snapshots d’erreurs)
  - [ ] Parse `--help`/`--version` simulés (fixtures texte)
  - [ ] `doctor` < 10s en environnement sain
  - [ ] Codes d’erreur déterministes selon scénarios

## Critères de sortie M0 (DoD)

- [ ] `multi-agents config validate` retourne OK/KO avec chemins d’erreur précis (text et json)
- [ ] `multi-agents doctor` liste versions, features, statut et remédiations; respecte les timeouts
- [ ] Conventions tmux et spéc NDJSON documentées et stables
- [ ] Table de compatibilité et remédiations publiées
- [ ] Exemples YAML fournis (min/complet/piégeux)
- [ ] Documentation M0 à jour
- [ ] Suite de tests M0 passée