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

## Reste

- **Régler les profils** dans l'atelier (Flo), puis les voir en jeu : allure avec le vrai rendu, lumière, caméra dans
  les zones basses, plaisir de parcours, densité des sprites et des créatures dans de très grandes zones.
- Des îlots de roche flottants peuvent rester dans les zones chaotiques à fort bruit (le générateur retire les
  bulles d'air fermées, pas les blocs de roche isolés).
- Le schéma « coupe-grotte » du wiki dessinait les poches du jalon K : retiré de la fiche, à refaire d'après une
  vraie grotte (piste wiki). Les deux captures de la fiche datent du jalon K.
- La génération prend 0,1 à 0,5 s par grotte dans le serveur (une fois par emplacement, et à chaque expédition
  vers « Grotte », I.12) ; la grille garde quelques dizaines de Mo en mémoire par grotte.
