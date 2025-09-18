## Milestone M4 — tmux Agents (REPL): Analyse approfondie

### 1. Contexte et objectifs
- L’orchestrateur CLI (Rust) pilote des providers (`gemini`, `claude`, `cursor-agent`) en one-shot (M2) et en REPL via tmux (M4), avec persistance SQLite et logs NDJSON. Voir `docs/overview.md`, `docs/roadmap.md`, `docs/workflows.md`.
- M4 introduit l’exécution REPL par agent dans tmux, les commandes `multi-agents agent run|attach|stop`, et l’observabilité NDJSON des sessions REPL.
- Standards: concurrency=3 (one-shot), timeouts (doctor, send, tmux 5s), exit codes standardisés (0–8). Voir `config/defaults.yaml`.

### 2. Exigences M4 (contrat)
- Commandes: `multi-agents agent run|attach|stop --project <name> --agent <name>` documentées dans `docs/cli-reference.md`.
- tmux: créer/attacher/arrêter en ≤5s; mapping erreurs → code 8 (`tmux_error`).
- Structure tmux:
  - Session: `proj:{project}`
  - Window: `{role}:{agent}`
  - 1 pane par agent REPL
- Logging NDJSON append-only vers `./logs/{project}/{role}.ndjson` activé dès le démarrage REPL.
- Événements NDJSON requis: `start`, `stdout_line`, `end` avec champs obligatoires (ts, level, project_id, agent_role, agent_id, provider, session_id si dispo, event, text/dur_ms).
- Idempotence: relancer `agent run` ne crée pas de doublons de fenêtres ni de pipes.

### 3. Architecture et responsabilités
- CLI couche commande: parse flags, résout projet/agent/role/provider depuis YAML/DB, applique timeouts/exit-codes.
- Gestionnaire tmux: encapsule `has-session`, `new-session`, `new-window`, `pipe-pane -o`, `send-keys`, `kill-window`, `attach`.
- Provider REPL: démarre le binaire provider en mode interactif avec prompt système du rôle; la sortie stdout est capturée par tmux et redirigée via `pipe-pane`.
- Journalisation: `pipe-pane -o` vers fichier NDJSON; pas de couleurs/ANSI; UTF-8; rotation gérée hors-produit.
- Persistance: sessions/messages SQLite (lecture/alignement pour identités); M4 peut se limiter au contrat NDJSON et enregistrement minimal des métadonnées si nécessaire.

### 4. Flux principaux
- Agent REPL (tmux) — voir `docs/workflows.md`:
  1) Créer session tmux `proj:{project}` si absente.
  2) Créer window `{role}:{agent}` avec commande REPL du provider.
  3) Activer `pipe-pane -o` vers `./logs/{project}/{role}.ndjson`.
  4) Émettre un événement `start` NDJSON.
  5) Recevoir/consigner `stdout_line` pour chaque ligne stdout du provider.
  6) À l’arrêt, consigner `end` avec `dur_ms`.
- Attach: `tmux attach -t proj:{project}` (mode interactif), ou message d’information si headless.
- Stop: `kill-window` ciblé sur `{role}:{agent}`; si dernière fenêtre, session conservée (décision: ne pas tuer session pour ne pas impacter d’autres agents).

### 5. Comportement « premier lancement » (désiré)
- Auto-détection de configuration lorsqu’une commande critique est invoquée:
  - Si `config/project.yaml` ou `config/providers.yaml` manquent → proposer `multi-agents config init` (non destructif) puis `multi-agents db init`.
  - `multi-agents doctor` recommandé pour valider PATH/versions providers avant `agent run`.
- Après ajout d’un projet/agent, `agent run` crée `proj:{project}`, `{role}:{agent}`, démarre le REPL provider et active `pipe-pane`.
- Option d’ouvrir la TUI (`multi-agents tui --project <name>`) après mise en place pour visibilité.

### 6. Design CLI `agent run|attach|stop`
- Syntaxe proposée:
  - `multi-agents agent run --project <name> --agent <name> [--role <role>] [--provider <prov>] [--model <model>] [--workdir <path>] [--no-logs] [--timeout-ms <int>]`
  - `multi-agents agent attach --project <name>`
  - `multi-agents agent stop --project <name> --agent <name>`
- Règles:
  - Résolution `role/provider/model` depuis `project.yaml` si omis; validation d’existence; messages d’erreur clairs.
  - `--no-logs` désactive `pipe-pane` (utile debug); `--workdir` change le répertoire d’exécution du provider.
  - Codes retour: 0 (OK), 2 (invalid_input), 5 (timeout), 8 (tmux_error).

### 7. Design gestionnaire tmux
- Nommage: session `proj:{project}`, window `{role}:{agent}`; pane index unique (0).
- Idempotence: vérifier `has-session`, `list-windows -F '#W'`, `list-panes` avant création; `pipe-pane -o` pour éviter double piping.
- Démarrage REPL: `new-window -- <provider_cmd>` avec prompt système injecté; surveillance de l’état de process (healthcheck optionnel).
- Attach: si non-interactif, impression d’un message « attach with: tmux attach -t proj:{project} ».
- Stop: `kill-window -t proj:{project}:{role}:{agent}`; si échec (absent) → code 0 idempotent avec avertissement.
- Timeouts: toutes opérations tmux bornées à 5s (par défaut `config/defaults.yaml`).

### 8. Spécification NDJSON
- Chemin: `./logs/{project}/{role}.ndjson` (créer dossiers si besoin).
- Événements:
  - `start`: métadonnées agent/provider; éventuellement `session_id` provider si accessible.
  - `stdout_line`: une ligne par sortie stdout (sans ANSI); champ `text`.
  - `end`: état final, durée `dur_ms`, motif d’arrêt si pertinent.
- Champs requis (exemple minimal):
```
{"ts":"2025-09-15T14:03:21.123Z","level":"info","project_id":"demo","agent_role":"backend","agent_id":"backend","provider":"gemini","session_id":null,"broadcast_id":null,"direction":"agent","event":"stdout_line","message_id":"...","text":"First response line","dur_ms":12}
```
- Conformité: NDJSON valide (1 JSON par ligne, UTF‑8), pas d’ANSI; voir `docs/roadmap.md`.

### 9. Timeouts et erreurs
- Valeur par défaut: `tmux_action_ms: 5000` dans `config/defaults.yaml`.
- Sur dépassement: retour code 5 (timeout) et message suggestif (vérifier tmux, permissions, PATH).
- Erreurs tmux (commande échouée): code 8; inclure le stderr nettoyé.
- Retries: un retry rapide autorisé pour conditions de course (`new-session` vs `new-window`).

Référence détaillée: `docs/specs/errors-and-timeouts.md`.

### 10. Intégration SQLite (côté REPL)
- Sessions: option d’enregistrer une session « repl » (statut actif, `provider_session_id` si disponible) pour aligner avec M3/M7.
- Messages: corrélation possible via `agent_id` + timestamps; M4 peut rester NDJSON-first et reporter enrichissement DB à M7.
- Nettoyage: `session cleanup` (>24h) reste pertinent même si REPL.

Note: aligner le schéma et conventions avec `docs/data-model.md` et `crates/db`.

### 11. Tests (TDD)
- Unitaires CLI: parsing, validation flags, mapping exit codes/erreurs.
- Contrat NDJSON: snapshots `start/stdout_line/end` et validateur (schéma minimal, UTF‑8, no ANSI).
- tmux simulé: abstraction exécution commandes (runner mock) pour `has-session/new-window/pipe-pane/kill-window`.
- Tests doc: exemples `docs/tmux.md` exécutables (dry-run) pour éviter dérive.

### 12. Risques et blocages
- tmux absent/non utilisable (WSL2/CI): bloquant pour attach; `agent run` doit échouer proprement (code 8) avec guidance.
- Permissions sur `./logs/...`: échec `pipe-pane`; prévoir fallback message clair et option `--no-logs`.
- PATH providers manquant: `doctor` doit détecter et bloquer avant M4.
- Concurrence/duplication: plusieurs `agent run` simultanés → mutex par agent (fichier lock) pour éviter fenêtres dupliquées.
- Échappement ANSI/tailles lignes: saturations; normaliser et borner taille.

### 13. Optimisations et axes d’amélioration
- Verrouillage par agent (file lock) pour sérialiser les opérations tmux.
- Healthcheck post-démarrage REPL (commande provider légère type `--version`).
- Options: `--logs-dir <path>`, `--no-logs`, `--workdir`.
- Métriques: durée de démarrage, échecs tmux catégorisés, taille des logs.
- Préparer M5: abstraction `send-keys` multi-cibles, agrégation de statuts, corrélation `broadcast_id`.
- Observabilité: enrichir NDJSON avec `broadcast_id`, `session_id` quand dispo; tracés corrélés pour M9.

### 14. Alignement M3 ↔ M4 ↔ M5
- M3 (Session Resume): réutilisation/stockage `provider_session_id` (Claude/Cursor/Gemini) s’applique aux REPL si exposé par provider.
- M4: n’exige pas la reprise provider-side mais doit supporter un identifiant interne pour corrélation/logs.
- M5 (Broadcast): en REPL, fan-out via `tmux send-keys` sur `{role}:{agent}` multiples; NDJSON conserve 1 événement par agent.

### 15. Critères d’acceptation (récapitulatif)
- CLI `agent run|attach|stop` disponible et documentée; `--help` clair.
- Démarrage REPL crée session/window/pane selon conventions, en ≤5s.
- `pipe-pane -o` actif; logs NDJSON `start/stdout_line/end` conformes; pas d’ANSI; UTF‑8.
- Idempotence: relance ne duplique pas; `stop` sans fenêtre retourne 0 avec avertissement.
- Codes: 0 OK; 2 invalid input; 5 timeout; 8 tmux error.

### 16. Références
- Docs internes: `docs/overview.md`, `docs/roadmap.md`, `docs/tmux.md`, `docs/workflows.md`, `config/defaults.yaml`.
- Bonnes pratiques externes:
  - Anthropic — Claude Code: Best practices for agentic coding: https://www.anthropic.com/engineering/claude-code-best-practices
  - Lorenz Hofmann-Wellenhof — Read These 4 Claude Code Resources: https://lorenzhw.substack.com/p/read-these-4-claude-code-resources?utm_campaign=post%2Cpost&utm_source=medium


