# Game design — Filons de minerai

Demandé par Flo (2026-11-23), tâches U.18 à U.21 de `PLAN-PLAYTEST.md`. Destruction à la pioche :
[`destruction.md`](destruction.md).

## L'idée

Des filons comme dans Deep Rock Galactic : une **bande de minerai prise dans la paroi**, qu'on attaque à la pioche et
qui part **morceau par morceau, là où l'on tape**. Jusqu'ici (U.14, U.15), un « filon » était une traînée de sprites
de minerai posés au pied d'une paroi, chacun ramassé d'un coup : un tas de pépites côte à côte.

## Les règles

- **Un filon est fait de blocs** pris dans la roche, à la surface d'une paroi, d'une voûte ou d'un sol. Il serpente
  sur sa longueur, a son épaisseur, entre d'un ou deux blocs dans la roche, et ressort çà et là d'un bloc (relief).
  Jamais sur le sentier ni dans la salle sûre.
- **Un minerai = une couleur** de roche (et une roche qui luit pour les gemmes, si on veut) ; le jeu le reconnaît à
  sa couleur exacte, qui ne doit être celle d'aucune roche d'ambiance.
- **À la pioche** : chaque bloc de minerai a la dureté de son minerai (coups, énergie par coup) ; quand il cède, il
  donne son objet (minerai ou gemme de Veloren) et, si le minerai a un `morceau`, les blocs du même minerai à cette
  distance du bloc visé partent avec lui et donnent aussi : les courants (cuivre, étain, fer, charbon) par 19 blocs au
  plus (1,5), les moyens (cobalt, argent) par 7 (1), les rares (or, gemmes) bloc par bloc (Flo, 2026-11-24). Le cône de U.17 n'emporte jamais un bloc de minerai : quand
  on creuse la roche autour, le filon reste et se dégage.
- **Des pépites sur le filon** (U.22, Flo, 2026-11-24) : pour qu'il ne soit pas qu'une bande de cubes, une part
  (`pepites`) des blocs du filon tournés vers le haut porte la pépite de Veloren de son minerai (`pepite`), hors du
  sentier et de la salle sûre. Elle se mine comme les pépites de Veloren (minerai en plus) et tombe, en donnant ce
  qu'elle donne, quand le bloc qui la porte cède. Pas sur les parois verticales ni au plafond : ces sprites n'ont pas
  d'orientation.
- **Tout se règle par minerai** dans `assets/icebreaker/grottes/minerais.ron` : couleur, lueur, coups, énergie,
  objet et quantité par bloc, morceau, forme du filon (longueur, épaisseur, profondeur, sinuosité, relief), poids
  par niveau de la grotte (où et combien il apparaît), poids dans la salle au minerai.
- **Combien** : le nombre de filons par zone et par niveau reste dans `grotte.ron` (`filons.veines.par_zone`), comme
  celui de la salle au minerai (`filons.salle.filons`), dont les parois se couvrent de filons.
- Les pépites isolées (sprites) restent possibles mais leur chance passe à 0 (`filons.chance`).

## Approches considérées

- **Blocs de couleur dans la roche** (retenue, Flo 2026-11-23) : rien à changer dans les genres de bloc de Veloren,
  le minerai survit au rechargement du terrain, la destruction de U.16 sert telle quelle. On perd l'allure à facettes
  des sprites de Veloren : un bloc reste un cube de terrain.
- Nos propres modèles voxel de cristaux posés contre la paroi (comme `piece-vivante`) : plus joli, beaucoup plus long ;
  à reprendre plus tard si les blocs ne suffisent pas.
- Un nouveau genre de bloc par minerai : touche au protocole de Veloren, peu de places libres ; écarté.

## Ce qui est laissé à qui code

La forme exacte de la marche le long de la paroi ; les valeurs de départ de `minerais.ron` (dureté croissante avec la
rareté) ; l'affichage dans l'atelier des grottes si la géométrie du filon vit dans `icebreaker-caverne`.
