# Wiki de l'équipe — site généré depuis les docs (jalon W)

Flo et Sayanel veulent un site façon wiki pour concevoir le jeu : voir ce qui est fini, en cours ou à vérifier en
jeu, et comprendre chaque mécanique à deux niveaux — la règle du jeu pour le joueur, puis « sous le capot » sans
lire le code. Référence d'ambiance : le wiki de Baldur's Gate 3 (thème sombre, barre latérale, fiches avec encart de
chiffres, tableaux de stats). Lecteurs : l'équipe seulement ; le dépôt GitHub est privé, donc pas de GitHub Pages.

## Choix (Flo, 2026-11-05)

- **Une seule source de vérité** : le site est généré depuis `docs/icebreaker/` et `assets/icebreaker/`. On n'écrit
  jamais dans le site ; on écrit dans la doc. La vulgarisation va dans de nouveaux fichiers `docs/icebreaker/wiki/`.
- **Générateur maison en Rust** : crate `icebreaker-wiki/` à la racine, membre du workspace
  (`pulldown-cmark` pour le Markdown, `minijinja` pour les gabarits, `git log` pour les commits). Pas de Node ni de
  Python : l'équipe a déjà la chaîne Rust, et un vrai moteur de gabarits est nécessaire pour les fiches (encarts,
  tableau de bord). Écarté : MkDocs Material / mdBook (tableau de bord et encarts difficiles sans plugins).
- **Hébergement sur le VPS existant** : Caddy sert le dossier généré en HTTPS avec un mot de passe ; un script
  `icebreaker-serveur/deployer-wiki.sh` génère puis envoie le site. Pas de CI : on régénère et déploie à la main,
  comme le serveur. En local : `cargo run -p icebreaker-wiki` depuis la racine, puis ouvrir `site/index.html`.
- **Langue** : français partout.

## Contenu du site

1. **Accueil / tableau de bord** : tableau des jalons (lettre, titre, état, tâches cochées / total, dernier commit),
   compteur « À vérifier en jeu », dernières tâches cochées. Source : `README.md`, `PLAN.md`, `HISTORIQUE.md`.
2. **Fiches mécaniques**, une page par domaine (à terre et mort, combat, classes et compétences, village, âmes,
   donjons et génération, grotte, clés et serrures, campagne et lieux…), à trois étages :
   - *La règle* : texte de vulgarisation (`docs/icebreaker/wiki/<domaine>.md`), encart à droite avec les chiffres
     clés lus dans le `.ron` du domaine (ex. `mort.ron` : 25 %, 30 s, 3 s).
   - *Le détail* : le fichier `design/<domaine>.md` rendu tel quel, repliable.
   - *Sous le capot* : systèmes et fichiers concernés (listés dans l'en-tête de la fiche), réglages complets du
     `.ron` (clé, valeur, commentaire), tâches liées avec hash, résumé du commit et son paragraphe « Testé ».
   En-tête de liaison (TOML) : `design = "combat.md"`, `reglages = ["combat.ron"]`, `taches = ["T", "C.3"]`,
   `systemes = ["server/src/sys/icebreaker_mort.rs"]`.
3. **Avancement** : une page par jalon avec les tâches de `PLAN.md` et `HISTORIQUE.md` et leurs commits ; sections
   « À vérifier en jeu » et « Plus tard ».
4. **Décisions**, **Questions / Idées** : rendu des fichiers existants, avec badges d'état (tranché / ouvert).
5. **Vision** : `VISION.md`, `design/piliers.md`.
6. Recherche plein texte côté client (index JSON, petit script) et navigation latérale par rubrique.

Design : thème sombre (bleu nuit / glace, accent chaud), titres en Cinzel, **pas d'émoji** (Flo, 2026-11-05) : les
pictogrammes sont les icônes en pixel art de Veloren (`assets/voxygen/element/`) et des SVG ; barre latérale fixe, largeur de lecture ~72 caractères,
encart flottant sur les fiches, tableaux à lignes alternées, badges d'état, hash cliquable vers GitHub. Une feuille
CSS maison, pas de framework ; fonctionne sans JS sauf recherche et repli.

## Tâches (worktree `Veloren-wiki`, branche `icebreaker-wiki`)

- [x] **W.1 Squelette du générateur** — crate `icebreaker-wiki`, Markdown rendu dans un gabarit à barre latérale,
  thème sombre ; 33 pages, vues avec Edge sans fenêtre. ad8ad8b7a3
- [x] **W.2 Tâches et commits** — tâches de PLAN, HISTORIQUE et PLAN-* liées à `git log` (sujet, corps, « Testé »,
  lien GitHub) ; tableau de bord en accueil et une page par jalon. Un hash de X (af89bed30e) est introuvable. c78e057b7d
- [x] **W.3 Réglages** — chaque `.ron` de `assets/icebreaker/` lu avec ses commentaires (groupes, listes) ; une page par
  fichier avec chiffres clés et tableau réglage / valeur / ce que ça fait. eb20c8b4f1
- [x] **W.4 Fiches mécaniques** — `wiki/<domaine>.md` (en-tête TOML : design, réglages, tâches, systèmes, chiffres) → page
  à trois étages ; sept fiches écrites (campagne, village, charrette, combat, mort, donjon, grotte), à relire. 8ff80b6298
- [x] **W.5 Décisions, questions, idées, vision** — page Questions (cartes Qn, badges, tranchées par date) et index des
  décisions (date, statut) ; vision et idées en pages Markdown. 8d4a8fe6ff
- [x] **W.6 Recherche et finitions** — recherche plein texte (index généré, touche `/`), rubriques repliables, petit
  écran vu à 600 px seulement ; recherche pas essayée dans un vrai navigateur. 8593daae75
- [ ] **W.7 Déploiement VPS** — *en cours : Veloren-wiki, 2026-11-05* — script `icebreaker-serveur/deployer-wiki.sh`
  écrit (Caddy, mot de passe, `ADRESSE` pour un nom de domaine) et note dans `CLAUDE.md` ; **pas encore lancé** : il
  faut choisir le mot de passe (et un nom de domaine, sinon `http://116.202.107.125:8090`).
- [ ] **W.8 Identité visuelle sans émoji** — *en cours : Veloren-wiki, 2026-11-05* — logo SVG, icône de Veloren par fiche
  (`icone` = chemin sous `assets/`), titres en Cinzel, en-tête de page avec icône et résumé, cartes des mécaniques en
  accueil, pictogrammes en SVG (plus aucun émoji ni symbole texte).
- [ ] **W.9 Schémas dessinés depuis les données** — module Rust qui produit du SVG avec les valeurs des `.ron` : frise
  à terre → mort → sac, carte de campagne (`lieux.ron`, `campagne.ron`), circuit charrette et registre, étages du
  donjon, niveaux de la grotte ; la fiche déclare son schéma dans l'en-tête.
- [ ] **W.10 Captures d'écran** — `docs/icebreaker/wiki/images/`, `![légende](images/x.png)` dans une fiche rendu en
  figure avec légende et zoom ; encadré « capture à prendre » tant qu'elle manque.
- [ ] **W.11 Icônes dans les tableaux** — icône de Veloren à côté des armes des classes, des états et des objets dans
  les tableaux de réglages et de design, quand elle existe.

Hors périmètre pour l'instant : accès public, édition dans le navigateur, mise à jour automatique à chaque push,
historique des valeurs des réglages dans le temps (`IDEES.md`).
