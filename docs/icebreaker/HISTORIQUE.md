# Icebreaker sur Veloren — historique des tâches faites

Tâches cochées de `PLAN.md`, déplacées ici telles quelles (texte, hash, ce qui n'a pas été vérifié en jeu) quand leur
jalon est fini. Le détail d'une tâche est dans son commit (`git show <hash>`). Les tâches restantes sont dans
`PLAN.md` ; la grotte (K.1 à K.5) garde ses tâches dans `PLAN-GROTTE.md`. Les notes « pas encore vu en jeu » sont
reprises dans la liste « À vérifier en jeu » de `PLAN.md` : quand un point est vu, le barrer là-bas, pas ici.

## Prototype du 2026-10-28

- [x] Traces de donjon : monstres tués et coffres ouverts ne reviennent plus — 3a8d51e0c5
- [x] Instance : petit monde avec un seul donjon choisi (`ICEBREAKER_INSTANCE_SITE`), ~4 s pour 2 km — 633cfd635d
- [x] Touches AZERTY par défaut — 2788a2682b
- [x] Titre « Icebreaker » au menu — 432606c4ca
- [x] Apparition devant le donjon, ville de départ ignorée en instance — 13e0816046, 34dcab4f61

## Jalon 1 — Des instances propres (la base de tout)

- [x] **I.1 Pas de faune sauvage en instance** — ni faune des chunks, ni oiseaux et monstres errants de rtsim ;
  restent les habitants du donjon et de la ville. Pas encore vérifié 5 min en jeu. — c1c1aa969c Vu en jeu (Flo,
  2026-11-07).
- [x] **I.2 Zone fermée avec horizon** — cercle autour du site (rayon du site + `icebreaker.zone_margin`, 120 blocs) ;
  le serveur remet le joueur sur le bord avec un message. À vérifier en jeu (pied, planeur, nage) ; montures non
  gérées. — b166c0e2e4 Vu en jeu (Flo, 2026-11-07).
- [x] **I.3 Personnages communs à toutes les instances** — base de personnages dans `ICEBREAKER_CAMPAIGN_DIR`, monde
  recréé à chaque lancement ; en instance, le waypoint sauvé (d'un autre monde) est ignoré. Vérifié : deux lancements
  successifs utilisent la même base ; pas encore vérifié en jeu avec un objet ramassé. — 159c42400d
- [x] **I.4 Lanceur unique** — `icebreaker.ps1 village | donjon <type> | route` : dossier et ports par lieu, graine au
  hasard, journée de 48 min. `donjon` (port 14110) et `village` (14100, V.1) marchent ; `route` arrive avec P.1. — 159c42400d
- [x] **I.5 Temps de carte** — 1 min réelle = 30 min de carte (journée de 48 min, I.4) ; horloge dans
  `campagne/horloge.ron`, écrite par l'instance qui a des joueurs (toutes les `icebreaker.clock_save_interval`, 5 s,
  et au départ du dernier), reprise par une instance vide à l'arrivée de son premier joueur : l'heure suit les joueurs
  et s'arrête quand personne ne joue. Vérifié avec le client texte : reprise à l'arrivée, ×30, continuité après
  relance. Deux instances jouées en même temps s'écrasent l'horloge (la dernière écrite gagne). — 4a688313bc
- [x] **I.6 Lieux au menu** — au menu principal, bloc « Campagne », un bouton par lieu
  (`assets/icebreaker/lieux.ron`) : lance `icebreaker.ps1` si le port est libre, puis connexion ; route grisée ;
  donjon généré sur 14130. Vérifié : lancement et connexion (client texte) ; pas encore cliqué en jeu. — fceac98232
- [x] **I.7 Monde unique** (`0016`) — `icebreaker.ps1 campagne` : village au centre, emplacements de donjon
  (`assets/icebreaker/campagne.ron` : 2 Gnarling, 2 générés) à ~1000 blocs ; une zone fermée par lieu ; menu : un
  seul bouton « Campagne ». Testé (démarrage en ~10 s, zones) ; pas encore en jeu. — cd1c0527cd
- [x] **I.8 Expédition par téléportation** — le départ groupé (V.9) téléporte vers un emplacement libre ; `/rentrer`
  ramène sur la place ; emplacement occupé tant qu'un membre y est.
  *Fin* : village → donjon → village sans changer de serveur.
  Pas de place libre : on attend. Testé (logique ; `/rentrer` depuis un emplacement au client texte) ; départ par
  le chef pas encore vu en jeu. — e1cdcaee54
- [x] **I.9 Remise à neuf** — quand le dernier membre rentre, terrain regénéré, créatures et coffres de retour.
  *Fin* : une deuxième expédition retrouve monstres et coffres.
  Traces de la zone oubliées, tronçons regénérés. Testé (client texte : créatures revenues) ; tués au combat et
  coffres pas encore vus en jeu. — 89e9fd2fb3

## Jalon 2 — Le village (première version)

- [x] **V.1 Instance village** — `icebreaker.ps1 village` (port 14100, `-Site Town` par défaut, ou CliffTown,
  DesertCity, SavannahTown, CoastalTown) : une seule ville au centre du petit monde, sans autre site ni faune ;
  apparition sur sa place principale ; zone fermée autour (I.2). Habitants : gardes, marchands, aventuriers, chats
  et chiens. Vérifié : génération, apparition sur la place, connexion, horloge reprise ; pas encore promené en jeu. — 5237f6c353
  Correctif (premier essai en jeu : place vide) : ville en taille maximale (`-TailleVille`, 1.0) et emplacement
  choisi parmi plusieurs essais (près du centre, sinon ailleurs) selon le nombre de parcelles construites ; 15
  graines essayées, toutes entre 131 et 170 parcelles. — a25deefcf2
- [x] **V.2 Entrepôt commun** — le « Magasinier », PNJ sur la place (à `icebreaker.storekeeper_offset` du point
  d'apparition) ; touche T : fenêtre d'échange de Veloren, son inventaire est l'entrepôt (36 cases), il accepte
  tout échange sans regarder la valeur. Contenu dans `campagne/entrepot.ron`, écrit après chaque échange, rendu au
  magasinier quand il réapparaît avec son tronçon de terrain. Vérifié : test aller-retour (objets et quantités),
  serveur qui démarre ; pas encore vérifié en jeu (dépôt par un joueur, retrait par un autre après redémarrage). — 20b7b6a616
- [x] **V.3 Caisse commune** — « Trésorier » sur la place (touche T) : n'accepte que l'or, total dans
  `campagne/caisse.ron`, affiché dans son nom et annoncé à tous. Testé (logique, démarrage) ; pas encore en jeu. — 9e06774934
  Correctif (premier essai en jeu : T « pending invite » après E) : les deux PNJ peuvent parler (SPEAK) et disent sur
  E qu'on échange avec T. Vérifié en jeu (échange ouvert après E puis T). — f7e8721149
- [x] **V.4 Bâtiments par dialogue** — un PNJ par bâtiment, ses choix ouvrent l'action (Veloren a déjà des dialogues
  à questions et réponses) : intendance (achat et vente, avec le commerce de Veloren), infirmerie (soin payant).
  *Fin* : on achète une potion à l'intendant et on se fait soigner à l'infirmerie.
  Intendant (marchand de Veloren, stock et prix de l'économie de la ville, T) et Infirmier (E : question, soin
  complet pour `icebreaker.heal_price`, 60 or) sur la place. Testé (stock, prix, démarrage) ; pas encore en jeu. — 7eb80e8bdc
- [x] **V.6 Vue du village de dessus** — au village, une touche passe en vue de dessus (caméra fixe, façon 2D) sur
  le vrai village ; un clic sur un bâtiment ouvre son panneau (niveau, stocks, actions). Le serveur envoie la liste des
  bâtiments et leur emprise.
  *Fin* : en vue de dessus, cliquer sur l'entrepôt affiche son contenu.
  Bâtiments = les 4 maisons les plus proches de la place, PNJ devant la porte. Touche U : caméra au-dessus, clic →
  le serveur répond par le panneau du bâtiment visé (contenu, total, prix ; pas de niveau ni d'action pour
  l'instant). Testé (clic → bâtiment, démarrage, PNJ devant les maisons) ; pas encore en jeu. — 4237d4e1a7
  Ajout : bâtiments nommés et surlignés sous la souris en vue de dessus ; pas encore vu en jeu. — e6aaf6d11a
- [x] **V.5 Retour au village** — commande `/rentrer` en instance : bilan dans le chat, le joueur repart vers le
  village (reconnexion à la main pour l'instant).
  *Fin* : donjon → `/rentrer` → village, avec le butin.
  Bilan à tous (or, puis « 3 × objet ») de ce que le personnage porte de plus qu'à son arrivée, puis déconnexion ;
  on rejoint le village par le menu. Testé (bilan, commande reçue) ; pas encore en jeu avec du butin. — 046140d4fb
- [x] **V.7 Maison « Départ »** — 5e bâtiment, « Chef d'expédition » à la porte : sur E, les destinations de
  `lieux.ron` (sauf le village). Village de départ à 7 maisons. Testé (démarrage, place du chef) ; dialogue pas
  encore vu en jeu. — fc773b6a5e
- [x] **V.8 Voyage automatique** — choisir une destination fait partir le joueur : le serveur village lance le
  serveur du lieu s'il ne tourne pas, le jeu s'y connecte seul ; `/rentrer` ramène au village de même.
  *Fin* : village → donjon → `/rentrer` → village sans passer par le menu.
  Le jeu revient au menu, se connecte et rejoue le même personnage. Testé (logique serveur, compilation) ; pas
  encore fait le voyage complet en jeu. — fa8e4b0c1c
- [x] **V.9 Départ groupé** — le premier inscrit choisit la destination, les autres rejoignent ; « Prêt » ; quand
  tous sont prêts, 5 s annoncées puis départ ensemble ; s'éloigner ou se déconnecter désinscrit.
  *Fin* : deux joueurs arrivent ensemble dans le donjon.
  Partir seul : choisir le lieu puis « Je suis prêt ». Testé (logique à deux joueurs, démarrage) ; pas encore
  en jeu. — 8a708aeddc
- [x] **V.10 Panneaux à boutons** (menu village en vue U, Flo) — plusieurs boutons par panneau, panneau redemandé
  chaque seconde ; Caisse : verser 10/50/100/tout (`/caisse`) ; Infirmerie : se soigner (`/soigner`). Testé (client
  texte) ; boutons pas encore vus en jeu. — c8fb316e52
- [x] **V.11 Départ depuis la vue U** — boutons du chef (destination, rejoindre, prêt/attendre, se désinscrire) ;
  liste des inscrits (et qui est prêt) à gauche, en direct, pour tout le village ; inscrit tant qu'on est au village.
  Commande `/expedition`. Testé (client texte : inscription, prêt, départ, retour) ; boutons et liste pas encore vus
  en jeu. — 16423be9bb
- [x] **V.12 Échanges à distance** — Entrepôt et Intendance : « Ouvrir l'échange » depuis le panneau, sans aller au PNJ.
  Commande `/echanger`. Testé (client texte : échange ouvert à ~85 blocs) ; fenêtre pas encore vue en jeu. — 6ba0e5f24f

## Jalon C — Le village qui grandit (Flo, 2026-10-28) — C.1 à C.3

C.4 à C.6 restent dans `PLAN.md`.

La ville complète est prévue dès le départ ; seules quelques maisons sont bâties, les autres sont des terrains à
bâtir. On y paie des chantiers avec l'or de la caisse ; la maison sort de terre pendant la partie. Réglages dans
`assets/icebreaker/village.ron`, état dans `campagne/village.ron`.

- [x] **C.1 Petit village** — au départ, les 4 maisons des bâtiments et 2 maisons d'habitation ; 2 habitants par
  maison bâtie. Graine du village gardée pour la campagne. Testé (démarrage, maison bâtie / terrain vide au client
  texte) ; pas encore vu en jeu. — 0df322747c
- [x] **C.2 Terrains à bâtir** — en vue de dessus, les terrains libres (« Terrain à bâtir », prix), cliquables.
  Les 6 maisons libres les plus proches de la place, en vert ; panneau : prix, durée. Pas encore vu en jeu. — 7e714733a2
- [x] **C.3 Commander un chantier** — bouton « Bâtir (300 or) » dans le panneau d'un terrain, payé par la caisse ;
  chantier et fin (temps de carte) dans la campagne.
  Aussi `/batir <terrain>` ; maison bâtie à l'heure, dessinée au prochain lancement en attendant C.4. Testé (client
  texte : refus, paiement, fin) ; bouton pas encore vu en jeu. Durée : `heures_chantier` (6 h) à baisser pour les
  essais. — 0d929a9171

## Jalon G — Donjon généré (branche `icebreaker-donjon`) — G.1 à G.10

L'IA du donjon (G.11 à G.16) et les donjons avancés restent dans `PLAN.md`.

Le générateur du projet Icebreaker (graphe de mission, salles préfaites, assemblage, peuplement, paramètres :
`design/generation.md`) traduit en Rust (`world/src/icebreaker/dungeon/`) et dessiné avec les blocs et les créatures
de Veloren (`world/src/site/plot/icebreaker_dungeon.rs`). Données dans `assets/icebreaker/donjons/`.

- [x] **G.1 Lieu « Genere » minimal** — `icebreaker.ps1 donjon -Site Genere` : une salle sous terre, un tunnel en
  marches qui remonte à la surface au centre du site, apparition à sa sortie. Une case = 5 × 5 blocs, un étage = 5
  blocs (`generation.ron` → `render`). `-Port` pour lancer deux serveurs du même lieu. Vérifié : génération, apparition
  devant le tunnel ; pas encore descendu en jeu. — 4a74ba8402
- [x] **G.2 Lecture des salles** — les 16 salles d'Icebreaker (copiées le 2026-10-28, b7a3308) dans
  `assets/icebreaker/donjons/salles/`, vérifiées comme dans le projet (rôles, portes `+`, fosses `*`, accès depuis
  chaque connecteur) ; `-Salle <nom>` dessine une seule salle. Contrôle des blocs : `cargo run -p veloren-world
  --example icebreaker_dungeon -- <graine>` génère le petit monde et vérifie que chaque case accessible a un sol et de
  la place au-dessus, marches du tunnel comprises. Vérifié : 16 salles valides ; entrée, antre du chef, sanctuaire,
  poste de garde, hall à escalier, réserve sans problème de blocs. — 78b128ebe8
- [x] **G.3 Graphe de mission** — `MissionGraphGenerator.cs` traduit (`graph.rs`) : chemin principal découpé en étages,
  verrous avec leur clé dans une branche avant eux, branches, boucles et chutes dans une même zone, règles de
  voisinage ; réglages `sizes`, `branch_roles`, `rules`, `goal_roles` de `generation.ron`. Vérifié : sur 1000 graines
  par taille, objectif atteignable, retour toujours possible, chef unique et au plus profond, maximums, garde avant
  chaque verrou, casernes après l'entrée ou une garde, nids loin du chemin principal. — 9e42282351
- [x] **G.4 Assemblage** — `DungeonAssembler.cs` traduit (`assemble.rs`, sans les murs communs des châteaux) : salles
  posées depuis l'entrée, couloirs creusés par A* avec escaliers, boucles et puits de chute ; le tunnel de surface prend
  une porte de l'entrée (nouvelle salle `entree_quatre_portes`). Le serveur génère un donjon `medium` « tuer le chef »
  (graine du monde) et l'écrit dans son journal. Vérifié : 100 graines par taille assemblées et finissables (tout
  accessible depuis le tunnel, retour toujours possible, chef au plus profond) ; contrôle des blocs sans problème pour
  les graines 1234, 7 et 99 ; pas encore parcouru en jeu. — 64b42945d5
- [x] **G.5 Peuplement** — `DungeonPopulator.cs` traduit (`populate.rs`) : créatures et coffres par rôle de salle
  (`population`), d'abord sur les propositions des salles, jamais devant une porte ; le chef garde toujours la salle
  de l'objectif. Créatures et coffres de Veloren par faction (`factions` : gnarling par défaut, adlet, cultist ;
  coffres et butin de Veloren). Les endormies sont notées dans le plan (majuscules) mais apparaissent éveillées.
  Vérifié : tests (un seul chef, dans la salle de l'objectif ; personne devant une porte ; donjon toujours
  finissable) ; contrôle des blocs et des apparitions (une créature par marqueur, 11 à 20 par donjon) pour 5 graines.
  Pas encore joué : à vérifier en jeu que le chef se tue au fond. — dee9e5fca1
- [x] **G.6 Paramètres de mission** — `icebreaker.ps1 donjon -Site Genere -Taille -Objectif -Region -Saison -Menace
  -Ennemis -Faction` (variables `ICEBREAKER_DONJON_*`) ; effets dans `generation.ron` → `parameters` (valeurs du
  projet Icebreaker : écarts qui s'ajoutent, facteurs qui se multiplient, chef et boss uniques). Le résumé du
  journal donne les paramètres appliqués. Vérifié : graine 1234, menace 1 = 8 salles, 7 créatures ; menace 5 = 14
  salles, un étage de plus, 36 créatures ; toutes les combinaisons région × saison × objectif se génèrent ; serveur
  lancé avec `-Menace 5 -Faction adlet -Region mountain`. — 7cc1eff81c
- [x] **G.7 Victoire** — tuer le chef (ou le boss) annonce à tous « Mission accomplie » avec le bilan (créatures
  tuées, coffres ouverts, d'après les traces du donjon). Vérifié avec le client texte (`!jouer`, `/goto` chez le
  chef, `/explosion`) : message reçu. Pas encore descendu à pied jusqu'au chef en jeu. — b8c0371170
- [x] **G.8 Verrous et clés** — porte verrouillée = mur de blocs de porte à clé avec sa serrure au milieu ; une clé
  différente par porte (`generation.ron` → `keys`), dans un coffre posé où le générateur l'a mise, consommée à
  l'ouverture ; une porte ouverte le reste. Vérifié (client texte) : sans clé et avec une autre clé, refus ; avec la
  bonne, la porte s'ouvre et la clé disparaît ; le coffre donne la clé ; contrôle des blocs, 4 graines. Pas encore
  ouvert à la main en jeu. — 6bf39d3a49
- [x] **G.9 Créatures endormies** — casernes et nids : couchées sur le dos, immobiles, « Zzz… » en bulle ; réveil
  (« Hein ?! ») si frappées, si un combat s'entend, ou si un joueur approche (6 blocs, 2,5 accroupi), et les
  endormies à 12 blocs avec elles (`generation.ron` → `sleep`). Vérifié (client texte) : couchées, endormies à 12
  blocs, la caserne se réveille à 4 blocs, le nid à l'explosion. Pas vu dans le vrai client (pose : à moitié dans
  le sol ?), approche accroupie non testée. — e0355bd6d5
- [x] **G.10 Éclairage selon les habitants** — chance de lumière par case de sol selon le rôle de la salle (nid :
  aucune ; garde, cuisine, chef : beaucoup ; caserne : peu), ×0,3 sans créature, peu dans les couloirs ; moitié
  torches au mur, moitié braseros (`generation.ron` → `lighting`). Vérifié : test (nid noir, salle habitée bien plus
  éclairée que vide, torches contre un mur), contrôle des blocs pour 4 graines, serveur lancé (graine 1234 : 27
  torches, 45 braseros). Pas vu en jeu (sens des torches à vérifier). — ee9fbfe5a4

## Jalon F — Forme et vie du donjon généré (Flo, 2026-11-03) — F.1 à F.6

F.7 (caméra) reste dans `PLAN.md`.

Constat de Flo après une descente : donjon **vide** (11 à 20 créatures, rien dans les couloirs), **immenses
couloirs vides**, **tout petit** (5 blocs par case et par étage : la caméra sort du donjon), salles rectangulaires
sans décor, escaliers qui débouchent sur un mur. Causes trouvées dans le code : portes hautes de 3 blocs quelle que
soit l'échelle ; `place_child` ajoute la taille de la salle à la distance dès que la porte ne fait pas face ;
`route` ne vérifie pas la case derrière le haut d'un escalier ; peuplement des salles seulement, `default_density:
"low"` ; aucun décor. Réglages dans `generation.ron` ; worktree `Veloren-donjon`. Ordre : F.1 à F.6, F.7 si besoin.
Les grottes façon Deep Rock Galactic sont un lieu à part : `PLAN-GROTTE.md` (jalon K, après F ; K.1 à K.5 faits le 2026-11-04, reste à voir dans le vrai client).

- [x] **F.1 Échelle** — `cell_size: 7`, `level_height: 7` (6 blocs libres, couloirs de 7 ; passer à 8/8 si la caméra
  sort encore) ; portes et mur de porte à clé hauts de `h - 2` sous un linteau, torche à `h - 2`, serrure à mi-hauteur,
  colonne 3 × 3 ; `check_blocks` juste pour `c` pair. Testé : tests ciblés, exemple sur 5 graines (0 problème), lieu
  Genere démarré (graine 1234) ; caméra pas encore vue en jeu. — 13e4cc97ba
- [x] **F.2 Couloirs courts, escaliers propres** — salles dessinées tournées et retournées (`variety.orientations: 8`,
  136 salles) ; porte en face du parent et, parmi elles, au même étage (couloir plat, la salle fait la descente) ;
  `spacing` (1, 3), décalage −1 à 1 ; boucles et chutes limitées à `max_extra_length` (24 cases, sinon abandonnées :
  elles reliaient des salles éloignées, c'étaient les couloirs immenses ; ~15 % sont creusées) ; derrière le haut
  d'un escalier de couloir, jamais de case praticable ; pas de lumière près d'un escalier ; `H` = puits dégagé avec
  des sprites `Ladder` au mur. Mesuré (test, 100 graines par taille) : 14 → 6 cases de couloir par salle. Testé :
  tests ciblés, exemple sur 5 graines, serveur Genere ; échelle et salles tournées pas encore vues en jeu. — 3a63b26e8b
- [x] **F.3 Donjon plus grand** — `sizes` : moyen chemin (7, 9), branches (3, 5), boucles (1, 2) ; grand chemin (9, 12),
  branches (5, 8), étages (3, 4) ; `search_limit` et `margin` inchangés (ça suffit). Mesuré : 13,7 salles en moyenne en
  moyen (test, 40 graines, au moins 10 chacun) ; assemblage vert sur 100 graines ; exemple sur 5 graines ; serveur
  Genere lancé (graine 7). Pas encore vu en jeu. — 198820aeea
- [x] **F.4 Plus d'ennemis** — `population` relevée (base ×2 environ, `default_density` reste « peu »), couloirs
  peuplés (`corridors` : un guetteur par tranche de 10 cases, jamais sur un escalier, un palier ni devant une porte),
  entrée 0–1 guetteur, sorte `beast` pour les nids (mandragora, hound, frostfang). Mesuré (test, 40 graines) : 42,5
  créatures par donjon moyen, ~4 en couloir ; exemple sur 5 graines (40 à 55 apparues) ; serveur Genere lancé
  (graine 99). Pas encore vu en jeu (dont les bêtes endormies des nids). — 7e0f652d71
- [x] **F.5 Salles de formes variées** — `RoomShapes.cs` traduit (`shapes.rs`) : coins coupés, L, T, croix, octogone,
  ellipse, grotte (regénérée si elle sort rectangulaire) ; 2 à 4 portes sur les pans droits, piliers et coffres par
  rôle, part `share` 0,5, 8–12 × 6–10 cases, `min_bar` 2, tirées par graine de donjon (`variety.shapes`) ; entrée,
  antres et sanctuaire restent dessinés ; 3 salles dessinées en plus (colonnade, galerie sur deux étages, caserne
  double). Mesuré (test, 40 graines) : 5,5 salles générées par donjon moyen ; validité des salles générées sur 20
  graines ; exemple sur 5 graines ; serveur Genere lancé. Pas encore vu en jeu (formes, galerie, échelles). — aaa6d77b89
- [x] **F.6 Décor par rôle** — section `decor` de `generation.ron` : par rôle (et couloirs), sprites de Veloren par leur
  nom, posés contre un mur (`wall`, tournés vers la salle), au milieu d'une case (`center`, `around` pour des chaises
  autour d'une table) ou semés sur les blocs de sol (`scatter`, chance par bloc) ; jamais sur une case prise
  (créature, coffre, clé, lumière) ni à côté d'une porte, d'un escalier ou d'un trou ; noms vérifiés au chargement.
  Cuisine (marmite, chaudron, tonneaux, table et chaises), caserne (couchages, caisses), réserve (tonneaux, étagères,
  sacs, foin), nid (os, toiles, champignons), garde (table, outils, heaume), prison (barreaux, chaînes), trésor et
  chambre forte (bougies, orbe), antre (feu, trophées, os), sanctuaire (bougies, sceau, tombes), couloirs (os,
  pierres, toiles). Testé : test de placement, tests ciblés, exemple sur 5 graines, serveur Genere ; rendu et sens
  des sprites pas encore vus en jeu. — bc767e06e3

## Jalon L — Combat létal (Flo, 2026-10-30)

- [x] **L.1 Dégâts façon Dark Souls** — `assets/icebreaker/combat.ron` multiplie les dégâts des attaques selon la
  cible : une arme de départ tue un gnarling en ~3 coups, un gnarling tue un joueur en 3 à 4 coups ; une créature
  plus forte encaisse plus. La mise à terre des humanoïdes reste. Testé : tests ciblés, le donjon Gnarling démarre ; nombre de coups en jeu, autres armes et boss pas vérifiés. — 373c5671c8
- [x] **L.2 Verrouiller une cible** — touche W : l'ennemi le plus proche du centre de l'écran (20 m), la caméra le
  suit ; saute à sa mort ou au-delà de 30 m (`assets/icebreaker/verrouillage.ron`). Testé : tests ciblés, le client
  compile ; pas vérifié en jeu. — df0932cf53
- [x] **L.3 Changer de cible à la molette** — pendant un verrouillage, molette vers le haut : ennemi à droite, vers le
  bas : à gauche, en faisant le tour ; plus de zoom. Même commit que L.2 (code entremêlé) ; pas vérifié en jeu. — df0932cf53
- [x] **L.4 Cercle sur la cible** — anneau doré qui respire, halo, point et quatre traits, par-dessus tout, au milieu
  du corps de la cible verrouillée. Testé : le client compile ; rendu pas vérifié en jeu. — 9c2b1d8a77

## Jalon R — La charrette et le registre commun (Flo, 2026-11-03)

On arrive au village en fantôme et on prend une recrue à la charrette ; les personnages sont au village (`0017`).
Réglages : `assets/icebreaker/classes.ron`, `registre.ron` ; état : `campagne/registre.ron`.

- [x] **R.1 Classes en données** — guerrier (épée, cuir clouté), mage (bâton, lin), archer (arc, cuir) :
  `classes.ron` et `classes/*.ron`. Testé (tests, et au client texte : arme et armure portées). — 8b1854d
- [x] **R.2 Arrivée en fantôme** — registre actif (`ICEBREAKER_REGISTRE`, campagne et village) : le jeu passe
  l'écran de personnages et entre en spectateur sur la place ; un fantôme ne part pas en expédition. Testé au
  client texte ; vrai client compilé, pas lancé (vue U d'un fantôme, sortie de zone non bloquée). — 9bd8d32d27
- [x] **R.3 La charrette** — 6e bâtiment de la vue U (village de départ à 8 maisons) : recrues au hasard, « Engager »
  crée le personnage au registre et le fait jouer ; renouvelée chaque jour de carte, complétée au minimum au retour
  d'un donjon (`/rentrer`) et si un fantôme n'a rien à prendre. Commande `/registre`. Testé au client texte ;
  retour de donjon pas essayé. — 9bd8d32d27
- [x] **R.4 Registre commun** — « Jouer » un libre (aussi d'un personnage à l'autre), « Rendre au registre » ; un
  joueur par personnage (lu des joueurs en jeu). Testé à deux au client texte (refus, reprise avec le même
  inventaire) ; pas encore avec un objet ramassé. — 9bd8d32d27
- [x] **R.5 Réserver** — « Réserver » / « Libérer », 5 par compte (`registre.ron`). Testé (tests ; refus à un autre
  joueur au client texte). — 9bd8d32d27
- [x] **R.6 Deux compétences par classe** — en plus des attaques de base, sur les touches 1 et 2 : guerrier
  (taillade en croissant, frappe cruelle), mage (onde de feu, ruée de feu), archer (appui, encoche lourde) ;
  `classes/competences/*.ron`. Testé (tests ; guerrier et archer engagés au client texte, compétences en base) ;
  mage en jeu et barre du vrai client pas vérifiés ; les recrues déjà créées n'en ont pas. — b75f07ff26
- [x] **R.7 Guerrier plus spectaculaire** — cascade (prérequis, touche 1), tranche-tourbillon et fendeur de terre
  (touches 2 et 3, posture tranchante). Testé (tests ; mage et archer engagés au client texte, compétences en
  base) ; pas de guerrier à la charrette ce jour-là, tourbillon et fendeur pas vus en jeu. — 1b7008aae2
- [x] **R.8 Trois postures pour le guerrier** — touche 1 riposte (défensive), 2 balayage lourd (lourde), 3
  tranche-tourbillon (tranchante) ; prérequis sur 4 et 5. Ordre des touches en données (`touches` de `classes.ron`).
  Testé (tests ; guerrier engagé au client texte, touches en base dans cet ordre) ; postures pas vues dans le vrai
  client. — 88b52bb7ee

## Jalon X — Interface et ergonomie du village (Flo, 2026-11-03)

Décision : `decisions/0018-refonte-interface-village.md`.
Reprendre ce que le vrai client a montré : robustesse et cohérence d'abord, le visuel reste celui de Veloren
(cadres, boutons, icônes et sprites existants ; pas de nouvelles textures ni de portrait 3D). Ordre : X.1, X.7,
X.2, X.3, X.8, X.4, X.5, X.6. Réglages dans `assets/icebreaker/vue_village.ron`, `village.ron`, `registre.ron`,
`verrouillage.ron`.
Fait le 2026-11-03 (branche `icebreaker-interface`, fusionnée), vérifié au client texte et au journal du vrai client,
puis vu en jeu par Flo le soir même : « c'est cool ».
- [x] **X.1 Vue U : entrée et sortie propres** — la vue se ferme d'elle-même quand le joueur s'éloigne
  (téléportation, `/rentrer`), meurt, ou que le serveur ne renvoie plus de bâtiments (hors du village : liste
  vide et message) ; hauteur, plan et rayon de sortie dans `vue_village.ron`. Vérifié au client texte (liste sur la
  place, vide au donjon après le départ, de retour après `/rentrer`) ; fermeture de la caméra pas encore vue en jeu. — eaa5413379
- [x] **X.7 « Campagne » amène sur la place, en fantôme** — pseudo pris sans invite, écran de personnages réduit à
  un chargement tant que le serveur n'a pas répondu, demande de fantôme gardée, voyage raté annulé, message d'accueil
  du fantôme (fr, en), « Campagne » en `.ftl` ; `ICEBREAKER_AUTO_CAMPAGNE=1` pour entrer sans clic (tests). Vérifié au
  journal du vrai client (menu → fantôme sans clic) et au client texte (accueil) ; pas encore vu à l'écran. — 4fb6ae42e8
- [x] **X.2 Clic juste en vue U** — le serveur envoie l'altitude de chaque bâtiment ; le client projette les
  coins avec la vraie matrice de la caméra, survol et clic sur le rectangle dessiné. Testé (projection, serveur,
  altitudes au client texte : de 201 à 257 sur un village) ; alignement pas encore vu en jeu. — 4e9faa3e09
- [x] **X.3 Un vrai panneau de bâtiment** — fenêtre dans les cadres de dialogue de Veloren (titre, croix, Échap),
  lignes avec icônes d'objets (`(Content, Vec<ItemKey>)`), boutons en colonne ; liste d'expédition du même style
  sous le panneau ; bandeau d'aide et « prêt » en `.ftl`. Testé (compilation, panneaux au client texte) ; rendu,
  icônes et Échap pas encore vus en jeu. — a2d4a8e6a6
- [x] **X.8 Charrette : voir les recrues** — chaque recrue et chaque fiche avec les icônes de son arme et de son
  torse ; les recrues du jour debout devant la charrette (`registre.ron` → `postes_recrues`), équipées et nommées,
  retirées quand engagées ou renouvelées ; E : « Engagez-moi à la charrette ». Testé (tests, client texte : icônes,
  recrues debout, PNJ parti à l'engagement) ; pas encore vu dans le vrai client. — db77c93340
- [x] **X.4 Identité visuelle des bâtiments** — en vue U, couleur et icône du HUD par sorte de bâtiment
  (`vue_village.ron` → `apparences`) ; dans le village, enseignes (sprites de Veloren) sur le rebord des fondations
  de part et d'autre de la porte (`village.ron` → `enseignes`). Testé (tests, enseignes vues au client texte) ;
  couleurs et icônes pas encore vues en jeu. — fe3400d39e
- [x] **X.5 Les PNJ dans leur maison** — magasinier, trésorier, intendant, infirmier et chef à l'intérieur
  (centre de la case de la porte, au plancher) ; la charrette et ses recrues restent dehors. Testé (client texte :
  les cinq dans l'emprise de leur maison, au plancher) ; E et T dedans pas encore essayés en jeu. — af89bed30e
- [x] **X.6 Anneau de verrouillage centré** — cause trouvée : les cercles sans position explicite étaient placés
  par conrod par rapport au widget dessiné avant ; posés en (0, 0) puis projetés ; `ancrage` et `rayon_anneau` dans
  `verrouillage.ron`. Testé (tests, compilation) ; centrage pas encore vu en jeu (sinon : régler `ancrage`). — 30ed3cd7b3
- [x] **X.9 Panneau fermé pour de bon** (Flo, 2026-11-04) — fermé (croix ou Échap), le panneau n'est plus redemandé
  chaque seconde ni rouvert par une réponse en route. Testé (compilation) ; pas encore vu en jeu. — 4cdf36023a
- [x] **X.10 Parties de campagne** (Flo, 2026-11-04) — « Campagne » propose de continuer une partie ou d'en créer une
  (nom tapé) ; une partie = `userdata-icebreaker/parties/<nom>` (`-Partie`), l'ancienne campagne devient « Partie 1 ».
  Testé (tests, compilation, script sur une copie) ; panneau et lancement depuis le menu pas encore vus en jeu. — c861af7b08

## Jalon 4 — Démo à l'équipe — S.1

S.2, S.3 et D.1 restent dans `PLAN.md`.

- [x] **S.1 Script de lancement Linux** — `icebreaker.sh`, même rôle que `icebreaker.ps1` (village, donjon), pour le
  serveur de test (VPS Hetzner CAX11, ARM64, 2 vCPU, 4 Go).
  *Fin* : `./icebreaker.sh village` démarre un serveur qui écoute sur 14100.
  Village et donjon démarrent (Git Bash + serveur Windows) ; pas encore lancé sur un vrai Linux. — df6e34479d

## Jalon J — Balise lumineuse (Flo, 2026-11-06) — vu en jeu le 2026-11-06

Une touche pose aux pieds une balise qui éclaire dans la couleur choisie, pour se repérer dans le donjon ; une seule
charge, rendue 10 s après la pose ou tout de suite en ramassant une balise (E). Réglages :
`assets/icebreaker/balise.ron` ; fiche wiki `wiki/balise.md`.

- [x] **J.1 Poser une balise** — objet donné à l'arrivée en donjon, touche `<`, `/balise poser`, recharge et
  « Balise prête », plafond de 6 au sol. Client texte ; lumière et touche pas vues en vrai client. ce1de1d86c
- [x] **J.2 Retirer une balise (E)** — éteinte, charge rendue à qui la ramasse ; effacées quand l'emplacement se
  vide (pas testé). Client texte. ae868b2ad1
- [x] **J.3 Menu des couleurs** — F9 ou `/balise` : panneau à boutons, montré hors vue de dessus. Panneau reçu au
  client texte ; affichage en jeu pas vu. 7a16e3223b
- [x] **J.5 Utiliser la balise, voir sa recharge** — l’utiliser (sac, barre de raccourcis) la pose ; la barre la
  grise et affiche les secondes qui restent (message `IcebreakerBaliseRecharge`). Message vu au client texte. d4d6273e8f
