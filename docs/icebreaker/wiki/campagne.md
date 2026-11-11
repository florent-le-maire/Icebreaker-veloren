+++
ordre = 1
titre = "La campagne et ses lieux"
icone = "voxygen/element/ui/generic/buttons/map.png"
schema = "carte-campagne"
resume = "Un seul serveur : le village au centre, des emplacements de donjon loin autour, une partie qui dure."
reglages = ["campagne.ron", "lieux.ron"]
taches = ["I", "S"]
systemes = [
  "server/src/icebreaker_slots.rs",
  "server/src/icebreaker_expedition.rs",
  "server/src/icebreaker_return.rs",
  "server/src/sys/icebreaker_zone.rs",
  "server/src/sys/icebreaker_clock.rs",
]
+++

## La règle

Une **partie** (dossier `userdata-icebreaker/parties/<nom>`) garde tout ce qui dure : les personnages, la caisse,
l'entrepôt, les maisons bâties, l'horloge. Au menu, le bloc « Icebreaker » a deux boutons, chacun avec sa ligne
d'explication : « Campagne » (le serveur tourne sur cet ordinateur) et « En ligne » (le serveur de l'équipe).
« Campagne » ouvre la liste des parties : la dernière jouée en tête avec « Continuer », les autres dessous, et un
champ pour en créer une. Pendant le lancement, l'écran montre les étapes : serveur lancé, préparation du monde,
connexion, arrivée au village.

Ce serveur génère **un seul monde** (décision 0016) : le village au centre et, loin de lui et les uns des autres,
des **emplacements** de donjon : deux Gnarling (un donjon de Veloren), deux de notre générateur, une grotte. Une
expédition qui part prend un emplacement libre de sa destination et y est **téléportée** ; l'emplacement reste
occupé tant qu'un membre y est, puis il est **remis à neuf** : monstres tués et coffres ouverts reviennent.

Chaque lieu est une **zone fermée** : un cercle autour du site, au-delà duquel le serveur vous ramène au bord avec
un message. Pas de faune sauvage ni de monstres errants dans les instances : seuls les habitants du donjon et de la
ville. Le **temps de carte** avance : une minute réelle fait trente minutes de carte, et un nouveau jour renouvelle
la charrette.

La commande `/rentrer` ramène au village et ouvre la fenêtre « Retour d'expédition » : le lieu, si la mission est
accomplie, les créatures tuées et les coffres ouverts sur le total, l'or et les objets rapportés. Les autres
joueurs n'en voient qu'une ligne dans le chat. Les lieux restent lançables seuls (`icebreaker.ps1 village`,
`donjon -Site Gnarling`…) pour tester.

## Pour la démo

Le serveur tourne sur le VPS (S.2) ; il reste le client pour les potes (S.3) et le pillage sur la route (jalon P),
puis la soirée de démo (D.1).

## En images

![Le menu principal : le bloc « Icebreaker », à part des boutons de Veloren](images/campagne-menu.png)

![Le choix de la partie : la dernière jouée en tête, la liste qui défile, le champ d'une nouvelle partie](images/interface-parties.png)

![Les étapes du lancement (ici vers un serveur d'essai, d'où l'adresse en titre)](images/interface-lancement.png)

![Le bilan au retour : la mission, les chiffres, ce qu'on rapporte](images/interface-bilan.png)

![Le message de bord de zone quand on s’éloigne trop d’un lieu](images/campagne-zone.png)
