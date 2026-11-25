+++
ordre = 10
titre = "La difficulté"
icone = "voxygen/element/ui/map/buttons/excl.png"
resume = "Chaque expédition part à une difficulté de 1 à 6 : plus d'ennemis, plus solides, plus dangereux, plus variés et plus habiles."
reglages = ["difficulte.ron"]
taches = ["DF"]
design = ["difficulte.md"]
systemes = [
  "common/src/icebreaker_difficulte.rs",
  "world/src/icebreaker/races.rs",
  "world/src/icebreaker/dungeon/populate.rs",
  "server/src/icebreaker_expedition.rs",
  "icebreaker-equilibrage/src/difficulte.rs",
]

[[chiffre]]
cle = "defaut"
nom = "Difficulté d'une expédition qui n'en choisit pas"
+++

## La règle

Au panneau **Départ**, une fois la destination choisie, le groupe choisit une **difficulté de 1 à 6**. N'importe quel
inscrit peut la changer, tout le monde est prévenu. Elle vaut pour le donjon généré, la grotte et la forêt. À
l'arrivée, le jeu la rappelle avec la forme et la graine du lieu.

Sans choix, l'expédition part en difficulté 3 : c'est le jeu tel qu'il était avant ce réglage. Les difficultés 1 et 2
sont plus douces.

## Ce qu'elle change

| | Comment |
|---|---|
| **Nombre** | chaque groupe d'ennemis grossit ou rétrécit ; le chef reste seul |
| **PV** | les PV de chaque ennemi sont multipliés |
| **Dégâts** | les coups de chaque ennemi sont multipliés |
| **Classes** | en difficulté 1, pas de mage ; en 2, peu ; en 5 et 6, de plus en plus |
| **Habileté** | les ennemis connaissent plus de coups : rang 1 jusqu'à la difficulté 3, rang 3 en 4 et 5, rang 5 en 6 |

Le terrain ne change pas : la forme et la taille du lieu se choisissent à part.

## À savoir pour régler

Les effets se **multiplient**. En difficulté 6, il y a plus d'ennemis, qui ont chacun plus de PV et frappent chacun
plus fort : le tableau « Les six difficultés » de l'équilibrage (`equilibrage/difficulte.md`) donne le total, compté
dans de vrais lieux générés. C'est lui qu'il faut regarder avant de toucher une ligne de `difficulte.ron`.

## Pour essayer

- En campagne : `/expedition difficulte 5` après s'être inscrit, ou les lignes « Difficulté » du panneau Départ.
- Un lieu seul : `.\icebreaker.ps1 donjon -Site Genere -Difficulte 5` (aussi `Grotte` et `Foret`).
- Au lieu d'entraînement : `/entrainement difficulte 5`, puis `/entrainement ennemi orc mage`.

## Pas encore

Une expédition plus difficile ne rapporte pas plus (question Q67). La difficulté n'est pas liée au niveau des zones
ni à leurs boss (Q68).
