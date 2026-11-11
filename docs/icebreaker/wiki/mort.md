+++
ordre = 5
titre = "À terre, mort et sac du mort"
icone = "voxygen/element/ui/generic/icons/skull_2.png"
schema = "frise-mort"
resume = "Tomber n'est pas mourir : un allié a trente secondes pour vous relever. Mourir, c'est laisser son sac en bas."
design = ["combat.md"]
reglages = ["mort.ron"]
taches = ["T"]
systemes = [
  "common/src/icebreaker_mort.rs",
  "server/src/sys/icebreaker_mort.rs",
  "server/src/icebreaker_mort.rs",
  "voxygen/src/icebreaker_mort.rs",
]

[[chiffre]]
cle = "delai_a_terre"
nom = "À terre avant de mourir"
unite = "s"

[[chiffre]]
cle = "pv_releve"
nom = "PV rendus à la relève"
pourcent = true

[[chiffre]]
cle = "duree_fouille"
nom = "Fouiller un sac"
unite = "s"
+++

## La règle

Quand vos points de vie tombent à zéro, vous ne mourez pas tout de suite : vous **tombez à terre** et vous rampez.
Une barre et un compte à rebours s'affichent (« Mort dans N s »), pour vous et au-dessus de votre corps pour vos
alliés. Un allié qui vient vous relever (touche d'interaction, E) vous remet debout avec un quart de vos points de
vie. Personne ne vient dans les trente secondes : vous mourez.

Mourir en expédition laisse un **sac du mort** au sol, avec tout ce que vous portiez dans l'inventaire ; votre
équipement porté reste sur le personnage. Un allié fouille le sac en trois secondes et en prend le contenu : rien
n'est perdu si l'équipe tient le coup, tout est perdu si elle rentre sans.

Une fois mort, l'écran de mort le dit sans détour : ce personnage est **perdu pour de bon**, il sort du registre.
Deux choix, rappelés avec leur touche :

- **Espace** (la touche de réapparition) : rentrer au village en fantôme, et y choisir un autre personnage de votre
  réserve.
- **Clic gauche** : regarder un allié encore vivant, en passant de l'un à l'autre ; un bandeau en haut de l'écran
  dit qui vous regardez et où en est sa vie. Sans allié en vie, ce choix est grisé.

Si le **dernier joueur vivant tombe**, l'expédition a péri : tout le monde rentre au village en fantôme, et les sacs
restent en bas jusqu'à ce que l'emplacement du donjon soit remis à neuf. Chacun retrouve au village la fenêtre
« Retour d'expédition », qui le dit : rien de rapporté, un personnage perdu.

## Pourquoi

C'est la règle qui rend le combat létal (voir la fiche Combat) jouable à plusieurs sans être punitif : une erreur
coûte un relevé, pas une mort. Elle oblige à rester groupés et donne une vraie raison de revenir sur ses pas pour
récupérer le sac d'un camarade.

## En images

![À terre : le temps qui reste, ce qu'on peut attendre, la touche d'abandon](images/interface-a-terre.png)

![L'écran de mort : ce qui est perdu, et les deux choix avec leur touche](images/interface-mort.png)

![Mort, on regarde un allié : son nom, sa vie, les deux touches](images/interface-allie-regarde.png)

![Le bilan d'une expédition qui a péri, retrouvé au village](images/interface-bilan-peri.png)

![Un allié à terre qui rampe, la barre et le compte à rebours « Mort dans N s » au-dessus de lui](images/mort-a-terre.png)

![Le sac du mort au sol et la fenêtre de fouille, vus par un allié](images/mort-sac.png)
