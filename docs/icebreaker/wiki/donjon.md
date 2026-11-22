+++
ordre = 6
titre = "Le donjon généré"
icone = "voxygen/element/ui/map/buttons/dungeon.png"
schema = "etages-donjon"
resume = "Un graphe de mission, des salles préfaites assemblées sur plusieurs étages, des verrous, des dormeurs."
design = ["generation.md", "creatures.md"]
reglages = ["donjons/generation.ron", "donjons/formes.ron", "butin.ron"]
taches = ["G", "F", "B.5", "B.14"]
systemes = [
  "world/src/icebreaker/dungeon/mod.rs",
  "world/src/icebreaker/dungeon/graph.rs",
  "world/src/icebreaker/dungeon/assemble.rs",
  "world/src/icebreaker/dungeon/populate.rs",
  "world/src/icebreaker/butin.rs",
  "world/src/site/plot/icebreaker_dungeon.rs",
  "server/src/sys/icebreaker_sleep.rs",
]

[[chiffre]]
cle = "cell_size"
nom = "Blocs par case"

[[chiffre]]
cle = "level_height"
nom = "Blocs par étage"

[[chiffre]]
cle = "profondeur"
nom = "Roche au moins au-dessus du donjon (blocs)"
+++

## La règle

Le lieu « Genere » est **notre** donjon, pas un de ceux de Veloren. Il naît en trois temps, repris du projet Godot :

1. Un **graphe de mission** : un chemin principal de l'entrée à l'objectif, des branches en cul-de-sac (dont celles
   qui cachent une clé), des verrous, des boucles, des chutes à sens unique vers le bas. La taille (small, medium,
   large) fixe les fourchettes.
2. L'**assemblage** : des salles préfaites (plans texte, un caractère par case, dans `assets/icebreaker/donjons/salles/`)
   posées une à une sur une grille à plusieurs étages, reliées par des couloirs courts et des escaliers propres.
   Depuis F.5, les salles ont des formes variées.
3. Le **peuplement** : créatures et coffres selon le rôle de chaque salle (caserne, cuisine, antre du chef…), un
   décor par rôle, un éclairage qui dépend des habitants. Plus un coffre est loin de l'entrée, plus il est riche
   (trois paliers, `butin.ron`) : de l'or, et une prise utile en bas (potion, bombe, nourriture) ou au village
   (matériaux, arme ou armure). Les créatures aussi lâchent le butin de leur palier ; le chef et le boss, gros.
   Les caisses en bois du décor donnent une petite prise (or, nourriture, bois, pierre, potion).

En bas, les **portes à clé** sont des murs de blocs avec leur serrure : il faut trouver la clé dans une branche.
Beaucoup de créatures **dorment** : elles ronflent sur le dos et se réveillent si on les frappe, si un combat fait
du bruit à portée, ou si on s'approche trop (moins si on est accroupi) ; une réveillée réveille ses voisines. La
**victoire**, c'est atteindre l'objectif de la mission. Le donjon est tout entier **sous le sol**, sans bouche en surface : l'expédition
arrive dans l'**abri**, une petite salle sûre (un feu, des couchages) accrochée à la salle d'entrée, comme dans la
grotte.

Comme la grotte, le donjon a des **formes** (`donjons/formes.ron`) : « plat », « profond », « labyrinthe »…
Chacune change le graphe, l'écart entre les salles ou la roche au-dessus. La graine en tire une, ou l'équipe la
choisit au départ. On les règle dans l'**atelier du donjon** (`.\icebreaker.ps1 donjon-atelier`). L'atelier montre
le plan étage par étage, avec des curseurs, et enregistre la forme.

## Ce qui reste à faire

L'**intelligence du donjon** (G.11 à G.16, `design/creatures.md` et `design/menace.md`) : appel à l'aide,
encerclement, déroute, alerte, embuscades. Et tout ce que propose `PLAN-DONJON.md` pour rendre la descente
intéressante : ressources, inventaire à formes, foreuse, trous, monstres à coop.

## En images

![Une salle de garde devant une porte verrouillée, avec ses dormeurs](images/donjon-salle-garde.png)
