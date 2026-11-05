+++
ordre = 3
titre = "Charrette, registre et classes"
icone = "voxygen/element/items/item_bag_leather_large.png"
schema = "circuit-charrette"
resume = "On arrive en fantôme, on engage une recrue, et le personnage appartient à l'équipe, pas au joueur."
design = ["classes.md"]
reglages = ["registre.ron", "classes.ron"]
taches = ["R"]
systemes = [
  "server/src/icebreaker_registre.rs",
  "server/src/icebreaker_charrette.rs",
  "server/src/icebreaker_classes.rs",
]

[[chiffre]]
cle = "recrues_par_jour"
nom = "Recrues par jour de carte"

[[chiffre]]
cle = "minimum_charrette"
nom = "Recrues au moins dans la charrette"

[[chiffre]]
cle = "places_registre"
nom = "Places au registre"

[[chiffre]]
cle = "reservations_par_compte"
nom = "Réservations par joueur"
+++

## La règle

Dans la campagne, on n'a pas « son » personnage. On arrive au village en **fantôme** (le spectateur de Veloren) et
on va à la **charrette**, où attendent les recrues du jour : classe au hasard, nom tiré au sort, équipement de
départ visible sur elles. En engager une l'inscrit au **registre** du village : un personnage que n'importe quel
joueur peut prendre, un seul à la fois.

Un joueur peut **réserver** quelques personnages pour lui seul. Les autres restent à tout le monde : si votre
guerrier est à l'infirmerie, vous prenez l'archer d'un camarade pour la prochaine sortie. Un personnage mort
revient au registre (voir la fiche À terre et mort).

Trois **classes** pour l'instant, décrites en données : guerrier (épée, trois postures), mage, archer. Une classe,
c'est un équipement de départ, les compétences débloquées de son arme, et l'ordre sur les touches 1, 2, 3 ; tout
le monde a aussi une potion, du fromage et une pioche pour les filons de la grotte.

## Pourquoi

Décision 0017 : le registre commun fait du village une vraie base d'équipe (on gère un roster, comme dans Darkest
Dungeon) et permet de changer de personnage pendant qu'un autre se soigne. Les neuf classes à combos de
`design/classes.md` restent une proposition pour plus tard.
