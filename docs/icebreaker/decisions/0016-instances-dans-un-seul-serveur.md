# 0016 — Les instances dans un seul serveur (prototype Veloren)

- **Date** : 2026-10-29
- **Statut** : proposé par Flo (« des instances, et toujours le même serveur quand on lance une expédition ») ;
  découpage à valider ; 2026-11-07 : Flo retient aussi l'option 2 pour le donjon généré et la grotte (un plan neuf
  à chaque expédition, `PLAN.md`, I.10 à I.12)

## Problème

Le prototype suit `0005` : un processus serveur par lieu (village, chaque donjon), le client se reconnecte en
changeant de lieu (V.8). En jeu, ça veut dire une fenêtre de serveur par lieu, un écran de connexion à chaque
expédition, un monde regénéré à chaque fois, une horloge et une sauvegarde à passer par fichier d'un serveur à
l'autre. Flo veut rester sur **un seul serveur** : l'expédition part dans une instance de ce même serveur.

`0005` visait Godot (singletons, un processus = un donjon). Veloren, lui, gère sans peine un grand monde avec
plusieurs groupes à des endroits différents : c'est son fonctionnement normal.

## Options

1. **Un monde, des emplacements d'instance** : le serveur génère un seul monde avec le village au centre et, loin de
   lui et loin les uns des autres, quelques **emplacements** de donjon (par exemple 2 Gnarling et 2 donjons de notre
   générateur). Une expédition prend un emplacement libre de la destination ; ses membres y sont téléportés, et
   `/rentrer` les ramène sur la place. Chaque lieu garde sa zone fermée (I.2). Quand le dernier membre est rentré,
   l'emplacement est remis à neuf (terrain regénéré, créatures et coffres de retour) pour l'expédition suivante.
   - Pour : un seul serveur, pas de reconnexion, horloge et personnages naturellement communs ; on garde presque tout
     le travail fait (village, bâtiments, donjon généré, jalon C).
   - Contre : un nombre fixe d'emplacements (2 groupes au plus par sorte de donjon en même temps) ; les paramètres
     de mission du donjon généré (G.6) sont tirés au lancement du serveur, pas à chaque expédition.
2. **Un monde regénéré par morceaux** : comme 1, mais le donjon d'un emplacement est regénéré à chaque expédition
   avec de nouveaux paramètres de mission.
   - Pour : chaque expédition a un donjon neuf.
   - Contre : Veloren génère ses sites une fois pour toutes au lancement ; les refaire en cours de partie touche au
     cœur de la génération du monde. Beaucoup plus long.
3. **Garder un serveur par lieu** (état actuel, V.8) en cachant mieux les reconnexions.
   - Contre : ne répond pas à la demande.

## Recommandation

**Option 1**, en gardant les lancements séparés (`icebreaker.ps1 donjon -Site …`) pour tester un donjon seul.
L'option 2 pourra venir plus tard, pour le donjon généré seulement, si les emplacements fixes lassent.

Ce qui change : le lanceur et les boutons du menu (un seul serveur « Campagne »), le départ (téléportation au lieu
du voyage de V.8, qui reste pour plus tard), la zone fermée (une par lieu), les traces de donjon (remises à zéro
par emplacement).
