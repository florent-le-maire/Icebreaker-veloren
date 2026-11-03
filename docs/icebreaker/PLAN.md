# Icebreaker sur Veloren — plan

Prototype du concept Icebreaker sur une copie modifiée de Veloren (décision : `decisions/0015-prototype-veloren.md`).
Vision et règles : `VISION.md`, `GAME_DESIGN.md`, `design/` (copiés du dépôt Icebreaker le 2026-10-28 ; les règles
écrites pour Godot se lisent comme des intentions, pas comme du code à reprendre).

Objectif du prototype : **une démo d'une soirée** où l'équipe enchaîne village → donjon → village → pillage sur la
route → village, pour trancher le choix du moteur avec Sayanel.

Méthode : une tâche = quelques heures au plus, avec un critère de fin vérifiable ; un commit par tâche (message en
français) ; on coche avec le hash. Le jeu doit rester lançable après chaque tâche.

## Déjà fait (prototype du 2026-10-28)

- [x] Traces de donjon : monstres tués et coffres ouverts ne reviennent plus — 3a8d51e0c5
- [x] Instance : petit monde avec un seul donjon choisi (`ICEBREAKER_INSTANCE_SITE`), ~4 s pour 2 km — 633cfd635d
- [x] Touches AZERTY par défaut — 2788a2682b
- [x] Titre « Icebreaker » au menu — 432606c4ca
- [x] Apparition devant le donjon, ville de départ ignorée en instance — 13e0816046, 34dcab4f61

## Architecture visée

- **Un seul serveur pour la campagne** (`0016`, remplace pour ce prototype le processus par instance de `0005`) :
  le village et des emplacements de donjon dans un même monde ; une expédition prend un emplacement libre. Les lieux
  restent lançables seuls pour tester un donjon.
- **Ce qui dure entre les instances** (personnages, inventaires, caisse et entrepôt du village, horloge, traces des
  donjons) vit dans un dossier de **campagne** commun à tous les processus, pas dans le monde d'une instance.
- Le client passe d'une instance à l'autre en se reconnectant (à la main d'abord, automatiquement ensuite).

## Jalon 1 — Des instances propres (la base de tout)

- [x] **I.1 Pas de faune sauvage en instance** — ni faune des chunks, ni oiseaux et monstres errants de rtsim ;
  restent les habitants du donjon et de la ville. Pas encore vérifié 5 min en jeu. — c1c1aa969c
- [x] **I.2 Zone fermée avec horizon** — cercle autour du site (rayon du site + `icebreaker.zone_margin`, 120 blocs) ;
  le serveur remet le joueur sur le bord avec un message. À vérifier en jeu (pied, planeur, nage) ; montures non
  gérées. — b166c0e2e4
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

## Jalon C — Le village qui grandit (Flo, 2026-10-28)

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
- [ ] **C.4 Chantier visible** — la maison sort de terre au fil du temps de carte, sans relancer.
- [ ] **C.5 Nouveaux habitants** — une maison finie amène des habitants.
- [ ] **C.6 Paliers** — l'or investi (300, 1000, 3000) ouvre des terrains plus loin de la place.

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

## Jalon G — Donjon généré (hors démo, branche `icebreaker-donjon`)

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

**Donjons avancés (idées, à découper le moment venu)** — aujourd'hui un seul lieu, le souterrain. Lieux envisagés
(le projet Icebreaker en décrit déjà quelques-uns : `design/generation.md`, « Derrière les murs et lieux ») :
- **La tour** : on monte au lieu de descendre, étages étroits, maçonnerie et ciel autour, chef au sommet.
- **Le marécage** : salles à ciel ouvert reliées par l'eau et la boue, nids nombreux (effets de la région Marais).
- **La forêt** : clairières reliées par des sentiers, sous les arbres ; nids et archers (région Forêt).
- **Le château** : salles collées par leurs murs, cour et remparts (assemblage « salles collées » du projet).
- **Le gouffre** (façon *Made in Abyss*) : un grand puits vertical, des salles accrochées à ses parois ; plus on
  descend, plus c'est dangereux et riche, et remonter coûte (fatigue, temps de carte, blessures ?).

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

## Jalon X — Interface et ergonomie du village (Flo, 2026-11-03) — *en cours : Veloren-interface*

Reprendre ce que le vrai client a montré : robustesse et cohérence d'abord, le visuel reste celui de Veloren
(cadres, boutons, icônes et sprites existants ; pas de nouvelles textures ni de portrait 3D). Ordre : X.1, X.7,
X.2, X.3, X.8, X.4, X.5, X.6. Réglages dans `assets/icebreaker/vue_village.ron`, `village.ron`, `registre.ron`,
`verrouillage.ron`.

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
- [ ] **X.5 Les PNJ dans leur maison** — magasinier, trésorier, intendant, infirmier et chef à l'intérieur
  (case de la porte ou sa voisine, au plancher) ; la charrette et ses recrues restent dehors.
  *Fin* : E et T marchent dans la maison ; échanges à distance et départ inchangés.
- [ ] **X.6 Anneau de verrouillage centré** — ancrage mesuré sur trois corps (gnarling, humanoïde, chef),
  décalage relatif et tailles dans `verrouillage.ron`.
  *Fin* : l'anneau est au milieu du corps, de près comme de loin.

## Jalon B — Boucles de jeu (proposition du 2026-11-02, à valider)

Ce qui rend l'enchaînement village → donjon → village intéressant (risque, butin, besoins du village) : règles et
tâches B.1 à B.13 dans `PLAN-BOUCLES.md`. Ce qu'on fait en bas (ressources du donjon, sac à formes, foreuse, trous,
garde et parade, monstres à coop) : tâches E.1 à E.16 dans `PLAN-DONJON.md`.

## Jalon 4 — Démo à l'équipe

- [x] **S.1 Script de lancement Linux** — `icebreaker.sh`, même rôle que `icebreaker.ps1` (village, donjon), pour le
  serveur de test (VPS Hetzner CX22, x86, 2 vCPU, 4 Go).
  *Fin* : `./icebreaker.sh village` démarre un serveur qui écoute sur 14100.
  Village et donjon démarrent (Git Bash + serveur Windows) ; pas encore lancé sur un vrai Linux. — df6e34479d
- [ ] **S.2 Serveur sur le VPS** — binaire Linux release, assets, un service systemd par lieu (village et donjon
  en même temps), pare-feu ouvert sur les ports de jeu ; `--no-auth` (on fait confiance aux potes au début).
  *Fin* : un pote se connecte à `<vps>:14100` depuis chez lui.
- [ ] **S.3 Client pour les potes** — adresse du serveur dans `lieux.ron` (pas de serveur local lancé si l'adresse
  est distante), zip du client à envoyer.
  *Fin* : un pote lance le zip, clique « Village » et arrive sur le VPS.
- [ ] **D.1 Soirée de test** : village → donjon → village → route → village, à deux ou plus, puis décision du moteur
  avec Sayanel (`0015`).

## Plus tard (à découper le moment venu)

- Passage automatique du client d'une instance à l'autre.
- Biome imposé pour terracotta, myrmidons et sahagins.
- Classes (`decisions/0010-classes-et-combos.md`) : capacités en fichiers de données.
- Recrues, traits, blessures, rang, registre commun (`design/village.md`).
- Carte stratégique façon Mount & Blade sur un grand monde Veloren, rencontres qui lancent les instances.

## Décisions (Flo, 2026-10-28)

1. **Personnages** : pour le prototype, chaque compte garde ses propres personnages (modèle de Veloren) ; le
   registre commun viendra plus tard. *Remplacée le 2026-11-03 par `0017` (charrette et registre commun, jalon R).*
2. **Village** : une vraie ville Veloren où l'on marche, **plus une vue de dessus façon 2D** où l'on clique sur les
   bâtiments pour voir les stocks et agir (V.6).
3. **Butin du convoi** : chacun garde ce qu'il ramène (règle de butin habituelle).
4. **Clés du donjon généré** : une clé différente par porte (G.8).
5. **Sommeil** : les endormis sont couchés, avec des « Zzz » au-dessus d'eux (G.9).

## Questions ouvertes

1. **PV entre deux lieux** : Veloren remet les PV au maximum à chaque connexion ; on revient donc soigné du donjon
   et l'infirmerie (V.4) ne sert qu'aux blessures prises au village. Sauver les PV dans la campagne, ou attendre
   les blessures (`design/village.md`) ?
2. **Gouffre** (donjons avancés, jalon G) : que coûte la remontée, et le gouffre est-il une seule mission très
   profonde ou une suite de descentes qu'on reprend où on s'était arrêté ?
3. **Charrette** (jalon R) : engager est-il gratuit (un fantôme n'a pas d'or ; proposé pour la démo) ou payé par la
   caisse ? Combien de places au registre (proposé : 10), et peut-on renvoyer un personnage ? Que devient un
   personnage du registre qui meurt (attendre B.3, ou il quitte le registre) ?
