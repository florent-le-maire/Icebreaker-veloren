# Game design — Génération des donjons

Graphe de mission, salles préfaites, assemblage, habitants crédibles, paramètres de mission (tâches 2.1 à 2.7).
Index des règles : [`docs/GAME_DESIGN.md`](../GAME_DESIGN.md).

## Génération des donjons (graphe de mission, tâche 2.1)

- Un donjon généré = un chemin principal de l'entrée à la salle de l'objectif (antre du chef, prison, chambre forte…
  selon l'objectif), qui descend d'étage en étage ; des branches en cul-de-sac (caserne, réserve, trésor…) ; des boucles.
- **Verrous** : une porte fermée sur le chemin principal ; sa clé est toujours dans une branche *avant* elle.
- **Chutes** : raccourcis à sens unique vers plus bas. Il existe toujours un autre chemin pour remonter : aucune chute
  ne piège le groupe.
- Boucles et chutes ne contournent jamais un verrou.
- Réglages par taille de donjon (`small`, `medium`, `large`) dans `game/data/dungeons/generation.json`.

## Salles préfaites (tâche 2.2)

Le générateur assemble des salles écrites à la main, une par fichier dans `game/data/dungeons/rooms/` (l'identifiant
est le nom du fichier). **Toute l'équipe peut en ajouter** : les tests `RoomCatalogTests` disent précisément ce qui ne va
pas. La version 2D a ses propres salles, plus grandes (`game2d/data/dungeons/rooms/`, au moins 17 × 11 cases murs
compris, une case = une tuile) : celles-ci agrandies par `tools/rooms2d/agrandir_salles.py`, puis retouchables à la
main (relancer le script les écrase). Une salle = un plan au format habituel (étages compris, voir « Plans de donjon ») avec en plus :
- une ligne `@roles: barracks, guardRoom` : rôles qu'elle peut tenir (`entrance`, `hall`, `guardRoom`, `barracks`,
  `storage`, `kitchen`, `nest`, `treasure`, `chiefLair`, `prison`, `vault`, `sanctum`) ;
- `+` : **porte possible**, dans un mur extérieur (le dehors d'un seul côté, du sol juste à l'intérieur). Le générateur
  en fait une porte s'il s'en sert, sinon elle reste un mur ;
- `*` : **fosse possible**, dans le sol, avec rien sous elle dans la salle : départ d'une chute vers une salle plus basse.
  Elle reste du sol si elle ne sert pas.

Règles vérifiées : au moins une porte possible ; depuis chaque connecteur on peut aller à tous les autres et à toute la
salle (en tenant compte des chutes) ; points d'apparition `S` dans les salles d'entrée seulement (et au moins un).
Les marqueurs (créatures, coffres) sont des propositions : le peuplement (2.5) les retire et place les habitants selon
le rôle de la salle, en commençant par les propositions de même sorte (un `a` sur un balcon y met l'archer).
Une salle qui peut se trouver au milieu d'une branche ou du chemin principal doit exister en version à deux portes au moins
(vérifié par les tests) ; mettre plusieurs portes possibles ne coûte rien (les inutilisées restent des murs) et aide
l'assemblage à placer boucles et chutes.

**Variété des salles en 2D** (tâches 2D.17 et 2D.18, réglages dans `game2d/data/dungeons/variety.json`) :
- chaque salle dessinée est aussi posée tournée et retournée (`orientations` : 1, 2, 4 ou 8), escaliers compris ;
- des **salles générées** de formes variées se mêlent aux salles dessinées : coins coupés, L, T, croix, octogone,
  cercle ou ellipse, grotte. Elles sont tirées pour chaque partie (même graine, mêmes salles), entourées de murs, avec
  2 à 4 portes possibles dans les pans droits, puis garnies selon le rôle (piliers, coffres). `share` règle leur part
  (0,5 : autant que de dessinées), `roles` dit quels rôles en reçoivent et avec quelles formes. Entrée, antres et
  sanctuaire restent dessinés à la main. Une branche ou un passage de grotte fait au moins `minBar` cases de large.
- ces rôles dessinés ont aussi leurs salles de forme marquante (2D.19, ex. `sanctuaire_rond`, `antre_rond`,
  `salle_trone` en T, `entree_t`) : la 2D peut avoir des salles qui n'existent pas en 3D.

## Assemblage du donjon (tâche 2.3)

- Les salles sont posées depuis l'entrée ; une salle de profondeur *n* dans le graphe est à l'étage *-n*.
- Les couloirs sont creusés automatiquement, entourés de murs, et ne restent qu'entre les étages de leurs deux bouts.
  Un escalier de couloir fait 2 cases de long par étage ; les chutes sont des puits à sens unique.
- Un couloir ne longe jamais une porte inutilisée (elle reste disponible) et ne s'ouvre jamais sur un autre couloir.
- Réglages dans `generation.json` → `assembly` (tentatives, essais, espacement, limite de recherche).

### Formes du plan (F.46 à F.49, Veloren)

Des réglages repris de donjon.bin.sh (ses idées, pas son code), sans changer de générateur : le graphe de mission
décide toujours des salles, des verrous et des clés. Ils sont dans `generation.ron` → `assembly` et se fixent par
preset (`donjons/formes.ron`, atelier du donjon). À leur valeur par défaut, une graine donne le même donjon qu'avant.

- **Silhouette** (`silhouette`, `etendue`) : salles et couloirs restent dans une forme, la même à chaque étage :
  Carre, Rectangle, Croix, Rond, Couronne, Diagonales, Losange, Hexagone (`Libre` : aucune). Le milieu de la forme
  est posé au plus près de l'entrée, pour que le donjon reste dans son emprise. `etendue` (cases du milieu au bord)
  vaut pour 18 salles et suit la racine du nombre de salles, sans dépasser 56.
- Dans une silhouette, l'assemblage part plutôt d'une porte qui a du champ devant elle, essaie trois fois plus, et
  d'essai en essai pose la salle plus loin de la porte (jusqu'à 14 cases). Une forme trop serrée fait échouer des
  graines ; après 6 graines sur 10, elle est abandonnée et le résumé du donjon le dit.
- **Couloirs** (`sinuosite`, `impasses`) : de 0 (le plus court) à 100 (un dédale), par un coût tiré pour chaque
  case ; ne se voit que si les salles sont écartées (`spacing`). Une impasse est un couloir sans issue de 2 à 5
  cases, creusé depuis un autre couloir ; elle ne touche rien d'autre de praticable (aucun passage autour d'un verrou).
- **Salles collées** (`collees`, en %) : la salle est posée contre celle d'où l'on vient, porte contre porte, au
  même étage ; chacune garde son mur (deux murs, deux portes). Si elle ne tient pas, un couloir.
- **Salles à forme tirée** (`formes_part`, `salle_largeur`, `salle_profondeur`) : leur part et leur taille ; les
  salles dessinées gardent la leur.

### Remplir la forme et relief (F.51 à F.57, Veloren)

Trois manques à côté de donjon.bin.sh : le donjon n'occupait que 18 à 33 % des cases de sa forme, ses couloirs ne
faisaient pas un labyrinthe, et il était presque plat. Mêmes principes : la mission ne change pas, tout est en
réglage (`assembly`, `vertical` de `generation.ron`, presets, curseurs de l'atelier), et à 0 une graine donne le
donjon d'avant. Le résumé du donjon dit la part de la silhouette occupée.

- **Labyrinthe de remplissage** (`remplissage`, 0 à 100 % de la roche libre) : une fois la mission assemblée, un
  labyrinthe est creusé dans le vide de la silhouette, à chaque étage. Chaque morceau part d'une case de couloir
  et avance de deux cases (couloir, mur, couloir) ; une case n'est creusée que si rien de praticable ne la touche
  hors celle d'où elle vient. **Le labyrinthe ne touche donc le donjon que par le pied de chaque morceau** : il ne
  contourne aucune porte à clé, et on en ressort toujours. `boucles_labyrinthe` ouvre des murs à l'intérieur d'un
  même morceau, `elagage` rebouche des culs-de-sac, `sinuosite` allonge ses couloirs. À 100 %, 98 à 99 % des cases
  de la forme sont prises. Les créatures de couloir y sont rares (`corridors.labyrinthe_per_cells`).
- **Salles en plus** (`salles_en_plus`) : des culs-de-sac hors mission, tirés et peuplés comme une salle de
  branche, posés loin de la salle où ils s'accrochent, là où la forme est vide.
- **Salles tout en hauteur** (`vertical`), générées avec les symboles des plans (`design/donjon.md`) :
  l'**escalier en Z** (`escaliers_z` : deux volées opposées l'une au-dessus de l'autre) et la **tour** (`tours`,
  `tour_etages`, `tour_largeur`, `tour_vide` : un puits carré, une volée par mur, un palier à portes par coin, un
  vide ou un pilier au milieu). Elles ne sont posées que là où le chemin descend d'autant d'étages qu'elles en
  montent : on entre par le haut, on repart par le pied. Avec `tours`, un changement d'étage du chemin principal
  peut descendre plusieurs étages d'un coup. Les tours restent sous le sol (F.31).
- Limites connues : un étage entier est rempli alors que les salles se partagent entre les étages, donc le
  labyrinthe est grand (4 500 cases de couloir pour 19 salles dans `labyrinthe` en taille moyenne) ; resserrer la
  forme fait échouer l'assemblage. Rien ne part encore des paliers du milieu d'une tour.

**Les six formes** (F.59, `donjons/formes.ron`, toutes tirées par la graine avec le même poids) : chacune pousse un
réglage que les autres n'ont pas.

| Forme | Ce qui la fait | En taille moyenne |
|---|---|---|
| `croix` | deux longues ailes, presque à plat, ni boucle ni chute, escaliers en Z | 15 salles, 50 cases de large |
| `forteresse` | anneau de salles carrées collées, un étage, portes à clé, presque pas de couloir | 3 cases de couloir par salle |
| `rond` | cercle serré sur 2 à 3 étages, boucles, salles à forme tirée, escaliers en Z | 44 cases de large |
| `labyrinthe` | carré rempli par le labyrinthe, couloirs sinueux, salles hors mission | 4 500 cases de couloir |
| `tour` | étroite et très profonde : tours à colimaçon, escaliers en Z, chutes, salles collées | 9 étages, 35 cases de large |
| `plat` | le donjon ordinaire sans silhouette, un seul étage, sans chute | 16 salles, 62 cases de large |

## Habitants crédibles (tâche 2.5)

Règles de voisinage (`generation.json` → `rules`), vérifiées sur 1000 graines par taille :
- la salle juste avant chaque porte fermée est une **salle de garde** ;
- la **caserne** part de l'entrée ou d'une salle de garde ; la **réserve** part d'une cuisine ;
- les **nids** sont à au moins deux salles du chemin principal ;
- au plus 1 trésor, 1 cuisine, 2 casernes, 2 réserves et 2 nids par donjon ;
- la salle de l'objectif est la plus profonde (antre du chef au fond).

Un rôle qu'aucune règle n'autorise est remplacé par une salle de garde.

Habitants et butin par rôle (`generation.json` → `population`, valeurs de départ) :

| Salle | Créatures | Coffres | État |
|---|---|---|---|
| Passage | 0–1 guetteur | – | éveillés |
| Salle de garde | 1–2 brutes, 0–1 archer | – | éveillés |
| Caserne | 2–3 brutes | – | **endormies** |
| Réserve | 0–1 guetteur | 1–2 | éveillés |
| Cuisine | 0–1 brute, 0–1 guetteur | – | éveillés |
| Nid | 2–3 brutes | – | **endormies** |
| Trésor | 1 brute, 0–1 archer (trésor gardé) | 2–3 | éveillés |
| Antre du chef | le chef, 1–2 archers | – | éveillés |
| Prison | 1–2 brutes | – | éveillés |
| Chambre forte | 1–2 brutes, 0–1 archer | 3–4 | éveillés |
| Sanctuaire | 1–2 archers | 1 | éveillés |

Jamais de créature ni de coffre sur un point d'apparition, une clé ou une case qui touche une porte. Ce sont des
maximums : la difficulté selon le nombre de joueurs (1.14) en retire. « Endormie » est noté dans le donjon généré mais
n'a pas encore d'effet en jeu (pas d'état sommeil chez les créatures, voir questions ouvertes).

## Paramètres de mission (tâche 2.6)

Une mission donne, en plus de la graine, de la taille et de l'objectif : une **région**, une **saison** et une
**menace** (1 à 5 ; remplacée le 2026-11-25 par la **difficulté 1 à 6** de l'expédition, `design/difficulte.md`,
qui change les ennemis sans changer le plan). Chacun change le donjon (`generation.json` → `parameters`) : salles et étages en plus ou en moins,
poids des rôles des salles, nombre de créatures (par sorte ou en tout), nombre de coffres. Les écarts s'additionnent et
les facteurs se multiplient ; le chef reste toujours unique ; les règles de voisinage (2.5) s'appliquent toujours.
Régions et saisons sont **provisoires** (la carte du monde arrive en phase 4) ; valeurs de départ :

La **Forêt** est aussi un lieu à part entière (jalon FO, `PLAN-FORET.md`, décidé le 2026-11-24) : deux étages, des
clairières reliées par des sentiers au sol et des plateformes reliées par des ponts dans les arbres ; tomber d'un
pont fait les dégâts de chute normaux, on atterrit sur un sol jouable et on peut toujours remonter. N'y vivent que
nos six espèces, une par forêt (Q66, 2026-11-25), pas de bêtes : un groupe dans chaque clairière (mêlé dans un
« camp », des guerriers serrés dans un « nid »), des tireurs sur les plateformes, le chef et ses gardes dans la
clairière du fond, la plus loin de l'arrivée par les sentiers. Tuer le chef accomplit la mission. Un coffre par
clairière, quelques-uns dans les cabanes et au bout des branches, plus riches loin de l'arrivée ; personne ni aucun
coffre près de l'arrivée (`assets/icebreaker/forets/foret.ron`).

| Paramètre | Effet |
|---|---|
| Forêt | nids ×2, cuisines ×1,5 ; archers ×1,5 |
| Montagne | +1 étage, +1 salle sur le chemin ; casernes ×2, nids ×0,5 ; brutes ×1,2 |
| Marais | nids ×3, trésor et cuisine ×0,5 ; guetteurs ×2 ; coffres ×0,7 |
| Printemps | nids ×1,5 |
| Été | guetteurs ×1,5 |
| Automne | réserves ×2, cuisines ×1,5 ; coffres ×1,3 |
| Hiver | casernes ×2, réserves ×0,5 ; créatures ×1,1 ; coffres ×0,8 |
| Difficulté 1 à 6 (ex-menace) | créatures ×0,70 à ×1,45, et PV, dégâts, classes, habileté : `design/difficulte.md` |
| Objectif « piller » | trésors ×3, réserves ×1,5 ; coffres ×1,3 |
| Objectif « nettoyer » | nids ×2 ; créatures ×1,2 |
| Objectif « sauver » | salles de garde et casernes ×1,5 |
| Objectif « explorer » | +1 étage, +1 salle ; créatures ×0,8 |
| Objectif « antre de boss » | créatures ×0,8 (le boss et ses renforts suffisent) |
| Ennemis faible / moyen / beaucoup | créatures ×1 / ×2 / ×3,5 (le chef et le boss restent uniques ; salles et coffres inchangés) |

Sans région ni saison : aucun effet ; sans difficulté : difficulté 3 ; sans nombre d'ennemis : faible. Le nombre d'ennemis
se choisit dans l'écran « Jouer » et l'atelier (liste « Ennemis ») ; en donjon moyen, ~14 / ~27 / ~50 créatures en
moyenne avant la difficulté selon le nombre de joueurs (40 graines). Les cases de sol libres plafonnent ce nombre.

## Donjon joué (tâche 2.7)

- Par défaut, le serveur **génère** le donjon (`dungeon.layout = "generated"` dans `game.json`) : graine au hasard, ou
  `--seed <n>` ; taille, objectif, région, saison et menace dans `dungeon.generation`, remplacés par `--size`,
  `--objective`, `--region`, `--season`, `--threat` (`none` retire région, saison ou menace), `--boss <id>` (boss de
  l'antre), `--density <low|medium|high>` (nombre d'ennemis). L'objectif ne change que
  la forme du donjon (décision du 2026-10-20) : la mission se gagne toujours en tuant le chef, qui garde **toujours**
  la salle de l'objectif (prison, chambre forte, nid, sanctuaire… en plus de ses habitants habituels), seul chef du donjon.
  Antre de boss : le boss tient ce rôle à la place du chef (voir `design/boss.md`).
- Un plan fait main reste possible : `--layout repaire` (ou `test_chef`, `test_etages`…).
- Tous les joueurs jouent le **même** donjon : le serveur leur envoie le plan. La graine est affichée en haut à droite
  pour rejouer un donjon qui a plu (ou déplu).
- Les portes fermées et les clés placées par le générateur n'ont **pas encore d'effet** : les portes sont ouvertes
  (voir questions ouvertes). Les créatures « endormies » dorment vraiment depuis la tâche H.1.


## Derrière les murs et lieux (tâches 2.9 et 2.10)

Chaque case vide du plan, et hors du plan, contient quelque chose selon le **lieu** (type de site) de la mission
(`generation.json` → `sites`, `defaultSite` ; décision `docs/decisions/0009`). On ne le voit pas encore en jeu : ça
servira quand un mur cède. Valeurs de départ :

| Lieu | Assemblage | Dans le bâtiment | Dehors, au sol | Dehors, au-dessus | Sous le sol |
|---|---|---|---|---|---|
| Souterrain (par défaut) | habituel | — | terre | — (le sol est l'étage le plus haut) | terre, puis roche à 2 étages sous le haut |
| Château | **salles collées** | maçonnerie | cour | ciel | — |
| Tour | habituel | maçonnerie | cour | ciel | — |
| Crypte, Mine naine, Donjon, Repaire | habituel | — | terre | — | comme le souterrain (la mine : roche dès 1 étage sous le haut) |

- **Salles collées** (château) : une salle peut poser ses murs sur ceux d'une autre ; trois fois sur quatre, la salle
  suivante est d'abord essayée contre la précédente, reliée par une porte dans leur mur commun, sans couloir. Entre deux
  étages, il faut toujours un couloir (escalier), d'au plus 2 cases. Sur 1000 graines, 60 % des murs de salle du
  château donnent sur une autre salle, un couloir, la cour ou le ciel (9 % sous terre), toujours finissable.
- **Le bâtiment** (château, tour) : la maçonnerie bouche les espaces de 2 cases ou moins entre les salles, et fait les
  planchers et les toits (case juste au-dessus ou au-dessous d'une case construite). Plus large, c'est le dehors : la
  cour au sol, le ciel au-dessus.
- Le donjon descend toujours depuis l'entrée : un bâtiment a son sol à l'**étage le plus bas** (on entre par le haut).
- **Roche-mère** : au-delà de 6 cases des salles, la terre et la roche deviennent de la roche-mère (rien ne la perce) :
  c'est la limite où l'on pourra creuser.
- Crypte, mine, donjon et repaire ne changent que l'habillage 2D (`docs/design/habillage.md`). En 2D, une mission
  sans lieu imposé en tire un parmi `randomSites` de `game2d/data/dungeons/tiles.json` (même graine, même lieu).
- Le lieu se choisit dans la liste « Lieu » (écran « Jouer », atelier), avec `--site` côté serveur ou dans l'outil
  d'aperçu (qui colore ces matières autour des salles), ou `dungeon.generation.site` dans `game.json`.
