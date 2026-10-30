# Game design — Destruction

Casser les murs du donjon (décision `docs/decisions/0009`, tâches 2.9 à 2.17). Ce qu'il y a derrière les murs selon le
lieu : [`generation.md`](generation.md) (« Derrière les murs et lieux »). Index des règles :
[`docs/GAME_DESIGN.md`](../GAME_DESIGN.md).

## Matières et outils (tâche 2.11)

Chaque case solide a une **matière** et des points de vie (`game/data/dungeons/materials.json`). Un coup retire
dégâts × facteur de l'outil pour cette matière ; à 0, la case **cède et devient du sol** couvert de gravats : ce qu'il y
avait derrière est ouvert (salle voisine, couloir, cour, ciel, ou matière suivante à creuser). Les dégâts s'additionnent
d'un coup à l'autre. Valeurs de départ :

| Matière | Où | PV | Arme | Bombe | Foreuse | Foreuse magique |
|---|---|---|---|---|---|---|
| Bois | portes | 40 | ×1 | ×2 | ×1 | ×1 |
| Porte renforcée | portes fermées à clé | 150 | — | ×1 | ×1 | ×1 |
| Pierre | murs, piliers | 200 | — | ×1 | ×1 | ×1,5 |
| Pierre renforcée | murs des chambres fortes et des antres (chef, boss) | 400 | — | ×1 | — | — |
| Maçonnerie | entre les salles d'un bâtiment, planchers, toits | 200 | — | ×1 | ×1 | ×1,5 |
| Terre | derrière les murs, sous terre | 60 | — | ×1 | ×1 | ×1 |
| Roche | sous terre, en profondeur | 250 | — | ×1 | ×1 | ×1,5 |
| Roche-mère | limite du donjon | — | — | — | — | — |

- Une **épée ne casse pas un mur** : les armes n'abîment que le bois.
- Une **bombe** touche toutes les cases du même étage autour d'elle (rayon en cases, diagonales comprises), avec les
  mêmes dégâts. Dégâts et rayon des outils : tâche 2.12.
- Une porte fermée à clé qui cède ne demande plus de clé (verrou contourné : décision 0009, point 8).
- Cour et ciel sont ouverts : rien à casser. Hors du plan, on ne creuse pas encore (le plan devra s'agrandir, 2.13).
- Le donjon garde la liste des cases qui ont cédé (`DungeonMatter.Changes`, sauvegarde en 2.16).

## Ce qui apparaît derrière (tâche 2.13)

Quand une case cède, ce qu'il y avait derrière elle devient réel : la terre, la roche ou la maçonnerie voisines
deviennent des murs de leur matière (blocs bruts de leur couleur), à creuser à leur tour ; une cour devient du sol, d'un
seul tenant, bordée de remparts au bord du plan ; le ciel reste du vide (on tombe). Une case cédée laisse des gravats de
sa matière. Le serveur envoie les cases changées à tous les joueurs, et toutes d'un coup à celui qui arrive.

## Outils (tâche 2.12)

Touche **R** : employer l'outil en main (vers le centre de l'écran, ramené à la direction de la grille la plus proche) ;
touche **3** : outil suivant. Le HUD montre l'outil en main et combien il en reste. Valeurs de départ
(`materials.json` → `equipment`, `startingEquipment`) :

| Outil | Départ | Effet |
|---|---|---|
| Bombe | 2 | posée jusqu'à 2 cases devant (contre le mur), explose après 2,5 s : 250 dégâts aux cases à 1 case autour (même étage), 45 aux corps à 4 m (alliés compris), bruit à 60 m |
| Foreuse | 1 | 6 s de travail, 60 dégâts/s à la case pleine juste devant ; quand elle cède, la suivante (tunnel) ; bruit à 24 m chaque seconde |
| Foreuse magique | 1 | 4 s, 120 dégâts/s, atteint jusqu'à 3 cases devant ; bruit à 8 m |

Une charge s'use à l'emploi. Les coffres en donnent (bombe 50 %, foreuse 30 %, foreuse magique 15 %, `loot.json`) : un
outil ramassé rejoint les outils du personnage, pas son butin. Une foreuse s'arrête sur ce qu'elle n'abîme pas (pierre
renforcée, roche-mère). Employer un outil range la carte.

## Ce qui ne tient plus tombe (tâche 2.14)

Un plancher repose sur ce qui est plein dans le carré de 3 × 3 cases juste en dessous (murs, piliers, portes, terre,
roche, maçonnerie). Quand une case cède, le plancher au-dessus d'elle et autour devient un **trou** s'il avait des appuis
au départ et n'en a plus aucun : ceux qui s'y tiennent tombent à l'étage du dessous (dégâts de chute habituels). Un
plancher sans appui dès la génération (au-dessus d'une grande salle) tient : c'est une voûte. Casser un pilier au milieu
d'une salle fait donc tomber le plancher qu'il portait ; casser un pan de mur, rarement. Salle d'essai :
`--layout test_effondrement`.

## Détruire fait du bruit (tâche 2.15)

Chaque explosion, et chaque seconde de foreuse, fait un bruit que les habitants entendent comme ceux du combat (même
ouïe, mêmes planchers qui l'étouffent) : ils viennent voir. Bombe : 60 m — une créature endormie l'entend à 18 m (×0,3),
de quoi réveiller la salle voisine, quand un combat (12 m) ne s'entend en dormant qu'à 3,6 m. Foreuse : 24 m, plus fort
qu'un combat ; foreuse magique : 8 m, discrète. La bombe désigne celui qui l'a posée aux créatures qu'elle blesse.

## Les créatures détruisent aussi (tâche 2.15b)

Une créature dont la ruée s'écrase contre le décor abîme la case heurtée, avec les facteurs de la bombe
(`creatures.json` → `crashDamage`) : brute 60 (une porte en bois cède du premier coup, un pilier de pierre en 4),
chef 100, chef de guerre orque 150 (un pilier en 2 charges : « attirez-le dans un pilier ! »). Les autres ne cassent
rien. Comme pour un joueur, ce qui ne tient plus tombe.
Les portes ne se ferment pas encore en jeu (question ouverte de la phase 2) : la brute ne « défonce » donc une porte
que si sa charge la percute.

## Le donjon garde ses traces (tâche 2.16)

Un donjon généré rejoué — même graine, même mission (taille, objectif, région, saison, menace, ennemis, lieu, boss) —
reprend les cases changées lors des parties précédentes : murs cassés, tunnels, planchers effondrés. Le serveur les
écrit à chaque changement dans `user://traces/<mission>.json` (en attendant la sauvegarde SQLite de la phase 3) et les
rejoue au lancement, avant l'arrivée des joueurs. `--fresh` les oublie pour une partie ; `game.json` →
`dungeon.keepTraces` (vrai par défaut) les coupe. Seules les cases sont gardées : les créatures tuées, les coffres
ouverts et le butin ramassé ne le sont pas encore.
