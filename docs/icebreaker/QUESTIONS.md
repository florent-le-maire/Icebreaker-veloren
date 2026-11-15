# Questions ouvertes et petites décisions

Source unique des questions pour l'équipe (Flo, Sayanel) et des petites décisions prises en route. Règle : une
question nouvelle s'écrit ici (pas dans un plan, qui se contente d'y renvoyer) ; une fois tranchée, elle passe dans
« Tranché » avec la date et l'endroit où la règle est écrite. Une décision importante (choix technique, changement de
cap) mérite une note dans `decisions/` (prochain numéro : 0021). Les numéros Qn ne changent pas, pour pouvoir les citer.

Chaque question : d'où elle vient, qui tranche, ce qui est proposé en attendant.

## 1. Stratégie et démo

- **Q1 Contenu et date de la démo D.1.** Le scénario (village → donjon → village → route → village) suppose le
  pillage sur la route (P.1 à P.4, pas commencé) ; S.2 et S.3 (VPS, client pour les potes) ne sont pas faits non plus.
  *Origine* : `PLAN.md`, jalon 4. *Qui* : Flo et Sayanel. *Proposé* : une démo sans route (village, donjon Gnarling,
  donjon généré, grotte), après une session de test dans le vrai client ; la route après, si Veloren est gardé.
- **Q2 Le moteur après la démo** : Veloren ou Godot (`decisions/0015`). *Qui* : Flo et Sayanel, après D.1.
- **Q3 Quand passer en mod** (`PLAN-MOD.md`, M.2 et suivantes). *Qui* : équipe. *Proposé* : après D.1, si Veloren
  est gardé ; sinon au moins faire les nouvelles fonctions directement dans le plugin pour ne pas grossir l'écart.
- **Q4 Proposer l'interface de plugin élargie à Veloren ?** Rien ne part vers `veloren` aujourd'hui ; une proposition
  passerait par une demande de fusion sur leur GitLab. *Origine* : `PLAN-MOD.md`. *Qui* : équipe.
- **Q5 Licence du plugin** : GPL comme le reste, ou autre si la double licence de `plugin/wit/` le permet.
  *Origine* : `PLAN-MOD.md`. *Qui* : équipe.
- **Q6 Garder la vue de dessus (V.6)** malgré le code client qu'elle impose au mod ? *Origine* : `PLAN-MOD.md`.
  *Proposé* : oui, elle est au cœur du village (`0018`).
- **Q7 Le prochain jalon.** En attente : B (boucles, proposition), E (mécaniques du donjon, proposition), P (route),
  C.4 à C.6, IA G.11 à G.16, F.7. *Qui* : Flo. *Proposé* : (1) session de test dans le vrai client sur la liste
  « À vérifier en jeu » de `PLAN.md` ; (2) bloc 1 des boucles, B.1 à B.4, ce qui fait peser l'expédition ; (3) S.2 et
  S.3 pour jouer à plusieurs ; la route et l'IA ensuite.

## 2. Boucle de jeu (`PLAN-BOUCLES.md`)

- **Q8 Valider le plan des boucles** (règles chiffrées, bâtiments à niveaux, contrats, 13 tâches). *Qui* : équipe.
  Il remplace les maisons de C.3 par des bâtiments nommés (B.8) : à trancher avant C.5 et C.6.
- **Q9** *(numéro réservé : « la mort en expédition » est tranchée, voir plus bas, 2026-11-04 et 2026-11-05).*
- **Q10 L'or des contrats** : à la caisse (proposé, pour pousser à bâtir ensemble) ou partagé entre les membres.
- **Q11 Entretien du village** : coûte-t-il de l'or chaque jour de carte (pression permanente, façon Darkest
  Dungeon) ? *Proposé* : non pour la démo, à essayer après. Variante « tribut du seigneur » : `IDEES.md`, idée 8.
- **Q12 Patrouilles toutes les 12 min** : trop punitif pour une première soirée ? *Proposé* : commencer à 20 min.
- **Q13** *(numéro réservé : « salle d'armes et arbre de compétences » est tranchée, voir plus bas, 2026-11-11).*
- **Q14** *(numéro réservé : la question « PV entre deux lieux » est tranchée, voir plus bas, 2026-10-29).*

## 3. Donjon généré et grotte (`PLAN-DONJON.md`, `PLAN-GROTTE.md`, jalon G)

- **Q15 Le gouffre** (donjons avancés) : que coûte la remontée, et est-ce une seule mission très profonde ou une
  suite de descentes qu'on reprend où on s'était arrêté ? *Origine* : `PLAN.md`. *Qui* : Flo.
- **Q16 Ressources du donjon à la sortie** (éclat, huile, corde, champignons, planches) : perdues (proposé : on les
  trouve sur place), ou gardées à l'entrepôt ? *Origine* : `PLAN-DONJON.md`.
- **Q17 Escalade de Veloren** : la couper en donjon (proposé, sinon les trous ne comptent pas) ? Et au village ?
- **Q18 Inventaire** : commencer par la capacité simple (E.3) et ne faire la grille à formes (E.16) que si la
  contrainte plaît ? *Proposé* : oui.
- **Q19 Foreuse** : outil pour tous, ou réservé à une classe (ingénieur, `decisions/0010`) ?
- **Q20 Monstres à coop pour la démo** : lesquels ? *Proposé* : porte-bouclier, ombre, chaman (les moins chers).
- **Q21 Ambiances de la grotte** : les trois (faites en K.3, en données) ou une seule pour la démo ? *Origine* :
  `PLAN-GROTTE.md`. *Proposé* : garder les trois, elles ne coûtent rien de plus.
- **Q22 Lumière de la grotte** : sombre où la lanterne compte (`design/donjon.md`, torche), ou assez de roche
  luisante pour jouer sans ? *Proposé* : lisible sans lanterne pour la première version ; le premier retour dans le
  vrai client (K.3 à K.5 pas encore vus) dira si les ambiances se reconnaissent.
- **Q23 Message de victoire** : G.7 dit « le chef du donjon est tombé » aussi dans la grotte (texte commun de
  `server/src/icebreaker_mission.rs`). Garder, ou un texte par lieu ?
- **Q24 Bêtes paisibles** (truffler, fungome, chauve-souris) comptées dans le bilan des créatures tuées : les garder
  comme habitants, ou ne compter que les agressives ?
- **Q25 La pioche** en mains secondaires de chaque classe (on change d'arme pour miner) : préférable à une pioche
  dans le sac ? La foreuse (E.6) la remplacera-t-elle ?
- **Q54 Remonter d'une grotte** (jalon U, `PLAN-CAVERNE.md`) : le générateur pose un sentier qu'on peut toujours
  suivre à pied, à l'aller comme au retour (lacets le long des puits) ; falaises et trous sont à côté. Garder ce
  sentier garanti, ou vouloir de vraies chutes sans retour à pied quand les cordes (E.7) existeront ? Lié à Q15.
  *Proposé* : garder le sentier tant qu'il n'y a pas de cordes. *Qui* : Flo, Sayanel.

## 4. Village et personnages (jalon R, `design/village.md`)

- **Q26 Engager à la charrette** : gratuit (un fantôme n'a pas d'or ; proposé pour la démo) ou payé par la caisse ?
  *Origine* : `PLAN.md`. *Qui* : Flo.
- **Q27 Places au registre** (proposé : 10) et peut-on renvoyer un personnage ?
- **Q28** *(numéro réservé : « un personnage du registre qui meurt » est tranchée, voir plus bas, 2026-11-05).*
- **Q29 Enseignes** (X.4) : le sprite `Sign` de Veloren n'a pas de texte ; une enseigne nommée demanderait un asset à
  nous. On s'en tient aux caisses, tonneaux et lanternes, ou on dessine un panneau par bâtiment ?
- **Q30 Recrues devant la charrette** (X.8) : sur le village d'essai, leurs postes tombaient 8 blocs au-dessus de la
  maison (pente, toit voisin ?). À voir en jeu d'abord ; si elles sont mal placées, les mettre sur le rebord des
  fondations comme les enseignes.
- **Q31 Changer de classe** à la salle d'armes : possible (cher) ou jamais (la classe fait l'identité de la recrue) ?
  *Origine* : `design/village.md`.
- **Q32 Paquetage** : combien de places, et les potions se partagent-elles entre joueurs en donjon ? Dans le prototype,
  c'est l'inventaire de Veloren qui joue ce rôle (voir Q18). *Origine* : `design/village.md`.
- **Q33 Le village peut-il être attaqué** quand le monde vit (phase 5) ? Si oui, l'entrepôt et la palissade deviennent
  importants. *Origine* : `design/village.md`.

## 5. Design à valider

- **Q34 Les piliers** (`design/piliers.md`) : à valider par l'équipe, puis à citer dans `CLAUDE.md` pour que chaque
  agent les applique.
- **Q35 La menace** (`design/menace.md`) : proposition ; son palier 1 est G.15 et G.16.
- **Q36 Neuf classes et combos** (`decisions/0010`, `design/classes.md`) : garder ce cap sur Veloren ? Le prototype a
  trois classes, refaites par le jalon A (`decisions/0020` : jeux d'armes, ultime, passif, maîtrise). Restent ouverts :
  les six autres classes et les combos de zones. *Qui* : Flo.
- **Q37 Les âmes** (`design/ames.md`) : Sayanel conçoit ; restent à définir le niveau d'âme (comment il monte) et les
  couleurs redistribuées à chaque partie.
- **Q38 Les idées** (`IDEES.md`) : lesquelles passent en tâches ? *Proposé* : curiosités (3) (relever un allié, 1, est fait : jalon T),
  dépouilles de chef (4) avec bâtiments débloqués (7), puis tribut (8) et mémorial (10).
- **Q40** *(numéro réservé : « l'interface Icebreaker » est tranchée, voir plus bas, 2026-11-11 ; Q39 était déjà
  pris par les worktrees, 2026-11-07).*
- **Q41 Le bilan de retour : chacun le sien, ou toute l'expédition ?** (`PLAN-INTERFACE.md`, N.8) La maquette montre
  « ce que chacun rapporte » ; en jeu chacun rentre seul par `/rentrer` et ne voit que ce qu'il rapporte lui-même,
  les autres n'en ont qu'une ligne dans le chat. *Proposé* : garder ainsi tant que le retour est individuel ; un
  bilan commun demande un retour groupé (toute l'expédition rentre ensemble). *Qui* : Flo et Sayanel.
- **Q42 Mourir au village** (`PLAN-INTERFACE.md`, N.7) : l'écran de mort dit « Mort en expédition » et « perdu pour
  de bon » même au village, où le personnage reste pourtant au registre (vu à l'essai avec `/kill`). Peut-on mourir
  au village en vrai jeu ? Si oui, que dit l'écran, et le personnage est-il perdu ? *Proposé* : non perdu, un texte
  à part. *Qui* : Flo.

## 6. Armes, classes et compétences (`PLAN-CLASSES.md`, jalon A)

- **Q43 Chiffres de départ** : points d'arme par expédition (proposé : 1 à 2), prix d'un jeu d'armes, d'un ultime,
  d'un passif, recharges (compétence 6 à 15 s, ultime 60 à 120 s). Tout est en données (`salle_armes.ron`,
  `jeux_armes/`) ; à régler après les premières parties. Repère (essai du 2026-11-11) : un gnarling tué seul donne 17
  de maîtrise ; le premier point d'arme en demande 100, le suivant 150. Le mage et l'archer n'ont pas été essayés
  contre de vraies cibles : leurs chiffres (dont l'inné du mage, −10 % de PV) sont un premier jet. *Qui* : Flo.
- **Q44 Choix de l'agent à confirmer** : (1) un jeu d'armes non appris ne donne que les attaques de base, sans
  blocage à l'équipement ; (2) la maîtrise se compte par jeu d'armes (grande épée et épée + bouclier séparées) et vient
  de l'expérience des ennemis tués ; (3) chaque classe a un inné non choisi (guerrier : PV et tenue ; mage : énergie ;
  archer : vitesse). *Qui* : Flo.
- **Q45 Ultimes et arme tenue** : un ultime de classe reprend le geste d'une arme de Veloren (la fureur vient de la
  hache, le séisme du marteau). Rendu avec une autre arme en mains à voir en jeu (A.14) ; s'il choque, un geste neutre
  par ultime. *Qui* : Flo, après A.14.
- **Q46 La suite des armes** : bascule entre deux jeux d'armes en combat (façon Guild Wars 2 ; la pioche irait dans le
  sac, voir Q25), et les jeux « deux épées » et « hache + épée » du guerrier (en données ; la hache à une main
  n'existe qu'en arme forgée). *Proposé* : après le jalon A. *Qui* : Flo.
- **Q48 Les compétences blessent les habitants du village.** Vu à l'essai du 2026-11-11 : un séisme lancé sur la
  place touche les recrues debout devant la charrette ; elles ripostent et tuent le personnage suivant dès son
  arrivée. Voisin de Q42 (mourir au village). *Proposé* : pas de dégâts au village hors des mannequins (zone sûre),
  ou des recrues qui n'attaquent jamais. *Qui* : Flo. *2026-11-15* : le Séisme frappe maintenant tout autour et bien
  plus fort (A.20), l'Envol et la Volée mortelle touchent aussi tout ce qui n'est pas du groupe : la question pèse plus.
- **Q47 Maquettes du jalon A** (A.2) : la barre de compétences et les panneaux de la charrette et de la salle d'armes
  (rangée 4 du canvas) conviennent-ils ? L'interface a été codée d'après elles sans attendre (A.6, A.8, A.11,
  A.12) : un avis contraire se corrige dans le code. Points à regarder : le passif en case ronde, posé à gauche de la
  barre (les cases 4 et 5 restent à Veloren), avec la même icône pour tous les passifs ;
  la salle d'armes en quatre onglets (Compétences, Armes, Ultimes, Passifs), un bouton d'un mot par ligne, le
  prix et la description dans un sous-titre sur plusieurs rangs (la liste défile dès trois compétences), deux
  boutons « Touche 1 : changer » et « Touche 2 : changer » en bas (maquette redessinée d'après le vrai client) ;
  le lot d'une recrue sur une seconde ligne. *Qui* : Flo.
- **Q49 Un village sans maison.** Une partie neuve tire la graine de son village au hasard ; celle de l'essai du
  2026-11-11 (459192) a donné une ville sans maison de Veloren : les sept bâtiments sont posés sur la place, à
  quelques blocs les uns des autres, et il n'y a aucun terrain à bâtir (jalon C). *Proposé* : à la création d'une
  partie, retirer une graine tant que la ville n'a pas assez de maisons. *Qui* : Flo.
- **Q50 Une identité par race ennemie (jalon Y).** Proposé (Flo, 2026-11-14, `PLAN.md`, jalon Y) : danari fourbes
  (achèvent les joueurs à terre, fuient blessés ; contre : les frapper ou les repousser), humains organisés (rondes
  entre les salles), orcs brutaux (rage sous la moitié des PV), nains tenaces (tiennent leur salle, plus d'armure),
  elfes tireurs (plus d'archers, voient de loin), draugr qui dorment (réveil au bruit, lents). Ces identités
  conviennent-elles à Sayanel (âmes, `design/ames.md`) ? Les danari tiennent le rôle des gobelins, alors que les
  gnarlings le tenaient jusqu'ici : à confirmer. Pas de vraie provocation en Veloren (le « revers méprisant » du
  marteau protège le guerrier, il n'attire pas) : en faut-il une ? Choix de cible (Y.9) : un danari seul
  n'achève pas (il vise d'abord un joueur debout), seuls des danari à plusieurs achèvent : est-ce bien l'idée ? *Qui* : Flo, Sayanel.

## 7. Lieu d'entraînement (`PLAN-ENTRAINEMENT.md`, jalon O)

- **Q51 Une menace par ennemi ?** Au lieu d'entraînement, un ennemi se choisit par race, rôle et nombre. La
  « menace » d'un donjon ne change aujourd'hui que le nombre de créatures, jamais leurs PV ni leurs dégâts.
  Faut-il une vraie menace par ennemi (PV, dégâts), ici et dans les donjons ? *Proposé* : non pour l'instant.
  *Qui* : Flo, Sayanel.
- **Q52 Garder ses réglages d'entraînement ?** Niveaux et touches repartent des défauts à chaque connexion (la
  classe et l'arme reviennent par l'inventaire). *Proposé* : ne rien garder. *Qui* : Flo.
- **Q53 Quels alliés à l'entraînement ?** Des créatures de Veloren dans le camp du joueur (garde de village,
  loup… liste dans `assets/icebreaker/arene.ron`), qui le suivent. Quelles créatures proposer, et faut-il un jour des
  alliés qui jouent nos classes (une IA pour nos compétences, un jalon à lui seul) ? *Qui* : Flo.

## Tranché

Petites décisions, par date ; les règles qui en sortent sont dans le fichier indiqué.

- **2026-10-19 (donjon, `design/donjon.md`)** : mort = cadavre qu'un allié peut relever ; déconnexion = cadavre ;
  chacun garde son butin ; tir allié activé ; verticalité (étages, fosses, saut).
- **2026-10-22 (village, Flo, `design/village.md`)** : (1) un seul village, commun à tout le serveur (or personnel +
  caisse commune) ; (2) pas de stress façon Darkest Dungeon, les blessures et les traits portent la tension ;
  (3) registre commun, tout le monde peut jouer tous les personnages ; (4) une scène 2D avec des menus pour
  commencer, les personnages qui bougent plus tard.
- **2026-10-28 (Flo, prototype Veloren)** : (1) chaque compte garde ses personnages — *remplacé le 2026-11-03 par
  `decisions/0017` (charrette et registre commun)* ; (2) village = une vraie ville Veloren où l'on marche, plus une
  vue de dessus façon 2D où l'on clique sur les bâtiments (V.6) ; (3) butin du convoi : chacun garde ce qu'il
  ramène ; (4) clés du donjon généré : une clé différente par porte (G.8) ; (5) sommeil : les endormis sont couchés,
  avec des « Zzz » (G.9).
- **2026-10-29 (`decisions/0016`)** : un seul serveur pour la campagne, emplacements de donjon dans le même monde.
  Règle au passage la question « PV entre deux lieux » (ex-question 1 du plan) : plus de reconnexion, les PV suivent
  le personnage ; reste à couper la régénération hors village (B.1, dans Q8).
- **2026-11-03 (Flo et Sayanel, `design/ames.md`)** : les gnarlings tiennent le rôle des gobelins ; le halfelin
  n'existe pas dans Veloren.
- **2026-11-03 (Flo, `PLAN-GROTTE.md`)** : la grotte se gagne comme le donjon, en tuant le chef au fond ; chaque
  classe part avec une pioche de Veloren (K.4).
- **2026-11-03 (`decisions/0017`, `0018`)** : charrette et registre commun ; interface du village robuste d'abord,
  visuel de Veloren.
- **2026-11-04 (Flo, `PLAN.md`, jalon T)** : à terre 30 s au plus (compte à rebours visible), relève à 25 % des PV ;
  mort en expédition : sac au sol avec l'inventaire hors équipement porté, fouillé en 3 s par un allié qui en garde
  le contenu ; le mort rentre au village en fantôme ou regarde un allié sur place (pas de caméra libre) ; quand le
  dernier vivant tombe, tous rentrent (ex-Q9, en partie).
- **2026-11-05 (Flo, `PLAN.md`, T.6)** : un personnage mort en expédition est perdu pour de bon : il quitte le
  registre et il est supprimé (ex-Q9 et Q28 ; pas de blessure ni de corps à rapporter pour l'instant).
- **2026-11-07 (Flo, Q39)** : tous les worktrees fusionnés sont supprimés avec leur branche (donjon, grotte, menu,
  mod, mort, serveur, wiki) ; une piste reprise se recrée depuis `master`.
- **2026-11-11 (Flo, Q40, `decisions/0019`, `PLAN-INTERFACE.md`)** : les maquettes de l'interface conviennent ;
  style proche de Veloren, panneau typé piloté par le serveur et envoyé au changement ; réalisation N.3 à N.9.
- **2026-11-11 (Flo, `decisions/0020`, `design/classes.md`, `PLAN-CLASSES.md`)** : le jeu d'armes en mains donne 2
  compétences d'arme, la classe un ultime (longue recharge) et un passif, sans niveau ; jouer une arme en expédition
  rapporte des points d'arme, dépensés à la salle d'armes pour débloquer ou monter (niveaux 1 à 5, les 3 et 5
  transforment) ; recrues tirées avec un lot au hasard, puis tout s'apprend ; changement d'arme libre en bas ;
  guerrier d'abord, puis mage et archer ; maquettes avant l'interface. Ex-Q13 : l'arbre de compétences de Veloren est
  coupé pour les personnages du registre, on ne dépense qu'à la salle d'armes.
