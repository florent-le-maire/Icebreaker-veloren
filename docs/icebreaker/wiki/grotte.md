+++
ordre = 7
titre = "La grotte"
icone = "voxygen/element/weapons/pickaxe.png"
schema = "coupe-grotte"
resume = "Des poches et des boyaux creusés dans la roche, des filons à la pioche, un chef au fond. Façon Deep Rock Galactic."
reglages = ["grottes/grotte.ron", "butin.ron"]
taches = ["K", "B.5"]
systemes = [
  "world/src/icebreaker/grotte/mod.rs",
  "world/src/site/plot/icebreaker_grotte.rs",
  "world/src/icebreaker/butin.rs",
]

[[chiffre]]
cle = "poches"
nom = "Poches par grotte"

[[chiffre]]
cle = "niveaux"
nom = "Niveaux de profondeur"

[[chiffre]]
cle = "ecart_niveau"
nom = "Blocs entre deux niveaux"

[[chiffre]]
cle = "chance"
nom = "Chance de filon par bloc de bord"
pourcent = true
+++

## La règle

La grotte est le deuxième lieu de notre fabrication, sans salles ni cases : tout est creusé en blocs. De la
**bouche** en surface, un boyau descend vers la première **poche**, un ellipsoïde au sol bosselé. De là, un
réseau de poches sur plusieurs niveaux, chacune accrochée à une poche du niveau au-dessus, avec quelques boucles
et des culs-de-sac étroits d'où sortent les bêtes.

Chaque grotte tire une **ambiance** (champignons, cristaux, ossuaire) : la couleur de la roche, la roche luisante,
les sprites au sol et au plafond, les créatures et le **chef**, qui attend dans la poche la plus profonde. Le
tuer, c'est la victoire.

Au pied des parois, des **filons** à miner à la pioche (chaque personnage en a une) : cuivre et étain près de la
surface, or et gemmes tout en bas. Au bout des culs-de-sac, parfois une créature et un coffre, d'autant plus
riche que sa poche est profonde (`butin.ron`).

## Pourquoi

Un lieu « Deep Rock Galactic » pour varier des donjons bâtis : on y descend pour la matière, et la forme libre du
réseau donne des combats dans des espaces ouverts plutôt que dans des couloirs. K.1 à K.5 sont faits mais pas
encore parcourus dans le vrai client.

## En images

![La bouche de la grotte en surface et le boyau d’entrée](images/grotte-bouche.png)

![Une poche d’ambiance « cristaux », avec un filon au pied de la paroi](images/grotte-poche.png)
