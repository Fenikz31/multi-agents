## M6.8 — Navigation & Interaction (Issue #54)

Objectif: Définir le système de navigation et d'interaction de la TUI (vues, raccourcis complets, menu contextuel, statut global, notifications/toasts).

### Vues et Navigation
- Vues principales: Project Select, Kanban, Sessions, Detail
- Transitions:
  - g k → Kanban, g s → Sessions, g d → Detail, g p → Project Select
  - h → Aide/Help; q → Quitter
- Focus model: header (status), body (content), footer (shortcuts)

### Raccourcis Clavier (keymap)

Global
- q: Quitter
- h: Aide
- /: Rechercher
- Tab / Shift+Tab: Changer de focus (header/body/footer)
- g k / g s / g d / g p: Aller à Kanban / Sessions / Detail / Project Select
- F: Toggle follow (logs)

Kanban
- ← →: Naviguer entre colonnes
- ↑ ↓: Sélectionner les tâches
- Tab / Shift+Tab: Changer le focus (colonnes/éléments)
- <: Déplacer la tâche à gauche
- >: Déplacer la tâche à droite
- Space: Passer la tâche au prochain statut
- n: Nouvelle tâche
- t: Trier
- /: Filtrer

Sessions
- ↑ ↓: Naviguer entre les sessions
- Enter: Attacher à la session
- S: Démarrer
- X: Arrêter
- r: Reprendre
- t: Trier
- /: Filtrer

Detail (Logs)
- ↑ ↓: Scroll
- g / G: Haut / Bas
- F: Follow on/off
- 1 / 2 / 3: Filtrer niveau info / warn / error
- /: Rechercher
- e: Exporter

### Menu Contextuel (Context Menu)
- Invocation: m
- Items contextuels par vue:
  - Kanban: move left/right, new, filter, sort
  - Sessions: resume, stop, start, filter by agent
  - Detail: follow on/off, level filter presets, export

### Indicateurs de Statut Global
- Zone header: icônes/symboles (● active, ◐ busy, ⚠ warn, ✖ error)
- Affiche: projet courant, vue, focus, messages système (dernière action)

### Notifications & Toasts
- Types: info (3s), success (2s), warn (4s), error (persistant jusqu’à dismiss)
- Position: coin inférieur droit; pile avec max 3 visibles; overflow en file
- API UI: enqueue_toast(type, message, ttl_ms?)

### Wireframes (ASCII)

Header/Body/Footer
```
┌ Multi-Agents TUI — Project:<name>  ● Active  View:Kanban  Focus:Body ──────┐
│ [Body content per view]                                                   │
└ q quit | h help | g k/s/d/p go-to | Tab focus | / search | m menu | F log │
```

Context Menu (example)
```
┌─ Menu ────────────────────────────┐
│ 1) Move Left                      │
│ 2) Move Right                     │
│ 3) New Task                       │
│ 4) Filter...                      │
│ 5) Sort by Priority               │
└───────────────────────────────────┘
```

Toasts
```
                  [ Saved successfully ✓ ]
            [ Export completed: logs-20250923.txt ]
```

Footer (cheat-sheet dynamique)
```
q quit | h help | / search | g k/s/d/p go-to | Tab focus | m menu | F follow
```

Header (statut global)
```
┌ Multi-Agents TUI — Project:<name>  ● Active  View:<view>  Focus:<Header|Body|Footer> ─┐
```

### Critères d’acceptation
- Navigation fluide (raccourcis go-to, Tab/Shift+Tab)
- Raccourcis complets documentés et visibles en footer
- Menu contextuel accessible (m) avec actions par vue
- Indicateurs de statut globaux cohérents
- Notifications/toasts non intrusives, empilables

Dernière mise à jour: 2025-09-23T14:12:00+02:00


