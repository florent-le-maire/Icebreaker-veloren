+++
ordre = 12
titre = "Roue des objets"
icone = "voxygen/element/ui/generic/buttons/map.png"
resume = "Tab maintenu ouvre une roue autour du personnage ; un coup de souris boit une potion, mange ou change d'arme."
reglages = ["roue.ron"]
taches = ["RO"]
systemes = [
  "common/src/icebreaker_roue.rs",
  "voxygen/src/hud/icebreaker_roue.rs",
  "voxygen/src/session/mod.rs",
]

[[chiffre]]
cle = "seuil"
nom = "Longueur du coup de souris qui choisit un secteur"
unite = "px"
+++

## La règle

Maintenir **Tab** ouvre une roue au centre de l'écran ; la relâcher la fait disparaître. Tant que la roue est
ouverte, la souris ne tourne plus la caméra : elle vise un des huit secteurs (nord, nord-est, est, sud-est, sud,
sud-ouest, ouest, nord-ouest). Un **coup de souris** franc dans une direction utilise tout de suite ce qu'il y a dans
ce secteur, puis la roue se ferme ; pour une autre action, relâcher et rappuyer sur Tab.

Pour l'instant la roue est la même pour tout le monde :

- en haut, la **carte** (elle arrive bientôt : rien ne se passe encore) ;
- à gauche, **changer d'arme** (ce que faisait Tab avant) ;
- à droite, **boire une potion** : la première trouvée dans le sac ;
- juste en dessous, **manger** : le premier plat trouvé dans le sac.

Un secteur dont on n'a rien dans le sac (pas de potion, rien à manger, pas d'arme de rechange) porte une **croix
rouge** et ne fait rien. Le chiffre en bas d'une case dit combien il en reste.

## Pourquoi

En plein combat, ouvrir le sac pour chercher une potion coûte la vie. La roue garde les gestes de survie sous une
seule touche, sans regarder ailleurs que le centre de l'écran. Plus tard, chacun pourra choisir ce qu'il met dans
ses huit secteurs.

## En images

![La roue ouverte : la carte en haut, l'arme à gauche, potion et nourriture à droite, barrées faute d'en avoir](images/roue.png)
