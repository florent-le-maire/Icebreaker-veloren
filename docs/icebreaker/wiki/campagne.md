+++
ordre = 1
titre = "La campagne et ses lieux"
icone = "voxygen/element/ui/generic/buttons/map.png"
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
l'entrepôt, les maisons bâties, l'horloge. Le bouton « Campagne » du menu lance ou rejoint son serveur.

Ce serveur génère **un seul monde** (décision 0016) : le village au centre et, loin de lui et les uns des autres,
des **emplacements** de donjon : deux Gnarling (un donjon de Veloren), deux de notre générateur, une grotte. Une
expédition qui part prend un emplacement libre de sa destination et y est **téléportée** ; l'emplacement reste
occupé tant qu'un membre y est, puis il est **remis à neuf** : monstres tués et coffres ouverts reviennent.

Chaque lieu est une **zone fermée** : un cercle autour du site, au-delà duquel le serveur vous ramène au bord avec
un message. Pas de faune sauvage ni de monstres errants dans les instances : seuls les habitants du donjon et de la
ville. Le **temps de carte** avance : une minute réelle fait trente minutes de carte, et un nouveau jour renouvelle
la charrette.

La commande `/rentrer` ramène au village avec le bilan de ce qu'on rapporte. Les lieux restent lançables seuls
(`icebreaker.ps1 village`, `donjon -Site Gnarling`…) pour tester.

## Pour la démo

Le serveur tourne sur le VPS (S.2) ; il reste le client pour les potes (S.3) et le pillage sur la route (jalon P),
puis la soirée de démo (D.1).
