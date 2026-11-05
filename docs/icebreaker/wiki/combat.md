+++
ordre = 4
titre = "Combat létal et verrouillage"
icone = "⚔️"
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

La conséquence, c'est qu'on ne fonce plus. On **verrouille** l'ennemi le plus proche du centre de l'écran (touche W
par défaut) : la caméra le suit, la molette passe à l'ennemi voisin, et un anneau doré respire sur lui. Trop loin,
le verrouillage saute.

Chaque classe a ses attaques de base et **trois compétences** sur les touches 1, 2 et 3. Le guerrier a trois
postures (riposte, balayage lourd, tranche-tourbillon) ; le mage et l'archer, les premières compétences de leur
arme dans l'ordre de Veloren.

## Pourquoi

Le pilier, c'est que la victoire vient de la coordination, pas du matériel : un combat court où chaque coup compte
oblige à se couvrir, à relever (voir la fiche À terre et mort) et à choisir ses affrontements. Le verrouillage
rend ces combats courts lisibles à la troisième personne.
