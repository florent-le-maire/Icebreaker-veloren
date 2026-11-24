# Game design — Filons de minerai

Demandé par Flo (2026-11-23 et 2026-11-24), tâches U.18 à U.25 de `PLAN-PLAYTEST.md`. Destruction à la pioche :
[`destruction.md`](destruction.md).

## L'idée

Des filons comme dans Deep Rock Galactic : une **bande de minerai prise dans la paroi**, qu'on attaque à la pioche et
qui part **morceau par morceau, là où l'on tape**. Avant (U.14, U.15), un « filon » était une traînée de sprites
de minerai posés au pied d'une paroi, chacun ramassé d'un coup : un tas de pépites côte à côte.

## Les règles

- **Un filon est fait de blocs** pris dans la roche, à la surface d'une paroi, d'une voûte ou d'un sol. Il serpente
  sur sa longueur, a son épaisseur, entre d'un ou deux blocs dans la roche, et ressort çà et là d'un bloc (relief).
  Jamais sur le sentier ni dans la salle sûre.
- **Chaque bloc de filon est un modèle à nous** (U.23, U.24, Flo, 2026-11-24) : un bloc de gangue bosselé qui porte
  son minerai — pépites brillantes pour les métaux, blocs à facettes pour le charbon, cristaux qui luisent pour les
  gemmes. Trois variantes par minerai, tirées et tournées par le jeu selon la position. Le modèle tient la place
  d'un bloc de roche : on marche dessus, on ne passe pas à travers, une explosion ne l'emporte pas.
- **Un minerai = un sprite** (`IcebreakerFilon…`), et c'est à lui que le jeu reconnaît le minerai.
- **À la pioche** : chaque bloc de minerai a la dureté de son minerai (coups, énergie par coup) ; miner du
  minerai ne coûte pas d'énergie (U.26, Flo, 2026-11-24 : `energie: 0` partout, la roche garde son coût) ; quand il cède, il
  donne son objet (minerai ou gemme de Veloren) et, si le minerai a un `morceau`, les blocs du même minerai à cette
  distance du bloc visé partent avec lui et donnent aussi : les courants (cuivre, étain, fer, charbon) par 19 blocs au
  plus (1,5), les moyens (cobalt, argent) par 7 (1), les rares (or, gemmes) bloc par bloc (Flo, 2026-11-24). Le cône
  de U.17 n'emporte jamais un bloc de minerai : quand on creuse la roche autour, le filon reste et se dégage.
- **Tout se règle par minerai** dans `assets/icebreaker/grottes/minerais.ron` : sprite, couleur (éclats, atelier),
  coups, énergie, objet et quantité par bloc, morceau, forme du filon (longueur, épaisseur, profondeur, sinuosité,
  relief), poids par niveau de la grotte (où et combien il apparaît), poids dans la salle au minerai.
- **Combien** : dans `grotte.ron` (`filons`) : le nombre de niveaux de la grotte (`niveaux`), de filons par zone et
  par niveau (`veines.par_zone`), et de filons de la salle au minerai (`salle.filons`), dont les parois se couvrent.
- **Limite connue** : un sprite ne s'affiche que jusqu'à la distance des sprites réglée dans le client (80 à 250
  blocs) ; au-delà, la case du filon paraît vide.

## Les modèles

`icebreaker-modeles/src/filons.rs` les écrit (`cargo run -p icebreaker-modeles <dossier d'images>`) dans
`assets/voxygen/voxel/sprite/icebreaker/filon_<minerai>_<1..3>.vox` ; `assets/voxygen/voxel/sprite_manifest.ron`
les branche. Un minerai de plus : une ligne dans la table `FILONS` du générateur, un sprite `IcebreakerFilon…` dans
`common/src/terrain/sprite/mod.rs` (il reste deux places parmi les ressources minables), son entrée au manifeste, et
son entrée dans `minerais.ron`.

## Approches considérées

- **Blocs de couleur dans la roche** (U.19, Flo 2026-11-23) : rien à changer dans les genres de bloc de Veloren.
  Abandonnée le 2026-11-24 : un filon restait une bande de carrés de couleur, et le jeu devait reconnaître un minerai
  à sa couleur exacte.
- Des pépites de Veloren posées sur les blocs de couleur (U.22) : elles ne tiennent que sur le dessus, ces sprites
  n'ayant pas d'orientation. Retirées avec U.24.
- Garder les cubes et les recouvrir de cristaux sur chaque face : trois poses par minerai, soit 33 sprites
  orientables pour 23 places libres ; écartée.
- **Chaque bloc est un sprite à nous** (retenue, Flo 2026-11-24) : marche pareil au sol, en paroi et au plafond ;
  le minage ne change pas.
- Un nouveau genre de bloc par minerai : touche au protocole de Veloren, peu de places libres ; écarté.

## L'atelier des grottes

Il pose les filons avec la même fonction que le jeu (`icebreaker_caverne::filon::poser`) et les montre en cubes de
la couleur du minerai. Sa grotte n'a ni le relief du monde ni les habitants du jeu : pour une même graine, les
filons de l'atelier donnent l'allure et le nombre, pas la place exacte de ceux du jeu.
