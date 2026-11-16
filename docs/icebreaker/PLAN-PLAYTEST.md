# Avant le playtest avec les potes — feuille de route

*Liste de Flo du 2026-11-16, mise en plan le même jour ; découpage **à valider** avant de coder.* Ce que le jeu doit
avoir avant d'inviter tous les potes à tester. Ce fichier reprend chaque point de la liste, dit ce qui existe déjà,
propose des tâches et renvoie les points flous à `QUESTIONS.md` (Q55 à Q62). Les règles des âmes sont dans
`design/ames.md`.

Les tâches gardent la lettre de leur jalon (F donjon généré, U grottes, B boucles, E donjon à jouer, L combat, X
interface du village) et se cochent ici ; les âmes ouvrent le jalon **AM**.

## Vue d'ensemble

| # | Chantier | Tâches | Worktree | État |
|---|---|---|---|---|
| 1 | Donjon : des pièces travaillées | F.10 à F.13 | `Veloren-donjon` | à valider (Q60) |
| 2 | Grotte : murs à casser, filons, salle au minerai | U.13 à U.15 | `Veloren-caverne` | à valider (Q59) |
| 3 | Missions : sablier, zones, types de mission, boss | B.16 à B.20 | `Veloren-boucles` | à valider (Q55, Q56) |
| 4 | Âmes | AM.1 à AM.8 | `Veloren-ames` | à valider (Q57, Q58) |
| 5 | Améliorations : village sans dégâts, bouton Rentrer | L.6, X.18 | `Veloren-village` | prêt à coder |
| 6 | Refonte des classes | — | `Veloren-classes` | à détailler par Flo |
| 7 | Craft et objets à trouver | E.17, E.18 | `Veloren-donjon` | à valider (Q61) |
| 8 | Équilibrage | — | — | en dernier |
| 9 | Le jour J : technique | — | `Veloren-serveur` | liste de contrôle |

**Ordre proposé** (à confirmer, Q62) : le chantier 5 d'abord (quelques heures, évite de perdre des personnages au
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

- [ ] **F.10 Catalogue des pièces** — la liste des fonctions (cuisine, dortoir, armurerie, réfectoire, réserve,
  forge, salle de garde, prison, chapelle, bibliothèque… à arrêter avec Flo et Sayanel) ; pour chacune : le mobilier
  attendu, les sprites de Veloren qui le font déjà, ceux qui manquent. Dans `salles.xlsx`.
  *Fin* : le tableau est validé et donne la liste des modèles à créer (F.13).
- [ ] **F.11 Une fonction par pièce** — chaque salle porte `@fonction:` ; deux ou trois plans par fonction, décorés
  avec ce qui existe ; le tirage varie les fonctions (pas deux fois la même à la suite, celles d'un thème ensemble).
  *Fin* : l'exemple de contrôle montre, pour dix graines, au moins cinq fonctions différentes par donjon.
- [ ] **F.12 Variations d'une pièce** — à partir d'un plan dessiné : groupes de mobilier tirés par la graine (présents
  ou non), meubles interchangeables, état de la pièce (rangée, saccagée, abandonnée), en plus du miroir et de la
  rotation déjà faits.
  *Fin* : la même cuisine tirée cinq fois donne cinq salles différentes dans l'exemple de contrôle.
- [ ] **F.13 Modèles voxel du donjon** — les modèles manquants de F.10, par lots de cinq environ (modèle `.vox`,
  déclaration du sprite, symbole de décor). Qui les dessine : Q60.
  *Fin* : chaque modèle du lot se pose par son symbole et se voit dans le vrai client.

## 2. Grotte : casser pour avancer, miner pour de bon (jalon U, suite)

**Ce que Flo demande** : des grottes séparées par des murs, pour obliger les joueurs à se servir de la destruction ;
des filons de minerai ; beaucoup moins de pépites isolées ; une salle avec beaucoup de minerai et quelques minerais
rares.

**Ce qui existe** : le générateur `icebreaker-caverne` (`PLAN-CAVERNE.md`) relie toutes les zones par un sentier
praticable à pied ; les minerais sont des sprites minables posés un par un au pied des parois (`filons` de
`grottes/grotte.ron`, déjà divisés par 8 en U.6) ; chaque classe a une pioche (K.4). Veloren sait déjà casser la
roche tendre (`WeakRock`) à la pioche et à l'explosion ; la roche ordinaire ne se casse pas. Le todo « Ajouter la
destructibilité » (2026-11-11) commence ici ; les matières et la foreuse restent dans `PLAN-DONJON.md` (E.5, E.6).

- [ ] **U.13 Murs à casser** — une part réglable des liens entre deux zones est bouchée par un mur de roche tendre,
  reconnaissable (couleur, fissures), d'une épaisseur réglable (`formes.ron`) ; le contrôle du sentier compte ces
  murs comme franchissables ; la salle sûre et sa première zone restent ouvertes. Outil : Q59.
  *Fin* : dans l'exemple de contrôle, les zones murées ne s'atteignent qu'en cassant, et toutes s'atteignent en
  cassant ; un mur tombe à la pioche dans le temps réglé.
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
- un **compte à rebours de 20 minutes**, affiché en sablier (forme pas arrêtée) ; ce qui se passe au bout n'est pas
  écrit dans la liste (Q55) ;
- des **zones** qui proposent des types de mission ;
- des **types de mission** : explorer 80 % des pièces ; tuer 80 % des ennemis ; collecter une chose posée exprès pour
  la mission ; battre un **boss**, mission qui se débloque quand on a passé des niveaux d'exploration de la zone ;
- des boss à ajouter.

**Ce qui existe** : une seule mission, tuer le chef (G.7, I.10), avec ses paramètres par emplacement (G.6) ; au
panneau Départ on choisit un lieu, pas une mission. `design/mission.md` décrit déjà « explorer 80 % » et « nettoyer »
pour l'ancien projet. Ce chantier remplace le tableau des contrats de `PLAN-BOUCLES.md` (B.9) : une mission tient le
rôle du contrat. Les patrouilles toutes les 12 minutes (B.4) se discutent avec le sablier (Q55).

- [ ] **B.16 Le sablier** — durée de l'expédition en données (20 min), comptée par le serveur depuis l'arrivée,
  affichée en sablier dans le HUD de donjon chez tous les membres ; annonces à 10, 5 et 1 minute ; au bout : la
  conséquence tranchée en Q55.
  *Fin* : le sablier se vide en 20 minutes chez deux joueurs à la fois, puis la conséquence arrive.
- [ ] **B.17 Explorer et nettoyer** — deux types de mission en plus de « tuer le chef » : explorer 80 % des pièces
  (des zones, dans la grotte), tuer 80 % des ennemis ; parts en données ; bandeau d'avancement (« Explorer : 5/10 »).
  *Fin* : chacun des trois types se gagne au client texte, dans un donjon généré et dans la grotte.
- [ ] **B.18 Collecter** — une chose posée exprès pour la mission, à rapporter en N exemplaires (laquelle : Q56).
  *Fin* : la mission est gagnée au N-ième objet ramassé, pas avant.
- [ ] **B.19 Zones et choix de la mission** — le panneau Départ liste des zones ; chaque zone propose des missions
  (type, lieu) ; réussir une mission monte le **niveau d'exploration** de la zone, gardé dans la campagne. Ce
  qu'est une zone et comment son niveau monte : Q56.
  *Fin* : une mission réussie monte le niveau de sa zone, encore affiché après une relance du serveur.
- [ ] **B.20 Mission de boss** — proposée quand la zone atteint le niveau réglé ; un boss distinct du chef (lesquels :
  Q56 ; « Plus tard » de `PLAN.md` en parlait déjà).
  *Fin* : la mission de boss n'apparaît qu'au niveau voulu, et se gagne en tuant le boss.

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
  chance d'en lâcher une, la durée au sol, les formes et les couleurs possibles. Un premier lot suffit (lequel : Q58).
  *Fin* : le fichier se lit ; un test vérifie que chaque type d'ennemi du lot a ses âmes et qu'il y a assez de couples
  forme et couleur pour toutes.
- [ ] **AM.2 L'âme tombe, puis s'éteint** — à la mort d'un ennemi, tirage dans sa table ; l'âme apparaît au sol,
  blanche si le village ne la connaît pas, avec sa forme et sa couleur sinon ; compte à rebours visible de tous
  (1 minute) ; elle disparaît au bout.
  *Fin* : une âme lâchée disparaît une minute après, au client texte.
- [ ] **AM.3 Lier une âme** — interagir avec l'âme ouvre « Voulez-vous lier cette âme ? » ; oui : elle passe dans un
  emplacement libre du personnage, son effet s'applique, elle reste jusqu'à sa mort et suit sa fiche au registre ;
  sans emplacement libre : refus. Elle ne va jamais dans le sac. Nombre d'emplacements : Q57.
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

Plus tard : la compétence qui permet de stocker des âmes (Q57), les âmes à effet actif si le premier lot n'en a pas.

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

Dans `QUESTIONS.md`, § 8 « Avant le playtest » (Q55 à Q62).
