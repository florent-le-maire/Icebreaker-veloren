# CLAUDE.md — Icebreaker sur Veloren

Copie modifiée de Veloren (Rust, GPL v3) qui sert de prototype au jeu **Icebreaker** (équipe : Flo, Sayanel).
Le projet principal, sa vision et sa méthode sont dans le dépôt `../Icebreaker` ; ce qu'il faut ici est copié dans
`docs/icebreaker/` : lire `docs/icebreaker/PLAN.md` (tâches, questions ouvertes) avant de coder, puis la vision ou le
fichier de `docs/icebreaker/design/` utile.

## Règles

- Avancer tâche par tâche selon `PLAN.md` ; présenter un nouveau découpage et attendre l'accord avant de coder.
- Un commit par tâche, message en français, détail dans le corps ; cocher la tâche avec le hash.
- Nos changements restent **regroupés et repérables** (commentaire « Icebreaker : … ») pour pouvoir suivre les
  mises à jour de Veloren.
- Réglages (rayons, temps, prix) en configuration ou en fichiers de données (`assets/`), pas en dur.
- Rien n'est poussé sur le dépôt de Veloren (`origin`) ; un dépôt à nous sera créé si l'équipe continue.

## Commandes

- Compiler le serveur : `cargo build -p veloren-server-cli` ; le client : `cargo build -p veloren-voxygen`.
- Tests ciblés : `cargo test -p <crate> <filtre>`.
- Lieu de la campagne : `powershell -ExecutionPolicy Bypass -File .\icebreaker.ps1 village` (port 14100) ou
  `donjon -Site Gnarling` (port 14110 ; route 14120 à venir), puis le client en multijoueur sur `localhost:<port>`.
  Personnages communs à tous les lieux dans `userdata-icebreaker\campagne`.
- Lancer depuis la racine du dépôt (le jeu y cherche `assets/`). Un serveur qui tourne verrouille son `.exe` :
  le renommer avant de recompiler plutôt que d'arrêter un serveur qu'on n'a pas lancé.
