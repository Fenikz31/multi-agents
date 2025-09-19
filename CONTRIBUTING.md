# Contribuer au projet multi-agents (GitLab)

Ce guide décrit comment contribuer avec GitLab (self‑hosted), la convention de branches/commits, et l’usage des templates d’issues.

## Flux Git recommandé

- Créez une branche depuis `main` (nommage: `<type>/<scope>`), exemples:
  - `feat/cli-broadcast`
  - `chore/m0-01-error-codes-and-timeouts`
  - `fix/doctor-timeout`
- Commits au format Conventional Commits (focus sur le "pourquoi"):
  - `feat(cli): add broadcast command with concurrency limit`
  - `chore(m0-01): define canonical error codes and default timeouts`
- Petites PR/MR, focalisées, avec description courte et claire.
- Pas de push sur `main` directement; passez par une Merge Request.

## Issues GitLab (templates)

- Les templates d’issues sont dans `.gitlab/issue_templates/`.
- Pour créer une issue M0 : dans GitLab → New Issue → "Choose a template" → sélectionnez `M0-XX-...`.
- Renseignez : description, critères d’acceptation, dépendances, tâches, risques.
- Ajoutez les labels (`m0`, `area:*`, `type:*`, `priority:*`) et la milestone `M0 – Config & Doctor`.

## Liens avec la documentation

- Spécifications stables : `docs/specs/` (ex: erreurs & timeouts).
- Roadmap & critères d’acceptation : `docs/roadmap.md`.
- Checklist jalon : `docs/m0-checklist.md`.

## Architecture et développement

Le CLI a été organisé en architecture modulaire pour améliorer la maintenabilité :

### Structure des modules
- **`cli/`** : Définitions et parsing des commandes CLI
- **`commands/`** : Implémentations des commandes (config, doctor, db, send, session, agent, init)
- **`utils/`** : Utilitaires partagés (constantes, gestion d'erreurs, timeouts)
- **`tmux/`** : Gestion des sessions tmux avec logique de retry
- **`logging/`** : Gestion des événements NDJSON
- **`providers/`** : Gestion des providers
- **`tests/`** : Suite de tests complète (24 tests unitaires et d'intégration)

### Bonnes pratiques

- Gardez `docs/` comme source de vérité pour les décisions stables.
- Évitez de versionner des artefacts éphémères (issues granulaires) — utilisez GitLab Issues.
- Respectez l'EditorConfig et les fins de ligne LF.
- Les données runtime (`/data`, `/logs`) et secrets (`.env`) ne sont pas commités.
- **Tests** : Ajoutez des tests unitaires pour les nouvelles fonctionnalités dans `tests/unit/`
- **Intégration** : Ajoutez des tests d'intégration dans `tests/integration/` pour les workflows complets
- **Modularité** : Respectez la séparation des responsabilités entre modules

## Hooks Git locaux (pré‑commit)

Pour éviter tout commit avec des warnings/erreurs Rust, installez le hook pré‑commit fourni :

```
scripts/install-git-hooks.sh
```

Le hook exécute automatiquement, et bloque le commit en cas d’échec :
- `cargo update`
- `RUSTFLAGS="-D warnings" cargo build --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `RUSTFLAGS="-D warnings" cargo test --workspace`

Vous pouvez inspecter/éditer le hook dans `.githooks/pre-commit`.

## Processus type (ex : M0‑01)

1) Créez la branche : `git checkout -b chore/m0-01-error-codes-and-timeouts`
2) Ajoutez vos fichiers (spec + defaults) et référencez-les depuis la roadmap.
3) Committez :
```
git commit -m "chore(m0-01): define canonical error codes and default timeouts"
```
4) Ouvrez une MR : associez l’issue M0‑01, ajoutez labels et milestone.

## Revue et intégration

- Demandez 1 relecteur minimum.
- MR verte (CI) et commentaires adressés → merge squash recommandé.
