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

## Bonnes pratiques

- Gardez `docs/` comme source de vérité pour les décisions stables.
- Évitez de versionner des artefacts éphémères (issues granulaires) — utilisez GitLab Issues.
- Respectez l’EditorConfig et les fins de ligne LF.
- Les données runtime (`/data`, `/logs`) et secrets (`.env`) ne sont pas commités.

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
