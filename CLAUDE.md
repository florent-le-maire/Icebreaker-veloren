# CLAUDE.md — Icebreaker sur Veloren

Copie modifiée de Veloren (Rust, GPL v3) qui sert de prototype au jeu **Icebreaker** (équipe : Flo, Sayanel).
Le projet principal, sa vision et sa méthode sont dans le dépôt `../Icebreaker` ; ce qu'il faut ici est copié dans
`docs/icebreaker/` : lire `docs/icebreaker/README.md` (fichiers, jalons, conventions), puis `PLAN.md` (état, tâches,
à vérifier en jeu) et `QUESTIONS.md` avant de coder, puis le fichier de `docs/icebreaker/design/` utile.

## Règles

- Avancer tâche par tâche selon `PLAN.md` ; présenter un nouveau découpage et attendre l'accord avant de coder.
- Début de session : traiter la boîte de l'équipe, remplie depuis le wiki local. Une question de `QUESTIONS.md` qui porte
  une `*Réponse de … (date)*` passe dans « Tranché » (date, fichier où la règle est écrite) après report dans le plan
  ou le design ; une entrée « Ouvert » de `A-FAIRE.md` devient tâche, question ou idée (avec l'accord de l'équipe),
  puis passe dans « Pris en compte » avec `→ <où>` ; une tâche marquée « Vu en jeu (…) » sort de la liste « À vérifier en
  jeu » de `PLAN.md`.
- Un commit par tâche, message en français, détail dans le corps ; cocher la tâche avec le hash. Ce qui n'est pas
  vérifié en jeu s'ajoute aussi à « À vérifier en jeu » de `PLAN.md` ; un jalon fini part dans `HISTORIQUE.md`.
- Nos changements restent **regroupés et repérables** (commentaire « Icebreaker : … ») pour pouvoir suivre les
  mises à jour de Veloren.
- Réglages (rayons, temps, prix) en configuration ou en fichiers de données (`assets/`), pas en dur.
- Notre dépôt est `origin` (GitHub, `git pull` / `git push` sur `master`) ; Veloren est le remote `veloren`, rien n'y est
  poussé. L'ancien `master` (avec l'historique de Veloren) est gardé sous l'étiquette `ancien-master`.
- Teste ce que tu écris (décision `0023`). **TDD** (skill `superpowers:test-driven-development` : test d'abord, vu
  en échec, puis le code) pour : les règles de jeu de `docs/icebreaker/design/` (le test cite la règle ; une règle
  qu'on n'arrive pas à tester est ambiguë → `QUESTIONS.md`) ; les générateurs (grottes, forêts, donjon, modèles), en
  testant des propriétés (salles reliées, abri présent, même graine → même résultat), pas des formes exactes ; chaque
  bug corrigé (d'abord un test qui le reproduit). **Pas de TDD** pour le ressenti, le rendu, la caméra, le son ; pour
  les valeurs de `assets/` (tester la règle, « difficulté 6 > difficulté 1 », jamais le chiffre) ; pour le code mêlé
  à Veloren dans `server/` ou `voxygen/` : en sortir la logique dans une fonction pure (`common/src/icebreaker_*.rs`
  ou un crate `icebreaker-*`), la tester, garder le branchement mince. Toujours : lancer le lieu concerné pour
  vérifier qu'il démarre sans erreur, et dire dans la tâche cochée ce qui n'a pas été vérifié en jeu.
- Reste simple : la solution la plus simple qui marche pour la tâche en cours.
- Fin de session : résumer ce qui a été fait, ce qui reste ; mettre à jour « État » de `PLAN.md` si le tableau a
  changé ; les questions pour l'équipe vont dans `QUESTIONS.md` (numéro Qn suivant), jamais dans un plan ; une
  question tranchée passe dans son « Tranché » avec la date. Une idée non décidée va dans `IDEES.md`.

## Travailler à plusieurs agents (repris de `../Icebreaker/CLAUDE.md`, §5 bis)

Plusieurs agents travaillent en même temps sur ce dépôt. La branche de référence est **`master`**,
ouverte dans le dossier principal `Veloren-test` ; Veloren d'origine reste lisible dans `veloren/master`.

- **Un agent = un worktree = une piste**, jamais à plusieurs dans le même dossier. Au démarrage, avant de toucher au
  code : `git worktree list` ; si le worktree de la piste existe, y travailler, sinon le créer :
  `git worktree add ../Veloren-<piste> -b icebreaker-<piste> master`, puis travailler **uniquement**
  dans ce dossier. Pistes, plans et worktrees : tableau des jalons dans `docs/icebreaker/README.md`.
- Le premier build d'un worktree refait toute la compilation dans son propre `target/` (long, c'est normal).
- Une simple question, une relecture ou une modification de doc d'une ligne n'ont pas besoin de worktree.
- Tâche finie et testée → commit sur la branche de la piste, `git rebase master`, puis fusion en avance
  rapide (`git merge --ff-only icebreaker-<piste>` depuis `Veloren-test`). Fusionner souvent, à chaque tâche.
- Si on doit malgré tout partager un dossier : committer uniquement ses propres changements via un index git
  temporaire (`GIT_INDEX_FILE`), jamais `git add -A` sur l'index partagé.
- Pour prendre une tâche, l'annoncer dans le plan : `— *en cours : <worktree>, <date>*`.
- Tâche cochée = court : 1 à 3 lignes de résultat (dont ce qui reste ou n'est pas vérifié) et le hash ; le détail va
  dans le corps du message de commit.
- Fichiers partagés (`PLAN.md`, `QUESTIONS.md`, `HISTORIQUE.md`, `CLAUDE.md`, `icebreaker.ps1`, `server/src/lib.rs`,
  `server/src/settings/mod.rs`) :
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
- Campagne (décision 0016) : `powershell -ExecutionPolicy Bypass -File .\icebreaker.ps1 campagne` (port 14100 : le
  village et les emplacements de donjon dans un seul serveur), ou le bouton « Campagne » du menu principal. Un lieu
  seul, pour tester : `village`, `donjon -Site Gnarling` (port 14110), `donjon -Site Genere` (`-Difficulte 1..6`,
  jalon DF ; en campagne `/expedition difficulte <1-6>`, au lieu d'entraînement `/entrainement difficulte <1-6>`) ; client en multijoueur
  sur `localhost:<port>`. Une partie (personnages, caisse, village…) = `userdata-icebreaker\parties\<nom>`, choisie
  par `-Partie <nom>` (défaut : la dernière jouée) ; au menu, `ICEBREAKER_AUTO_PARTIE=<nom>` avec l'auto-clic.
- Atelier des grottes (jalon U, `docs/icebreaker/PLAN-CAVERNE.md`) : `.\icebreaker.ps1 caverne` compile le générateur
  `icebreaker-caverne` pour le navigateur (`rustup target add wasm32-unknown-unknown` une fois) et sert la page à
  curseurs sur http://localhost:8096 ; « Enregistrer le preset » écrit le profil dans
  `assets/icebreaker/grottes/formes.ron` et committe ; en campagne : `/expedition 3 <preset>`. En jeu : `donjon -Site Grotte -Profil gouffre`.
  Sans la page : `cargo run --release -p icebreaker-caverne --example coupes -- <profil> <graine> <dossier>`.
- Atelier des salles (F.14 à F.17, `docs/icebreaker/PLAN-PLAYTEST.md`) : `.\icebreaker.ps1 salles` sert sur
  http://localhost:8097 l'éditeur où l'on meuble les salles du donjon au bloc près avec les objets de Veloren
  (`?salle=<nom>` ouvre une salle) ; « Enregistrer » contrôle la salle comme le jeu, écrit
  `assets/icebreaker/donjons/salles/<nom>.txt` (`@meuble:`, `@manque:` pour un modèle qui manque) et committe. En
  jeu : `donjon -Site Genere -Salle <nom>`. Une pièce vivante (nos modèles voxel générés par
  `icebreaker-modeles`, salle serrée, vérifiée en jeu) : skill `piece-vivante` (`.claude/skills/piece-vivante`).
- Atelier du donjon (F.34, `docs/icebreaker/PLAN-PLAYTEST.md`) : `.\icebreaker.ps1 donjon-atelier` sert sur
  http://localhost:8098 le donjon généré en 3D (ou à plat, étage par étage), avec des curseurs ; « Enregistrer le preset » écrit
  le profil dans `assets/icebreaker/donjons/formes.ron` et committe. En jeu : `donjon -Site Genere -Profil <preset>`,
  en campagne `/expedition 2 <preset>`. Le donjon généré est tout sous le sol : on arrive dans l'abri (F.31).
- Atelier des forêts (jalon FO, `docs/icebreaker/PLAN-FORET.md`) : `.\icebreaker.ps1 foret` compile le générateur
  `icebreaker-foret` pour le navigateur et sert la page à curseurs sur http://localhost:8099 ; « Enregistrer le preset »
  écrit le profil dans `assets/icebreaker/forets/formes.ron` et committe ; en jeu : `donjon -Site Foret -Profil immense`,
  en campagne `/expedition 6 <preset>`. Sans la page :
  `cargo run --release -p icebreaker-foret --example coupes -- <profil> <graine> <dossier>`.
- Classeur d'équilibrage (jalon EQ, `docs/icebreaker/PLAN-PLAYTEST.md`, § 8) : `.\icebreaker.ps1 equilibrage` (ou
  `cargo run -p icebreaker-equilibrage`) relit les chiffres du jeu avec les structures du jeu et réécrit
  `docs/icebreaker/equilibrage.xlsx` (Synthèse, Combat, Ennemis, Progression, Économie, Expéditions, Réglages) ; on essaie
  une valeur dans l'onglet Hypothèses, on la règle pour de bon dans le fichier qu'il indique. À relancer et committer
  quand un chiffre de `assets/icebreaker/` change. Sans tableur : les mêmes tableaux sont écrits en Markdown dans
  `docs/icebreaker/equilibrage/` (lire `synthese.md` d'abord ; ils sont aussi des pages du wiki), et `cargo run -q -p
  icebreaker-equilibrage -- --resume [filtre]` affiche la vue d'ensemble. Régler un chiffre (mesurer, quel fichier changer, vérifier) : skill
  `equilibrage` (`.claude/skills/equilibrage`).
- Lieu d'entraînement (jalon O, `docs/icebreaker/PLAN-ENTRAINEMENT.md`) : `.icebreaker.ps1 entrainement` (port
  14150) ; tout s'y règle par `/entrainement` (classe, arme, niveau, ennemi, allie, nettoyer, soin…) ou par les deux
  cases du plan (touche U). Sans registre : `ICEBREAKER_AUTO_PERSONNAGE=1` fait entrer le vrai client dans le monde
  sans clic (avec `ICEBREAKER_AUTO_CAMPAGNE=1` et `ICEBREAKER_AUTO_ADRESSE`).
- IA des ennemis (jalon IA, `docs/icebreaker/PLAN-IA.md`, décision `0024`) : la décision de nos six espèces est dans le
  crate pur `icebreaker-ia` (`cargo test -p icebreaker-ia`), ses réglages dans `assets/icebreaker/ia/profils.ron`, son
  branchement dans `server/src/sys/agent/icebreaker_ia.rs` ; `/entrainement ia` fait dire son état à chaque ennemi.
- Client texte pour tester sans affichage : `cargo build -p veloren-client --example chat_cli --features
  "plugins bin_bot"`, puis sur l'entrée : pseudo, `localhost:<port>`, un mot de passe quelconque, et des lignes de
  chat (`/commande args` part en commande). Laisser ~40 s entre deux connexions coupées net : le serveur n'accepte
  la suivante qu'une fois la précédente expirée. Avec `ICEBREAKER_JOUER=1`, la ligne `!jouer` entre dans le monde
  avec un personnage (pour `/goto`, `/explosion`… ; admin : `veloren-server-cli --no-auth admin add <pseudo> admin`
  avec le même `VELOREN_USERDATA`) ; puis `!ouvrir x y z` (serrure, coffre), `!bloc x y z`, `!sac`, `!voisins`, `!echange`.
- Vrai client sans clic : `ICEBREAKER_AUTO_CAMPAGNE=1` fait cliquer « Campagne » au menu (pseudo pris dans
  `<userdata>/voxygen/settings.ron`, `VELOREN_USERDATA` pour un dossier à part) ; journal dans `<userdata>/voxygen/logs`.
  Avec `ICEBREAKER_AUTO_ADRESSE=localhost:<port>`, il va droit à ce serveur sans rien lancer ; avec
  `ICEBREAKER_AUTO_PLAN=charrette,caisse,/expedition 2`, il ouvre le plan du village 25 s après l'entrée, clique
  chaque bâtiment (ou envoie la commande) et prend une capture à chaque pas (dossier `screenshots_path` de ses
  réglages ; `fullscreen: (enabled: false)` pour une fenêtre) ; `fermer` ferme le plan, `regarder` fait le clic
  gauche d'un mort, `touche 1` appuie la touche de compétence 1, `onglet 3` ouvre le troisième onglet du panneau,
  `rituel` fait la touche d'interaction devant le portail ou l'autel le plus proche, `lanterne` allume la lanterne, `cap 180` tourne la caméra vers le haut du plan (0 vers le bas).
  `/aller autel` et `/aller portail` (admin) téléportent devant l'autel ou le portail le plus proche.
  `ICEBREAKER_AUTO_MENU=menu|parties` : une capture du menu ou du choix de la partie.
- Wiki de l'équipe (jalon W, `docs/icebreaker/PLAN-WIKI.md`) : `cargo run -p icebreaker-wiki` écrit `site/` (ouvrir
  `site/index.html`) depuis `docs/icebreaker/` et `assets/icebreaker/` ; la vulgarisation s'écrit dans
  `docs/icebreaker/wiki/<domaine>.md` (en-tête TOML : design, réglages, tâches, systèmes, chiffres, icone, schema) ;
  pas d'émoji ; les captures d'écran vont dans `docs/icebreaker/wiki/images/` (`![légende](images/x.png)`), celles des créatures dans `images/creatures/<id>.png`. Sur le VPS :
  `MOT_DE_PASSE=… icebreaker-serveur/deployer-wiki.sh root@<ip>` (Caddy, mot de passe). En local avec saisie :
  `cargo run -p icebreaker-wiki -- serve` depuis `Veloren-test` (http://localhost:8095, `--nom Sayanel`) : répondre aux
  questions et noter des todos, chaque saisie fait un commit sur master.
- Client à télécharger (S.5, S.6) : GitHub Actions « Release du client » (nouveau numéro dans
  `assets/icebreaker/version.ron` poussé sur master → Release `v<version>` ; bouton « Run workflow » → pré-release
  `dernier-build`) produit `Icebreaker-windows.zip` et `Icebreaker-mac.zip` ; à la main :
  `icebreaker-serveur/paquet-client.sh` (`docs/icebreaker/MAINTENANCE.md`, § 3).
- Lancer depuis la racine du dépôt (le jeu y cherche `assets/`). Un serveur qui tourne verrouille son `.exe` :
  le renommer avant de recompiler plutôt que d'arrêter un serveur qu'on n'a pas lancé.
