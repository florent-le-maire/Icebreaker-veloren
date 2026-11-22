# Avant le playtest avec les potes — feuille de route

*Liste de Flo du 2026-11-16, mise en plan le même jour ; ses réponses aux questions (Q55 à Q62) sont reportées ici.
On code chantier par chantier, avec son accord.* Ce que le jeu doit
avoir avant d'inviter tous les potes à tester. Ce fichier reprend chaque point de la liste, dit ce qui existe déjà,
propose des tâches ; restent ouvertes dans `QUESTIONS.md` : Q58 (le tableau des âmes) et Q61 (le craft). Les règles des âmes sont dans
`design/ames.md`. Sur le wiki : page « Feuille de route » (W.18).

Les tâches gardent la lettre de leur jalon (F donjon généré, U grottes, B boucles, E donjon à jouer, L combat, X
interface du village) et se cochent ici ; les âmes ouvrent le jalon **AM**.

## Vue d'ensemble

| # | Chantier | Tâches | Worktree | État |
|---|---|---|---|---|
| 1 | Donjon : des pièces travaillées | F.10 à F.18 | `Veloren-donjon` | catalogue et atelier des salles faits (F.10, F.14 à F.17) ; salles à meubler dans l'atelier (F.18), puis F.11 à F.13 |
| 2 | Grotte : murs à casser, filons, salle au minerai | U.13 à U.16 | `Veloren-caverne` | tranché |
| 3 | Missions : sablier, zones, types de mission, boss | B.16 à B.20 | `Veloren-boucles` | tranché ; B.19 et B.20 peuvent suivre le premier playtest |
| 4 | Âmes | AM.1 à AM.9 | `Veloren-ames` | attend le tableau de Sayanel (Q58) ; AM.6 peut suivre le premier playtest |
| 5 | Améliorations : village sans dégâts, bouton Rentrer | L.6, X.18 | `Veloren-village` | prêt à coder |
| 6 | Refonte des classes | — | `Veloren-classes` | à détailler par Flo ; pas obligatoire avant le premier playtest |
| 7 | Craft et objets à trouver | E.17, E.18 | `Veloren-donjon` | attend (Q61 : « on verra au moment ») |
| 8 | Équilibrage | — | — | en dernier |
| 9 | Le jour J : technique | — | `Veloren-serveur` | liste de contrôle |

**Ordre proposé** (Flo, 2026-11-16 : le développement démarre quand on veut ; zones et boss, pouvoirs des ennemis
et refonte des classes ne sont pas obligatoires avant le premier playtest) : le chantier 5 d'abord (quelques heures, évite de perdre des personnages au
village) ; puis la grotte (2) et les missions B.16 et B.17 (3), qui donnent un but à chaque descente ; puis les âmes
(4) ; les pièces du donjon (1) en parallèle, parce qu'elles demandent des dessins et des modèles ; le craft (7) ;
l'équilibrage (8) quand le reste ne bouge plus ; la liste technique (9) la veille. Les chantiers 1, 2, 3 et 4
touchent des dossiers différents : un agent et un worktree par chantier.

## 1. Donjon : des pièces travaillées (jalon F, suite)

**Ce que Flo demande** : des donjons faits de pièces travaillées, chacune avec sa fonction et sa décoration
(cuisine, couchage, armurerie…) ; voir si on peut les améliorer et en générer des variations ; ajouter les modèles
voxel qu'on s'attend à trouver dans un donjon.

**Ce qui existe** : trente salles dessinées en texte avec leur décor (F.9 : symboles de décor, `@theme:` militaire,
culte, bestial ou artisan, `@poids:`, un thème dominant par donjon), des formes variées (F.5), un décor par rôle
(F.6), le tableur `docs/icebreaker/salles.xlsx` pour les retours (`cargo run -p icebreaker-salles`). Rien de tout
cela n'a été regardé dans le vrai client. Le décor vient des sprites de Veloren ; aucun modèle à nous.

**Constat du 2026-11-16 (cuisine vue dans le vrai client)** : une case fait 7 blocs et ne porte qu'un objet, la
salle paraît vide. **Tranché (Flo, même jour)** : les meubles se posent au bloc près dans un atelier web (comme
celui des grottes) ; les murs, portes et escaliers restent dessinés par cases ; un bloc « ? » étiqueté (« trône »)
marque un modèle qui manque. D'où F.14 à F.17, avant F.11 à F.13.

- [x] **F.10 Catalogue des pièces** — `assets/icebreaker/donjons/pieces.ron` : 18 pièces, leur mobilier chez Veloren,
  31 modèles à créer en cinq lots ; feuilles « Pièces » et « Modèles » de `salles.xlsx` ; `@fonction:` sur 22
  salles. La liste reste à relire par Flo et Sayanel (colonne « Retours »). `42a2c5e0a9`
- [x] **F.14 Meubles au bloc près** — `@meuble: <sprite> x y z ori [miroir]` et `@manque: "libellé" x y z l p h`
  (format : `design/donjon.md`) ; posés par le générateur, tournés avec la salle, plus de décor par rôle dans une
  salle meublée. Cuisine meublée à la main, vue en salle seule dans le vrai client. Pas vu : une salle meublée
  tournée ou en miroir dans un donjon entier. `36b4b95b1c`
- [x] **F.15 Atelier des salles : serveur et objets** — `.\icebreaker.ps1 salles` (port 8097) : objets de Veloren
  et leurs modèles, salles (anciens symboles convertis en meubles), enregistrement après le contrôle du jeu, avec
  un commit ; une salle refusée n'est pas écrite. `cbc461ae0a`
- [x] **F.16 Atelier : vue 3D et pose** — salle en 3D, recherche (anglais et français), poser, tourner, miroir,
  déplacer, dupliquer, supprimer, annuler ; bloc « ? ». Essayé dans Edge sans fenêtre, pas encore à la main ; le
  sens de chaque objet n'a pas été comparé un à un avec le jeu. `f6595cfb2c`
- [x] **F.17 Atelier : plan, fiche, enregistrement** — cases à peindre, rangées à ajouter, fiche, nouvelle salle,
  copie, suppression ; les blocs « ? » remontent dans la feuille « Modèles ». Depuis, sur les
  retours de Flo : gomme, rangées à retirer, étages à ajouter ou retirer, escaliers, échelles et vide à peindre
  (une salle à étages faite dans l'atelier n'a pas encore été vue en jeu). `f6595cfb2c`
- [ ] **F.18 Meubler les salles** (Flo, Sayanel, dans l'atelier) — chaque pièce du catalogue a au moins une salle
  meublée ; les blocs « ? » posés donnent la liste de F.13.
  *Fin* : un donjon généré ne montre plus de salle vide.
- [ ] **F.11 Une fonction par pièce** — chaque salle porte `@fonction:` ; deux ou trois plans par fonction, décorés
  avec ce qui existe ; le tirage varie les fonctions (pas deux fois la même à la suite, celles d'un thème ensemble).
  *Fin* : l'exemple de contrôle montre, pour dix graines, au moins cinq fonctions différentes par donjon.
- [ ] **F.12 Variations d'une pièce** — à partir d'un plan dessiné : groupes de mobilier tirés par la graine (présents
  ou non), meubles interchangeables, état de la pièce (rangée, saccagée, abandonnée), en plus du miroir et de la
  rotation déjà faits.
  *Fin* : la même cuisine tirée cinq fois donne cinq salles différentes dans l'exemple de contrôle.
- [ ] **F.13 Modèles voxel du donjon** — les modèles manquants de F.10, par lots de cinq environ (modèle `.vox`,
  déclaration du sprite, symbole de décor), dessinés par l'agent (Flo, 2026-11-16) ; Flo juge sur capture.
  *Fin* : chaque modèle du lot se pose par son symbole et se voit dans le vrai client.

Portail et autel (Flo, 2026-11-20, `design/donjon.md`, « Portail et autel ») : deux salles qui portent un objet du
jeu ; la suite (allumer, traverser, cristal, résurrection) est sous le jalon T de `PLAN.md` (T.10 à T.12).

- [x] **F.19 Rôles et salles du portail et de l'autel** — rôles `portal` et `altar`, salles `salle_portail` et
  `salle_autel` ; l'objet se place par `@manque: "portail"` ou `"autel de résurrection"`. Tests verts. f70843906e
- [x] **F.20 Au moins une de chaque par donjon** — cul-de-sac accroché à une salle peu chargée, jamais à l'objectif ;
  une salle en plus dans ~4 % des petits donjons et ~12 % des grands (`special_rooms` de `generation.ron`). Tests
  verts ; pas parcouru dans un donjon entier. 012a872883
- [x] **F.21 Modèles voxel : portail (éteint, allumé), autel, cristal** — écrits par
  `common/examples/icebreaker_portail_vox.rs`. Portail éteint et autel vus en capture ; portail allumé et cristal
  vus sur l'image de contrôle seulement. 8a678167de
- [x] **F.22 Le donjon fait apparaître le portail et l'autel** — entité immobile à la place dite par la salle, avec
  une lueur (`portail.ron`). Vu en capture dans les deux salles seules. 727467233f, 6350a54f3a, bc044191f9

Donjon souterrain (Flo, 2026-11-22) : comme la grotte, on arrive dans une salle sûre, sous terre, sans tunnel ni
bouche en surface ; le donjon est tout entier sous le sol ; un atelier web règle sa forme comme celui des grottes.

- [x] **F.31 Salle sûre, tout sous le sol** — salle `abri.txt` (rôle `safeRoom`) en cul-de-sac sur l'entrée, plus de
  tunnel ; roche au-dessus de tout (`placement.profondeur`). Vu en capture : arrivée dans l'abri, sous terre (donjon
  seul) ; pas essayé en expédition de campagne. 8995919e92
- [x] **F.32 Calme autour de l'arrivée** — personne à moins de `abri.calme` (4) cases ; les rondes n'y entrent pas ;
  éclairé par son feu de camp et ses appliques. Tests verts, abri vide en capture. 10880b5d62
- [x] **F.33 Presets de donjon** — `donjons/formes.ron` (standard, plat, profond, labyrinthe) ; panneau Départ,
  `/expedition 2 <preset>`, `-Profil`. `-Profil plat` vu au journal du serveur ; pas essayé en campagne. a4fc4ad98a
- [x] **F.34 Atelier du donjon** — `.\icebreaker.ps1 donjon-atelier` (port 8098) : plan par étage, curseurs,
  « Enregistrer le preset » (commit). Vu dans Edge sans affichage ; pas encore essayé par l'équipe. 01f05e67c4
- [x] **F.35 Vue 3D de l'atelier du donjon** — three.js comme la grotte : une case = un cube, étages écartés, rampes,
  billes (créatures, coffres), arrivée ; « Jusqu'à l'étage N » coupe le dessus ; le plan reste en onglet. Vu dans Edge sans
  affichage. 0805b919df

Cuisine vivante (Flo, 2026-11-21, ambiance de la cuisine de Dark Souls 3 sans la copier) : les salles paraissent vides,
Veloren n'a presque rien pour une cuisine. Nos modèles sont générés par du code ; la vie vient des traces d'activité,
de la densité et de la hauteur ; la cuisine passe à 6×5 cases. Premier lot de six modèles, puis essai en jeu (F.13).

- [x] **F.23 Générateur de modèles** — crate `icebreaker-modeles` (formes, matières bruitées, `.vox`, image de
  contrôle) ; `cargo run -p icebreaker-modeles [dossier d'images]`. b19856d9c0
- [x] **F.24 Nos sprites dans le jeu** — six sprites `Icebreaker…` (`0xD4` à `0xD9`), manifeste, lueur et étincelles
  de l'âtre, `pieces.ron`. Pas vu : la collision au contact. 736ee1f58b
- [x] **F.25 Lot cuisine** — âtre de briques, billot, viande pendue, ustensiles pendus, vaisselle, préparation en
  cours ; recettes dans `icebreaker-modeles/src/cuisine.rs`. Vus dans le vrai client. b19856d9c0, 736ee1f58b
- [x] **F.26 Nouvelle cuisine 6×5** — 86 meubles, trois portes ; vue dans l'atelier et dans le vrai client
  (salle seule, lanterne allumée) ; pas vue dans un donjon généré. 150961aae8

Sol, murs et plafond (Flo, 2026-11-22) : tout le donjon a la même pierre. Le générateur sait dessiner des matériaux
(dallage, planches, pierre taillée, poutres…) au bloc près ; une salle choisit les siens, sinon le thème du donjon
décide.

- [x] **F.27 Matériaux du donjon** — `materiaux.ron` : 6 sols, 5 murs, 3 plafonds, un habillage par thème ; dessin
  bloc par bloc (`Fill::Sampling`). d4bba130cb
- [x] **F.28 Chaque salle son habillage** — `@sol:`, `@murs:`, `@plafond:` ; mur partagé : chaque côté le sien ; couloirs
  et tunnel au thème du donjon. Pas vu : un donjon généré entier. d4bba130cb
- [x] **F.29 Atelier : choisir et voir** — trois listes, sol au motif et murs à la couleur du matériau ; un nom
  inconnu est refusé à l'enregistrement. Le plafond ne se voit pas dans la vue. 1d7aa6079b
- [x] **F.30 La cuisine habillée en jeu** — carrelage, enduit et poutres vus dans le vrai client (salle seule,
  lanterne) ; étape « sol, murs, plafond » dans le skill `piece-vivante`. Commit « F.30 ».

Salle du trône royale (Flo, 2026-11-22, ambiance d'une salle du trône de marbre, sans copier) : épurée mais stylée,
un tapis qui mène jusqu'au trône sur son estrade. Nef de 5×8 cases, sol de marbre à cadres sombres, piliers, vitrail
et chandeliers ; peu d'objets, chacun travaillé.

- [x] **F.36 Sol de marbre à cadres** — motif `Cadres` (cadre sombre par case, compté depuis le coin de la case), sol
  `marbre`, mur `pierre_claire`, atelier aussi ; vu en jeu (salle seule). 443238313b
- [x] **F.37 Lot trône** — trône, estrade, marche, tapis (droit, bout, sur la marche), pilier à vigne, vitrail, chandelier ;
  `icebreaker-modeles/src/trone.rs`. Vus dans le vrai client. 8e882c6737
- [x] **F.38 Sprites du trône** — neuf sprites `0xDA` à `0xE2`, collisions, lueurs (vitrail 12, chandelier 16),
  manifeste, `pieces.ron`. Pas vu : les collisions au contact. 89a355377d
- [x] **F.39 Nouvelle salle du trône 5×8** — 116 meubles, tapis de la porte au trône par la marche du milieu ;
  pas vue dans un donjon généré. 9ad28ba17d
- [x] **F.40 La salle du trône vue** — atelier et vrai client (salle seule, lanterne, sans le chef qui tuait le
  personnage d'essai). 2042a1c765

Grande salle du trône (Flo, 2026-11-22) : « fais tout », et une salle grande, surtout haute, pour montrer
l'importance du lieu. Nef de 7×10 cases sur deux étages (14 blocs sous la voûte), lot 2 de modèles, gardes.

- [x] **F.41 Lot trône 2** — bannière, statue, lustre, rosace, grand vitrail, pilier haut ; vus dans le vrai client. 280191db4b
- [x] **F.42 Sprites du lot 2** — `0xE3` à `0xE8`, lueurs, `pieces.ron`. Pas vu : les collisions au contact. 0377b8bd59
- [x] **F.43 Grande salle du trône** — 7×10 cases, 14 blocs sous la voûte, 185 meubles ; pas vue dans un donjon généré. 455152560f
- [x] **F.44 La grande salle vue** — atelier et vrai client (salle seule, sans créatures) ; pas `cap` du client sans les mains. f4ba4e0460

## 2. Grotte : casser pour avancer, miner pour de bon (jalon U, suite)

**Ce que Flo demande** : des grottes séparées par des murs, pour obliger les joueurs à se servir de la destruction ;
des filons de minerai ; beaucoup moins de pépites isolées ; une salle avec beaucoup de minerai et quelques minerais
rares.

**Ce qui existe** : le générateur `icebreaker-caverne` (`PLAN-CAVERNE.md`) relie toutes les zones par un sentier
praticable à pied ; les minerais sont des sprites minables posés un par un au pied des parois (`filons` de
`grottes/grotte.ron`, déjà divisés par 8 en U.6) ; chaque classe a une pioche (K.4). Veloren sait déjà casser la
roche tendre (`WeakRock`) à la pioche et à l'explosion ; la roche ordinaire ne se casse pas. **Tranché (Flo,
2026-11-16)** : la pioche casse aussi la roche dure, contre de l'énergie. Le todo « Ajouter la
destructibilité » (2026-11-11) commence ici ; les matières et la foreuse restent dans `PLAN-DONJON.md` (E.5, E.6).

- [x] **U.16 La pioche casse tout** — tranché par Flo (2026-11-22) : tout est destructible à la pioche, dans la
  Grotte et le donjon généré, plus ou moins facilement selon la matière (`assets/icebreaker/destruction.ron` : coups
  et énergie par coup ; roche 5 coups à 15, bois 3 à 10, roche tendre 1 à 5). À sec, le coup ne fait rien ; hors de
  ces lieux, règles de Veloren. Testé : 4 tests de `veloren-common`, serveur de grotte lancé. Pas vérifié en jeu :
  creuser au vrai client (visée, bruit, arrêt à sec), dans la grotte et le donjon. — 8c11626f51
- [x] **U.13 Murs à casser** — une part des passages entre deux zones (`murs_part`, 0,5 ; `murs_epaisseur`, 3 blocs,
  dans `formes.ron` et l'atelier) est rebouchée par de la roche ordinaire, au milieu de la galerie ; jamais la sortie
  de la salle sûre ni un puits ; un mur qu'on contournerait à pied tout près n'est pas posé. Le sentier est contrôlé
  « en cassant » ; créatures, coffres et cristaux aussi derrière les murs. Marqueur violet dans l'atelier. Testé :
  10 tests du générateur, 160 grottes (aucune coupée, 3 à 5 murs par grotte), contrôle en jeu sur 12 mondes
  (0 problème : zones murées non atteintes sans casser, toutes atteintes en cassant). Pas vérifié en jeu : trouver
  un mur (rien ne le signale) et le percer ; la page de l'atelier. — HASH_U13
- [ ] **U.14 Filons, et presque plus de pépites** — un filon est une traînée serrée de minerai le long d'une paroi
  (longueur et minerais par niveau dans `grotte.ron`), visible de loin ; les pépites isolées sont divisées par dix
  environ (réglage).
  *Fin* : l'exemple de contrôle compte dix fois moins de pépites isolées et au moins un filon par zone à partir du
  niveau 2.
- [ ] **U.15 La salle au minerai** — une par grotte, à l'écart du sentier, derrière un mur à casser : beaucoup de
  minerai courant et quelques minerais rares (or, cobalt, gemmes) ; marquée dans l'atelier.
  *Fin* : chaque grotte de l'exemple de contrôle en a une, atteinte en cassant.

Reste du jalon U, inchangé : régler les profils dans l'atelier et voir une grotte de chaque forme en jeu.

## 3. Missions : quêtes et primes (jalon B, bloc 4)

**Ce que Flo demande** :
- un **compte à rebours de 20 minutes**, affiché en sablier (forme pas arrêtée) ; au bout, **échec sec** : la mission
  s'arrête et tous ceux qui ne sont pas rentrés perdent leur personnage (Flo, 2026-11-16) ;
- des **zones** qui proposent des types de mission. Une zone est un lieu avec un thème : donjon humain, donjon elfe,
  grotte au réglage 1 avec des gobelins, grotte au réglage 2 avec des nains… Son niveau monte avec les expéditions
  qu'on y fait ;
- des **types de mission** : explorer 80 % des pièces ; tuer 80 % des ennemis ; collecter une chose posée exprès pour
  la mission (pour commencer : des **champignons violets**, à rapporter en nombre dans une caisse posée à l'arrivée) ;
  battre un **boss** ;
- des boss à ajouter : au niveau 2 de la zone on rencontre un boss, et on reste bloqué à ce niveau tant qu'il n'est
  pas battu ; un autre au niveau 4 ; le dernier de la zone au niveau 6.

**Ce qui existe** : une seule mission, tuer le chef (G.7, I.10), avec ses paramètres par emplacement (G.6) ; au
panneau Départ on choisit un lieu, pas une mission. `design/mission.md` décrit déjà « explorer 80 % » et « nettoyer »
pour l'ancien projet. Ce chantier remplace le tableau des contrats de `PLAN-BOUCLES.md` (B.9) : une mission tient le
rôle du contrat. Les patrouilles toutes les 12 minutes (B.4) restent à discuter (Q12).

- [ ] **B.16 Le sablier** — durée de l'expédition en données (20 min), comptée par le serveur depuis l'arrivée,
  affichée en sablier dans le HUD de donjon chez tous les membres ; annonces à 10, 5 et 1 minute ; au bout : échec de
  la mission, ceux qui sont rentrés gardent leur personnage, ceux qui sont encore en bas le perdent (comme une mort,
  T.6) et reviennent au village en fantôme.
  *Fin* : le sablier se vide en 20 minutes chez deux joueurs à la fois ; à zéro, le joueur resté en bas perd son
  personnage, celui qui est rentré le garde.
- [ ] **B.17 Explorer et nettoyer** — deux types de mission en plus de « tuer le chef » : explorer 80 % des pièces
  (des zones, dans la grotte), tuer 80 % des ennemis ; parts en données ; bandeau d'avancement (« Explorer : 5/10 »).
  *Fin* : chacun des trois types se gagne au client texte, dans un donjon généré et dans la grotte.
- [ ] **B.18 Collecter** — des champignons violets posés exprès dans le lieu (nombre posé et nombre demandé en
  données) ; une caisse à l'arrivée ; la mission est gagnée quand le nombre demandé est déposé dans la caisse.
  *Fin* : déposer le dernier champignon gagne la mission ; les avoir dans le sac ne suffit pas.
- [ ] **B.19 Zones et choix de la mission** — les zones en données (lieu, thème : race, réglage de grotte) ; le panneau
  Départ les liste, chacune avec son niveau et ses missions ; une expédition faite dans la zone monte son **niveau
  d'exploration** (combien d'expéditions par niveau : réglage), gardé dans la campagne.
  *Fin* : une expédition monte le niveau de sa zone, encore affiché après une relance du serveur.
- [ ] **B.20 Boss de zone** — aux niveaux 2, 4 et 6 (réglage) la zone a un boss ; tant qu'il n'est pas battu, le niveau
  de la zone ne monte plus ; le boss du niveau 6 est le dernier. Quelle créature pour chaque boss : à choisir en
  prenant la tâche (« Plus tard » de `PLAN.md` parlait déjà d'un boss distinct du chef).
  *Fin* : une zone au niveau 2 reste au niveau 2 après une expédition sans tuer le boss, et passe à 3 après.

## 4. Âmes (jalon AM)

**Ce que Flo demande** : la liste complète est reprise en règles dans `design/ames.md` (« Règles », mises à jour le
2026-11-16). En bref : chaque type d'ennemi (race et rôle) a ses âmes, trois par exemple pour un guerrier orc ; un
ennemi tué en lâche une avec une faible chance ; l'âme reste au sol une minute, compte à rebours visible ; on ne la
porte pas, on la **lie** à un personnage qui a un emplacement libre, après confirmation, jusqu'à sa mort ; blanche
tant que le village ne la connaît pas, elle prend une forme et une couleur à elle quand un personnage qui la porte
rentre vivant, et entre au glossaire du village ; les ennemis ont eux aussi un pouvoir tiré de leurs âmes.

**Ce qui existe** : rien de codé. `design/ames.md` donne les races, les rôles, les stats et les effets de Veloren
dont une âme peut se servir. Les ennemis par race et par rôle existent (jalon H).

- [ ] **AM.1 Les âmes en données** — `assets/icebreaker/ames.ron` : par race et par rôle, les âmes (nom, effet), la
  chance d'en lâcher une, la durée au sol, les formes et les couleurs possibles. D'après le tableau de Sayanel, que Flo donnera au moment de coder (Q58).
  *Fin* : le fichier se lit ; un test vérifie que chaque type d'ennemi du lot a ses âmes et qu'il y a assez de couples
  forme et couleur pour toutes.
- [ ] **AM.2 L'âme tombe, puis s'éteint** — à la mort d'un ennemi, tirage dans sa table ; l'âme apparaît au sol,
  blanche si le village ne la connaît pas, avec sa forme et sa couleur sinon ; compte à rebours visible de tous
  (1 minute) ; elle disparaît au bout.
  *Fin* : une âme lâchée disparaît une minute après, au client texte.
- [ ] **AM.9 Niveau du personnage** (à faire avant AM.3 ; Flo, 2026-11-16) — un personnage va du niveau 1 au niveau 6,
  figé à 6 ; il gagne de l'expérience en battant des monstres et en finissant des missions (barème en données) ; le
  niveau suit sa fiche au registre et se lit dans son build. La maîtrise d'arme (jalon A) reste à part.
  *Fin* : un personnage passe au niveau 2 après assez de monstres ou une mission finie, et ne dépasse pas 6 (test).
- [ ] **AM.3 Lier une âme** — interagir avec l'âme ouvre « Voulez-vous lier cette âme ? » ; oui : elle passe dans un
  emplacement libre du personnage, son effet s'applique, elle reste jusqu'à sa mort et suit sa fiche au registre ;
  sans emplacement libre : refus. Elle ne va jamais dans le sac. Autant d'emplacements que le niveau du personnage (AM.9).
  *Fin* : un personnage lié garde son âme et son effet après un retour au village et une relance ; il la perd en mourant.
- [ ] **AM.4 Les emplacements dans le build** — une zone « Âmes » dans la fiche du personnage : emplacements, âmes
  liées, leur effet ; maquette d'abord, dans le canvas de l'interface.
  *Fin* : Flo valide la maquette, puis la zone se voit dans le vrai client.
- [ ] **AM.5 Découverte au retour et glossaire** — un personnage qui rentre vivant avec une âme inconnue la fait
  connaître : un couple forme et couleur encore libre lui est donné pour la partie ; la première expédition rentrée
  le fixe, les suivantes n'y changent rien ; le glossaire du village liste les âmes connues (forme, couleur, effet).
  Mort avant le retour : rien n'est découvert.
  *Fin* : deux retours avec la même âme donnent une seule entrée au glossaire, et l'âme tombe ensuite en couleur.
- [ ] **AM.6 Des ennemis à pouvoirs** — chaque ennemi reçoit à sa création un des effets des âmes de son type, tiré au
  hasard, sans lien avec l'âme qu'il lâchera.
  *Fin* : sur dix guerriers orcs, les trois effets de leurs âmes se rencontrent (test), et l'un d'eux se voit en combat.
- [ ] **AM.7 Résistances chiffrées** — seulement si le lot d'âmes en demande (« 30 % de résistance au feu ») : la
  résistance par effet décrite à la fin de `design/ames.md`.
  *Fin* : une âme de résistance au feu réduit d'autant une brûlure (test).
- [ ] **AM.8 Wiki** — fiche « Âmes » : la règle, le glossaire, les réglages.

Plus tard : la compétence qui permet de stocker des âmes (à concevoir), les âmes à effet actif si le premier lot n'en a pas.

## 5. Améliorations

### Village

**Ce que Flo demande** : supprimer les dégâts au village, sauf à l'aire d'entraînement. Tranche Q48 (un Séisme sur la
place blessait les recrues, qui tuaient le personnage suivant).

- [ ] **L.6 Pas de dégâts au village** — dans l'emprise du village, aucune attaque, compétence, projectile ou
  explosion ne blesse un joueur, une recrue ou un habitant ; exception : un rayon réglable autour des mannequins
  (`entrainement.ron`), où les mannequins encaissent comme aujourd'hui. Les chutes ne changent pas.
  *Fin* : un Séisme sur la place ne retire de PV à personne ; les mannequins donnent toujours leur bilan.
- [ ] **X.18 Bouton « Rentrer »** (todo de Flo, 2026-11-11) — un bouton dans le HUD de donjon qui fait `/rentrer`,
  après « Rentrer au village ? Oui / Non ».
  *Fin* : le bouton ramène au village après « Oui », rien après « Non ».

L'amélioration du village lui-même (bâtiments à niveaux, B.8) n'est pas dans la liste : elle attend.

### Personnage

Rien de listé pour l'instant (à détailler par Flo). Ce qui améliore déjà un personnage : la salle d'armes (jalon A)
et, bientôt, les âmes (chantier 4).

## 6. Refonte des classes

À détailler plus tard par Flo. État du jalon A : `PLAN-CLASSES.md` (trois classes codées, ultimes refaits le
2026-11-15, maquettes en attente d'avis, Q43 à Q47). Aucune tâche tant que la refonte n'est pas décrite.

## 7. Craft et objets à trouver (jalon E, suite du bloc 2)

**Ce que Flo demande** : un système de fabrication et des objets à trouver dans les grottes et les donjons ; le todo
du 2026-11-11 parlait de feuilles et d'ingrédients pour fabriquer des potions en bas.

**En attente** (Flo, 2026-11-16 : « on verra au moment », Q61) : les deux tâches ci-dessous sont une première idée.

**Ce qui existe** : le butin par profondeur (B.5, B.14, `butin.ron`) donne or, potions, matériaux, armes ; Veloren a
des plantes à cueillir et des recettes (potions comprises) qu'on fabrique près d'une station. `PLAN-DONJON.md`
propose des ressources qui ne servent qu'en bas (E.4) et `PLAN-BOUCLES.md` une forge au village (B.11) : à relier.

- [ ] **E.17 Ingrédients à cueillir** — plantes, feuilles et champignons de Veloren posés dans le donjon (cuisine,
  réserve) et dans la grotte (selon l'ambiance), plus rares à l'entrée qu'au fond ; réglages dans `butin.ron`.
  *Fin* : l'exemple de contrôle pose des ingrédients dans chaque lieu ; on les cueille au client texte.
- [ ] **E.18 Fabriquer une potion** — une recette qui prend ces ingrédients ; où on fabrique : Q61.
  *Fin* : avec les ingrédients d'une descente, on obtient une potion à l'endroit prévu.

Le reste du craft (armes, armures, autres recettes) est à décrire : Q61.

## 8. Équilibrage

En dernier, quand les chantiers au-dessus ne changent plus les chiffres. Tout se règle en données
(`assets/icebreaker/`), sans code. À passer en revue, au lieu d'entraînement (`.\icebreaker.ps1 entrainement`) puis en
campagne à deux :

- la difficulté des ennemis : « extrêmement dur » le 2026-11-13, corrigé en H.6, pas rejoué ;
- les compétences et les ultimes des trois classes, sans le plafond de 40 % par coup (Q43, liste « À vérifier en
  jeu » de `PLAN.md`, jalon A) ;
- le butin et l'or d'une descente (`butin.ron`), le minerai après U.14 et U.15 ;
- la durée du sablier, la chance de lâcher une âme, la force de leurs effets.

## 9. Le jour J : liste technique

Ce qui fait échouer une soirée même si le jeu est prêt. Détail des commandes : `MAINTENANCE.md`.

- Redéployer le serveur du VPS et refaire le zip du client **depuis le même commit** (le protocole a changé
  plusieurs fois depuis le dernier déploiement).
- Lancer le zip chez un pote avant la soirée : autre réseau, PC sans outils de développement (S.3) ; deux joueurs
  avec deux pseudos sur le VPS (S.4).
- Jouer une boucle entière à deux dans le vrai client : charrette, départ groupé, mission, retour, bilan, deuxième
  départ (I.8, I.9, I.12, V.9, V.11).
- À terre, relève, sac du mort et « regarder un allié » à trois joueurs (T.2 à T.4, T.8, T.9).
- Le village de la partie du VPS a des maisons (Q49).
- Le serveur tient avec tous les joueurs prévus (jamais essayé au-delà de deux).
- Une copie de la partie du VPS juste avant, et une fiche des touches pour les potes.

## Questions pour l'équipe

`QUESTIONS.md`, § 8 « Avant le playtest » : Q58 (tableau des âmes) et Q61 (craft) restent ouvertes ; Q55, Q56, Q57,
Q59, Q60 et Q62 sont tranchées (2026-11-16). Les patrouilles en plus du sablier : Q12.
