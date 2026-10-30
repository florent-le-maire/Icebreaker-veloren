# Game design — Donjon

Décisions de l'équipe sur le donjon, étages, mouvement vertical et format des plans texte.
Index des règles : [`docs/GAME_DESIGN.md`](../GAME_DESIGN.md).

## Donjon (décisions du 2026-10-19)

- **Mort** : le personnage devient un cadavre ; un allié peut le relever sur place (phase 1).
  La résurrection coûteuse, le nécromancien et le changement de personnage arrivent en phase 3.
- **Déconnexion** en plein donjon : le personnage devient un cadavre.
- **Butin** : chacun garde ce qu'il ramasse.
- **Tir allié** : activé (`combat.friendlyFire`).
- **Verticalité** : plusieurs étages, mezzanines et fosses ; on peut sauter ; la charge du guerrier fait tomber dans les
  trous (ADR 0004).

## Étages

- Un étage fait 4 m. Le sol d'un étage est le plafond de celui du dessous.
- **Escalier** (`^ > v <`) : une rampe qui monte d'un étage sur une case (45°), entre deux rambardes : on n'y
  entre que par le bas, dans le sens de la montée.
- **Échelle** (`H`) : posée contre le bord du trou qui donne sur un palier (de préférence avec un mur derrière elle en
  bas). On s'y accroche avec **Interagir (E)**, au pied, dans le trou ou depuis le palier en haut ; on monte en
  poussant vers l'échelle (3 m/s, `movement.climbSpeed`), on descend en s'en écartant, on reste accroché sans rien
  toucher. En haut, on sort en continuant sur le palier ; en bas, tout pas qui ne monte pas fait lâcher. F lâche
  l'échelle n'importe où : on tombe (un étage : sans dégâts). Les créatures, elles, s'accrochent toutes seules en
  avançant depuis le bas et redescendent en sautant dans le trou ; les bots appuient sur Interagir. Salle de test :
  `--layout test_echelle`.
- **Trou** (`_`) : on tombe jusqu'au premier sol en dessous (dégâts de chute au-delà de 4,5 m).
- Salle de test : `--layout test_etages` (mezzanine, escalier, échelles, fosse vers une cave).

## Mouvement vertical (gravité, saut, chutes)

Réglages : section `movement` de `game.json`.
- Joueurs et créatures tombent (pesanteur 20 m/s²), cadavres compris.
- **Saut** : Espace, 1,1 m de haut, seulement debout, au sol et pas étourdi (le serveur décide). De quoi franchir un
  fossé de 2 à 3 m en courant.
- **Chute** : sans dégâts jusqu'à 4,5 m (un étage fait 4 m), puis 10 PV par mètre au-delà. Une chute peut tuer.
- Tombé hors du donjon (sous -30 m) : retour au point d'apparition (ou au point de départ pour une créature).
- Salle de test : `--layout test_mouvement` (rampe, plates-formes de 8 m, fossé de 2 m, chute).
- **Attaque plongeante** (toutes les classes ; réglages `combat.plunge` de `game.json`, capacité `plunge` de
  `abilities.json`) : l'attaque principale (clic gauche) lancée en l'air, à au moins **3 m** au-dessus du sol, remplace
  l'attaque par un piqué (18 m/s). À l'atterrissage, frappe autour de soi (3 m) : **30 dégâts + 5 par mètre** de chute
  au-delà de 3 m, et étourdit 0,6 s. Celui qui plonge se réceptionne : **aucun dégât de chute**. Un saut simple ne suffit
  pas (1,1 m) ; sauter d'un étage ou d'une mezzanine, oui. La chute réelle doit aussi faire 3 m : retombé sur un rebord
  plus haut (fossé franchi), le plongeon est raté, sans impact. Tir allié comme les autres attaques (il blesse les alliés
  pris dans la zone si `combat.friendlyFire`).

## Carte d'exploration (piste C, règles de Flo du 2026-10-20)

- **G** (réassignable) sort ou range la carte : le personnage tient une feuille, la caméra passe à la première
  personne, la feuille occupe le bas de l'écran ; **le jeu continue**.
- **Chacun sa feuille** : on n'y voit que ce que son propre personnage a exploré. Les autres voient qu'on lit sa
  carte (une feuille blanche en main), pas ce qu'elle dit.
- Une salle apparaît **en entier** quand on y a **mis les pieds** (zone = sol relié sans passer une porte). Une porte
  qui mène à une salle inconnue reste ouverte sur du blanc.
- Dessiné : salles, couloirs, portes, piliers, trous, échelles, escaliers ; **rien d'autre** (ni créatures, ni
  coffres, ni alliés, ni « vous êtes ici »). **Une feuille par étage connu**, en pile : la carte s'ouvre sur l'étage où
  l'on est, la molette fait passer à l'étage connu au-dessus ou en dessous. La feuille est à l'italienne.
- Carte en main : vitesse × `map.moveFactor` (0,7), **pas d'attaque** (une attaque range la carte) ; avancer suit le
  regard, gauche et droite font un pas de côté, la souris tourne le personnage (plus lentement qu'en temps normal).
  À terre, la carte est rangée.
- **En 2D** (tâche 2D.33, façon Hollow Knight) : **Tab tenu** montre la carte en transparence par-dessus le jeu, sur
  l'étage montré ; le personnage tient une feuille dans ses mains (les autres la voient), vitesse ×
  `topDown.mapMoveFactor` (0,45), ni coup ni course. La carte s'ouvre sur l'étage où l'on est ; carte ouverte, la
  molette passe à l'étage connu au-dessus ou en dessous (pas de zoom), sans changer l'étage montré du jeu ; rangée,
  elle change l'étage montré comme avant. Où l'on est et ses alliés : seulement si
  `topDown.mapShowsPlayers` (non par défaut, règle ci-dessus).
- On **garde la feuille** toute l'expédition, mort et relève comprises ; un nouveau donjon donne une feuille vierge.
  La garder d'une partie à l'autre viendra avec la sauvegarde (phase 3).

## Vue et brouillard en 2D (tâche 2D.31, Flo le 2026-10-25)

- On voit **tout autour de soi**, jusqu'à `topDown.sightRadius` cases (18), ce qui est **en ligne de vue** : murs et
  piliers cachent ce qui est derrière ; le mobilier (on voit par-dessus) et les portes non.
- **Chacun sa mémoire** (comme sa carte) : jamais vu = noir ; déjà vu = assombri (`topDown.fogSeenAlpha`, 0,6), le décor
  reste mais pas les créatures, le butin ni les tirs ; vu = normal.
- Les créatures hors de vue ne se montrent pas ; les alliés si (sous le brouillard dans le noir).
- **Salles sombres** (tâche 2D.30) : une part des salles est éclairée (`topDown.torch.litShare`, 0,4 ; l'entrée
  toujours, même tirage pour tous). Ailleurs, on ne voit que ce qu'éclaire une torche : **tout le monde en porte une**
  (pas d'objet à choisir en 2D), portée `topDown.torch.radius` (8 cases) ; celle d'un allié éclaire aussi ce qu'on voit
  d'elle. La lumière s'assombrit vers son bord et vacille ; éteinte à terre. Les créatures voient toujours dans le noir.

## Lumière et torche (demande du 2026-10-21)

- **Toutes les salles ne sont pas éclairées** : une part des zones de la carte (salles et couloirs séparés par des
  portes) a ses torches murales allumées (`visuals.json`, `dungeon.torch.litShare`, 0,4 au départ) ; ailleurs, les
  supports sont éteints et **on n'y voit rien** (ni lumière d'ambiance ni « soleil », Main.tscn) : seule une torche,
  celle d'un allié ou la lumière d'une salle voisine éclaire. La salle d'entrée est toujours éclairée. Tirage d'après
  le plan (mêmes salles pour tous).
- **Torche** : un objet de main gauche comme le bouclier (piste E) : on la prend en faisant défiler la main gauche
  (touche 2) ; Maj + clic droit reste la compétence de l'arme principale. **Pas avec une arme à deux mains** (elle
  reste à la ceinture, éteinte). Changement refusé en plein coup, en ruée, étourdi ; éteinte à terre.
- **Clic droit maintenu, torche en main : on la lève** (posture bras levé) : la lumière monte au-dessus de la tête,
  éclaire plus fort et plus loin (8 m → 16 m). On peut toujours frapper de la main droite.
- Pour l'instant purement visuel : les créatures voient aussi bien dans le noir (question ouverte, phase 1).
- **Cible, pas encore faite** : l'éclairage décidé par le générateur selon les habitants (ceux qui voient dans le noir,
  comme les gobelins, vivent dans des salles noires ; ceux qui ont besoin de lumière les éclairent). Découpage proposé
  dans `docs/roadmap/phase-2.md`.

## Plans de donjon

Fichiers texte dans `game/data/dungeons/` (le plan chargé est choisi par `dungeon.layout` dans `game.json`).
Une case = **4 m** (murs de 4 m de haut, échelle du kit KayKit), un caractère par case :

| Caractère | Case |
|---|---|
| `#` | mur |
| `.` | sol |
| `O` | pilier (obstacle sur du sol) |
| `D` | porte (passage de 2 m, orientée selon les murs voisins) |
| `S` | point d'apparition des joueurs (sol) |
| `b` `a` `g` `c` | marqueurs de créature (sol) : brute, archer, guetteur, chef ; en majuscule (`B` `A` `G` `C`), la créature dort |
| `x` | le boss du plan (sol, `X` endormi) : la créature de la ligne `@boss: <id>` (antre de boss) |
| lettre générique | n'importe quelle créature de `creatures.json` (gobelin…) : déclarée en tête du plan par `@creature: k goblin` (lettres possibles : `efijklnpqrtuwyz`), majuscule = endormie |
| `$` | coffre (sol) |
| `m` | mannequin d’entraînement (sol) |
| `_` | trou : pas de sol, on tombe à l'étage du dessous (fosse, vide au-dessus d'une salle haute) |
| `H` | échelle (sur du sol) : monte par le trou `_` de l'étage du dessus |
| `^` `>` `v` `<` | escalier qui monte vers le nord / l'est / le sud / l'ouest (une case de long, un étage de haut) |
| espace | vide (roche) |

Les lignes qui commencent par `;` sont des commentaires. En tête du plan, `@boss: <id>` et `@creature: <lettre> <id>`
donnent le boss et les lettres génériques ; `@room: <rôle> <colonne> <ligne> <étage>` donne le rôle d'une salle par une de
ses cases de sol, pour son décor (piste V). Le générateur écrit ces lignes lui-même. `@zone: <type> <colonne> <ligne>
<étage>` (piste S) pose une zone de `combat/zones.json` sur cette case pour toute la partie, à personne : elle agit
sur tout le monde (brasier, flaque ; plan `test_combos`).

**Étages** (tâche 1.18, ADR 0004) : une section `[étage N]` par étage, étages consécutifs ; 0 = étage de l'entrée,
positif au-dessus, négatif en dessous (caves). Toutes les sections sont alignées sur le même coin en haut à gauche. Un
plan sans section n'a qu'un étage (l'étage 0). Exemple : `game/data/dungeons/test_etages.txt`. Règles vérifiées à la lecture :
- un escalier a du sol à son pied (côté opposé à la montée) et à son arrivée à l'étage du dessus, et un trou `_` juste
  au-dessus de lui (il le traverse) ;
- une échelle a un trou `_` au-dessus d'elle, avec du sol autour de ce trou pour en descendre ;
- un trou finit toujours sur du sol, une échelle ou un escalier (on peut traverser plusieurs étages) : pas de trou
  sans fond, pas de chute sur un mur.
Une case a un plafond quand la case de l'étage du dessus n'est ni vide ni un trou. Mezzanine = sol à l'étage 1
au-dessus de trous, salle haute = murs sur deux étages.


Contraintes de conception : passages d’au moins 2 m (les personnages font 1 m de large et la navigation
retire leur rayon) ; tout le sol doit être accessible depuis les `S` (vérifié par les tests unitaires).
Option de lancement `--bot` : le client suit la navigation de marqueur en marqueur, pratique pour tester un plan.
