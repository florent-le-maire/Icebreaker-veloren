+++
ordre = 7
titre = "La grotte"
icone = "voxygen/element/weapons/pickaxe.png"
resume = "Des cavernes creusées dans la roche, plates ou en gouffre selon la graine, des filons à la pioche, un chef au fond. Façon Deep Rock Galactic."
reglages = ["grottes/grotte.ron", "grottes/minerais.ron", "grottes/formes.ron", "butin.ron"]
taches = ["K", "U", "B.5", "B.14"]
systemes = [
  "icebreaker-caverne/src/lib.rs",
  "icebreaker-caverne/src/filon.rs",
  "common/src/icebreaker_minerais.rs",
  "world/src/icebreaker/grotte/mod.rs",
  "world/src/site/plot/icebreaker_grotte.rs",
  "world/src/icebreaker/butin.rs",
]

[[chiffre]]
cle = "emprise"
nom = "Tronçons réservés de chaque côté de l'entrée"

[[chiffre]]
cle = "par_poche"
nom = "Créatures par zone, à l'entrée"

[[chiffre]]
cle = "par_niveau"
nom = "Créatures en plus par niveau"

[[chiffre]]
cle = "niveaux"
nom = "Niveaux de la grotte, de l'entrée au fond (minerais et butin plus riches)"
+++

## La règle

La grotte est le deuxième lieu de notre fabrication, sans salles ni cases : tout est creusé en blocs. Chaque grotte
tire une **forme** par sa graine :

- le **gouffre** : des zones empilées, reliées par des puits où l'on descend en lacets le long de la paroi ;
- la **plaine** : d'immenses salles basses, des forêts de stalactites, presque pas de dénivelé ;
- le **chaos** : des salles pleines de bosses, de trous et de ponts de roche, sur plusieurs hauteurs ;
- le **mixte** : un peu de tout.

L'équipe arrive dans une **salle sûre**, sous terre, à un endroit tiré par la graine : aucune créature autour, et le
chef à l'autre bout. La grotte n'a pas de sortie en surface : on rentre par `/rentrer`. De la salle sûre, un
**sentier** relie toutes les zones : on peut toujours le suivre à pied, à l'aller comme au retour. Falaises, trous et replats sont à côté. De petites **niches** s'ouvrent au bord des
zones : parfois une bête, parfois un coffre.

Chaque grotte tire aussi une **ambiance** (champignons, cristaux, ossuaire) : la couleur de la roche, la roche
luisante, les sprites au sol et au plafond, les créatures. Le **chef** attend dans la zone la plus loin de l'entrée.
Le tuer, c'est la victoire.

Dans les parois, des **filons** à miner à la pioche (chaque personnage en a une), comme dans Deep Rock Galactic :
une bande de minerai prise dans la roche, de la couleur de son minerai, qui serpente le long de la paroi et ressort
çà et là d'un bloc. Elle part **morceau par morceau, là où l'on tape** : chaque bloc a la dureté de son minerai et
donne son minerai en cédant ; les minerais courants (cuivre, étain, fer, charbon) partent par gros morceaux, le
cobalt et l'argent par petits, l'or et les gemmes bloc par bloc. Chaque bloc de filon a le modèle de son minerai : une gangue bosselée à pépites brillantes pour les métaux,
des blocs noirs à facettes pour le charbon, des cristaux qui luisent pour les gemmes. En creusant la roche autour, le filon reste. Un à
trois filons par zone ; cuivre et étain près de l'entrée, or et gemmes au fond. Chaque minerai se règle à part
(`minerais.ron` : dureté, ce qu'il donne, longueur, épaisseur, profondeur, sinuosité, relief, où il
apparaît) ; l'atelier des grottes montre des filons en cubes de couleur, posés comme dans le jeu mais pas aux
mêmes places.

Une partie des passages entre deux zones est **bouchée par la roche** : la galerie s'arrête, il faut creuser. Dans la
grotte, la pioche casse tout, bloc après bloc, contre de l'énergie ; à sec, on attend. Le minerai, lui, se mine sans énergie. La salle sûre où l'on arrive
s'ouvre toujours sur sa zone.

Chaque grotte cache une **salle au minerai**, au bout d'une galerie murée : ses parois couvertes de filons : fer, cuivre,
étain et charbon surtout, de l'argent, un peu d'or, de cobalt et de gemmes. Personne ne la garde : son mur suffit. Coffres et créatures donnent le butin de leur niveau (`butin.ron`), le chef bien plus.

## Régler la forme

`.\icebreaker.ps1 caverne` ouvre l'**atelier** : une page avec un curseur par réglage et la grotte en 3D, qui se
refait à chaque changement. C'est le même générateur que le jeu. Trois curseurs « Étendue sur X / Y / Z » donnent
la forme d'ensemble : ronde à 1 partout, en boyau ou en galette quand on en resserre un.

Des réglages qui plaisent s'enregistrent en **preset** (un nom, un poids) : « Enregistrer le preset » l'écrit dans
`formes.ron`, avec un commit. Au village, une expédition vers la Grotte choisit sa forme au panneau Départ : « au
hasard » ou l'un des presets. En arrivant, l'équipe lit le nom du preset et la graine de sa grotte ; les mêmes dans
l'atelier montrent sa forme (au terrain près : en jeu la grotte se cale sous le sol réel).

## Pourquoi

Un lieu « Deep Rock Galactic » pour varier des donjons bâtis : on y descend pour la matière, et la forme libre
donne des combats dans des espaces ouverts plutôt que dans des couloirs. La première version (jalon K) faisait des
poches reliées par des boyaux : on y sentait encore des salles et des couloirs. Le jalon U l'a remplacée.

## En images

Les captures ci-dessous datent de la première forme (jalon K) ; elles sont à refaire.

![La bouche de la grotte en surface et le boyau d’entrée](images/grotte-bouche.png)

![Une poche d’ambiance « cristaux », avec un filon au pied de la paroi (avant U.19 : des sprites)](images/grotte-poche.png)

![Debout sur un filon de cuivre : chaque bloc du filon a le modèle de son minerai (U.24)](images/grotte-filon.png)
