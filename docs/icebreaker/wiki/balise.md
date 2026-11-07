+++
ordre = 8
titre = "Balise lumineuse"
icone = "voxygen/element/ui/bag/backgrounds/lantern.png"
resume = "Une touche pose à vos pieds une lumière de la couleur choisie, pour retrouver son chemin dans le donjon."
design = ["donjon.md"]
reglages = ["balise.ron"]
taches = ["J"]
systemes = [
  "common/src/icebreaker_balise.rs",
  "server/src/icebreaker_balise.rs",
  "server/src/sys/icebreaker_balise.rs",
]

[[chiffre]]
cle = "delai_recharge"
nom = "Recharge après une pose"
unite = "s"

[[chiffre]]
cle = "max_au_sol"
nom = "Balises au sol par joueur"

[[chiffre]]
cle = "max_total"
nom = "Balises au sol en tout"
+++

## La règle

En arrivant dans un donjon, chaque personnage reçoit une **balise lumineuse**. La touche « Poser une balise » (`<`
par défaut) en pose une à vos pieds : elle reste au sol et éclaire dans votre couleur. La touche « Couleur de la
balise » (F9), ou `/balise` dans le chat, ouvre un panneau pour choisir la couleur : blanc, rouge, vert, bleu, jaune
ou violet.

La balise n'a **qu'une charge**. Après une pose, il faut attendre dix secondes avant d'en poser une autre ; le chat
dit « Balise prête » quand c'est possible. Ramasser une balise posée (E) l'éteint et **rend la charge tout de
suite**. N'importe quel joueur peut ramasser une balise, même celle d'un autre.

Un joueur a au plus deux cents balises au sol, et il n'y en a pas plus de deux cents en tout, tous joueurs
confondus : au-delà, la plus ancienne s'éteint. Les balises restent jusqu'à la fin
de l'expédition. On ne peut pas en poser au village.

## Pourquoi

Les salles sombres et les couloirs qui se ressemblent font qu'on se perd vite. Une balise marque le chemin du retour,
une salle déjà fouillée ou un point de rendez-vous. La couleur dit qui l'a posée ou ce qu'elle veut dire. Rendre la
charge quand on ramasse une balise invite à les reprendre en revenant sur ses pas, au lieu d'en semer partout.

## En images

![Une balise posée au sol qui éclaire un couloir du donjon en vert](images/balise-sol.png)

![Le panneau de choix de la couleur de la balise](images/balise-couleurs.png)
