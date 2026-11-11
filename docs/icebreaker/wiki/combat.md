+++
ordre = 4
titre = "Combat létal et verrouillage"
icone = "voxygen/element/weapons/swords_crossed.png"
schema = "portee-verrouillage"
resume = "Quelques coups suffisent, dans un sens comme dans l'autre. On verrouille sa cible façon Dark Souls."
design = ["combat.md"]
reglages = ["combat.ron", "verrouillage.ron"]
taches = ["L", "R.6", "R.7", "R.8"]
systemes = [
  "common/src/icebreaker_combat.rs",
  "voxygen/src/session/icebreaker_lock.rs",
]

[[chiffre]]
cle = "degats_contre_creatures"
nom = "Dégâts contre les créatures"
unite = "×"

[[chiffre]]
cle = "degats_contre_joueurs"
nom = "Dégâts contre les joueurs"
unite = "×"

[[chiffre]]
cle = "creatures_projectile"
nom = "Tirs contre les créatures"
unite = "×"

[[chiffre]]
cle = "creatures_rayon"
nom = "Rayons contre les créatures, par touche"
unite = "×"

[[chiffre]]
cle = "plafond_coup_creature"
nom = "Part des PV qu'un coup peut retirer à une créature, au plus"

[[chiffre]]
cle = "rayon"
fichier = "verrouillage.ron"
nom = "Portée du verrouillage"
unite = "blocs"

[[chiffre]]
cle = "rayon_perte"
fichier = "verrouillage.ron"
nom = "Le verrouillage saute à"
unite = "blocs"
+++

## La règle

Dans Veloren, un combat dure longtemps : on tape, on boit des potions, on retape. Chez nous, les dégâts de toutes
les attaques sont **multipliés** : une épée de départ tue un gnarling en trois coups environ, et un gnarling tue un
joueur de 100 PV en trois ou quatre coups. Seules les attaques comptent : les pièges et l'environnement font les
dégâts de Veloren. Une créature plus forte encaisse plus, comme avant.

Contre les créatures, le multiplicateur dépend de l'attaque : on reste plus faible de loin. Il faut cinq à six
flèches ou boules de feu pour un gnarling, et le lance-flammes, qui touche trois fois par seconde, fait à chaque
touche la moitié d'une flèche. Surtout, **un coup ne retire jamais plus de 40 % des PV d'une créature** : même un
tir chargé ne tue pas d'un coup. Ce plafond monte avec les bonus de dégâts (potions, puis forge et âmes) : c'est en
gagnant en puissance qu'on finit par abattre un ennemi d'un seul coup.

La conséquence, c'est qu'on ne fonce plus. On **verrouille** l'ennemi le plus proche du centre de l'écran (touche W
par défaut) : la caméra le suit, la molette passe à l'ennemi voisin, et un anneau doré respire sur lui. Trop loin,
le verrouillage saute.

Le jeu d'armes qu'on tient donne les attaques de base et **deux compétences** sur les touches 1 et 2 ; la classe
donne l'**ultime** de la touche 3. Toutes ont une recharge. Les règles sont dans la fiche
[Classes, armes et compétences](classes.html) ; chaque classe, ses jeux d'armes et leurs compétences sont dans la
page [Classes et jeux d'armes](../competences/classes.html), et toutes les capacités de Veloren dans la rubrique
[Compétences](../competences/index.html).

## Pourquoi

Le pilier, c'est que la victoire vient de la coordination, pas du matériel : un combat court où chaque coup compte
oblige à se couvrir, à relever (voir la fiche À terre et mort) et à choisir ses affrontements. Le verrouillage
rend ces combats courts lisibles à la troisième personne.

## En images

![L’anneau de verrouillage sur un gnarling, la caméra tournée vers lui](images/combat-verrouillage.png)

![Un gnarling tué en trois coups d’épée de départ](images/combat-trois-coups.png)
