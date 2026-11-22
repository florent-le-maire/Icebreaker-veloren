# Jalon U — Grottes organiques et atelier de réglage

*Plan validé par Flo le 2026-11-15 ; U.1 à U.5 codés le même jour.* Worktree `Veloren-caverne`, branche
`icebreaker-caverne`. Question pour l'équipe : `QUESTIONS.md`, Q54.

## Pourquoi

La Grotte du jalon K était faite de poches en ellipsoïde reliées par des tubes droits : on y sentait encore « des
couloirs qui donnent sur des salles ». Flo veut des grottes à la Deep Rock Galactic : selon la graine, très
verticales ou très étalées, d'immenses zones plates à stalactites, des salles pleines de bosses et de trous ; et un
outil où l'on bouge des paramètres en voyant la grotte se refaire.

Décidé avec Flo (2026-11-15) : l'aperçu est une **page web 3D** ; le nouveau générateur **remplace la forme de la
Grotte** (ambiances, filons, habitants, chef et butin restent ceux du jalon K).

## Comment c'est fait

Un seul générateur, `icebreaker-caverne/` : un crate sans dépendance à Veloren, utilisé par le jeu
(`world/src/icebreaker/grotte/mod.rs`) et compilé en WebAssembly pour la page. Ce qu'on voit dans l'atelier est la
grotte que le jeu creuse pour la même graine et les mêmes réglages (sur terrain plat).

1. **Zones** posées depuis l'entrée, à côté ou en dessous de la précédente selon `verticalite` ; elles se touchent
   ou se recouvrent selon `ecart`. Deux types : **plate** (contour irrégulier, sol presque plat, plafond en relief,
   stalactites, stalagmites, piliers) et **chaotique** (bruit 3D : bosses, trous, ponts de roche, replats). Des
   **niches** au bord des zones (coffres, bêtes seules).
2. **Liens** : une galerie large et sinueuse si la pente le permet ; sinon un **puits** creusé à l'écart, où le
   sentier descend en hélice le long de la paroi.
3. **Sentier** : posé en dernier (roche sous les pieds, 4 blocs d'air), il relie la bouche à chaque zone avec des
   marches d'un bloc au plus, dans les deux sens. Le générateur le vérifie à pied ; si une zone n'est pas atteinte
   (deux sentiers qui se coupent), il essaie la graine dérivée suivante (6 au plus).
4. **Grille** : un bit par bloc (air ou roche), plus la nature de la roche (sentier, concrétion). Le jeu creuse
   d'après elle ; les sols où poser habitants, coffres et cristaux en sont tirés (ceux qu'on atteint à pied).
5. **Profils** (`assets/icebreaker/grottes/formes.ron`) : `gouffre`, `plaine`, `chaos`, `mixte`. La graine en tire un
   selon les poids ; `-Profil` du lanceur (`ICEBREAKER_GROTTE_PROFIL`) en demande un.

Les réglages d'un profil sont tous des nombres bornés, déclarés une fois (`forme!` dans `icebreaker-caverne/src/lib.rs` :
nom, défaut, bornes, libellé) : l'atelier en tire ses curseurs, le jeu son fichier.

**L'atelier** : `.\icebreaker.ps1 caverne` compile le générateur (cible `wasm32-unknown-unknown`, à installer une
fois : `rustup target add wasm32-unknown-unknown`) et ouvre `icebreaker-caverne/web/index.html`. Graine, profil, un
curseur par réglage ; la grotte en 3D vue de l'intérieur des parois, vues perspective / dessus / côté, coupe
horizontale ; « Copier les réglages » donne le bloc `forme` à coller dans `formes.ron`. three.js vient d'internet.
`?profil=gouffre&graine=3&vue=cote&coupe=-40` ouvre sur une grotte précise.

Sans la page : `cargo run --release -p icebreaker-caverne --example coupes -- <profil> <graine> <dossier>` écrit deux
images (dessus, côté) ; `-- stats` compte les grottes réussies au premier essai.

## Tâches

- [x] **U.1 Générateur** — crate `icebreaker-caverne` : zones plates et chaotiques, niches, galeries, puits, sentier
  vérifié à pied, grille de bits, profils de `formes.ron`. Testé : 5 tests (même graine = même grotte, 4 profils sur
  6 graines praticables et sous terre, profils différents) ; 160 grottes : 94 % au premier essai, aucune coupée ;
  0,1 à 0,4 s par grotte. — ad7337f8f8
- [x] **U.2 Atelier web** — page à curseurs et vue 3D, commande `caverne` du lanceur. Testé par captures Edge sans
  fenêtre (plaine, gouffre, chaos avec coupe). Pas essayé à la main : glisser les curseurs, copier dans le
  presse-papiers. — 705287f9ee
- [x] **U.3 Types et profils** — fait avec U.1 : `verticalite`, zones plates et chaotiques, puits pour les liens
  raides, quatre profils tirés par la graine. Les valeurs des profils sont un premier jet : **à régler par Flo dans
  l'atelier**. — ad7337f8f8
- [x] **U.4 Dans le jeu** — la Grotte prend sa forme du générateur : `Pocket` / `Bore` et les réglages `entree`,
  `poche`, `reseau`, `relief` de `grotte.ron` sont retirés ; chef dans la zone la plus loin le long du sentier,
  créatures sur les sols atteints à pied, coffres au fond des niches, filons au pied des parois hors du sentier,
  cristaux luisants hors du sentier ; niveaux (filons, butin) par distance à l'entrée. La roche qui touche l'air
  prend les couleurs de l'ambiance. La grotte suit le terrain qui descend (jusqu'à 150 blocs de plus) et ne passe
  pas sous le fond du monde. Testé : 10 tests de `veloren-world`, exemple `icebreaker_grotte` sur 12 graines (0
  problème : toutes les zones atteintes à pied dans les blocs générés, habitants apparus une fois et debout, coffres
  posés), serveur lancé. Pas vu en jeu. — 0f42ff8c39
- [x] **U.5 Docs** — ce plan, `PLAN-GROTTE.md`, `PLAN.md`, `README.md`, `CLAUDE.md`, fiche wiki de la grotte, Q54.
- [x] **U.6 Retours de Flo après essai en jeu (2026-11-15)** — plafonds bien plus hauts (zone plate 30 blocs par
  défaut, 34 au profil plaine ; zones chaotiques 60 à 72 ; galeries 18) ; sprites de sol et de plafond et filons
  divisés par 8 environ (`grotte.ron`) ; descentes : les zones se placent assez à l'écart pour une **pente en
  lacets** (le sentier va d'un bord à l'autre d'une salle en pente, virages à plat, les lacets s'ouvrent les uns
  sur les autres comme des corniches), les puits en hélice ne servent plus que pour une part réglable des descentes
  (`puits_part`, 0,2) ou quand la place manque ; créatures à l'écart des parois, cristaux à l'écart des créatures.
  Testé : tests, 160 grottes (aucune coupée, 12 graines dérivées au plus), exemple sur 12 mondes (0 problème),
  capture de l'atelier. Pas revu en jeu. — 5078857ef8
- [x] **U.7 Salle sûre** (demande de Flo, 2026-11-15) — la grotte n'a plus de bouche en surface : elle est entièrement
  sous terre, sa première zone sous l'origine du site. Une **salle sûre** (petite poche calme, `abri_rayon`) est
  accrochée à une zone tirée par la graine ; on y arrive. Le chef est dans la zone la plus loin de la salle sûre le
  long du sentier, et les niveaux (filons, butin) se comptent depuis elle. Marqueur vert dans l'atelier. Testé :
  tests du générateur (une salle sûre dans chaque grotte, fond à plus de 60 blocs), 160 grottes sans sentier coupé.
  — 0ba4c2865f
- [x] **U.8 Arrivée sous terre** — le serveur fait apparaître l'équipe dans la salle sûre (`dungeon_arrival_under`,
  `arrival_pos` : lieu seul, emplacement de campagne, grotte refaite pour une expédition) ; aucune créature à moins
  de `population.calme` blocs (30) de l'arrivée, ni filon ni coffre dans la salle. On ressort par `/rentrer`. Testé :
  10 tests de `veloren-world`, exemple sur 12 mondes (0 problème, le contrôle part de la salle sûre), serveur lancé
  (apparition à z = 60 sous un sol vers 200). Pas vérifié : un vrai joueur qui arrive, en lieu seul et en campagne ;
  le retour par `/rentrer` depuis le fond. — 0ba4c2865f
- [x] **U.9 Forme d'ensemble** (demande de Flo, 2026-11-15) — `etendue_x`, `etendue_y`, `etendue_z` (0,2 à 1) : les
  zones tiennent dans une ellipse autour de l'origine (plus dans un carré), la profondeur se resserre d'autant ;
  à 1 partout la grotte est ronde, 1 / 0,3 / 1 donne un boyau le long de X. Testé : profils resserrés sur chaque
  axe, 160 grottes sans sentier coupé. — 2a79cf2751
- [x] **U.10 Presets enregistrés** — `.\icebreaker.ps1 caverne` sert l'atelier sur http://localhost:8096 (bin
  `atelier`) : « Enregistrer le preset » (nom, poids) écrit le profil dans `formes.ron` et committe ce fichier,
  « Supprimer » le retire ; le preset et la graine sont en clair sous la grotte. Essayé au curl et par une capture
  de la page servie ; pas essayé : les clics dans la page, la commande `caverne` elle-même. — d63ca3924f
- [x] **U.11 Preset choisi au départ** — au panneau Départ, une expédition vers la Grotte propose « au hasard » et
  chaque preset (`/expedition forme <preset>`, ou `/expedition 3 gouffre`) ; au départ l'emplacement est refait
  avec cette forme et une graine neuve (32 bits), et l'équipe lit « Grotte « gouffre », graine … » en arrivant.
  Essayé au client texte en campagne ; pas vu dans le vrai client : les lignes de forme du panneau. — e32595d713

## Reste

- **Destruction (2026-11-22)**, U.13 à U.16 de `PLAN-PLAYTEST.md` (chantier 2) : la pioche casse tout contre de
  l'énergie (`destruction.ron`), murs à casser entre les zones (`murs_part`, `murs_epaisseur`), filons le long des
  parois, salle au minerai derrière son mur (`minerai_rayon`). Fait et testé ; à voir en jeu, et à régler.
- **Régler les profils** dans l'atelier (Flo), puis les voir en jeu : allure avec le vrai rendu, lumière, caméra dans
  les zones basses, plaisir de parcours, densité des sprites et des créatures dans de très grandes zones.
- Des îlots de roche flottants peuvent rester dans les zones chaotiques à fort bruit (le générateur retire les
  bulles d'air fermées, pas les blocs de roche isolés).
- Le schéma « coupe-grotte » du wiki dessinait les poches du jalon K : retiré de la fiche, à refaire d'après une
  vraie grotte (piste wiki). Les deux captures de la fiche datent du jalon K.
- La génération prend 0,1 à 0,5 s par grotte dans le serveur (une fois par emplacement, et à chaque expédition
  vers « Grotte », I.12) ; la grille garde quelques dizaines de Mo en mémoire par grotte.
