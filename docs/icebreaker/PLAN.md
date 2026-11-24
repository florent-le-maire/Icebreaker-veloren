# Icebreaker sur Veloren — plan

Prototype du concept Icebreaker sur une copie modifiée de Veloren (décision : `decisions/0015-prototype-veloren.md`).
Point d'entrée des docs : `README.md` (fichiers, jalons, conventions). Vision et règles : `VISION.md`,
`GAME_DESIGN.md`, `design/` (copiés du dépôt Icebreaker le 2026-10-28 ; les règles écrites pour Godot se lisent comme
des intentions, pas comme du code à reprendre). Tâches faites : `HISTORIQUE.md`. Questions pour l'équipe et petites
décisions : `QUESTIONS.md`.

Objectif du prototype : **une démo d'une soirée** où l'équipe enchaîne village → donjon → village → pillage sur la
route → village, pour trancher le choix du moteur avec Sayanel.

Méthode : une tâche = quelques heures au plus, avec un critère de fin vérifiable ; un commit par tâche (message en
français) ; on coche avec le hash, puis on déplace les tâches cochées dans `HISTORIQUE.md` quand le jalon est fini.
Le jeu doit rester lançable après chaque tâche.

## État au 2026-11-04

- **Ce qui se joue** (`icebreaker.ps1 campagne`, un seul serveur, `0016`) : le village (vue de dessus U, entrepôt,
  caisse, intendance, infirmerie, chantiers payés par la caisse, charrette et registre commun de personnages, départ
  groupé), deux emplacements de donjon Gnarling, deux de notre générateur (salles de formes variées, verrous et clés,
  dormeurs, décor), une grotte façon Deep Rock Galactic (formes tirées par la graine, filons, chef au fond), combat létal et verrouillage
  de cible, à terre et relève, sac du mort (jalon T). Détail et hashes : `HISTORIQUE.md`, `PLAN-GROTTE.md`.
- **Presque rien n'a été vu dans le vrai client** : la plupart des tâches ont été vérifiées par des tests et le client
  texte ; seuls les jalons X (interface du village) et N (interface refaite) ont été joués par Flo. Liste : « À
  vérifier en jeu » plus bas.
- **Interface refaite (jalon N, 2026-11-11)** : menu, fenêtre du village, bandeaux, HUD de donjon, bilan de retour
  (`PLAN-INTERFACE.md`, tâches dans `HISTORIQUE.md`). Jouée par Flo le 2026-11-11 : « tout est bon pour moi ».
  Le protocole a changé : le serveur du VPS est à redéployer avant d'y rejouer avec ce client.
- **Ce qui manque à la démo D.1** : le pillage sur la route (P.1 à P.4, pas commencé) ; serveur sur le VPS et zip du
  client faits (S.2, S.3). Le contenu de la démo est à confirmer ou à réduire : `QUESTIONS.md`, Q1.
- **Propositions en attente de l'équipe** : boucles de jeu (`PLAN-BOUCLES.md`, B.1 à B.13), mécaniques du donjon
  (`PLAN-DONJON.md`, E.1 à E.16), mod (`PLAN-MOD.md`), piliers, menace et âmes (`design/`), idées (`IDEES.md`).
- **Prochaine étape (Flo, 2026-11-16)** : la feuille de route d'avant playtest, `PLAN-PLAYTEST.md` : pièces
  travaillées du donjon, murs et filons de la grotte, missions et sablier, âmes, village sans dégâts, craft,
  équilibrage. Réponses de Flo reportées (2026-11-16) ; restent Q58 (tableau des âmes) et Q61 (craft).
- **Jalon U (2026-11-15)** : la grotte prend sa forme d'un nouveau générateur (`PLAN-CAVERNE.md`) : gouffre, plaine,
  chaos ou mixte selon la graine, zones plates à stalactites, zones pleines de bosses et de trous, puits, et un
  sentier toujours praticable à pied. Atelier web (`.\icebreaker.ps1 caverne`) pour régler les formes et
  les enregistrer en presets, que l'équipe choisit au panneau Départ (U.9 à U.11). Vérifié en
  test et par l'exemple de contrôle (12 mondes générés) ; pas vu en jeu, profils à régler par Flo.
- **Destruction dans la grotte (2026-11-22)** : la pioche casse tout, dans la grotte et le donjon généré, contre de
  l'énergie (plus ou moins de coups selon la matière, `destruction.ron`) ; des murs de roche bouchent une part des
  passages de la grotte ; le minerai est en filons le long des parois, et une salle au minerai attend derrière un
  mur (U.13 à U.16, `PLAN-PLAYTEST.md`). Depuis le 2026-11-24, chaque bloc de filon est un sprite à nous, au modèle
  de son minerai (U.23 à U.25, `design/filons.md`). Grotte vue en jeu (Flo, 2026-11-24 : « c'est top »). Client et
  serveur sont à recompiler ensemble (nouveaux sprites).
- **Jalon A (2026-11-11)** : armes, classes et compétences (`PLAN-CLASSES.md`, `decisions/0020`) codé pour les trois
  classes : jeux d'armes, 2 compétences d'arme + ultime + passif, recharges, niveaux 1 à 5, lot tiré à la charrette,
  maîtrise en expédition, salle d'armes (7e bâtiment). Vérifié en test, au client texte (de bout en bout en
  campagne) et par captures du vrai client (barre, passif, salle d'armes) ; personne n'y a encore joué à la main
  (liste « À vérifier en jeu »). Restent : l'avis de Flo sur les maquettes (A.2, Q47) et sur Q43 à Q49. Le
  protocole a changé : client et serveur à recompiler ensemble, VPS à redéployer.
- **Ultimes refaits (2026-11-15)** après l'essai de Flo (`PLAN-CLASSES.md`, A.19 à A.23) : plafond de 40 % par coup
  retiré, Envol à la place de Fureur, Fournaise en laser, Volée mortelle en pluie visée, Séisme, Bénédiction et Œil
  d'aigle renforcés. Vus partir dans le vrai client ; équilibre et relève à essayer à la main. Le protocole change
  encore.
- **Portail, cristal et autel (2026-11-20)** : chaque donjon généré a une salle du portail (allumé en 5 s, puis
  traversé pour rentrer, chacun pour soi) et une salle de l'autel (relève les morts dont on porte le cristal, pour la
  moitié de la vie du moment). La mort n'est définitive que si le mort quitte le donjon ou si plus aucun vivant n'y
  reste (`design/donjon.md`). Le protocole a changé : client et serveur à recompiler ensemble, VPS à redéployer.

## Architecture visée

- **Un seul serveur pour la campagne** (`0016`, remplace pour ce prototype le processus par instance de `0005`) :
  le village et des emplacements de donjon dans un même monde ; une expédition prend un emplacement libre. Les lieux
  restent lançables seuls pour tester un donjon.
- **Ce qui dure entre les instances** (personnages, inventaires, caisse et entrepôt du village, horloge, traces des
  donjons) vit dans un dossier de **campagne** commun à tous les processus, pas dans le monde d'une instance.
- Le client passe d'une instance à l'autre en se reconnectant (à la main d'abord, automatiquement ensuite).

## Jalons finis (détail dans `HISTORIQUE.md`)

Prototype du 2026-10-28 ; jalon 1 (instances propres, I.1 à I.9) ; jalon 2 (village, V.1 à V.12) ; L (combat
létal) ; R (charrette et registre) ; X (interface du village) ; S.1 (script Linux) ; VF (village fixe). Finis en partie : C (C.1 à C.3),
G (G.1 à G.10), F (F.1 à F.6). Grotte : K.1 à K.5 (`PLAN-GROTTE.md`). Mod : M.1 (`PLAN-MOD.md`).

## Jalon C — Le village qui grandit (Flo, 2026-10-28)

La ville complète est prévue dès le départ ; seules quelques maisons sont bâties, les autres sont des terrains à
bâtir. On y paie des chantiers avec l'or de la caisse ; la maison sort de terre pendant la partie. Réglages dans
`assets/icebreaker/village.ron`, état dans `campagne/village.ron`. Fait : C.1 petit village, C.2 terrains à bâtir,
C.3 commander un chantier (maison dessinée au prochain lancement en attendant C.4). `PLAN-BOUCLES.md` (B.8) propose
de remplacer les maisons par des bâtiments nommés à niveaux : à trancher avant C.5 et C.6.

- [ ] **C.4 Chantier visible** — la maison sort de terre au fil du temps de carte, sans relancer.
- [ ] **C.5 Nouveaux habitants** — une maison finie amène des habitants.
- [ ] **C.6 Paliers** — l'or investi (300, 1000, 3000) ouvre des terrains plus loin de la place.

## Jalon X — Interface du village, suite (Flo, 2026-11-06)

- [x] **X.11 Plan schématique du village** — la vue U ne déplace plus la caméra : panneau centré dessiné par le HUD,
  une case colorée par bâtiment à sa place réelle, mise à l'échelle ; point du joueur ; personnage immobile ; U ou Échap
  ferme ; `vue_village.ron` → `taille`, `marge`, `couleur_joueur`. Testé (6 tests, client entre en jeu) ; plan,
  clic, Échap et fantôme en vol pas encore vus en jeu. — 249040251e
- [x] **X.12 Viser la cible verrouillée** — tant qu'un ennemi est verrouillé, le réticule disparaît et les attaques
  (arc, bâton, mêlée) partent de la main vers le cercle ; angle de caméra inchangé ; touche dans Contrôles. Testé
  (test ciblé, client compilé), vu en jeu. — c9d929adf2
- [x] **X.13 Le tir ne dérive pas avec le tireur** — flèches, sorts et objets lancés ne reprennent plus la vitesse du
  tireur (`combat.ron` → `part_vitesse_tireur: 0.0`) : on touche en se déplaçant de côté, une cible qui change de
  direction esquive ; vaut aussi pour les archers ennemis. Testé (test ciblé, donjon démarre), vu en jeu. — f2b57a7e8d
- [x] **X.14 Zone d'entraînement** — 3 mannequins près de la place (`entrainement.ron`) ; ils ne meurent pas et
  donnent à chaque attaquant son bilan (dégâts, coups, dégâts/s) après une série. Testé (3 tests, client texte : posés
  au sol) ; coups, bilan et remise des PV pas vus dans le vrai client. — 2fadb68
- [x] **X.15 Mannequins sur sol plat** — rangée posée sur le sol plat le plus proche (`recherche`, `marge`) ; 2
  immobiles (le recul est annulé) et 2 avec le recul de Veloren. Testé (4 tests, client texte : tous au même niveau) ;
  recul annulé pas vu dans le vrai client. — b3e8769
- [x] **X.16 Clavier AZERTY ou QWERTY** — deux boutons dans Réglages > Contrôles : touches de base (AZERTY) ou
  QWERTY (WASD, 1 à 0, cible sur Z). Testé (3 tests, boutons vus au vrai client) ; clic pas fait en jeu. — ced08d6028
- [x] **X.17 Supprimer une partie** — bouton « Supprimer » sur chaque ligne du choix de la partie, confirmé par
  « Oui / Non » ; efface le dossier de la partie (et `derniere.txt` s'il la désignait), refusé si son serveur tourne.
  Testé (test ciblé, lignes et confirmation vues au vrai client) ; clic réel sur « Oui » pas fait. — 964b6d6a59

## Jalon L — Combat, suite (Flo, 2026-11-07)

- [x] **L.5 Dégâts selon le type d'attaque** — contre une créature : épée 3–4 coups, arme à distance 5–6, lance-flammes
  moins par touche ; un coup retire au plus 40 % des PV max, plafond relevé par les bonus de dégâts (`combat.ron`).
  Testé (5 tests ciblés, donjon démarre) ; nombre de coups pas vu en jeu. — d295d68c42

## Jalon I — suite : un donjon neuf à chaque expédition (Flo, 2026-11-07)

Option 2 de `0016` pour le donjon généré et la grotte : l'emplacement garde sa place, son plan est retiré à chaque
expédition. Les paramètres de mission restent ceux du lanceur (`ICEBREAKER_DONJON_*`) en attendant les contrats.

- [x] **I.10 Mission et dormeurs par emplacement** — chaque emplacement (et le lieu seul) a sa mission et ses
  dormeurs. Testé (2 tests ciblés, campagne démarre : 3 missions) ; annonce et dormeurs pas vus en jeu. — 14fd998fa4
- [x] **I.11 Plan remplaçable dans le monde** — plan du donjon généré et de la grotte remplaçable
  (`regenerate_slot`) dans une emprise réservée (`generation.ron` → `placement.emprise`, `grotte.ron` → `emprise`).
  Testé (2 tests ciblés, exemples sans problème, campagne démarre). — 5711768528
- [x] **I.12 Plan neuf à chaque expédition** — au lancement du serveur et à chaque remise à neuf (I.9), un nouveau
  plan ; arrivée, mission et dormeurs recalculés. Testé (client texte : entrée puis sortie, deux fois, deux plans
  neufs) ; nouveau plan dessiné pas vu dans le vrai client. — ee8758bfcf

## Jalon 3 — Pillage sur la route

- [ ] **P.1 Instance route** — un tronçon de route dans un paysage, zone fermée (I.2).
  *Fin* : on apparaît au bord de la route.
- [ ] **P.2 Convoi** — marchands, gardes et bêtes de somme qui avancent le long de la route.
  *Fin* : le convoi traverse la zone d'un bout à l'autre si on ne fait rien.
- [ ] **P.3 Défense et fuite** — les gardes protègent, les marchands fuient ; la rencontre finit quand le convoi est
  pillé, détruit ou sorti de la zone.
  *Fin* : les trois fins se produisent en test.
- [ ] **P.4 Butin du convoi** — marchandises sur les bêtes et les chariots ; au retour, elles alimentent la bourse et
  l'entrepôt.
  *Fin* : un convoi pillé rapporte or et matériaux au village.

## Jalon G — Donjon généré (branche `icebreaker-donjon`)

Le générateur du projet Icebreaker (graphe de mission, salles préfaites, assemblage, peuplement, paramètres :
`design/generation.md`) traduit en Rust (`world/src/icebreaker/dungeon/`) et dessiné avec les blocs et les créatures
de Veloren (`world/src/site/plot/icebreaker_dungeon.rs`). Données dans `assets/icebreaker/donjons/`. Fait : G.1 à
G.10 (lieu « Genere », salles, graphe, assemblage, peuplement, paramètres de mission, victoire, verrous et clés,
créatures endormies, éclairage selon les habitants).

**IA du donjon** (`design/creatures.md`, tâches H du projet ; `design/menace.md`, palier 1) — en s'appuyant sur ce
que Veloren fait déjà (vue, ouïe, combat, fuite). G.11 à G.14 d'abord ; G.15 et G.16 à redécouper après.
- [ ] **G.11 Attaque surprise** — un coup sur une créature endormie fait ×1,5 dégâts, « Surprise ! » au-dessus d'elle.
  *Fin* : frapper un dormeur fait plus mal qu'une créature éveillée.
- [ ] **G.12 Appel à l'aide** — l'archer sous 75 % de ses PV et le guetteur dès le premier coup crient « À l'aide ! »
  (24 blocs, une fois toutes les 6 s) ; brutes et chef qui l'entendent prennent l'agresseur pour cible même sans le
  voir ; les dormeurs à portée se réveillent.
  *Fin* : frapper l'archer fait venir la brute de la salle voisine.
- [ ] **G.13 Déroute à la mort du chef** — « Le chef est tombé ! » : les créatures à 24 blocs fuient (brute 4 s,
  archer 8 s, guetteur 12 s), « Fuite ! » au-dessus d'elles, puis se reprennent.
  *Fin* : tuer le chef disperse sa garde.
- [ ] **G.14 Le guetteur messager** — un guetteur qui repère un joueur crie « Des intrus ! » et court prévenir la
  créature combattante la plus proche, qui vient voir.
  *Fin* : on voit le guetteur s'enfuir et des renforts revenir avec lui.
- [ ] **G.15 Niveau d'alerte du donjon** — calme, inquiet, en alerte, mobilisé : monte avec les cris, combats et morts,
  redescend lentement ; chaque niveau s'annonce (texte d'ambiance, torches qui s'allument dans les salles sombres,
  dormeurs qui se lèvent).
  *Fin* : un combat bruyant met le donjon en alerte, une entrée discrète le laisse calme.
- [ ] **G.16 Embuscade préparée** — la salle prévenue par le guetteur se poste autour de sa porte et attend (~40 s)
  avant d'aller chercher les intrus ; derrière la porte, c'est « trop calme ».
  *Fin* : on se fait cueillir en passant la porte.

**Donjons avancés (idées, à découper le moment venu)** — aujourd'hui deux lieux, le souterrain et la grotte. Lieux
envisagés (le projet Icebreaker en décrit déjà quelques-uns : `design/generation.md`, « Derrière les murs et lieux ») :
- **La tour** : on monte au lieu de descendre, étages étroits, maçonnerie et ciel autour, chef au sommet.
- **Le marécage** : salles à ciel ouvert reliées par l'eau et la boue, nids nombreux (effets de la région Marais).
- **La forêt** : clairières reliées par des sentiers, sous les arbres ; nids et archers (région Forêt).
- **Le château** : salles collées par leurs murs, cour et remparts (assemblage « salles collées » du projet).
- **Le gouffre** (façon *Made in Abyss*) : un grand puits vertical, des salles accrochées à ses parois ; plus on
  descend, plus c'est dangereux et riche, et remonter coûte (fatigue, temps de carte, blessures ?). `QUESTIONS.md`, Q15.

## Jalon F — Forme et vie du donjon généré (Flo, 2026-11-03)

Constat de Flo après une descente : donjon vide, couloirs immenses, tout petit (la caméra sort du donjon), salles
rectangulaires sans décor. Fait : F.1 échelle 7 blocs, F.2 couloirs courts et escaliers propres, F.3 donjon plus
grand, F.4 plus d'ennemis, F.5 salles de formes variées, F.6 décor par rôle ; tout cela reste à voir dans le vrai
client. Réglages dans `generation.ron` ; worktree `Veloren-donjon`. La grotte façon Deep Rock Galactic est un lieu
à part : `PLAN-GROTTE.md` (jalon K).

- [ ] **F.7 Caméra en donjon** (si F.1 ne suffit pas) — mode « loin » de `camera.rs` : refuser un point de caméra
  séparé du joueur par de la roche.
  *Fin* : en couloir, dos au mur, la caméra ne sort jamais du donjon.
- [x] **F.8 Donjon jamais au bord du monde** — entrée à 16 tronçons au moins du bord, donjon refait (autre graine)
  s'il passe à moins d'un tronçon du bord ; réglages `placement` de `generation.ron`. Campagne lancée : donjons
  placés sans dépassement ; descente jusqu'au bout pas faite en jeu. de7388d606
- [x] **F.9 Trente salles décorées** — le décor se dessine dans le plan (symboles `=&!?|` etc., table `decor.symbols` de
  `generation.ron`), chaque salle a `@theme:` (militaire, culte, bestial, artisan) et `@poids:` ; un thème dominant par
  donjon pèse `theme_bias` fois plus au tirage. 20 salles décorées + 10 nouvelles ; tableur `docs/icebreaker/salles.xlsx`
  (`cargo run -p icebreaker-salles`) pour les retours. Tests ciblés verts ; pas vu en jeu. 2fd39d92cc

## Jalon T — À terre, mort et sac du mort (Flo, 2026-11-04)

À 0 PV on tombe à terre ; un allié relève (E) avec 25 % des PV ; passé 30 s à terre, on meurt. Mort en
expédition : un sac au sol garde l'inventaire hors équipement porté, un allié le fouille 3 s pour le prendre. Le mort
rentre au village en fantôme (puis joue un perso de sa réserve) ou regarde un allié sur place ; si le dernier vivant
tombe, tous rentrent. Le personnage mort est perdu : il quitte le registre et il est supprimé. Réglages :
`assets/icebreaker/mort.ron` ; worktree `Veloren-mort`.

- [x] **T.1 Relever à 25 % et mort après 30 s à terre** — client texte à deux : relevé à 25/100 PV, mort ~30 s
  après une seconde chute. 63f57ce830
- [x] **T.2 Visuel du temps qui reste à terre** — barre et « Mort dans N s » (soi, au-dessus des autres). Client
  compilé, pas vu à l'écran. bab7a8345d
- [x] **T.3 Le sac du mort** — sac au sol, fouillé en 3 s par un allié, contenu dans son inventaire. Client texte : B
  récupère l'or de A ; modèle du sac au sol pas vu. 634f96f275
- [x] **T.4 Après la mort : village ou regarder un allié** — R : fantôme au village (vu au client texte) ; clic :
  allié suivant, pas vu dans le vrai client. 61b3637e5c
- [x] **T.5 Le dernier tombe : retour au village** — client texte : B meurt seul en bas, message et retour en
  fantôme. 3e9abc6ef2
- [x] **T.6 Le mort est perdu pour de bon** — fiche retirée du registre, personnage supprimé de la base. Client
  texte : registre à 0/10 après la mort, plus dans la base ; texte de mort du vrai client pas vu. 186c82f80d
- [x] **T.7 Plantage du client au passage en fantôme** — caméra libre sans touche enfoncée : NaN dans la caméra
  avant le point de départ du serveur, le son plantait (build debug). Vu en jeu (Flo, 2026-11-08). 4d205f18b0
- [x] **T.8 Écran noir puis plantage en regardant un allié** — l'allié regarde à la verticale (le sac à ses pieds) :
  tangage NaN, gardé par la caméra, plantage des ombres. Tangage borné, NaN refusé ; tests, pas revu en jeu. 00c6f0101c
- [x] **T.9 Une barre de vie à terre** — à terre avec 90 % des PV max (`pv_a_terre`, ~3 coups) au lieu de 1 PV ;
  le décompte s'arrête pendant la relève. Client texte à deux ; coups réels et barre pas vus en jeu. c988dcc9f8

Portail, cristal et autel (Flo, 2026-11-20, `design/donjon.md`) : la mort n'est plus définitive tant que le mort
reste en bas avec un vivant. Salles et modèles : F.19 à F.22 de `PLAN-PLAYTEST.md` ; worktree `Veloren-donjon`.

- [x] **T.10 Allumer le portail et rentrer en le traversant** — 5 s d'interaction (E), puis chacun pour soi
  (`portail.ron`). Client texte : allumé, puis retour ; vrai client : traverser ramène sur la place avec le bilan.
  Pas vus : la jauge, le libellé « Allumer », l'annulation par un coup. 3df46f71fa
- [x] **T.11 Le cristal du mort** — à son nom, en première place de son sac ; retiré au retour au village. Client
  texte à deux : « cristal de Beta » porté par Alpha, disparu après `/rentrer`. Un cristal porté à la déconnexion
  revient sans nom ; nom pas vu dans l'inventaire du vrai client. 9d53db33a2
- [x] **T.12 L'autel** — consomme les cristaux portés, relève leurs morts près de l'autel à 25 %, coûte la moitié
  de la vie du moment (`mort.ron`, `autel`). Client texte à deux : refus à 1 PV, puis Beta relevé. Pas vus : PV
  exacts, plusieurs morts d'un coup, écran de mort et retour de « regarder un allié ». 55bf68888b

## Jalon A — Armes, classes et compétences (Flo, 2026-11-11)

Le jeu d'armes en mains donne 2 compétences d'arme, la classe un ultime et un passif ; jouer une arme en expédition
rapporte des points d'arme, dépensés à la salle d'armes ; recrues tirées avec un lot au hasard. Règles :
`design/classes.md` (section Veloren) ; tâches A.0 à A.18 : `PLAN-CLASSES.md`.

## Jalon H — Habitants : races ennemies et monstres solitaires (Flo, 2026-11-13)

Les ennemis sont des humanoïdes d'une des six races (humain, orc, nain, elfe, draugr, danari), une seule par donjon
ou grotte (tirée par la graine du lieu), dans les rôles des classes (guerrier, archer, mage, équipement de
`classes/`) ; des monstres solitaires, sans race (araignées dans la grotte) ; un chef de la race avec les capacités du
chef gnarling, aussi dans la salle du boss. Pas de comportement de race pour l'instant. Worktree `Veloren-races`.

- [x] **H.1 Les ennemis par rôle** : `assets/icebreaker/ennemis/` (guerrier, archer, mage, chef au bâton du chef
  gnarling), `world/src/icebreaker/races.rs`. Testé ; pas vu en jeu. `584586c8b3`
- [x] **H.2 et H.3 Une race par donjon généré, monstres solitaires** (un seul commit, code commun) : faction
  `races` par défaut, `especes` et `solitaires` dans `generation.ron`, `-Race` au lanceur. Testé ; pas vu en jeu. `a62bc950be`
- [x] **H.4 La grotte peuplée d'une race** : `habitants`, `chef`, `especes` et `part_solitaires` dans `grotte.ron`, les
  ambiances n'ont plus que leurs `solitaires` (araignée partout) ; wiki des créatures à jour. Testé ; pas vu en jeu. `745a871e9d`
- [x] **H.5 La campagne sans le fort gnarling** : 4 donjons générés + la grotte (`campagne.ron`), plus de lieu
  « Donjon » (fort gnarling, lançable seul) ; `design/ames.md`. Campagne lancée : 4 donjons de races différentes. `b5c25349d5`
  Vu en jeu (Flo, 2026-11-13) : ça marche, mais « extrêmement dur ».
- [x] **H.6 Ennemis des races moins durs** : `icebreaker_pv` dans les fiches (guerrier 50, archer et mage 40, chef
  150 ; un humanoïde a 100), plastron et jambières seulement, plus de survie à 1 PV pour un ennemi. Testé ; à rejouer.

## Jalon Y — Une identité par race ennemie (Flo, 2026-11-14)

Chaque race a un trait de comportement lisible et contrable (`design/ames.md`, identités à valider : `QUESTIONS.md`,
Q50). Réglages : `assets/icebreaker/ennemis/races.ron` ; code : `common/src/icebreaker_races.rs`. Worktree
`Veloren-races-ia` (supprimé, à recréer pour la suite).

- [x] **Y.1 à Y.3 Réglages par race, danari qui achèvent, humains en ronde** (un commit, code commun) : danari :
  joueur à terre à 20 blocs en tête des cibles, sauf celui qui vient de les frapper ; fuite à 30 %. Humains : rondes
  de 3 guerriers sur 3 salles, aller et retour. Testé (tests ciblés ; client texte : une ronde humaine fait 0→1→2→1→0
  dans le journal, donjon danari sans erreur) ; achèvement et rondes pas vus dans le vrai client. ed852019b5
- [ ] **Y.4 Orcs en rage** — sous la moitié des PV, une fois : dégâts et vitesse en plus.
- [ ] **Y.5 Nains tenaces** — ils tiennent leur salle (poursuite bornée), plus d'armure.
- [ ] **Y.6 Elfes tireurs** — plus d'archers à la composition, voient de plus loin.
- [ ] **Y.7 Draugr dormeurs** — presque tous endormis (réveil au bruit), lents.
- [ ] **Y.8 Wiki** — rubrique Créatures : le comportement de chaque race.
- [x] **Y.9 Choix de cible** (Flo, 2026-11-14) : parmi les joueurs repérés, un joueur debout passe avant un joueur à
  terre (tous à terre : achevés), les ennemis se répartissent (`commun` de `races.ron`) ; les danari n'achèvent qu'à
  plusieurs. Corrige Y.2 : un ennemi ne rechoisissait jamais sa cible en combat. Testé (tests ciblés ; client texte à
  deux, orcs et danari, journal des cibles) ; pas vu dans le vrai client. 5b39c1b941

## Jalon B — Boucles de jeu (proposition du 2026-11-02, à valider)

Ce qui rend l'enchaînement village → donjon → village intéressant (risque, butin, besoins du village) : règles et
tâches B.1 à B.13 dans `PLAN-BOUCLES.md`. Ce qu'on fait en bas (ressources du donjon, sac à formes, foreuse, trous,
garde et parade, monstres à coop) : tâches E.1 à E.16 dans `PLAN-DONJON.md`.

## Jalon 4 — Démo à l'équipe

Fait : S.1 (`icebreaker.sh`, même rôle que `icebreaker.ps1`).

- [x] **S.2 Serveur sur le VPS** — campagne sur `116.202.107.125:14100` (service `icebreaker-veloren@campagne`,
  `icebreaker-serveur/deployer.sh`), terrain pré-généré relu au démarrage (~1 s). Vérifié au client texte depuis le
  PC ; pas encore au vrai client ni depuis chez un pote. `66960fc9a4`
- [x] **S.3 Client pour les potes** — bouton « En ligne » (`adresse` dans `lieux.ron`), zip par
  `icebreaker-serveur/paquet-client.sh`. Zip décompressé à part : le vrai client entre dans le monde du VPS ; pas
  encore lancé chez un pote. `b3e3f64e8c`
- [x] **S.4 Pseudo au menu** — champ « Ton pseudo » au-dessus des lieux ; sans pseudo, l'écran de saisie au lieu
  d'un pseudo vide (vu sur le VPS : deux joueurs `username=""`, le même compte, l'un éjectait l'autre). Champ vu en
  capture du menu ; connexion à deux au VPS pas encore vérifiée.
- [x] **S.5 Release du client Windows + Mac** — `.github/workflows/release-client.yml` (bouton « Run workflow » ou
  étiquette `v*`) ; `paquet-client.sh` fait aussi le zip Mac (`Icebreaker.command`). Workflow pas encore lancé sur
  GitHub ; zip Windows refait en local avec un faux binaire seulement. `6e81e32d`
- [x] **S.6 Release à chaque nouvelle version** — numéro dans `assets/icebreaker/version.ron` (0.1.0) ; poussé sur
  master avec un numéro nouveau, il donne la Release `v<version>`. Lecture du numéro et test de l'étiquette essayés
  en local ; pas encore vu sur GitHub. `7f8714bd`
- [x] **S.7 Release même si un système échoue** — publiée avec les zips présents (1er run : pas de Mac libre chez
  GitHub, job annulé) ; « Re-run failed jobs » complète la même Release. Essayé en local avec un faux `gh` ; pas
  encore vu sur GitHub. `dd991211`
- [ ] **D.1 Soirée de test** : village → donjon → village → route → village, à deux ou plus, puis décision du moteur
  avec Sayanel (`0015`). Contenu à confirmer (`QUESTIONS.md`, Q1).

## Avant le playtest avec les potes (Flo, 2026-11-16)

Ce que le jeu doit avoir avant d'inviter tout le monde : `PLAN-PLAYTEST.md`. Ses tâches gardent la lettre de leur
jalon et se cochent là-bas : F.10 à F.13 (pièces du donjon), U.13 à U.16 (grotte), B.16 à B.20 (missions), AM.1 à
AM.9 (âmes), L.6 (plus de dégâts au village), X.18 (bouton « Rentrer »), E.17 et E.18 (craft).

## À vérifier en jeu

Ce que les tâches cochées n'ont vérifié qu'en test ou au client texte (détail dans `HISTORIQUE.md` et
`PLAN-GROTTE.md`). À parcourir dans le vrai client, en campagne, à deux si possible ; barrer ce qui est vu, ouvrir une
tâche pour ce qui casse.

- [ ] Thème du village (2026-11-24, `musique.ron`) : il démarre au village, à bas volume, tourne en boucle, ne repart
  pas du début au passage de fantôme à personnage et s'éteint en fondu au départ en expédition.

**Village fixe (VF.1 à VF.5)**
- [ ] Faire le tour du village : chaque PNJ à sa porte (la caisse dans la Guilde), maisons d'aplomb sur le sol
  plat, bordure en pente douce ; une recrue engagée à la charrette.
- [ ] Une fois, de nuit (graine 777), la caméra a montré le village vu d'en dessous, terrain absent ; de jour, rien.
- [ ] VF.3 : un lieu mis au palier 1 dans `village_plan.ron` reste nu, `/batir <id>` le fait sortir de terre
  au lancement suivant.
- [ ] VF.4 : de près, les rangs de tombes du cimetière et l'entrée de chaque clôture.
- [ ] VF.5 : le plan à l'échelle dans une ville tirée au hasard (doit rester la grille).

**Donjon souterrain (F.31 à F.35, `PLAN-PLAYTEST.md`)**
- [ ] En campagne, `/expedition 2` (donjon généré) : sortir de l'abri par la porte vers l'entrée ; personne ne vient
  dans l'abri ; aucun jour au plafond dans tout le donjon. (Arrivée dans l'abri, feu allumé, vue par Claude dans le
  vrai client le 2026-11-22.)
- [ ] Choisir une forme au panneau Départ (le bouton) : vu par Claude le 2026-11-22 avec `/expedition 2 plat`
  (« Donjon « plat », graine N » à l'arrivée, même plan dans l'atelier avec ce preset et cette graine).
- [ ] L'atelier du donjon (`.\icebreaker.ps1 donjon-atelier`) en main : vue 3D (tourner, couper les étages), curseurs, enregistrer un preset, le retrouver
  au panneau Départ.

**Labyrinthe et relief du donjon (F.51 à F.57, `PLAN-PLAYTEST.md`)**
- [ ] `donjon -Site Genere -Profil labyrinthe` : se perdre dans le labyrinthe et en ressortir ; est-il trop grand, trop
  vide ? Trouver une salle hors mission. La porte à clé ne se contourne pas.
- [ ] `donjon -Site Genere -Profil tour` : descendre une tour par ses volées (les marches se montent sans sauter),
  regarder dans le vide, y tomber (dégâts de chute : Q64) ; un escalier en Z, monté et descendu.
- [ ] L'atelier du donjon en main : les groupes « Remplissage » et « Relief », une tour en vue 3D.
- [ ] Les six formes (F.59 : croix, forteresse, rond, labyrinthe, tour, plat) se reconnaissent-elles à pied, sans
  regarder le plan ?

**Formes du donjon (F.45 à F.50, `PLAN-PLAYTEST.md`)**
- [ ] `donjon -Site Genere -Profil forteresse` : deux salles collées, portes dos à dos (on passe, les portes
  s'ouvrent, rien ne coince) ; `-Profil labyrinthe` : un couloir qui serpente, à pied.
- [ ] En campagne, plusieurs `/expedition 2` de suite : la forme change (« Donjon « croix », graine N » à l'arrivée).
- [ ] L'atelier du donjon en main : le menu « Forme du plan », le bord doré de la silhouette, les groupes de curseurs
  (Couloirs, Salles), régler l'étendue des presets à l'œil.

**Portail, cristal et autel (F.19 à F.22, T.10 à T.12)**
- Un donjon généré entier : trouver la salle du portail et celle de l'autel, voir qu'elles ne gênent pas le chemin.
- Portail : « Allumer » (E), la jauge de 5 s, l'annulation par un coup ; le portail allumé de face (surface bleue,
  chevrons lumineux) ; le traverser à deux, l'un après l'autre ; le dernier vivant parti, les morts sont renvoyés.
- Mort : le texte de l'écran de mort ; le sac contient « Cristal de <nom> » (bulle et inventaire).
- Autel : « Relever les morts » (E) avec deux cristaux ; les relevés ont 25 % de vie, le prieur la moitié de la
  sienne ; un mort qui regardait un allié retrouve son personnage ; refus à 1 PV.

**Cuisine vivante (F.23 à F.26, `PLAN-PLAYTEST.md`)**
- La cuisine dans un donjon généré, avec ses habitants : se faufiler entre la table, le billot et l'âtre ; on
  bute sur l'âtre (bloc devant, au milieu) et le billot, on passe sous les ustensiles et la viande pendus.
- Sol, murs et plafond (F.27 à F.29) dans un donjon généré : un mur entre deux salles d'habillage différent, les
  couloirs et le tunnel au thème du donjon, les thèmes militaire, culte, bestial.
- Les salles plus anciennes ont leurs meubles adossés en ori 2 : à revoir dans l'atelier (4 contre le mur du haut).

**Salle du trône (F.36 à F.44, `PLAN-PLAYTEST.md`)**
- La grande salle (F.43) dans un donjon généré : elle demande 7×10 cases sur deux étages, voir si le générateur la
  place souvent ; la hauteur et la lumière (lustres, rosace) avec les gardes et le chef.
- La salle du trône dans un donjon généré, avec le chef : monter la marche puis l'estrade (0,5 puis 1,0) par le
  tapis ; on bute sur les piliers (le fût seulement), le trône et les chandeliers. En salle seule, le chef (case `c`)
  a tué le personnage d'essai avant toute capture : vérifier que c'est bien un combat et pas autre chose.

**Atelier des salles (F.14 à F.17, `PLAN-PLAYTEST.md`)**
- `.\icebreaker.ps1 salles` : ouvrir une salle, la meubler, l'enregistrer, puis la voir par `donjon -Site Genere
  -Salle <nom>` : chaque meuble est à la place et dans le sens montrés par l'atelier (surtout les meubles adossés
  à un mur et ceux en miroir).
- Une salle meublée sort tournée ou en miroir dans un donjon entier, ses meubles avec elle.

**Fantôme du village (jalon FT, `PLAN-FANTOME.md`)**
- Marcher et sauter en fantôme : il glisse, saute haut, retombe lentement ; la caméra le suit dès l'arrivée.
- À deux : chacun voit le fantôme de l'autre ; le pseudo au-dessus n'a pas été vu dans les captures.
- Le fantôme ne peut ni attaquer ni ramasser ; parler à un PNJ ; mourir en expédition ramène au village en fantôme.

**Clavier (X.16)**
- Cliquer « Clavier QWERTY » puis « Clavier AZERTY » dans Réglages > Contrôles ; le choix tient après relance.

**Lieu d'entraînement (jalon O, `PLAN-ENTRAINEMENT.md`)**
- O.7 : le bouton « Entraînement » du menu lance le serveur et entre dans le monde sans écran des personnages.
- O.4, O.5, O.8 : tomber à terre sous les coups des ennemis, être relevé, mourir, réapparaître à l'entrée ; les
  alliés suivent et ne frappent pas le joueur ; les tirs ennemis vers l'entrée ; le bilan des mannequins.
- O.3 : chaque compétence du mage et de l'archer sur de vrais ennemis, aux niveaux 1, 3 et 5.

**Campagne et village**
- I.2 zone fermée : sortie à pied, au planeur, à la nage ; montures non gérées.
- I.8 / V.9 / V.11 : départ groupé décidé par le chef, à deux joueurs, depuis la vue U ; `/rentrer` ; emplacement
  occupé tant qu'un membre y est.
- I.9 remise à neuf : monstres tués au combat et coffres ouverts de retour à la deuxième expédition.
- I.10 : en campagne, tuer le chef d'un donjon généré ou de la grotte annonce « Mission accomplie » ; les dormeurs
  d'un emplacement Genere dorment (des « Zzz » vus au client texte).
- I.12 : deux expéditions de suite vers « Genere » ou « Grotte » donnent deux plans différents, dessinés entiers
  (aucun reste de l'ancien, pas de tronçon vide), arrivée devant l'entrée.
- V.2 entrepôt : dépôt par un joueur, retrait par un autre après redémarrage ; V.12 fenêtre d'échange à distance.
- V.4 : acheter une potion à l'intendant, se faire soigner (60 or).
- V.5 : bilan au retour avec du vrai butin.
- C.1 à C.3 : maison bâtie contre terrain vide, bouton « Bâtir (300 or) », maison dessinée après relance
  (`heures_chantier` à baisser pour l'essai).
- X.1 fermeture de la vue U au départ et à la mort ; X.11 plan : cases à leur place, clic, point du joueur, Échap, fantôme en vol ; X.3 et X.9
  panneau, icônes, Échap, fermeture pour de bon ; X.4 couleurs et icônes des bâtiments ; X.5 E et T sur les PNJ dans
  leur maison ; X.8 recrues devant la charrette (posées 8 blocs trop haut sur le village d'essai ? `QUESTIONS.md`, Q30).
- X.10 : panneau « Campagne » (continuer, nouvelle partie), passage de l'ancienne campagne en « Partie 1 ».
- X.17 : « Supprimer » puis « Oui » efface la partie de la liste ; refus si son serveur tourne.
- R.6 à R.8 : barre de compétences du vrai client ; mage en jeu ; tourbillon, fendeur et les trois postures du
  guerrier ; les recrues créées avant R.6 n'ont pas de compétences. *Remplacé par le jalon A pour les trois classes.*

**Armes, classes et compétences (jalon A, `PLAN-CLASSES.md`)**
- A.4 : guerrier dans le vrai client : touches 1, 2 (arme en mains) et 3 (ultime), gestes et animations ; clic gauche
  et clic droit inchangés (plus de postures) ; l'ultime (fureur) avec une épée en mains (`QUESTIONS.md`, Q45).
- A.5 : une touche tenue ne relance pas ; la recharge se voit dans la barre (A.6).
- A.6 : la barre : infobulles au survol (compétences, passif), case grisée faute d'endurance ; la case ronde du
  passif se lit-elle, à gauche du bouton du journal ? (icônes, bord doré de l'ultime, secondes de recharge et case
  du passif vus en capture, guerrière et mage) ; la barre d'un archer.
- A.7 : prendre en mains une arme non apprise (marteau pour un guerrier à l'épée) : cases 1 et 2 vides et un message,
  une seule fois ; l'arbre de compétences de Veloren refuse un point.
- A.8 : deux recrues guerrières à la charrette : lots et armes différents, visibles dans le panneau et sur les recrues
  debout devant la charrette.
- A.9 : tuer des créatures en expédition : annonce « un point d'arme gagné » ; au retour, la maîtrise dans la fenêtre
  du bilan.
- A.10, A.15 : chaque compétence aux niveaux 3 et 5 (geste, effet), équilibre des seize compétences du guerrier.
- A.14 : fureur, séisme et rempart avec chacun des quatre jeux d'armes : le geste est-il lisible (`QUESTIONS.md`, Q45) ?
- A.13 : PV max d'un guerrier (inné et passif) vus par un coéquipier ; le passif ne tombe ni à terre ni à la mort.
- A.11, A.12 : la salle d'armes à la souris (le panneau et ses quatre onglets sont vus en capture) : chaque bouton
  cliqué (Débloquer, Monter, Apprendre, Activer, « Touche 1 : changer »), la liste qui défile à la molette, un
  personnage qui a un point à dépenser ; apprendre un jeu d'armes avec le sac plein ; la barre suit tout de suite
  ce qu'on a changé ; l'onglet « Compétences » dépasse un peu de son bouton.
- A.16 : mage au bâton et au sceptre : effets sur de vraies cibles (brûlure, soin d'un allié, protection, hâte),
  gestes des copies du sceptre, trois ultimes ; l'inné (−10 % de PV) le rend-il trop fragile ?
- A.17 : archer : saignement, ralentissement, bond arrière, trois ultimes ; équilibre.
- A.19 à A.27 (ultimes refaits, 2026-11-15) : l'Envol relève-t-il un allié à terre ; le laser de Fournaise arrêté
  par un mur ; la Volée mortelle : le cercle suit-il bien le regard, est-elle assez dense, tient-elle sous un
  plafond ; les trois frappes du Séisme ; l'équilibre de tous (plus tard) ; sans le plafond de 40 %, les grosses attaques d'arme tuent-elles
  trop vite.

**Combat (donjon Gnarling)**
- L.1 : nombre de coups pour tuer et mourir (cible : 3 et 3 à 4), autres armes, boss.
- L.5 : gnarling en 5 à 6 flèches ou boules de feu, ~10 touches de lance-flammes ; tir chargé et onde de feu ne tuent
  pas d'un coup (plafond 40 %). *Plafond retiré le 2026-11-15 (A.19) : ils peuvent tuer d'un coup.*
- L.2 à L.4, X.6 : verrouillage W, molette, anneau centré (sinon régler `ancrage` dans `verrouillage.ron`).

**Donjon généré**
- F.1 : la caméra reste-t-elle dans le donjon à l'échelle 7 ? Sinon 8/8, puis F.7.
- F.2 salles tournées, puits à échelles `H` ; F.3 taille ; F.4 bêtes endormies des nids ; F.5 formes, galerie sur
  deux étages ; F.6 rendu et orientation des sprites de décor  ; F.9 décor dessiné : rendu, orientation (mur/centre), cohérence d'un thème par donjon; G.10 sens des torches.
- F.8 : un donjon généré de la campagne se finit en entier (aucune salle coupée par le bord du monde).
- G.7 : descendre à pied jusqu'au chef et le tuer ; G.8 ouvrir une porte à clé à la main ; G.9 pose des dormeurs
  (à moitié dans le sol ?), approche accroupie.
- B.5 (et grotte) : ouvrir un coffre près de l'entrée puis un du fond : or et prise utile, plus riche au fond
  (`butin.ron`) ; rien d'inutile. B.14 : ce que lâchent les créatures, le chef, le boss.

- U.16 (la grotte est vue en jeu, Flo, 2026-11-24) : dans un donjon généré, percer un mur de pierre et une porte en
  bois à la pioche ; au village, la roche ne se creuse pas.

**Mort (jalon T)**
- T.2 : barre « Mort dans N s » au-dessus du corps d'un allié (position, lisibilité), à deux joueurs.
- T.3 : modèle du sac au sol, libellé « Fouiller le sac », fouille interrompue si on bouge.
- T.4 : à trois joueurs ou plus, le clic passe d'un allié regardé au suivant (corrigé en N.7, vu avec un seul allié).
- T.8 : mort, regarder un allié qui fouille le sac en visant droit vers le bas : plus d'écran noir ni de plantage.
- T.9 : à terre, environ 3 coups pour achever (barre rouge au-dessus du corps = PV) ; « Mort dans N s » figé
  pendant la relève, à deux joueurs.

**Habitants (jalon H)**
- H.1 à H.4 : un donjon généré et une grotte d'une seule race (corps, noms, armes des trois rôles), le chef humanoïde
  qui lance les attaques de feu et les totems du chef gnarling (animations d'un corps humanoïde ?), sa force face à
  un groupe ; araignées et autres solitaires dans les nids et les poches.
- H.5 : la campagne à quatre donjons générés, sans fort gnarling ; le départ ne propose plus « Donjon ».
- H.6 : difficulté des ennemis des races (PV, armure, plus de survie à 1 PV) ; si trop dur encore : moins de tireurs.
- Y.2, Y.9 (à deux ou plus) : les ennemis se répartissent entre les joueurs et laissent un joueur à terre tant qu'un
  autre est debout (trop dur, trop mou ?) ; des danari à plusieurs vont achever le joueur à terre, les frapper ou les
  repousser les détourne ; ils fuient blessés. Y.3 (`-Race human`) : des groupes de guerriers passent de salle en salle et
  reviennent (bloqués dans un escalier ou une porte ?).

**Divers**
- M.1 : téléchargement du plugin par le vrai client.
- S.2 / S.3 : le zip du client chez un pote (autre réseau, PC sans outils de dev) ; un départ en expédition sur le VPS.
- S.5 : lancer la Release du client sur GitHub (Actions → « Release du client ») ; le zip Mac sur un vrai Mac
  (Apple Silicon), le zip Windows chez un pote. S.6 : le push de `version.ron` 0.1.0 donne la Release `v0.1.0`.
- W.15 : prendre en jeu les captures des créatures du wiki (rubrique Créatures, nom du fichier sur chaque carte).
- Balises : plafond 200 par joueur et 200 au total (la plus ancienne s'éteint) ; vu en test seulement.

## Plus tard (à découper le moment venu)

- Passage automatique du client d'une instance à l'autre.
- Biome imposé pour terracotta, myrmidons et sahagins.
- Races ennemies au-delà des jalons H et Y : les compétences Icebreaker des classes pour les ennemis, un boss distinct du chef, des totems aux couleurs de la race (ce sont encore ceux du chef gnarling).
- Classes au-delà du jalon A (`decisions/0010-classes-et-combos.md`, Q36) : les six autres classes, les combos de zones.
- Recrues, traits, blessures, rang (`design/village.md`, `PLAN-BOUCLES.md` B.10).
- Carte stratégique façon Mount & Blade sur un grand monde Veloren, rencontres qui lancent les instances.

## Décisions et questions

Petites décisions tranchées et questions ouvertes pour l'équipe : `QUESTIONS.md`. Décisions importantes
(choix technique, changement de cap) : `decisions/`.
