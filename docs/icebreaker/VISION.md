# Vision du jeu et grandes phases

Document de conception d'origine (sorti de `CLAUDE.md` le 2026-10-20). À lire pour une décision de design ou pour
découper une nouvelle phase. Les règles fixées sont dans `docs/design/`, l'avancement dans `docs/ROADMAP.md`.

## Vision du jeu

### Le donjon
- Temps réel, coop, chaque joueur a une classe.
- Habitants crédibles : les créatures ont une logique (façon de combattre, de se défendre, de vivre ensemble).
- L'objectif de la mission (sauver, piller, nettoyer, explorer) change la façon d'aborder le lieu.
- Jamais deux fois le même : région, saison, état du monde et menace font varier le donjon.
- Le donjon garde des traces : boss tué, passage effondré, créature qui a pris la place d'une autre.
- Environnement modifiable : tout mur peut céder aux bons outils (bombe, foreuse), et derrière il y a ce que le lieu contient (salle, cour, ciel, terre, `docs/decisions/0009`) ; ponts et barricades constructibles à des emplacements prévus. L'anéantissement total (gros sort de mage) remplace le donjon par sa version détruite (un cratère), qui reste sur la carte du monde.

### Le village
- On y revient entre les expéditions ; il grandit avec nous.
- Les expéditions transforment le village (butin, personnes sauvées, pertes).
- Le village transforme les classes (déblocage, entraînement, soins, équipement).
- Boucle : partir → revenir plus riche ou plus abîmé → améliorer le village → repartir plus fort.

### Le monde
- Un monde qui vit sans les joueurs : factions, armées, convois, villages avec leurs propres projets.
- Carte façon Mount & Blade : un petit personnage (à pied ou à cheval) qui voyage et croise ce qui circule.
- Le monde change à plusieurs vitesses : local (heures/jours), régional (semaines), global (saisons/années).
- Les décisions ont des conséquences durables (convoi pillé → ville affamée, faction offensée qui se souvient…).

### Les rencontres
- Une rencontre sur la carte (embuscade, camp, entrée de donjon) ouvre une **scène séparée** jouée en temps réel.
- Elle reflète l'état de la carte au moment de la rencontre (lieu, climat, adversaires et leur état).
- Elle est **isolée** : ce qui passe à côté sur la carte n'y apparaît pas (choix assumé).

### Le temps
- Le monde ne ralentit jamais : **1 minute réelle = 30 minutes sur la carte**, en permanence, y compris pendant les instances.
- Le monde **s'arrête quand tous les joueurs sont déconnectés**.
- Renforts : un joueur à X heures de route sur la carte arrive dans une instance après X × 2 minutes réelles (trajet converti en délai). Si l'instance est finie avant, il arrive trop tard.

| Élément | Durée carte | Durée réelle |
|---|---|---|
| Vitesse de base | 30 min | 1 min |
| Une journée | 24 h | 48 min |
| Donjon classique | 15 à 30 h | 30 à 60 min |
| Gros donjon | ~2 jours | 1 h 30 |
| Construction au village | 6 h à 3 jours | 12 min à 2 h 30 |
| Une saison | 15 jours | 12 h |
| Une année (4 saisons) | 60 jours | ~48 h |

Toutes ces valeurs sont des **paramètres de configuration**, jamais codées en dur : elles seront ajustées en testant.

### Jouer ensemble
Chaque joueur choisit son activité en même temps que les autres : intendance du village (gestion), pillage sur les routes (rencontres), expédition en donjon. Les activités se répondent (le butin nourrit le village, l'intendant prépare les sorties).

### Les personnages
- Recrues aléatoires et faibles (façon Darkest Dungeon), qui progressent en compétences et équipement.
- 1 ou 2 traits de personnalité par recrue (peureux, robuste, cleptomane…).
- Mort → cadavre. Trois options : résurrection coûteuse (ramener le corps, bon bâtiment, séquelle) ; revenir en mort-vivant sous la tutelle d'un nécromancien (avec inconvénients, ex. rejet par certaines factions) ; prendre un autre personnage.

## Grandes phases


Chaque phase se termine par une version jouable et une **démo à l'équipe**. Ne passe pas à la phase suivante sans validation.

### Phase 0 — Fondations
- Créer la structure du dépôt, la solution .NET, le projet Godot.
- Rédiger `docs/ROADMAP.md` (découpage détaillé en tâches de cette feuille de route), `docs/ARCHITECTURE.md` et `docs/GAME_DESIGN.md`.
- Mettre en place les tests, un script de build client et serveur headless, un README expliquant comment lancer le jeu.
- **Critère de fin** : un client et un serveur se lancent, deux clients se connectent au serveur et se voient bouger dans une scène vide.

### Phase 1 — Le cœur : un donjon en coop (la phase la plus importante)
- Déplacement 3D vue de dessus, caméra, contrôles.
- 2 classes jouables (ex. guerrier et mage) avec 2-3 capacités chacune.
- 2-3 types de créatures avec une IA simple mais lisible (une qui charge, une qui garde ses distances, une qui fuit et appelle des renforts).
- Un donjon construit à la main avec des modules, un objectif (« tuer le chef »), du butin.
- Mort d'un personnage → cadavre, version simple.
- **Critère de fin** : 2 à 4 amis jouent ensemble 15 minutes et trouvent ça amusant. Si ce n'est pas amusant, on itère ici avant d'aller plus loin.

### Phase 2 — Profondeur du donjon
- Génération procédurale à partir de modules, avec paramètres (région, menace, saison).
- Objectifs de mission différents (sauver, piller, nettoyer, explorer) qui changent la lecture du donjon.
- Destruction systémique (bombe, foreuse : ce qu'il y a derrière le mur dépend du lieu, `docs/decisions/0009`) et emplacements constructibles (ponts, barricades).
- Créatures qui vivent ensemble (groupes, territoires, alarme).
- Persistance des traces : un donjon revisité garde boss tué, passages effondrés, etc.

### Phase 3 — Le village et les personnages
- Village jouable entre deux expéditions : bâtiments, constructions qui prennent du temps de jeu.
- Recrutement de recrues aléatoires avec traits de personnalité.
- Progression (compétences, équipement) et liens village ↔ classes.
- Les trois options après la mort (résurrection, nécromancien, nouveau personnage).
- Sauvegarde complète dans SQLite.
- **Critère de fin** : la boucle partir → revenir → améliorer → repartir fonctionne sur plusieurs soirées.

### Phase 4 — Le temps et la carte du monde
- Horloge du monde côté serveur (1 min réelle = 30 min carte, paramétrable), pause quand plus personne n'est connecté.
- Carte du monde jouable : déplacement à pied/à cheval, lieux (village, donjons), routes.
- Rencontres : entrer dans une instance générée à partir de l'état de la carte, puis réinjecter le résultat dans le monde.
- Plusieurs groupes en parallèle (un au village, un sur les routes, un en donjon).
- Renforts : conversion d'un trajet en délai d'arrivée dans une instance.
- **Décision technique à préparer** (ADR) : comment le serveur fait tourner plusieurs instances en même temps (mondes séparés dans un même processus ou un processus par instance). Proposer, prototyper, puis faire valider.

### Phase 5 — Le monde vivant
- `WorldSim` : factions, villages, convois, armées avec des objectifs simples.
- Trois vitesses : local (heures/jours), régional (semaines), global (saisons/années).
- Conséquences des actions des joueurs (convoi pillé → pénurie, faction offensée → mémoire).
- Le donjon anéanti devient un cratère sur la carte.
- Commencer très simple (quelques factions, quelques règles), mesurer, puis enrichir.

### Phase 6 — Contenu et finitions
- Plus de classes, créatures, bâtiments, traits, régions.
- Équilibrage des valeurs de temps et de progression.
- Sons, effets, interface, menus, tutoriel.
