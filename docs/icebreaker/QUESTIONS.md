# Questions ouvertes et petites décisions

Source unique des questions pour l'équipe (Flo, Sayanel) et des petites décisions prises en route. Règle : une
question nouvelle s'écrit ici (pas dans un plan, qui se contente d'y renvoyer) ; une fois tranchée, elle passe dans
« Tranché » avec la date et l'endroit où la règle est écrite. Une décision importante (choix technique, changement de
cap) mérite une note dans `decisions/` (prochain numéro : 0019). Les numéros Qn ne changent pas, pour pouvoir les citer.

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
- **Q9 La mort en expédition** : sac au sol + blessure (proposé), ou plus dur, le personnage reste mort jusqu'à ce
  qu'on rapporte son corps (`design/village.md`). Le registre commun (R.4) rend la version dure possible dès
  maintenant. Lié à Q28.
- **Q10 L'or des contrats** : à la caisse (proposé, pour pousser à bâtir ensemble) ou partagé entre les membres.
- **Q11 Entretien du village** : coûte-t-il de l'or chaque jour de carte (pression permanente, façon Darkest
  Dungeon) ? *Proposé* : non pour la démo, à essayer après. Variante « tribut du seigneur » : `IDEES.md`, idée 8.
- **Q12 Patrouilles toutes les 12 min** : trop punitif pour une première soirée ? *Proposé* : commencer à 20 min.
- **Q13 Salle d'armes** : bloquer l'arbre de compétences de Veloren hors du village, ou le laisser libre pour la démo.
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

## 4. Village et personnages (jalon R, `design/village.md`)

- **Q26 Engager à la charrette** : gratuit (un fantôme n'a pas d'or ; proposé pour la démo) ou payé par la caisse ?
  *Origine* : `PLAN.md`. *Qui* : Flo.
- **Q27 Places au registre** (proposé : 10) et peut-on renvoyer un personnage ?
- **Q28 Un personnage du registre qui meurt** : attendre B.3 (sac et blessure), ou il quitte le registre ? Lié à Q9.
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
  trois classes en données (R.1, R.6 à R.8). *Qui* : Flo.
- **Q37 Les âmes** (`design/ames.md`) : Sayanel conçoit ; restent à définir le niveau d'âme (comment il monte) et les
  couleurs redistribuées à chaque partie.
- **Q38 Les idées** (`IDEES.md`) : lesquelles passent en tâches ? *Proposé* : relever un allié (1), curiosités (3),
  dépouilles de chef (4) avec bâtiments débloqués (7), puis tribut (8) et mémorial (10).

## 6. Atelier

- **Q39 Worktrees fusionnés** : `Veloren-classes`, `-combat`, `-interface`, `-menu`, `-mod`, `-serveur`, `-village`
  et la branche `icebreaker-traces` sont à 0 commit devant `master` (2026-11-04). Les supprimer (`git worktree
  remove`, libère un `target/` chacun) ou les garder pour la suite de leur piste ? *Qui* : Flo.

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
