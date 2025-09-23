## M6.4 — UX TUI: Recherche & Conception (Issue #50)

Description courte: Définir l'expérience utilisateur de la TUI (Kanban, Sessions, Logs) avec wireframes, hiérarchie d'information, navigation et raccourcis, système de thèmes (couleurs/typographie), et patterns d'interaction.

### Objectifs
- Clarifier les besoins utilisateur (personas synthétiques)
- Proposer 3 wireframes basse fidélité: Kanban, Sessions, Logs
- Définir la hiérarchie d'information et la navigation globale
- Spécifier les raccourcis clavier
- Établir un système de thèmes (couleurs/typographie)
- Documenter les patterns d'interaction

---

### Personas (synthèse)
- DevOps Lead: suit l'avancement (Kanban), veut diffuser des messages (broadcast), surveille logs.
- Backend Engineer: reprend des sessions, lit/filtre logs, bascule entre agents.
- Product Owner: visualise état des tâches et transitions, lecture seule.

---

### Navigation Globale (modèle mental)
- Vues principales: Kanban (K), Sessions (S), Logs (L)
- Barre d'en-tête: titre application + projet courant
- Footer d'aide: rappels des raccourcis contextuels

Transitions clés:
- h → Aide/Help
- k → Kanban
- s → Sessions
- l → Logs (réservé; à implémenter)
- q → Quitter

---

### Raccourcis Clavier (proposés)
- Global:
  - q: Quitter
  - h: Aide/Help
  - k/s/l: Aller à Kanban/Sessions/Logs
  - ← → ↑ ↓: Navigation/focus
  - Enter: Sélection/valider
  - /: Filtre/Recherche contextuelle
- Kanban:
  - ← →: Changer de colonne
  - ↑ ↓: Changer de tâche sélectionnée
  - m: Déplacer tâche vers colonne suivante
  - f: Saisir filtre (par titre/assignee)
  - t: Basculer tri (Priorité/Création/Mise à jour/Titre)
- Sessions:
  - ↑ ↓: Sélectionner session
  - Enter: Ouvrir détails
  - r: Reprendre session
  - F: Filtrer par agent/provider
- Logs:
  - ↑ ↓: Scroll
  - g/G: Aller début/fin
  - F: Suivre (tail -f)
  - /: Filtrer par texte/niveau

---

### Hiérarchie d'information
- Kanban: priorité visuelle sur colonnes (Todo/Doing/Done), puis cartes (titre, priorité, assignee), méta (compteurs, filtre, tri)
- Sessions: liste triée par last_activity, badges (provider, agent), état session; panneau détail sur Enter
- Logs: en-tête (agent/provider/session), zone de log, barre d’état (filtre, mode follow)

---

### Wireframes (ASCII, basse fidélité)

Kanban

```
┌ Multi-Agents TUI — Project: <name> ──────────────────────────────────────────┐
│ Kanban Board | Columns: 3 | Total: 42 | Completed: 8 | Filter: "api"       │
├───────────────────────────┬───────────────────────────┬──────────────────────┤
│ ▸ To Do (21)              │   Doing (8)               │   Done (13)          │
│ [H][#123] Setup TUI       │ [M][#220] Refactor state  │ [L][#090] README     │
│ [M][#124] API client      │ [H][#221] NDJSON tail     │ [M][#091] Migrations │
│ [L][#130] Typo pass       │ [L][#240] Theme tokens    │ ...                  │
│ ...                       │ ...                       │                      │
└───────────────────────────┴───────────────────────────┴──────────────────────┘
  ← →: columns, ↑ ↓: tasks, Enter: open, m: move, /: filter, t: sort, q: quit
```

Sessions

```
┌ Multi-Agents TUI — Project: <name> ──────────────────────────────────────────┐
│ Sessions (sorted by last activity)   Filter: agent:"backend" provider:"claude"│
├──────────────────────────────────────────────────────────────────────────────┤
│ ▸ [active] #sess-7d2c  backend/agent-a  claude  2m ago                       │
│   [idle]   #sess-2a10  frontend/agent-b cursor  12m ago                      │
│   [idle]   #sess-8fa1  devops/agent-c   gemini  1h ago                       │
├──────────────────────────────────────────────────────────────────────────────┤
│ Details: #sess-7d2c  provider_session_id=abc123  messages: 58                │
│ Actions: Enter=open  r=resume  d=details  F=filter                           │
└──────────────────────────────────────────────────────────────────────────────┘
  ↑ ↓: select, Enter: open, r: resume, F: filter, q: quit
```

Logs

```
┌ Multi-Agents TUI — Project: <name> ──────────────────────────────────────────┐
│ Logs — agent: backend/agent-a  provider: claude  session: #sess-7d2c         │
├──────────────────────────────────────────────────────────────────────────────┤
│ 10:04:22.120 start {"pid":1234,"args":[...]}                                │
│ 10:04:23.331 stdout_line "Connected"                                         │
│ 10:05:01.901 stdout_line "Message sent"                                      │
│ ...                                                                          │
├──────────────────────────────────────────────────────────────────────────────┤
│ Mode: follow  Filter: "error"  g/G: home/end  ↑ ↓: scroll  /: filter        │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

### Thèmes (couleurs & typographie)
- Modes: Dark (par défaut), Light
- Palette (Dark):
  - primary: cyan (accents, focus)
  - secondary: gray (borders, separators)
  - text: white/gray-200
  - danger: red; warning: yellow; success: green; info: blue
- Typographie (ratatui `Style` tokens):
  - title: bold
  - subtitle: bold + accent
  - body: normal
  - caption: dim

Implémentation: s’appuyer sur `crate::tui::themes::{Theme, ThemeKind, ThemePalette, Typography}`

---

### Patterns d’interaction
- Focus clair (inversion/accents) sur l’élément actif
- Feedback immédiat sur actions (barre de statut)
- Raccourcis persistants en footer; aide contextuelle via `h`
- Filtrage inline performant; tri cohérent entre vues
- Idempotence: opérations non destructives par défaut (confirmation si besoin)

---

### Intégration technique (propositions)
- Clavier: mapper `l` pour route Logs (à ajouter au runtime)
- Kanban: exposer `filter`, `sort_by` et actions `move_to_column` via handlers
- Sessions: ajouter vue détail (pane 70/30) avec Enter
- Logs: vue dédiée avec mode follow, filtres et raccourcis

---

### Critères d'acceptation (issue #50)
- Conception UX validée (ce document)
- Wireframes des 3 vues principales présents ci-dessus
- Hiérarchie d’information et navigation définies
- Patterns d’interaction et raccourcis listés

---

Dernière mise à jour: 2025-09-23T08:59:58+02:00


