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
  dormeurs, décor), une grotte façon Deep Rock Galactic (poches, filons, chef au fond), combat létal et verrouillage
  de cible. Détail et hashes : `HISTORIQUE.md`, `PLAN-GROTTE.md`.
- **Presque rien n'a été vu dans le vrai client** : la plupart des tâches ont été vérifiées par des tests et le client
  texte ; seul le jalon X (interface du village) a été joué par Flo. Liste : « À vérifier en jeu » plus bas.
- **Ce qui manque à la démo D.1** : le pillage sur la route (P.1 à P.4, pas commencé) et le serveur pour les potes
  (S.2, S.3). Le contenu de la démo est à confirmer ou à réduire : `QUESTIONS.md`, Q1.
- **Propositions en attente de l'équipe** : boucles de jeu (`PLAN-BOUCLES.md`, B.1 à B.13), mécaniques du donjon
  (`PLAN-DONJON.md`, E.1 à E.16), mod (`PLAN-MOD.md`), piliers, menace et âmes (`design/`), idées (`IDEES.md`).
- **Prochaine étape proposée** : une session de test dans le vrai client sur la liste « À vérifier en jeu », puis le
  bloc 1 des boucles (B.1 à B.4), puis S.2 et S.3 pour jouer à plusieurs (`QUESTIONS.md`, Q7).

## Architecture visée

- **Un seul serveur pour la campagne** (`0016`, remplace pour ce prototype le processus par instance de `0005`) :
  le village et des emplacements de donjon dans un même monde ; une expédition prend un emplacement libre. Les lieux
  restent lançables seuls pour tester un donjon.
- **Ce qui dure entre les instances** (personnages, inventaires, caisse et entrepôt du village, horloge, traces des
  donjons) vit dans un dossier de **campagne** commun à tous les processus, pas dans le monde d'une instance.
- Le client passe d'une instance à l'autre en se reconnectant (à la main d'abord, automatiquement ensuite).

## Jalons finis (détail dans `HISTORIQUE.md`)

Prototype du 2026-10-28 ; jalon 1 (instances propres, I.1 à I.9) ; jalon 2 (village, V.1 à V.12) ; L (combat
létal) ; R (charrette et registre) ; X (interface du village) ; S.1 (script Linux). Finis en partie : C (C.1 à C.3),
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

## Jalon B — Boucles de jeu (proposition du 2026-11-02, à valider)

Ce qui rend l'enchaînement village → donjon → village intéressant (risque, butin, besoins du village) : règles et
tâches B.1 à B.13 dans `PLAN-BOUCLES.md`. Ce qu'on fait en bas (ressources du donjon, sac à formes, foreuse, trous,
garde et parade, monstres à coop) : tâches E.1 à E.16 dans `PLAN-DONJON.md`.

## Jalon 4 — Démo à l'équipe

Fait : S.1 (`icebreaker.sh`, même rôle que `icebreaker.ps1` ; pas encore lancé sur un vrai Linux).

- [ ] **S.2 Serveur sur le VPS** — *en cours : Veloren-serveur, 2026-11-04* — binaire Linux release, assets, un service systemd par lieu (village et donjon
  en même temps), pare-feu ouvert sur les ports de jeu ; `--no-auth` (on fait confiance aux potes au début).
  *Fin* : un pote se connecte à `<vps>:14100` depuis chez lui.
- [ ] **S.3 Client pour les potes** — adresse du serveur dans `lieux.ron` (pas de serveur local lancé si l'adresse
  est distante), zip du client à envoyer.
  *Fin* : un pote lance le zip, clique « Village » et arrive sur le VPS.
- [ ] **D.1 Soirée de test** : village → donjon → village → route → village, à deux ou plus, puis décision du moteur
  avec Sayanel (`0015`). Contenu à confirmer (`QUESTIONS.md`, Q1).

## À vérifier en jeu

Ce que les tâches cochées n'ont vérifié qu'en test ou au client texte (détail dans `HISTORIQUE.md` et
`PLAN-GROTTE.md`). À parcourir dans le vrai client, en campagne, à deux si possible ; barrer ce qui est vu, ouvrir une
tâche pour ce qui casse.

**Campagne et village**
- I.2 zone fermée : sortie à pied, au planeur, à la nage ; montures non gérées.
- I.8 / V.9 / V.11 : départ groupé décidé par le chef, à deux joueurs, depuis la vue U ; `/rentrer` ; emplacement
  occupé tant qu'un membre y est.
- I.9 remise à neuf : monstres tués au combat et coffres ouverts de retour à la deuxième expédition.
- V.2 entrepôt : dépôt par un joueur, retrait par un autre après redémarrage ; V.12 fenêtre d'échange à distance.
- V.4 : acheter une potion à l'intendant, se faire soigner (60 or).
- V.5 : bilan au retour avec du vrai butin.
- C.1 à C.3 : maison bâtie contre terrain vide, bouton « Bâtir (300 or) », maison dessinée après relance
  (`heures_chantier` à baisser pour l'essai).
- X.1 fermeture de la vue U au départ et à la mort ; X.2 alignement des clics sur les maisons en pente ; X.3 et X.9
  panneau, icônes, Échap, fermeture pour de bon ; X.4 couleurs et icônes des bâtiments ; X.5 E et T sur les PNJ dans
  leur maison ; X.8 recrues devant la charrette (posées 8 blocs trop haut sur le village d'essai ? `QUESTIONS.md`, Q30).
- X.10 : panneau « Campagne » (continuer, nouvelle partie), passage de l'ancienne campagne en « Partie 1 ».
- R.6 à R.8 : barre de compétences du vrai client ; mage en jeu ; tourbillon, fendeur et les trois postures du
  guerrier ; les recrues créées avant R.6 n'ont pas de compétences.

**Combat (donjon Gnarling)**
- L.1 : nombre de coups pour tuer et mourir (cible : 3 et 3 à 4), autres armes, boss.
- L.2 à L.4, X.6 : verrouillage W, molette, anneau centré (sinon régler `ancrage` dans `verrouillage.ron`).

**Donjon généré**
- F.1 : la caméra reste-t-elle dans le donjon à l'échelle 7 ? Sinon 8/8, puis F.7.
- F.2 salles tournées, puits à échelles `H` ; F.3 taille ; F.4 bêtes endormies des nids ; F.5 formes, galerie sur
  deux étages ; F.6 rendu et orientation des sprites de décor ; G.10 sens des torches.
- F.8 : un donjon généré de la campagne se finit en entier (aucune salle coupée par le bord du monde).
- G.7 : descendre à pied jusqu'au chef et le tuer ; G.8 ouvrir une porte à clé à la main ; G.9 pose des dormeurs
  (à moitié dans le sol ?), approche accroupie.

**Grotte**
- K.3 : les trois ambiances se reconnaissent-elles, la roche luisante suffit-elle sans lanterne (`QUESTIONS.md`, Q22) ;
  caméra dans les boyaux.
- K.4 : miner un filon à la pioche, changement d'arme ; K.5 : créatures des culs-de-sac posées un bloc trop haut ou
  trop bas.

**Divers**
- M.1 : téléchargement du plugin par le vrai client.
- S.2 : vrai client sur le VPS (`116.202.107.125:14100`), depuis chez un pote ; un départ en expédition là-bas.

## Plus tard (à découper le moment venu)

- Passage automatique du client d'une instance à l'autre.
- Biome imposé pour terracotta, myrmidons et sahagins.
- Classes (`decisions/0010-classes-et-combos.md`) : capacités en fichiers de données.
- Recrues, traits, blessures, rang (`design/village.md`, `PLAN-BOUCLES.md` B.10).
- Carte stratégique façon Mount & Blade sur un grand monde Veloren, rencontres qui lancent les instances.

## Décisions et questions

Petites décisions tranchées et questions ouvertes pour l'équipe : `QUESTIONS.md`. Décisions importantes
(choix technique, changement de cap) : `decisions/`.
