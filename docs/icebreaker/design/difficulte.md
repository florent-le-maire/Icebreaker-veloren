# Game design — Difficulté des expéditions

Décidé le 2026-11-25 (Flo). Jalon DF, tâches dans `PLAN-PLAYTEST.md` (§ 8). Chiffres :
`assets/icebreaker/difficulte.ron`.

## La règle

Chaque expédition a une **difficulté de 1 à 6**. Le groupe la choisit librement au départ, comme la forme du lieu.
C'est un seul curseur : il vaut pour le donjon généré, la grotte et la forêt, et il remplace la « menace 1 à 5 » du
donjon (`design/generation.md`, « Paramètres de mission »).

On dit « difficulté », jamais « niveau » : le mot sert déjà au personnage, à l'alerte, aux jeux d'armes et aux
filons.

La difficulté règle cinq choses :

| Ce qui change | Comment | Crans |
|---|---|---|
| Nombre d'ennemis | un facteur sur les effectifs du lieu ; le chef et le boss restent uniques | 6 |
| PV des ennemis | un facteur sur les PV de leur fiche | 6 |
| Dégâts des ennemis | un facteur sur les dégâts qu'ils infligent | 6 |
| Classes et compositions | un poids par rôle (guerrier, archer, mage) quand le lieu tire un ennemi ; poids 0 : absent | 4 |
| Habileté | le rang des compétences de l'ennemi (1, 3, 5), et son cran dans `ia/profils.ron` : délai de réaction, temps mort entre deux attaques, roulade, parade, sens un peu plus fins (jalon IA) | 3 |

Le **terrain** ne dépend pas de la difficulté dans cette version : la forme et la taille se choisissent à part. Un
labyrinthe rend l'expédition plus longue, pas plus dangereuse, et le jeu n'a pas encore de piège.

## Le tableau de départ

Linéaire, à régler avec le classeur d'équilibrage (onglet Difficulté). **La difficulté 3 est le jeu d'avant ce
réglage** : les essais l'ont jugé très dur (2026-11-13), les difficultés 1 et 2 sont donc plus douces.

| Difficulté | Nombre | PV | Dégâts | Habileté | Guerrier / archer / mage |
|---|---|---|---|---|---|
| 1 | ×0,70 | ×0,60 | ×0,35 (mage ×0,60) | rang 1 | 3 / 1 / 0 |
| 2 | ×0,85 | ×0,80 | ×0,65 (mage ×0,80) | rang 1 | 3 / 2 / 1 |
| 3 (par défaut) | ×1 | ×1 | ×1 | rang 1 | le tirage du lieu |
| 4 | ×1,15 | ×1,20 | ×1,15 | rang 3 | le tirage du lieu |
| 5 | ×1,30 | ×1,40 | ×1,30 | rang 3 | 2 / 2 / 2 |
| 6 | ×1,45 | ×1,60 | ×1,45 | rang 5 | 2 / 2 / 3 |

Le chef garde ses compétences (rang 5) à toutes les difficultés ; il prend les facteurs de PV et de dégâts.

Les dégâts des difficultés 1 et 2 ne sont plus sur la droite (Flo, 2026-11-25, essai au lieu d'entraînement) : en
difficulté 1 un guerrier ou un archer tuait un mage en un coup. D'après le classeur, la flèche chargée de l'archer
fait 135 à la difficulté 3 contre un mage de 90 PV qui en retire 17 % : à ×0,35 il faut 3 flèches chargées. Le mage
ennemi frappe déjà moins fort : il a son propre facteur (`degats_roles`), moins baissé. **À la difficulté 3, cette
flèche tue toujours un mage en un coup.**

## Ce qu'il faut savoir en réglant

- **Les effets se multiplient.** En difficulté 6, un groupe a 1,45 fois plus d'ennemis, qui ont chacun 1,6 fois plus
  de PV : il y a 2,3 fois plus de PV à abattre, pendant que 1,45 fois plus d'ennemis frappent 1,45 fois plus fort
  (2,1 fois plus de dégâts reçus par seconde), sans compter l'habileté. « Linéaire » ligne par ligne ne l'est donc
  pas pour le joueur : c'est le temps pour tuer et le temps pour mourir du classeur qu'il faut regarder.
- **Les poids ne créent pas de rôle.** Ils pèsent un tirage entre les rôles que le lieu propose à cet endroit : une
  salle de guerriers reste une salle de guerriers. Si la difficulté interdit tous les rôles proposés (un poste qui
  n'a que des mages, en difficulté 1), le tirage du lieu est gardé.
- **L'habileté a deux leviers** : le rang des compétences (Veloren, `crans` de `difficulte.ron`) et le cran de
  l'ennemi dans `ia/profils.ron` (IA.5 : au cran « base » il attend 0,8 s avant son premier coup et 1,4 s après
  chaque attaque, sans rouler ni parer ; « vétéran » roule, pare et enchaîne). Le classeur ne mesure pas l'habileté :
  elle s'essaie au lieu d'entraînement. Les règles de cible des races ne dépendent pas de la difficulté.

## Pas dans cette version

- La récompense selon la difficulté : même butin partout pour le moment (Flo, 2026-11-25, ex-Q67).
- Le lot d'expéditions et les zones (B.19, B.20), règle ci-dessous.
- Le terrain, les pièges, la difficulté selon le nombre de joueurs (`design/creatures.md`, pas codée).

## Le lot d'expéditions (Flo, 2026-11-25, ex-Q68 ; à coder avec les zones, B.19)

Aujourd'hui le groupe choisit librement sa difficulté. Avec les zones, le panneau Départ proposera un **lot
d'expéditions**, chacune avec sa difficulté :

- la difficulté d'une expédition est **plafonnée par le niveau de sa zone** ;
- dans le lot, toutes zones confondues, il y a **toujours au moins une expédition de difficulté 1**. Exemple :
  grotte 5, 4, 5, 3 ; forêt 3, 2, 3 ; donjon 6, 6, 1.

La raison : un village qui n'a plus que de nouveaux aventuriers doit toujours pouvoir les faire monter de niveau.

## Qui on affronte (Flo, 2026-11-25, DF.8)

Seulement nos **six espèces** (humain, orc, nain, elfe, draugr, danari), dans les trois lieux. Les monstres
solitaires de Veloren sont retirés : dans la grotte (araignées, trolls des cavernes, onis…), les niches n'ont plus
qu'un coffre et les zones ne sont peuplées que par la race ; dans le donjon, les nids sont des dortoirs de guerriers
endormis. Les factions de Veloren (gnarlings, adlets, cultistes) ne restent que derrière `-Faction` du lanceur, pour
un essai.
