## M6.9 — Accessibilité & Responsive (Issue #55)

Objectif: Définir et valider l’accessibilité complète et l’adaptation responsive de la TUI (ratatui), avec support clavier exhaustif, thèmes clair/sombre/haut contraste, indicateurs a11y, et modes d’affichage alternatifs.

### Personas
- Opérateur DevOps en environnement SSH (contraste élevé requis, police lisible)
- Développeur backend sur terminal partagé (navigation clavier rapide, repères focus)
- SRE sur petit écran/VM (layout compact, reflow minimal)

### Principes & Critères A11y (adaptés TUI)
- Focus visible et persistant (colonnes, listes, éléments)
- Navigation clavier complète: pas de piège; retour visuel de focus
- Lisibilité: contraste suffisant (HC ≥ 7:1), tailles min. typographiques
- Feedback non intrusif: toasts, statut global, libellés explicites
- Alternatives visuelles: thèmes clair/sombre/HC, mode compact

### Thèmes (Clair / Sombre / Haut Contraste)
- Palette: primary, secondary, surface, text, success, warning, error
- HC: accentuer `text` et `primary`, bordures renforcées, focus inversé
- Typographie: body, subtitle, caption; tailles +1 pas en HC (optionnel)

### Responsive (Tailles d’écran)
- XS (≤ 80x24):
  - Header 1 ligne; Footer 1 ligne (cheat-sheet condensée)
  - Vues: Kanban en 1 colonne (tabs pour colonnes), Sessions liste compacte
- S (≈ 120x30):
  - Kanban 2 colonnes; Sessions liste + détails abrégés
- M+ (≥ 160x40):
  - Kanban 3 colonnes; Sessions étendues; Detail avec padding

### Indicateurs A11y
- Focus highlight: inversion + cadre primary
- Badges lisibles (état session, priorité tâche) avec alternatives HC
- Statut global: icône d’état, projet, vue, focus, dernière action

### Modes d’affichage alternatifs
- Compact: densité augmentée (marges/bordures réduites)
- High-Density: typographie -1 pas, interlignes serrés (optionnel)
- HC Mode: palette dédiée + outlines renforcés

### Keymap A11y/Responsive (compléments)
- g T: cycle thèmes (clair → sombre → HC)
- g M: cycle modes (normal → compact → high-density)
- g R: cycle layouts (XS → S → M+) pour test

### Wireframes (ASCII)

Header/Body/Footer (HC)
```
┌ Multi-Agents — Project:<name>  ● Active  View:Kanban  Focus:Body ───────────┐
│ [Body: selon vue; colonnes réduites en XS; 1→2→3 colonnes selon largeur]   │
└ q quit | h help | g k/s/d/p go-to | Tab focus | gT theme | gM mode | gR sz │
```

Kanban XS (1 colonne + onglets)
```
┌ Kanban [To Do|Doing|Done] (Tab pour changer) ───────────────────────────────┐
│ ▶ [P1] Setup TUI                                                             │
│   [P2] Load sessions                                                         │
└ ← → colonnes | ↑ ↓ tâches | </> déplacer | n new | / filtre | t tri         │
```

### Acceptation
- Support clavier complet; focus visible partout
- Thèmes clair/sombre/HC commutables à chaud (gT)
- Layouts adaptatifs XS/S/M+ conformes
- Indicateurs a11y présents et lisibles
- Modes compact/high-density disponibles (gM)

Dernière mise à jour: 2025-09-23T14:20:00+02:00


