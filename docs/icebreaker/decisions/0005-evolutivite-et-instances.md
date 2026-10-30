# 0005 — Évolutivité de l'architecture et instances en parallèle

- **Date** : 2026-10-20
- **Statut** : accepté le 2026-10-20 (l'équipe suit les recommandations : pas d'ECS, option 2 pour les instances)

## Problème

Le projet doit tenir jusqu'à la version finale (phases 4 à 6 : plusieurs groupes en parallèle, carte du monde,
`WorldSim`, village persistant). On a relu le code à la fin de la piste A.4 pour vérifier que la construction actuelle
le permet, et pour répondre à une question de l'équipe : **faut-il passer à une architecture Entité-Composant-Système
(ECS) ?**

## Constat

### Ce qui tient

- **Logique en C# pur, séparée de Godot** : `Combatant`, `CreatureBrain`, `CreatureBehaviors`, `AttackChain`,
  la génération des donjons, les catalogues… vivent dans `src/Gameplay`, sans Godot, avec ~35 fichiers de tests.
  Les nœuds Godot ne font que déplacer, afficher et relayer le réseau. C'est la propriété la plus importante pour durer.
- **Contenu en données** (armes, classes, créatures, salles, butin) : la phase 6 (beaucoup de contenu) ne demandera
  pas de toucher au code.
- **Serveur autoritaire** qui vérifie l'expéditeur de chaque RPC.
- **Génération déterministe par graine** : utile plus tard pour `WorldSim` et la sauvegarde.

### Ce qui ne tiendra pas en l'état

1. **Singletons statiques** : `CombatManager.Instance`, `DungeonBuilder.Current`, le compteur statique
   `Creature._nextId` (environ 100 usages dans `game/scripts`). Deux donjons dans un même processus se mélangeraient.
   Le réseau n'a pas non plus de filtrage de visibilité : tous les clients reçoivent tout. Les catalogues en lecture
   seule de `GameConfigFile` (`Combat`, `Creatures`, `Loot`) ne posent pas de problème.
2. **Le personnage est lié à la connexion** : un joueur est identifié par son `PeerId` (identifiant réseau) et son
   butin vit sur le nœud (`Player.Loot`). Les recrues, traits, équipement et la mort (phase 3) demandent un personnage
   persistant, indépendant de la connexion.
3. **Fichiers qui grossissent trop** : `Player.cs` (~1 200 lignes) mélange entrées du client, logique serveur, RPC et
   affichage ; `CombatManager` (~550) et `Creature` (~450) prennent le même chemin. Ça ralentit la lecture et, avec
   plusieurs personnes (ou plusieurs sessions de Claude) en parallèle, c'est la première source de conflits Git.
4. `WorldSim` et `Persistence` sont encore vides : normal à ce stade.

## Question 1 — ECS ?

**Options**

1. **Bibliothèque ECS en C#** (Arch, Friflo, DefaultEcs) pour tout le jeu.
2. **Composition à la manière de Godot** : des nœuds enfants spécialisés et des petites classes C# pures assemblées
   (déjà en partie le cas : `Stamina` et `Poise` optionnels dans `Combatant`, interfaces `ICombatBody` / `IDasher`).
3. **Approche orientée données** (tableaux + systèmes, sans bibliothèque) **seulement dans `WorldSim`**.

**Recommandation : 2 pour le donjon, 3 envisagée pour `WorldSim` en phase 5 après mesure.**

Un ECS sert quand on traite des milliers d'entités semblables. Une instance de donjon compte au plus 10 joueurs et
quelques dizaines de créatures. Godot est construit autour de l'arbre de nœuds : un ECS lutterait contre la physique,
la navigation et `MultiplayerSynchronizer`, pour un gain nul. En revanche `WorldSim` pourra simuler des milliers de
villageois, convois et armées sans affichage : là, des tableaux de données parcourus par des « systèmes » peuvent
valoir le coup. On mesurera avant de choisir.

## Question 2 — Plusieurs instances en parallèle (préparation de la phase 4)

C'est la décision que `CLAUDE.md` demande de préparer pour la phase 4. On l'avance parce qu'elle dit s'il faut retirer
les singletons dès maintenant (chaque semaine en ajoute).

**Options**

1. **Tout dans un seul processus** : chaque instance est une sous-scène avec son propre monde physique
   (`World3D`) et sa navigation ; filtrage réseau par instance (`SceneMultiplayer` / visibilité des
   `MultiplayerSynchronizer`).
   - Pour : un seul programme à déployer, échanges directs entre le monde et les instances.
   - Contre : retirer tous les singletons, isoler physique, navigation et réseau par instance ; un plantage dans un
     donjon arrête tout le monde.
2. **Un processus « monde » + un processus Godot headless par instance**.
   - Processus monde : horloge (1 min réelle = 30 min carte), `WorldSim`, village, carte, SQLite. Il ne s'arrête
     jamais tant qu'un joueur est connecté (cohérent avec « le monde ne ralentit pas pendant les instances »).
   - Processus d'instance : le code de donjon actuel, presque tel quel. Il reçoit au lancement l'état de la rencontre
     (lieu, saison, menace, participants) et renvoie le résultat à la fin (butin, morts, traces du donjon).
   - Les clients changent de serveur en entrant / sortant d'une instance (adresse + jeton donnés par le monde).
   - Pour : les singletons restent valables (un processus = un donjon) ; isolement naturel (réseau, plantages) ;
     coïncide avec le choix « rencontre = scène isolée ».
   - Contre : un protocole entre le monde et les instances (petit : lancement, renforts, résultat) ; mémoire du VPS
     (CAX11, 4 Go) : un serveur Godot headless coûte quelques centaines de Mo, donc quelques instances simultanées
     au plus — à mesurer, puis passer à un VPS plus gros si besoin.

**Recommandation : option 2.** Elle garde le code actuel du donjon, isole les pannes et colle à la vision. Le
processus monde peut être un programme .NET sans Godot (C# pur : `WorldSim`, `Persistence`), ou un Godot headless
sans scène 3D si on veut réutiliser ENet tel quel : à trancher au prototype de la phase 4.

## Conséquences et actions proposées

Pas de grand refactoring maintenant : la phase 1 n'est pas encore validée en session de test. Les actions sont
proposées dans la piste **R** de `docs/roadmap/piste-R-architecture.md` :

- **R.1** (à faire au prochain gros changement de `Player.cs`) : découper `Player` en nœuds enfants (entrées du
  client, contrôle serveur, affichage), même principe ensuite pour `Creature` et `CombatManager`.
- **R.2** (début de la phase 3) : modèle `Character` en C# pur dans `Gameplay` (identifiant propre, classe, traits,
  équipement, inventaire, état de mort) ; le nœud `Player` y fait référence ; `PeerId` ne sert plus qu'au réseau.
- **R.3** (phase 4) : prototype « processus monde + processus d'instance » : lancer une instance, y envoyer deux
  clients, récupérer le résultat.
- Option 2 retenue : les singletons du donjon restent (un processus = un donjon). Les nouveaux ne doivent rien
  supposer de plus (pas d'état du monde ou du village dans un singleton du donjon).
- **ECS** : on n'en met pas. Question rouverte en phase 5, pour `WorldSim` seulement, mesures à l'appui.
