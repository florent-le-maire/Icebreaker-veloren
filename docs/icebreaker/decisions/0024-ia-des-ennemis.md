# 0024 — Une IA à nous pour les six espèces, branchée en un point de Veloren

- **Date** : 2026-11-26
- **Statut** : plan validé par Flo le 2026-11-26 ; à relire par Sayanel (questions Q69 à Q73).
- **Tâches** : jalon IA, `PLAN-IA.md`. Reprend G.11 à G.16 et Y.4 à Y.8 de `PLAN.md`.

## Problème

Nos ennemis (humain, orc, nain, elfe, draugr, danari) décident avec l'arbre de comportement de Veloren, retouché à
trois endroits (Y.1 à Y.3, Y.9 : environ 90 lignes dans `choose_target`, `do_combat` et `terrain.rs`). Chaque salle
est un combat isolé, et la lecture du code (2026-11-26) montre pourquoi :

- un coup de mêlée fait un bruit de volume 2 (6 blocs, jamais assez fort pour qu'on vienne voir), une mort n'en fait
  aucun, et un ennemi en combat n'écoute rien (`handle_sounds_heard` n'est appelé qu'au repos) ;
- les cris passent par le contrôleur avec `time: 1.0` au lieu de l'heure : ils sont oubliés aussitôt
  (`common/systems/src/controller.rs`) ;
- `stop_pursuing` ne joue pas pour un humanoïde (pas de `patrol_origin`, et notre reciblage remet le chrono à zéro) :
  l'abandon ne vient que de la jauge de conscience, après environ 18 s, pendant lesquelles l'ennemi vise la vraie
  position du joueur, à travers les murs ;
- quand il lâche, il oublie tout : pas de recherche ;
- l'habileté ne règle que le rang des compétences ; le bâton n'utilise que 3 sorts sur 8.

Les identités de race ne sont pas figées (Q50) : la base doit supporter beaucoup de changements, et suivre les mises à
jour de Veloren.

## Options

1. **Retoucher Veloren sur place**, comme Y.2 et Y.9 : un patch par règle dans l'agent et dans `attack.rs` (8 690
   lignes). Peu cher au début ; les conflits de fusion et la logique non testable grandissent avec chaque règle.
2. **Un cerveau à nous, branché en un point** : un nœud dans `BehaviorTree::root()` prend la main pour nos six
   espèces ; la décision est une fonction pure (`tick(observation) → intention`) dans un crate sans dépendance à
   Veloren ; Veloren reste l'exécutant (chemins, visée, déclenchement des compétences).
3. **Notre propre système d'agents** : on retire `Agent` à nos ennemis. Aucun conflit, mais il faut réécrire chemins,
   réflexes et boîte de réception avant le premier résultat visible.

## Décision

Option 2, en mode « étrangleur » : le cerveau possède l'état et la cible dès la première tâche, et délègue à Veloren
ce qui n'est pas encore remplacé (errance du repos, manière de frapper).

- **Crate `icebreaker-ia`** (pur) : perception, états (repos, enquête, combat, fuite), événements, réactions (un
  fichier par réaction, inscrites dans un registre), tirage pondéré à l'événement, choix de cible (les fonctions de
  `icebreaker_races.rs`, déplacées avec leurs tests), habileté, et un mini-monde de test pour jouer des scènes.
- **Profils en données** (`assets/icebreaker/ia/`) : `commun` + race + rôle + cran, fusionnés champ par champ. Un
  profil dit, par événement, quelles réactions sont tirées et avec quels poids. Pas de rôle fixe dans un groupe.
- **Branchement** : un champ `icebreaker_ia` dans `Agent`, une ligne dans `root()`, un adaptateur
  (`server/src/sys/agent/icebreaker_ia.rs`) qui construit l'observation et exécute l'intention.
- **Entre ennemis, des boîtes** : le système d'agents tourne en parallèle. Un cerveau écrit des faits dans sa boîte de
  sortie ; un système séquentiel (`icebreaker_ia_monde`) les livre aux cerveaux à portée, produit les bruits de combat
  et de mort, et tiendra la mémoire du lieu. Les ordres (rondes, plus tard le maître du jeu) arrivent par la même
  boîte d'entrée qu'un message d'allié ; le type d'un ordre ne contient ni joueur ni position de joueur.
- **`attack.rs`** : gardé intact pour tout ce qui est hors périmètre ; **enveloppé** pour l'épée, l'arc et le bâton
  du chef (le cerveau décide quand appeler `attack()` et filtre les entrées du contrôleur selon le cran) ;
  **remplacé** pour le bâton de nos mages par une table de sorts en données, puis pour les autres armes seulement si
  l'enveloppe ne suffit pas.

## Conséquences

- Les lignes à nous dans les fichiers de Veloren baissent : environ 5 ajoutées, les 90 de Y.2 et Y.9 retirées.
- Une mise à jour de Veloren ne crée pas de conflit dans l'agent ; un changement de son API casse la compilation dans
  le seul adaptateur. Un handler enveloppé peut changer de comportement sans prévenir : les scènes du lieu
  d'entraînement servent à le voir.
- Le cerveau ne reçoit que ce qu'il perçoit : la position d'un joueur non perçu n'est pas dans son observation. Le
  principe « les créatures ne trichent pas » de `design/menace.md` devient une propriété testée.
- Changer une identité de race, c'est changer du RON ; au pire écrire un fichier de réaction. Le tronc ne bouge pas.
- La difficulté 3 n'est plus « le jeu d'avant » dès que la perception change : le classeur d'équilibrage ne mesure
  pas l'IA, il faut un essai au lieu d'entraînement par tâche.
- Les murs n'arrêtent aucun son tant que les salles ne sont pas exportées au serveur (IA.13).
- C'est un jalon entier (quinze tâches). Les règles de `design/creatures.md` écrites pour le prototype Godot qui ne
  se transposent pas sont listées dans ce fichier, section « Pour Veloren ».
