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

- **Un processus serveur par instance** (`0005`) : village, donjon, route. Chacun génère son petit monde au
  lancement (graine + type de lieu + paramètres).
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

## Jalon 2 — Le village (première version)

- [x] **V.1 Instance village** — `icebreaker.ps1 village` (port 14100, `-Site Town` par défaut, ou CliffTown,
  DesertCity, SavannahTown, CoastalTown) : une seule ville au centre du petit monde, sans autre site ni faune ;
  apparition sur sa place principale ; zone fermée autour (I.2). Habitants : gardes, marchands, aventuriers, chats
  et chiens. Vérifié : génération, apparition sur la place, connexion, horloge reprise ; pas encore promené en jeu. — 5237f6c353
  Correctif (premier essai en jeu : place vide) : ville en taille maximale (`-TailleVille`, 1.0) et emplacement
  choisi parmi plusieurs essais (près du centre, sinon ailleurs) selon le nombre de parcelles construites ; 15
  graines essayées, toutes entre 131 et 170 parcelles.
- [ ] **V.2 Entrepôt commun** — un coffre du village dont le contenu est partagé et sauvegardé dans la campagne.
  *Fin* : un joueur y dépose un objet, un autre le retrouve après redémarrage.
- [ ] **V.3 Caisse commune** — chaque joueur garde son or (règle de butin) et peut en verser dans la caisse.
  *Fin* : versement visible par tous, gardé au redémarrage.
- [ ] **V.4 Bâtiments par dialogue** — un PNJ par bâtiment, ses choix ouvrent l'action (Veloren a déjà des dialogues
  à questions et réponses) : intendance (achat et vente, avec le commerce de Veloren), infirmerie (soin payant).
  *Fin* : on achète une potion à l'intendant et on se fait soigner à l'infirmerie.
- [ ] **V.6 Vue du village de dessus** — au village, une touche passe en vue de dessus (caméra fixe, façon 2D) sur
  le vrai village ; un clic sur un bâtiment ouvre son panneau (niveau, stocks, actions). Le serveur envoie la liste des
  bâtiments et leur emprise.
  *Fin* : en vue de dessus, cliquer sur l'entrepôt affiche son contenu.
- [ ] **V.5 Retour au village** — commande `/rentrer` en instance : bilan dans le chat, le joueur repart vers le
  village (reconnexion à la main pour l'instant).
  *Fin* : donjon → `/rentrer` → village, avec le butin.

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

## Jalon 4 — Démo à l'équipe

- [ ] **D.1 Soirée de test** : village → donjon → village → route → village, à deux ou plus, puis décision du moteur
  avec Sayanel (`0015`).

## Plus tard (à découper le moment venu)

- Passage automatique du client d'une instance à l'autre.
- Notre générateur de donjon à partir des briques de Veloren (région, saison, menace, objectif).
- Biome imposé pour terracotta, myrmidons et sahagins.
- IA : alerte, embuscades, fuite et appel à l'aide (`design/creatures.md`, `design/menace.md`).
- Classes (`decisions/0010-classes-et-combos.md`) : capacités en fichiers de données.
- Recrues, traits, blessures, rang, registre commun (`design/village.md`).
- Carte stratégique façon Mount & Blade sur un grand monde Veloren, rencontres qui lancent les instances.

## Décisions (Flo, 2026-10-28)

1. **Personnages** : pour le prototype, chaque compte garde ses propres personnages (modèle de Veloren) ; le
   registre commun viendra plus tard.
2. **Village** : une vraie ville Veloren où l'on marche, **plus une vue de dessus façon 2D** où l'on clique sur les
   bâtiments pour voir les stocks et agir (V.6).
3. **Butin du convoi** : chacun garde ce qu'il ramène (règle de butin habituelle).

## Questions ouvertes

(aucune pour l'instant)
