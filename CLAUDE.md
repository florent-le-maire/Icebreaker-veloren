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
- Teste ce que tu écris : tests ciblés pour la logique, et lancer le lieu concerné pour vérifier qu'il démarre sans
  erreur ; dire dans la tâche cochée ce qui n'a pas été vérifié en jeu.
- Reste simple : la solution la plus simple qui marche pour la tâche en cours.
- Fin de session : résumer ce qui a été fait, ce qui reste, et ajouter les questions pour l'équipe dans la section
  « Questions ouvertes » du plan concerné.

## Travailler à plusieurs agents (repris de `../Icebreaker/CLAUDE.md`, §5 bis)

Plusieurs agents travaillent en même temps sur ce dépôt. La branche de référence est **`icebreaker-traces`**,
ouverte dans le dossier principal `Veloren-test`.

- **Un agent = un worktree = une piste**, jamais à plusieurs dans le même dossier. Au démarrage, avant de toucher au
  code : `git worktree list` ; si le worktree de la piste existe, y travailler, sinon le créer :
  `git worktree add ../Veloren-<piste> -b icebreaker-<piste> icebreaker-traces`, puis travailler **uniquement**
  dans ce dossier. Pistes actuelles : `Veloren-donjon`, `Veloren-mod` (plan `docs/icebreaker/PLAN-MOD.md`).
- Le premier build d'un worktree refait toute la compilation dans son propre `target/` (long, c'est normal).
- Une simple question, une relecture ou une modification de doc d'une ligne n'ont pas besoin de worktree.
- Tâche finie et testée → commit sur la branche de la piste, `git rebase icebreaker-traces`, puis fusion en avance
  rapide (`git merge --ff-only icebreaker-<piste>` depuis `Veloren-test`). Fusionner souvent, à chaque tâche.
- Si on doit malgré tout partager un dossier : committer uniquement ses propres changements via un index git
  temporaire (`GIT_INDEX_FILE`), jamais `git add -A` sur l'index partagé.
- Pour prendre une tâche, l'annoncer dans le plan : `— *en cours : <worktree>, <date>*`.
- Tâche cochée = court : 1 à 3 lignes de résultat (dont ce qui reste ou n'est pas vérifié) et le hash ; le détail va
  dans le corps du message de commit.
- Fichiers partagés (`PLAN.md`, `CLAUDE.md`, `icebreaker.ps1`, `server/src/lib.rs`, `server/src/settings/mod.rs`) :
  modifications courtes et localisées, pour que les fusions restent simples.
- Lire juste ce qu'il faut : dans `docs/icebreaker/design/`, seulement le domaine utile ; l'historique se cherche
  dans `git log`.

## Quand demander à l'équipe

Avant de changer un choix technique, quand une règle de jeu est ambiguë ou manquante, quand une tâche s'avère bien
plus grosse que prévu, ou pour choisir entre « fidèle à la vision » et « faisable rapidement ». Pour une décision
importante, écrire une note dans `docs/icebreaker/decisions/` (problème, options, recommandation).

## Commandes

- Compiler le serveur : `cargo build -p veloren-server-cli --features plugins` (sans la feature, le mod n'est pas
  chargé) ; le client : `cargo build -p veloren-voxygen`. Le mod (`plugin/icebreaker`) : `.\icebreaker.ps1 mod`
  (cible `wasm32-wasip2`, plan dans `docs/icebreaker/PLAN-MOD.md`).
- Tests ciblés : `cargo test -p <crate> <filtre>`.
- Lieu de la campagne : `powershell -ExecutionPolicy Bypass -File .\icebreaker.ps1 village` (port 14100) ou
  `donjon -Site Gnarling` (port 14110 ; route 14120 à venir), puis le client en multijoueur sur `localhost:<port>`.
  Personnages communs à tous les lieux dans `userdata-icebreaker\campagne`.
- Client texte pour tester sans affichage : `cargo build -p veloren-client --example chat_cli --features
  "plugins bin_bot"`, puis sur l'entrée : pseudo, `localhost:<port>`, un mot de passe quelconque, et des lignes de
  chat (`/commande args` part en commande). Laisser ~40 s entre deux connexions coupées net : le serveur n'accepte
  la suivante qu'une fois la précédente expirée.
- Lancer depuis la racine du dépôt (le jeu y cherche `assets/`). Un serveur qui tourne verrouille son `.exe` :
  le renommer avant de recompiler plutôt que d'arrêter un serveur qu'on n'a pas lancé.
