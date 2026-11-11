+++
ordre = 3
titre = "Classes, armes et compétences"
icone = "voxygen/element/weapons/swordshield.png"
resume = "Le jeu d'armes en mains donne deux compétences, la classe un ultime et un passif ; la maîtrise des armes se gagne en expédition et se dépense à la salle d'armes."
design = ["classes.md"]
reglages = ["salle_armes.ron", "registre.ron"]
taches = ["A"]
systemes = [
  "common/src/icebreaker_competences.rs",
  "server/src/icebreaker_registre.rs",
  "server/src/icebreaker_charrette.rs",
  "server/src/sys/icebreaker_armes.rs",
  "server/src/icebreaker_salle.rs",
  "voxygen/src/icebreaker_competences.rs",
]

[[chiffre]]
cle = "premier_point"
nom = "Maîtrise pour le premier point d'arme"

[[chiffre]]
cle = "pas_point"
nom = "Maîtrise en plus pour chaque point suivant"

[[chiffre]]
cle = "maitrise_par_experience"
nom = "Maîtrise par point d'expérience reçu"

[[chiffre]]
cle = "cout_debloquer"
nom = "Points pour débloquer une compétence"

[[chiffre]]
cle = "cout_niveaux"
nom = "Points pour les niveaux 2, 3, 4, 5"

[[chiffre]]
cle = "prix_jeu_armes"
nom = "Or pour apprendre un jeu d'armes"

[[chiffre]]
cle = "prix_ultime"
nom = "Or pour apprendre un ultime"

[[chiffre]]
cle = "prix_passif"
nom = "Or pour apprendre un passif"

[[chiffre]]
cle = "competences_tirees"
fichier = "registre.ron"
nom = "Compétences d'arme tirées avec une recrue"
+++

## La règle

**Le jeu d'armes.** C'est ce qu'on tient en mains : épée et bouclier, grande épée, marteau à deux mains, hache à deux
mains pour le guerrier, bâton ou sceptre pour le mage, arc pour l'archer. Il donne les attaques de
base, celles de Veloren (clic gauche, clic droit, garde), et un lot de **compétences d'arme** à choisir. On en place
**deux**, sur les touches 1 et 2. En expédition on change de jeu d'armes en changeant d'arme dans l'inventaire : les
compétences suivent ce qu'on tient. Un jeu d'armes qu'on n'a pas appris, ou qui n'est pas celui de sa classe, ne
donne que les attaques de base : les touches 1 et 2 restent vides et un message le dit.

**La classe.** Elle ne change pas avec l'arme. Elle donne un **ultime** sur la touche 3, à longue recharge (une ou
deux minutes), un **passif** (un seul est actif à la fois) et un **inné** qu'on ne choisit pas : ce qui fait la
classe, par exemple des PV en plus et plus de tenue pour le guerrier. Ultimes et passifs n'ont pas de niveau : on en
apprend d'autres.

**La recharge.** Toute compétence en a une, visible dans la barre : la case se grise et affiche les secondes qui
restent. Le passif actif est dans une case ronde, à gauche de la barre.

![La barre d'une guerrière à la hache : la compétence de la touche 1 en recharge (4 s), puis son ultime, bord doré (58 s)](images/classes-barre.png)

**Les niveaux.** Une compétence d'arme va du niveau 1 au niveau 5. Aux niveaux 2 et 4, des chiffres changent (dégâts,
portée, recharge). Aux niveaux 3 et 5, la compétence **se transforme** : un coup de plus, une zone plus large, un
effet nouveau (saignement, ralentissement). La page Classes et jeux d'armes de la rubrique Compétences dit, pour
chaque compétence, ce qu'apporte chaque niveau.

**La maîtrise et les points d'arme.** Tuer des ennemis en expédition, avec un jeu d'armes en mains, fait monter la
**maîtrise de ce jeu d'armes** : la grande épée et l'épée et bouclier montent chacune de leur côté. Chaque palier de
maîtrise donne un **point d'arme** : le premier à 100, puis chaque point demande 50 de plus que le précédent. Les
mannequins ne meurent pas : ils ne donnent rien. Le gain est annoncé en jeu et le bilan du retour de donjon le
résume.

**La salle d'armes.** C'est le seul endroit où l'on dépense, au village. Des points d'arme servent à **débloquer** une
compétence du jeu d'armes ou à la **monter** d'un niveau ; de l'**or** de la bourse sert à **apprendre** un jeu d'armes
(ses armes sont alors remises), un ultime ou un passif. C'est là aussi qu'on place ses deux compétences, son ultime
et son passif. Chaque ligne dit ce que fait la compétence, ce qu'apporte son niveau suivant et ce qu'il coûte ; un
cadre doré marque les deux compétences placées. Les chiffres sont dans l'encart ci-dessus.

![La salle d'armes d'une mage au sceptre : ses deux compétences placées, une troisième à débloquer](images/classes-salle.png)

**Les recrues.** Chaque recrue de la charrette tire un petit lot au hasard : un jeu d'armes qu'elle maîtrise, deux
compétences, un ultime, un passif, et les armes qui vont avec, qu'elle porte devant la charrette. Deux guerriers
peuvent donc jouer très différemment. Ensuite tout s'apprend à la salle d'armes.

**La mort.** Elle reste définitive : la maîtrise et les niveaux partent avec le personnage. Pas de montée de niveau
en expédition, tout se règle au village.

L'arbre de compétences de Veloren est coupé pour les personnages du registre, et un donjon lancé seul, hors registre,
ne change pas.

## Où en est-on

Les trois classes sont dans les données : le guerrier (quatre jeux d'armes), le mage (bâton, et sceptre de soutien)
et l'archer (arc), chacun avec quatre compétences par jeu d'armes et leurs niveaux, trois ultimes, un inné et quatre
passifs. La barre, le tirage des recrues, la maîtrise, les niveaux et **la salle d'armes** (septième bâtiment du
village : débloquer, monter, apprendre, placer) sont en place. Tout a été essayé en test et au client texte, de la
charrette au retour d'expédition ; dans le vrai client, la barre et la salle d'armes ont été vues en capture, mais
personne n'y a encore joué à la main : le reste est dans « À voir en jeu ». Le mage et l'archer n'ont pas encore
frappé de vraies cibles : leurs chiffres sont un premier jet.

Questions ouvertes pour l'équipe : les chiffres de départ, tous à régler en jouant (Q43) ; trois choix de l'agent à
confirmer, dont un jeu d'armes non appris qui ne donne que les attaques de base (Q44) ; un ultime lancé avec une arme
qui n'est pas celle de son geste (Q45) ; la bascule entre deux jeux d'armes et les jeux « deux épées » et « hache +
épée » (Q46) ; les maquettes de la barre et des panneaux (Q47) ; les compétences qui blessent les habitants du
village (Q48).

## Sous le capot

Chaque jeu d'armes est un fichier de `assets/icebreaker/jeux_armes/` : sa classe, ce qu'on tient, les armes remises
et ses compétences, chacune avec son nom, sa description, sa recharge, la capacité de Veloren qu'elle lance et ses
niveaux. Un ultime ou un passif vient de `ultimes/<classe>.ron` et `passifs/<classe>.ron` ; un passif est une liste
d'effets permanents de Veloren (PV max, réduction de dégâts, vitesse). Une classe de plus, c'est de nouveaux
fichiers : aucun code à changer. Les points de maîtrise et les prix sont dans `salle_armes.ron`.

Une compétence n'est pas une nouvelle attaque : elle réutilise une **capacité de Veloren** (le balayage lourd de
l'épée, la fureur de la hache) avec son animation. Un niveau 2 ou 4 la lance avec des multiplicateurs ; un niveau 3
ou 5 la remplace par une copie modifiée du fichier de la capacité, sous `assets/icebreaker/abilities/<classe>/`.

À chaque lancer de la touche 1, 2 ou 3, un crochet unique dans le code de Veloren (`handle_ability`) regarde ce que
le personnage a appris, le niveau, la recharge, et lance la bonne capacité. La recharge est tenue par le serveur : le
client la lit pour griser la case, et une touche tenue ne relance pas.

Ce que le personnage a appris est dans sa **fiche** du registre : par jeu d'armes, la maîtrise, les points dépensés,
les compétences connues avec leur niveau et les deux placées ; puis les ultimes et les passifs connus et choisis. Une
ancienne fiche reçoit le lot par défaut de sa classe. Le serveur en tire deux composants : l'un, visible du seul
joueur, porte ce qu'il a appris et ses fins de recharge ; l'autre porte l'inné et le passif et se voit de tous, sinon
les PV maximum d'un coéquipier seraient faux. Les effets du passif sont appliqués à chaque tic par le système des
effets de Veloren.

La maîtrise vient de l'expérience que Veloren attribue quand un ennemi meurt : on en crédite le jeu d'armes en
mains. Le nombre de points d'arme se calcule depuis la maîtrise avec les seuils de `salle_armes.ron`, rien d'autre
n'est stocké. Le tirage d'une recrue pioche dans les jeux d'armes de sa classe ; le nombre de compétences tirées est
dans `registre.ron`.

La page Classes et jeux d'armes du wiki est lue directement dans ces fichiers : elle montre toutes les classes qu'ils
décrivent, et celle de Veloren qu'une compétence reprend se retrouve dans les tableaux de la rubrique Compétences.

## Pourquoi

Décision 0020 : avant, une classe n'était qu'un équipement de départ et quelques compétences de Veloren, sans
recharge ni niveau ; les classes se ressemblaient et améliorer un personnage n'avait aucun intérêt. On veut des
classes à forte identité, un vrai ressenti par arme façon Guild Wars 2, et des recrues qui ne se ressemblent pas.
Le format de deux compétences d'arme, un ultime et un passif garde la barre lisible, et faire gagner la maîtrise par
jeu d'armes donne une raison d'essayer une autre arme.

## Voir aussi

Les fiches Charrette, registre et classes, Combat létal et verrouillage, et À terre, mort et sac du mort ; la page
Classes et jeux d'armes de la rubrique Compétences.
