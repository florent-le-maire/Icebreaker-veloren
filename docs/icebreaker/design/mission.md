# Game design — Mission, butin et interface

Le chef et l'objectif, le butin, le HUD.
Index des règles : [`docs/GAME_DESIGN.md`](../GAME_DESIGN.md).

## Le Chef et la mission

- **Chef** (`c` dans les plans, `chief` dans `creatures.json`) : une brute renforcée (600 PV, frappe de 40, charge).
  Capacité spéciale : **onde de choc** (25 dégâts et étourdi 1,2 s à 4,5 m autour de lui, recharge 9 s), annoncée
  1,1 s à l'avance par un cercle : on l'évite en s'éloignant. Il la déclenche dès que sa cible est dans le rayon, même dans son dos.
- **Objectif « tuer le chef »** : affiché en haut à gauche. La mission est **réussie** quand tous les chefs du donjon sont
  abattus, **échouée** quand tous les personnages présents sont à terre en même temps.
- Fin de mission : écran de résultat (durée depuis l'arrivée du premier joueur, créatures abattues). On reste ensuite dans
  le donjon ; pour recommencer, on relance le serveur (pour l'instant).
- Un plan sans chef n'a pas d'objectif.

### Objectifs en 2D (tâches 2D.25 et 2D.35)

- En mode essai, l'hôte choisit la mission sur l'écran titre (au village : à la porte) : objectif, taille, région,
  saison, menace, nombre d'ennemis, lieu (au hasard ou imposé). Le choix est gardé pour la fois suivante et vaut pour
  toutes les missions de la partie (chacune avec sa graine). L'antre de boss n'est pas proposé avant la tâche 2D.29.
- La salle visée est la dernière du chemin principal. Ce qui gagne la mission (Flo, 2026-10-25) :
  - **tuer le chef** : abattre le ou les chefs (sinon toutes les créatures) ;
  - **nettoyer** : abattre toutes les créatures ;
  - **piller** : trouver la salle au trésor (un personnage debout entre dans la chambre forte) ;
  - **sauver** : trouver le prisonnier dans la prison et l'escorter jusqu'à l'entrée. Un personnage debout à
    `topDown.prisonerFreeRange` (1,5 case) le délivre (« Merci ! Je vous suis. ») ; il suit ensuite le personnage
    debout le plus proche à `prisonerFollowDistance` (1,2 case), à `prisonerSpeed` (4 m/s), escaliers compris. Enfermé,
    les créatures le laissent ; délivré, elles s'en prennent à lui (coups et tirs, `prisonerHealth` 80 PV, barre bleue),
    pas les coups des joueurs. **S'il meurt, la mission échoue** (Flo, 2026-10-25, tâche 2D.42) ;
  - **explorer** : visiter `topDown.exploreShare` (80 %) des salles du donjon (une salle compte quand un personnage
    debout y met les pieds).
- Le bandeau dit l'objectif, la salle visée et son étage, et l'avancement : « Piller : trouver la salle « Chambre
  forte » (étage -1) », « Sauver : escorter le prisonnier jusqu'à l'entrée », « Explorer : visiter les salles (5/10) ».

## Butin

- **Chacun garde ce qu'il ramasse** (décision d'équipe). Données : `game/data/loot/loot.json` (objets avec leur valeur en
  or, et une table par créature ou `chest` : or entre deux bornes, chaque objet tiré avec sa chance).
- Une créature abattue lâche son butin au sol ; le premier personnage **debout** qui passe à `loot.pickupRadius` (1,5 m)
  le ramasse. Un coffre (`$`) s'ouvre quand un personnage debout s'en approche à `loot.chestOpenRadius` (2,5 m) : son
  contenu revient à celui qui l'ouvre.
- Les objets sont pour l'instant des trésors revendables (valeur en or) ; l'équipement viendra avec le village (phase 3).
- HUD : or et nombre d'objets sous les PV. Écran de fin de mission : butin de chacun et sa valeur.

## Interface (HUD)

- En bas à gauche : classe et PV du joueur. En bas au centre : ses trois capacités (touche, nom) ; une capacité en
  recharge est grisée avec les secondes restantes. À 0 PV : « À terre » au centre de l’écran, comment être relevé, et
  l’avancement du relevé.
- Barres de vie au-dessus des autres joueurs (vertes) et des mannequins / créatures (rouges), cachées à 0 PV.

