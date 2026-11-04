# Icebreaker en mod Veloren — plan

Proposition (2026-10-28) pour passer d'une **copie modifiée** de Veloren à un **mod** : la logique Icebreaker dans un
plugin et des fichiers de données, la copie de Veloren réduite au strict nécessaire. À valider avec l'équipe ; rien
n'est codé. À lancer **après la démo D.1** et seulement si l'équipe garde Veloren (`decisions/0015`).

## Pourquoi

- Suivre les mises à jour de Veloren sans conflits : aujourd'hui 17 commits touchent ~25 fichiers du moteur
  (`server`, `world`, `rtsim`, `voxygen`).
- Un mod se distribue seul : le serveur envoie ses plugins aux clients à la connexion (cache `server-plugins`), les
  joueurs n'ont rien à installer en plus.
- La licence des fichiers d'interface du plugin (`plugin/wit/LICENSE`) est double : à vérifier si elle laisse le
  plugin sous une autre licence que la GPL (question 3).

## Ce que Veloren permet déjà sans toucher au moteur

| Moyen | Ce qu'on peut y mettre |
|---|---|
| **Plugin WASM** (`assets/plugins/*.plugin.tar`, interface `plugin/wit/veloren.wit`, v0.0.1) | au chargement ; arrivée d'un joueur (accepter ou refuser) ; **commandes de chat** à nous ; message à un joueur ; lire le nom et les PV d'une entité ; squelettes et animations de nouveaux corps |
| **Données du plugin** (fusionnées avec celles du jeu) | objets, recettes, traductions des objets, corps de plugin (`plugin_bodies.ron`) |
| **Surcharge d'assets** (`VELOREN_ASSETS_OVERRIDE`) | remplace n'importe quel fichier d'`assets/` : PNJ et leurs équipements (`common/entity`, `loadout`), butin (`loot_tables`), capacités (`abilities`), commerce (`trading`), traductions (`voxygen/i18n`), donjons et sites en données (`world/`) |
| **Réglages du serveur** (`settings.ron`) | `day_length`, taille du monde (`map_file` → `x_lg`, `y_lg`), graine, `battle_mode`, taille du groupe, dossier de données |
| **Réglages du client** (`VELOREN_USERDATA`) | un `settings.ron` fourni par le lanceur (touches AZERTY) |

**Ce que l'interface du plugin ne permet pas** : génération du monde, `rtsim`, apparition des PNJ, horloge du jeu,
lire ou changer une position, inventaires et coffres, commerce, dialogues, mort d'une entité, tâche à chaque tick,
fichiers sur disque, et rien côté client hors animations (pas d'interface, pas de caméra).

**Conclusion** : un mod sur le Veloren officiel ne garderait que des commandes de chat (`/rentrer`, bilan) et des
données — environ 15 % du prototype. Pour garder **le maximum**, on passe par trois couches et on **étend
l'interface du plugin** dans notre copie, de façon générique (utile à tout mod, pas seulement au nôtre) pour pouvoir
la proposer un jour à Veloren (question 2).

## Les trois couches visées

1. **Données** (`icebreaker-assets/`, monté par `VELOREN_ASSETS_OVERRIDE` et dans le `.plugin.tar`) : réglages
   (rayons, temps, prix), PNJ, butins, objets, traductions, touches.
2. **Plugin `icebreaker`** (Rust compilé en WASM, `plugin/icebreaker/`) : toute la règle du jeu — traces, zone,
   horloge, campagne, entrepôt, caisse, bâtiments, retour, convoi.
3. **Moteur** (notre copie de Veloren) : seulement **l'interface du plugin élargie** (crochets génériques) et ce qui
   ne peut pas passer par un plugin (génération du monde, vue de dessus du client), chacun derrière un réglage.

## Où va chaque fonctionnalité

| Fonctionnalité | Aujourd'hui | Cible | Ce qu'il manque à l'interface |
|---|---|---|---|
| Traces de donjon | `rtsim` + serveur | plugin | mort d'une entité, ouverture d'un coffre, **filtre d'apparition** des PNJ, stockage du plugin |
| Instance à un seul site (I.1 sites, V.1) | `world/civ` | moteur, en **réglage générique** (`world.sites` : liste de sites imposés, taille, emplacement) | — (la génération reste en Rust natif : trop lourde pour du WASM) |
| Pas de faune (I.1) | `terrain`, `rtsim` | réglage générique (`world.wildlife = false`) ou filtre d'apparition | filtre d'apparition |
| Zone fermée (I.2) | système serveur | plugin | **tick** du plugin, position d'une entité, téléporter |
| Personnages communs (I.3) | dossier de campagne | réglage du dossier de base (déjà là) ; waypoint ignoré → plugin | arrivée dans le monde, changer le waypoint |
| Lanceur (I.4) | `icebreaker.ps1` | inchangé (il écrit les réglages et pose le plugin) | — |
| Horloge de campagne (I.5) | système serveur | plugin | lire et régler l'heure, départ d'un joueur, tick, stockage |
| Touches AZERTY, titre | code client | données (`settings.ron` fourni, traduction) | — |
| Entrepôt commun (V.2) | en cours dans le moteur | plugin | inventaire d'une entité (lire, ajouter, retirer), interaction avec un PNJ ou un coffre |
| Caisse commune (V.3) | — | plugin (commande `/caisse` d'abord) | inventaire (or) |
| Bâtiments par dialogue (V.4) | — | plugin + données | ouvrir un dialogue à choix, réponse du joueur, ouvrir le commerce d'un PNJ, soigner |
| Vue de dessus (V.6) | — | **moteur client**, isolé derrière un réglage ; le plugin envoie la liste des bâtiments | message de données du plugin vers le client |
| Retour au village (V.5) | — | plugin (déjà possible : commande + bilan dans le chat) | — pour la version à la main |
| Instance route (P.1) | — | réglage générique (`world.sites` avec une route) | — |
| Convoi (P.2, P.3) | — | plugin + données (PNJ) | **faire apparaître** un PNJ depuis un fichier d'entité, lui donner un but (aller à, suivre, fuir), alignement |
| Butin du convoi (P.4) | — | plugin + `loot_tables` | inventaire, mort d'une entité |

## Interface du plugin à ajouter (`veloren.wit` v0.1)

Regroupée en paquets ; chacun ouvre plusieurs fonctionnalités.

- **E1 Événements** : `tick(dt)` (avec une fréquence demandée par le plugin), `player-left`, `entity-died(uid,
  attaquant)`, `sprite-interacted(joueur, position, type)` (coffres), `npc-interacted(joueur, pnj)`.
- **E2 Entités** : position, téléporter, alignement, inventaire (lister, ajouter, retirer, or), soigner.
- **E3 Monde** : lire et régler l'heure du jour ; filtre d'apparition (`should-spawn(entité, position) -> bool`) ;
  faire apparaître un PNJ depuis un fichier `common/entity` ; but d'un PNJ (aller à, suivre, fuir).
- **E4 Stockage** : un dossier réservé au plugin, partageable entre processus (le dossier de campagne), lire et
  écrire des octets. C'est ce qui remplace nos sauvegardes dans `rtsim` et dans `campagne/`.
- **E5 Dialogues et commerce** : ouvrir un dialogue à choix, recevoir la réponse, ouvrir le commerce d'un PNJ.
- **E6 Client** : message de données du plugin vers le client (pour la vue de dessus) — seulement si V.6 est gardé.
- **Échappatoire** : `run-command(commande, args)` en tant que serveur. Donne d'un coup `/time`, `/tp`, `/give_item`,
  `/spawn`… Utile pour avancer vite ; à remplacer par des appels propres quand ils existent.

## Tâches

Même méthode que `PLAN.md` : une tâche = quelques heures, un critère de fin, un commit, le jeu lançable après chaque
tâche. On bascule une fonctionnalité à la fois : la version moteur reste jusqu'à ce que la version plugin marche,
puis on la retire.

### Jalon M1 — Socle du mod

- [x] **M.1 Plugin vide chargé** — `plugin/icebreaker/` construit par `icebreaker.ps1 mod` ; serveur compilé avec
  `--features plugins`. Vérifié au client texte : chargé au démarrage, `/icebreaker` répond. Pas vérifié : le
  téléchargement du plugin par voxygen (le client texte ne l'installe pas). — 5cf1352c2b
- [ ] **M.2 Données à part** — `icebreaker-assets/` monté par `VELOREN_ASSETS_OVERRIDE` dans le lanceur ; on y
  déplace le titre (traduction) et les touches AZERTY (`settings.ron` client fourni) ; on retire le code client
  correspondant. *Fin* : titre et touches inchangés, `voxygen` revenu à l'identique de Veloren pour ces points.
- [ ] **M.3 Retour au village par le plugin (V.5)** — `/rentrer` et le bilan dans le chat, sans toucher au moteur.
  *Fin* : comme V.5, avec zéro ligne de moteur.

### Jalon M2 — Interface élargie (moteur)

- [ ] **M.4 Tick, départ, position, téléporter (E1, E2 partiel)**. *Fin* : un plugin de test renvoie au centre un
  joueur qui s'éloigne.
- [ ] **M.5 Stockage du plugin (E4)**. *Fin* : une valeur écrite par une instance est relue par une autre.
- [ ] **M.6 Heure du jour (E3 partiel)**. *Fin* : le plugin de test avance l'heure d'une heure.
- [ ] **M.7 Mort, coffres, filtre d'apparition (E1, E3)**. *Fin* : le plugin de test empêche une créature
  d'apparaître et voit un coffre s'ouvrir.

### Jalon M3 — Bascule des fonctionnalités faites

- [ ] **M.8 Zone fermée dans le plugin** (I.2) ; retrait de `server/src/sys/icebreaker_zone.rs`.
- [ ] **M.9 Horloge dans le plugin** (I.5) ; retrait de `server/src/sys/icebreaker_clock.rs`.
- [ ] **M.10 Traces de donjon dans le plugin** ; retrait de `rtsim/src/data/dungeon_trace.rs` et des crochets
  dans `inventory_manip`, `entity_manipulation`, `terrain`. Les traces passent dans le dossier de campagne (cohérent
  avec l'architecture visée).
- [ ] **M.11 Génération en réglages génériques** — `world.sites` (sites imposés, taille, emplacement) et
  `world.wildlife` remplacent `ICEBREAKER_INSTANCE_SITE` et `world/src/icebreaker.rs` ; même résultat pour donjon et
  village.

*Fin du jalon* : `icebreaker.ps1 village` et `donjon` donnent le même jeu qu'avant, et `git diff master` ne montre
plus que l'interface du plugin et les réglages génériques.

### Jalon M4 — Nouvelles fonctionnalités directement en mod

- [ ] **M.12 Inventaires (E2)** puis **entrepôt (V.2)** et **caisse (V.3)** dans le plugin (le travail V.2 en cours
  dans le moteur est repris ici s'il n'est pas fini avant).
- [ ] **M.13 Dialogues et commerce (E5)** puis **bâtiments (V.4)**.
- [ ] **M.14 Apparition et buts des PNJ (E3)** puis **convoi (P.1 à P.4)**.
- [ ] **M.15 Vue de dessus (V.6)** : client à part, derrière un réglage, nourri par le plugin (E6).

## Ce qui restera dans le moteur

- L'interface du plugin élargie (E1 à E6) — candidate à une proposition à Veloren.
- Les réglages génériques de génération (`world.sites`, `world.wildlife`) — candidats aussi.
- La vue de dessus du client (V.6), tant que Veloren n'a pas de plugins d'interface.
- Plus tard : notre générateur de donjon. En WASM il serait trop lent ; il pourra devenir un **format de données**
  (salles, briques) lu par un générateur natif générique.

## Risques

- **Interface instable** : `veloren.wit` est en 0.0.1 et peu utilisée ; Veloren peut la changer. Garder nos ajouts
  dans un fichier `icebreaker.wit` séparé quand c'est possible.
- **Coût du tick en WASM** : quelques joueurs par instance, quelques dizaines de PNJ, ça doit tenir ; à mesurer en
  M.4 (critère : moins de 1 ms par tick).
- **Proposition à Veloren incertaine** : si elle est refusée, l'interface élargie reste un petit écart à maintenir,
  toujours bien plus petit qu'aujourd'hui.
- **Effort** : environ 15 tâches avant de retrouver le jeu actuel plus les jalons 2 et 3 ; ne pas le lancer avant
  la décision du moteur.

## Questions ouvertes

Dans `QUESTIONS.md`, § 1 « Stratégie et démo » (Q3 à Q6).
