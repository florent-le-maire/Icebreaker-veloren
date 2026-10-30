# Boss

Règles des boss (piste B, [`roadmap/piste-B-boss.md`](../roadmap/piste-B-boss.md)), d'après les boss d'AmbianceurZinzin
([`IMPORT_BOSS_AMBIANCEUR.md`](../IMPORT_BOSS_AMBIANCEUR.md)). Valeurs : `game/data/creatures/bosses.json` et
`creatures.json`.

## Séquence d'actions (B.1)

- Un boss est une créature de comportement `boss` (`creatures.json` : PV, vitesse, cerveau, une frappe au contact)
  plus une entrée de même id dans `bosses.json` : sa **séquence d'actions**, jouée dans l'ordre, en boucle.
- Une action toutes les `actionInterval` secondes, comptées **depuis la fin de la précédente**, et seulement au
  combat. Entre deux actions, il poursuit sa cible et frappe au contact.
- Ses actions n'ont pas de recharge : c'est l'intervalle qui les rythme. Elles partent **même de loin**, et on les
  esquive pendant leur annonce :
  - une zone autour de lui ou un cri partent dès que c'est leur tour ;
  - un coup en arc attend que la cible soit à portée ;
  - un coup visé (ruée, tir) attend que le boss voie sa cible bien en face ; la ruée file sur toute sa longueur.
- Il ne lance rien s'il est étourdi, ni contre une cible à un autre étage (il va la chercher).
- **Seuils de PV** : chacun se déclenche une seule fois, du plus haut au plus bas, même si un gros coup en passe
  plusieurs. Phase connue : `rage` (enchaîne à `rageInterval`).
- Un boss **compte comme un chef** pour la mission (« tuer le chef ») ; la mission « Antre de boss » vient en B.5.

## Punir un boss : récupération, mur, gros corps (B.2)

Valable pour toutes les créatures, pensé pour les boss.
- **Coup en trois temps** : armement (annoncé, immobile), coup, puis **récupération** (`recovery` de la capacité) :
  immobile, elle ne lance rien. C'est la fenêtre où l'on frappe. Pour une ruée, la récupération suit la course.
- **Ruée dans un mur** : si la course finit contre le décor (mur, pilier), pas contre un corps, la créature est
  **étourdie** `wallStun` secondes au lieu de récupérer (« Étourdi » s'affiche). Esquiver une charge en se plaçant
  devant un pilier est la parade attendue.
- L'intervalle d'un boss ne court pas pendant sa récupération ni pendant un étourdissement.
- **Taille du corps** (`bodyRadius`, `bodyHeight` dans `creatures.json`, 0,5 × 2 m par défaut) : collision, zones
  touchées (une zone qui atteint son bord le touche), contact d'une ruée. Plus gros = plus facile à toucher.

## Renforts (B.3)

- Une action de boss de forme `summon` appelle des créatures (`summons` : nombre par sorte) ; les sortes alternent
  (gobelin, kamikaze, gobelin…). Au plus `maxSummoned` renforts **du même boss debout à la fois** : au-delà, l'appel
  ne fait venir personne.
- **Ils entrent par les portes de la salle** du boss (décision du 2026-10-20) : juste derrière chaque porte, à tous les
  étages de la salle (une porte sur la mezzanine compte), la plus proche d'abord, puis à tour de rôle. Une salle sans
  porte (plan de test) les fait venir en cercle autour du boss, comme dans AmbianceurZinzin.
- Ils savent tout de suite qui attaquer : la cible du boss, même s'ils ne la voient pas encore.
- Ils ne lâchent pas de butin (pas de table dans `loot.json`) : on ne farme pas un boss.
- **Gobelin** : faible et rapide, une brute sans charge. **Kamikaze** : fonce et, à moitié du rayon de son explosion,
  allume une mèche de 0,7 s (immobile, zone annoncée) puis explose (3,4 m, 35 dégâts) et meurt. On s'en sort en
  s'éloignant pendant la mèche. Tué, il explose quand même après une mèche de
  0,45 s (zone annoncée), comme dans AmbianceurZinzin (décision de Flo) : on le tue de loin.

## Barre de boss (B.4)

- En haut au centre : nom du boss, barre de PV, et une **consigne** pendant l'annonce d'une action (`hints` de
  `bosses.json`, reprises d'AmbianceurZinzin : « CHARGE : écartez-vous de sa ligne ! »). Sans consigne pour cette
  action : rien, ou « INVULNÉRABLE » quand aucun coup ne le touche.
- Elle montre le boss **le plus proche qui se bat** ; un boss qui dort ou monte la garde ne s'annonce pas. Elle
  disparaît à sa mort.

## Mission « Antre de boss » (B.5)

Décision de l'équipe (2026-10-20) : une mission à part (objectif `bossLair`), à côté de « tuer le chef ».
- **Tirage du boss** (`generation.json` → `bossLairs`) : poids de base × facteur de la région × facteur de la saison,
  parmi les boss dont la fourchette de menace contient celle de la mission. Même graine, même boss. `--boss <id>`
  l'impose.
- **Un antre fait pour lui** : la salle au bout du chemin principal (rôle `bossLair`) est prise parmi celles qui
  déclarent ce boss (`@boss: orcchief` dans le fichier de la salle). Une salle d'antre conçue pour ses mécaniques :
  arène dégagée et piliers pour le Chef orque (charges), autels pour la Liche… Le jeu refuse un boss sans antre.
- Le boss se place sur le marqueur `x` de l'antre ; le plan porte l'en-tête `@boss: <id>` (plans faits main aussi).
- L'objectif s'affiche « Objectif : vaincre <nom du boss> » ; la mission est réussie à sa mort (il compte comme chef).
- Premier boss : le **Chef de guerre orque** (charge 12 m, cri de guerre : 3 gobelins + 1 kamikaze, massue 4,7 m),
  dans `antre_orque` (arène 8 × 6 cases, quatre piliers, trois entrées). Modèle provisoire : squelette guerrier agrandi.

## Un donjon à l'image du boss (B.6)

- Le boss tiré change tout le donjon, comme une région : `generation.json` → `bossLairs.<id>.dungeon` (nom affiché,
  salles favorisées, facteurs de créatures) et, en plus, **`extraCreatures`** : ses troupes ajoutées par rôle de salle.
- Chef orque → « Horde orque » : casernes ×2, moitié moins de brutes squelettes, gobelins dans les passages, salles
  de garde, casernes (endormis) et cuisines, parfois un kamikaze dans les réserves.
- Ces créatures n'ont pas de lettre à elles : le plan leur donne une **lettre générique** (décision de Flo,
  2026-10-20, voir `design/donjon.md`).
- **L'antre à son image** (B.6b) : `visuals.json` → `bossLairs.<id>` : décor posé contre les murs de l'antre, à l'étage
  du boss (ni devant une porte, ni sur un marqueur, ni à côté du boss : le milieu reste libre pour le combat), et une
  lumière de sa couleur au-dessus de lui. Décor seulement (pas de collision). Chef orque : étendards rouges, armes
  brisées, tonneaux, éboulis, pièces ; lumière rouge orangé.

## En 2D (tâche 2D.29)

- Mêmes règles qu'en 3D : séquence, ruée (étourdi 2 s dans un mur ou un pilier, pas contre un corps), renforts par les
  portes de la salle, barre de boss en haut au centre avec la consigne de l'action annoncée. Les ruées des brutes et
  du chef partent aussi en 2D (elles restaient en recharge), et le kamikaze y apparaît.
- Les durées d'armement et de récupération sont celles de la 2D (`topDown.actionSpeed`) ; la portée de la ruée
  n'est pas allongée (12 m, comme en 3D).
- « Antre de boss » se choisit sur l'écran titre ; bandeau « Antre de boss : vaincre <nom> (étage N) » ; la mission
  est réussie à sa mort. Les renforts tués ne comptent pas dans « nettoyer ».
- Pas encore : décor de l'antre (bannières, lumière), corps du boss dessiné plus grand, mèche du kamikaze abattu
  montrée à l'écran. Le Chef orque est dur en 2D (un joueur seul immobile perd 80 % de ses PV en 4 s) : à juger en
  jouant.
