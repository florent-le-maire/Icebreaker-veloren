# Game design — Habillage du donjon 2D

Piste T (`docs/roadmap/piste-T-tuiles.md`). Le donjon 2D est dessiné en tuiles LPC (décision 0012), en trois couches
qui suivent la génération : le **lieu** donne l'architecture, le **rôle de chaque salle** son mobilier, les
**occupants** leurs traces. Tout est tiré de la graine : le même donjon a le même habillage chez tous les joueurs.

## Lieux et architecture (tâche T.2)

Données : `game2d/data/dungeons/tiles.json` (planches, morceaux nommés, thème de chaque lieu) ; au style « Tiny »
(décision 0013, `topDown.sprites`), `tiles-tiny.json` : mêmes noms, pris dans les planches Tiny (cases de 16 px
dessinées ×2, murs d'une case de haut), un équivalent Tiny pour chaque objet du mobilier, ou `null` : l'objet n'est
pas posé dans ce style. Au style Tiny, un objet ne bloque que la case de son image (taille de `dressing.json` ignorée),
et le décor ne prend jamais une créature de Tiny Creatures (elle passerait pour un habitant ; tâche 2D.44).

| Lieu (`site`) | Sol | Murs | Pilier |
|---|---|---|---|
| Repaire (`lair`, et le souterrain par défaut) | roche moussue | roche brune | stalagmite |
| Mine naine (`mine`) | terre | roche étayée de bois | stalagmite |
| Donjon (`fortress`) | terre battue | brique rouge | stalagmite |
| Crypte (`crypt`) | dalles sombres | brique violette | statue de moine |
| Château (`castle`) | dalles claires | pierre de taille | colonne |
| Tour (`tower`) | plancher | brique grise | colonne |

- Une mission 2D sans lieu imposé (`--site`) en tire un parmi `randomSites`.
- Un mur vu depuis le sud (une salle juste en dessous sur le plan) se dessine **de face**, et sa face monte sur la roche
  au-dessus (2 ou 3 cases selon la planche) ; sinon on voit son **dessus** (couleur du thème), bordé côté salle.
- Par un trou, on voit l'étage du dessous assombri. Les échelles sont dessinées en traits (pas de tuile LPC).

## Mobilier par rôle de salle (tâche T.3)

Données : `game2d/data/dungeons/dressing.json` (`DungeonDressing`, C# pur). Chaque sorte de mobilier a une image (morceau
de `tiles.json`), une place en cases (1 × 1 à 3 × 3), une couche (debout, ou à plat comme un tapis ou des os), une
place préférée (contre un mur, au milieu, n'importe où), et dit si elle bloque et en combien de coups elle casse.

- Chaque rôle de salle a ses lignes de mobilier (`rooms`), remplacées rôle par rôle dans certains lieux (`sites`) : la
  caserne a des paillasses, mais des lits de fer au château et des cercueils dans la crypte ; la mine a des lanternes,
  du minerai et des outils. La tour s'habille comme le château (`siteAliases`).
- Une salle plus grande que `referenceArea` (90 cases libres) a d'autant plus d'objets ; une ligne `[1, 1]` (trône,
  chaudron) reste unique.
- Rien ne se pose sur un marqueur (créature, coffre), un point d'apparition, une zone, ni à côté d'une porte, d'un
  escalier, d'une échelle ou d'un trou. Un objet qui bloque ne coupe jamais une salle en morceaux ; un objet sans place
  est laissé de côté.
- Tiré de la graine de la mission : même mobilier chez le serveur et chez tous les clients.

## Mobilier qui bloque et se casse (tâche T.4)

- Le mobilier bloquant occupe ses cases dans le plan (`Tile.Furniture`, `%` dans un plan texte) : joueurs et créatures
  ne le traversent pas, les créatures le contournent (chemins), mais on **voit par-dessus** (meubles bas).
- Chaque coup d'un joueur qui l'atteint (même arc que contre une créature) lui ôte un point ; à zéro (`health`, 1 à 5
  selon la sorte), il casse : ses cases redeviennent du sol et il reste des **débris** qu'on traverse (`debris` : bois,
  ou pierre pour statues, rochers, armures…). Les créatures ne le cassent pas (pour l'instant).
- Le serveur décide et annonce chaque objet cassé à tous ; un joueur qui arrive reçoit la liste de ceux déjà cassés.
- Le décor à plat (tapis, os, flaques…) ne bloque pas et ne se casse pas. Pas de coffre dans le mobilier : les seuls
  coffres sont ceux du butin (2D.16), pour ne pas tromper.

## Traces des occupants (tâche T.5)

- Le donjon est **tenu** par le peuple du plus grand nombre de ses créatures (`creaturePeople` : brute, chef, chef
  orque → orcs ; archer, guetteur, gobelin, kamikaze → gobelins ; égalité : ordre alphabétique).
- Chaque salle reçoit, en plus du mobilier de son rôle, les **traces** de ce peuple (`traces`) : chez les orcs, feux de
  camp, os rongés, sang, crânes empilés ; chez les gobelins, foin, sacs et paniers entassés, saletés, mousse, rats,
  toiles. Un château tenu par des orcs reste un château (armures, candélabres), avec des affaires d'orcs.
- Pièces LPC manquantes (bannières, totems, crânes sur piques) : plus tard ; d'ici là les deux peuples se distinguent
  par ces quelques traces.
