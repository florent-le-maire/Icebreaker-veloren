+++
ordre = 13
titre = "Carte de l'expédition"
icone = "voxygen/element/ui/generic/buttons/map.png"
resume = "Un objet du sac ouvre une carte en volume de ce qu'on a exploré, qu'on tourne à la souris ; les objectifs y sont en rouge."
design = ["donjon.md"]
reglages = ["carte.ron"]
taches = ["CA"]
systemes = [
  "common/src/icebreaker_carte.rs",
  "voxygen/src/session/icebreaker_carte.rs",
  "voxygen/src/hud/icebreaker_carte.rs",
  "server/src/sys/icebreaker_carte.rs",
]

[[chiffre]]
cle = "rayon"
nom = "Jusqu'où on découvre autour de soi"
unite = "blocs"

[[chiffre]]
cle = "hauteur"
nom = "Jusqu'où on découvre vers le haut et le bas"
unite = "blocs"

[[chiffre]]
cle = "maille"
nom = "Côté d'une case de la carte"
unite = "blocs"
+++

## La règle

En arrivant dans un lieu d'expédition (donjon, grotte, forêt), chacun reçoit dans son sac une **carte de
l'expédition**. On l'ouvre en l'utilisant dans le sac, ou par le **haut de la roue des objets** (Tab tenu, un coup de
souris vers le haut). Échap, ou le même geste, la referme.

La carte montre **en volume** ce que son personnage a exploré : sols, murs, marches, galeries, dans les couleurs du
terrain. Elle est vide à l'arrivée et se remplit en marchant, même fermée. On ne découvre que ce qu'on atteint par
l'air autour de soi : un mur ou une porte fermée cache la salle d'à côté. Chacun a la sienne ; un nouveau lieu donne
une carte vierge.

Carte ouverte, **la souris la fait tourner** (la caméra ne bouge plus) et **la molette zoome** ; le jeu continue et
on peut marcher. Elle s'ouvre tournée dans le sens où le personnage regarde, et reste centrée sur lui.

Deux repères seulement :

- **les objectifs, en rouge**, visibles dès l'arrivée même dans une zone pas encore explorée ; aujourd'hui c'est le
  chef (ou le boss) à tuer. Un objectif trop loin reste collé au bord de la carte, dans sa direction. Une fois
  rempli, il devient gris ;
- **moi**, une flèche dorée au centre, tournée où je regarde.

Ni les alliés, ni les ennemis, ni les coffres, ni le portail n'y sont.

## Pourquoi

Dans une grotte sur plusieurs niveaux, une carte à plat ne dit pas si le chef est au-dessus ou en dessous. Une carte
en volume qu'on tourne le montre d'un coup d'œil, et l'objectif rouge donne toujours une direction sans faire le
chemin à la place des joueurs.

## En images

![La carte dans une grotte : la galerie explorée, la flèche du joueur au centre, l'objectif rouge en bas](images/carte-grotte.png)
