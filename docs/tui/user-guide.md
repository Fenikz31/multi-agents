## TUI — Guide Utilisateur

### Objectifs
- Naviguer entre les vues (Kanban, Sessions, Aide)
- Gérer le focus et la sélection
- Changer de thème et densité d’affichage

### Lancer le TUI
```bash
multi-agents tui --project demo [--refresh-rate 200]
```

### Raccourcis Clavier
- q / Ctrl+C: Quitter
- gT: Cycle thème (Clair → Sombre → Haut Contraste)
- gM: Cycle densité (Normal → Compact → Haute Densité)
- h: Aide · k: Kanban · s: Sessions
- Flèches/PgUp/PgDn/Home/End: Navigation
- Tab / Shift+Tab: Changer de focus
- Enter: Sélection/Activation

### Kanban
- Colonnes: To Do, Doing, Done
- Filtrage (via champs prochainement), navigation par colonnes/éléments

### Sessions
- Liste des sessions avec statut et durée
- Filtrage et tri (selon implémentation courante)

### Conseils
- Pour des grands jeux de données, augmentez `--refresh-rate` (ex: 300)
- Les thèmes influencent le contraste des composants

### Problèmes fréquents
- Terminal non restauré: relancez une commande `reset` ou relancez le TUI (restauration automatique via RAII)
- Erreurs DB: exécuter `multi-agents db init` et vérifier permissions de `./data/`


