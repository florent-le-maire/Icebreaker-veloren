# 0022 — Un village fixe, dessiné d'après le croquis de Flo

- **Date** : 2026-11-21
- **Statut** : choisi et validé par Flo le 2026-11-21. Remplace, pour la forme du village, « une vraie ville Veloren »
  (Tranché du 2026-10-28) : on y marche toujours, mais la ville n'est plus tirée au hasard.
- **Tâches** : jalon VF, `PLAN.md`

## Problème

Le village était une ville de Veloren tirée au hasard (`generate_city`) : les bâtiments étaient « les maisons les
plus proches de la place », le relief n'était pas aplani, et une graine pouvait donner une ville sans maisons (Q49).
Flo veut un **hub unique** à la façon de *Darkest Dungeon* ou *Monster Hunter* : à chaque partie, **le même
village, presque plat**, disposé comme son croquis (`design/images/village-croquis.png`), où l'on pourra ajouter des
bâtiments plus tard.

## Options

1. **Garder la ville de Veloren et choisir la graine** : plus de village sans maisons, mais la disposition change
   d'une partie à l'autre et le relief reste.
2. **Un site construit à la main d'après un fichier de données**, avec les bâtiments de Veloren (maison, atelier,
   taverne) posés à leur place et une seule altitude pour tout le village.
3. **Des modèles voxel faits à la main** (prefabs) : fidèle au dessin, mais chaque bâtiment est à modéliser.

## Décision

**Option 2.** Le plan est dans `assets/icebreaker/village_plan.ron` (en tuiles de 6 blocs, nord en haut) : la
place (où l'on apparaît), les chemins pavés, et les lieux (id, forme Veloren, rectangle, côté de la porte, palier).
Le terrain du village est mis à une seule hauteur, avec une bordure de raccord en pente douce. Ajouter un bâtiment
plus tard = une ligne dans ce fichier (et une variante de `Building` s'il a un panneau).

Correspondance du croquis (Flo) : Guilde = **charrette** (recrues, registre) ; Boutique = **intendance** ;
Entrepôt = entrepôt ; Salle d'armes = salle d'armes ; Expédition = **départ** ; Forge (+ écurie) = **forge**
(nouvelle) ; champs et cimetière en décor pour l'instant ; **l'infirmerie est supprimée**. La caisse n'est pas sur
le croquis (Q63).

## Conséquences

- La graine de la partie ne change plus la disposition ; Q49 (village sans maison) disparaît.
- Les bâtiments sont trouvés par leur id dans le plan, plus par la distance à la place.
- Un lieu pas encore construit reste un terrain nu à sa place fixe (paliers du jalon C).
- Les maisons de Veloren gardent leur tirage (couleurs, étages) : la silhouette varie un peu selon la graine.
