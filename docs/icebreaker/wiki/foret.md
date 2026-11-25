+++
ordre = 9
titre = "La forêt"
icone = "voxygen/element/ui/map/buttons/tree.png"
resume = "Une forêt à deux étages : clairières et sentiers au sol, plateformes et ponts dans les arbres. Une espèce l'habite, son chef attend dans la clairière du fond."
reglages = ["forets/foret.ron", "forets/formes.ron", "butin.ron"]
taches = ["FO"]
systemes = [
  "icebreaker-foret/src/lib.rs",
  "icebreaker-foret/src/grille.rs",
  "world/src/icebreaker/foret.rs",
  "world/src/site/plot/icebreaker_foret.rs",
  "world/src/icebreaker/butin.rs",
]

[[chiffre]]
cle = "camp"
nom = "Ennemis d'une clairière « camp » (de, à)"

[[chiffre]]
cle = "nid"
nom = "Ennemis d'une clairière « nid » (de, à)"

[[chiffre]]
cle = "garde"
nom = "Gardes autour du chef (de, à)"

[[chiffre]]
cle = "guetteurs"
nom = "Part des plateformes qui portent un tireur"

[[chiffre]]
cle = "coffres_hauts"
nom = "Part des cabanes et des bouts de branche qui portent un coffre"

[[chiffre]]
cle = "calme"
nom = "Blocs sans ennemi ni coffre autour de l'arrivée"
+++

## La règle

La forêt est le troisième lieu de notre fabrication, après le donjon et la grotte. Elle se joue sur **deux étages** :

- **au sol**, des clairières reliées par des sentiers qui serpentent entre les troncs ;
- **en hauteur**, des plateformes autour des arbres, reliées par des ponts de planches ou par de grosses branches
  sur lesquelles on marche.

On monte par un escalier en spirale autour d'un tronc, par une échelle, ou par l'intérieur d'un **très grand arbre
creux** : une porte au pied, un escalier dedans, une porte à chaque plateforme. Au bord de certaines plateformes,
des **cabanes** éclairées par une lanterne. Certaines branches ne mènent nulle part : elles s'arrêtent dans un
bouquet de feuilles.

**Tomber n'est pas mourir** : on prend les dégâts de chute habituels, on atterrit sur un sol jouable, et l'on peut
toujours remonter. Depuis l'arrivée, chaque clairière, chaque plateforme et chaque cabane s'atteint à pied : le
générateur le vérifie, et refait la forêt sinon.

L'équipe arrive dans une clairière **au bord** de la forêt. Personne autour, aucun coffre : c'est la zone calme.

## Qui y vit

Une forêt est habitée par **une seule espèce**, tirée par sa graine parmi nos six : humain, orc, nain, elfe, draugr,
danari. Pas de bêtes.

- Chaque clairière a son groupe. Un **camp** mêle guerriers, archers et mages, dispersés dans la clairière ; un
  **nid** est une meute de guerriers serrés au milieu.
- Sur les plateformes, des **tireurs** (archers et mages), postés hors des troncs : ils voient le sol.
- Dans la clairière **du fond**, la plus loin de l'arrivée en suivant les sentiers, le **chef** et ses gardes.

**Tuer le chef accomplit la mission**, comme dans la grotte. On rentre ensuite par `/rentrer`.

Chaque clairière a un **coffre**, et quelques autres se cachent en hauteur : au fond d'une cabane, au bout d'une
branche sans issue. Coffres et créatures donnent le butin de `butin.ron`, plus riche loin de l'arrivée ; le chef
donne bien plus.

## Régler la forme

`.\icebreaker.ps1 foret` ouvre l'**atelier** (http://localhost:8099) : un curseur par réglage, la forêt en 3D qui
se refait à chaque changement, une case « Feuillage » pour voir l'étage haut. C'est le même générateur que le jeu.
On y règle la densité des arbres, la part de très grands arbres et d'arbres creux, le nombre de clairières, la part
d'arbres à plateformes, leur hauteur, le nombre de niveaux, la part de ponts, de branches, de cabanes, de lanternes.

Des réglages qui plaisent s'enregistrent en **preset** : « Enregistrer le preset » l'écrit dans `formes.ron`, avec
un commit. Quatre presets au départ :

- **clairsemee** : des arbres espacés, quelques plateformes de guet ; on joue surtout au sol ;
- **passerelles** : une forêt serrée dont l'étage haut est un vrai réseau ;
- **geante** : de très grands arbres creux, trois niveaux de plateformes, de longs ponts ;
- **immense** : des troncs larges comme des tours, des plateformes à plus de cent blocs du sol.

Au village, une expédition vers la Forêt choisit sa forme au panneau Départ : « au hasard » ou l'un des presets
(`/expedition 6 <preset>`). Pour essayer la forêt seule : `.\icebreaker.ps1 donjon -Site Foret -Profil geante`.

L'atelier ne montre pas les habitants : leur nombre se règle dans `foret.ron`.

## Pourquoi

Un lieu à l'air libre, tout en hauteur, pour varier des donjons bâtis et de la grotte : on y choisit entre le sol,
où les groupes attendent, et les arbres, d'où l'on voit loin mais d'où l'on tombe. Nos propres arbres plutôt que
ceux de Veloren : leurs branches partent au hasard et couperaient ponts et plateformes, et l'atelier ne montrerait
pas au bloc près ce qu'on joue.

## En images

![Une forêt « geante » vue de dessus : clairières en vert clair, sentiers, troncs, plateformes et ponts](images/foret-dessus.png)

![Un tireur posté sur une plateforme, autour d'un très grand arbre](images/foret-guetteur.png)

![D'une plateforme à l'autre : ponts, lianes, et le sol en contrebas](images/foret-plateformes.png)

![Une clairière et son coffre, au pied des racines](images/foret-clairiere.png)
