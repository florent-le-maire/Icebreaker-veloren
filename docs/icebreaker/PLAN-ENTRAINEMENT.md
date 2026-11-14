# Jalon O — Lieu d'entraînement et de playtest

*Plan validé par Flo le 2026-11-14.* Worktree `Veloren-entrainement`, branche `icebreaker-entrainement`. Questions
pour l'équipe : `QUESTIONS.md`, Q51 à Q53.

## Pourquoi

Valider une fonctionnalité de combat demande aujourd'hui de passer par la charrette (classe tirée au hasard), de
gagner des points en expédition pour monter une compétence, et d'aller chercher des ennemis dans un donjon où la mort
est définitive. Le mage et l'archer n'ont jamais été essayés sur de vraies cibles.

Le jalon O crée un **lieu à part** où l'on choisit librement classe, armes, compétences et niveaux, où l'on fait
venir ennemis et alliés, et où mourir renvoie vivant à l'entrée. Rien de ce qu'on y fait ne touche la campagne.

## Règles décidées avec Flo (2026-11-14)

| Sujet | Règle |
|---|---|
| Lieu | À part : `icebreaker.ps1 entrainement` (port 14150), son bouton au menu, sa propre partie. Pas de registre, pas de charrette. |
| Personnage | Tout est libre et gratuit : classe, jeu d'armes, compétences placées, niveau (1 à 5), ultime, passif. |
| Ennemis | Les races du jalon Y, choisies par race, rôle (guerrier, archer, mage, chef) et nombre. Pas de « menace » par ennemi (Q51). |
| Alliés | Des créatures de Veloren placées dans le camp du joueur ; pas d'allié qui joue nos classes (Q53). |
| Mort | « À terre » comme en expédition ; à la mort, on réapparaît seul à l'entrée, PV pleins, personnage et sac gardés. |
| Pilotage | La commande `/entrainement`, et deux cases sur le plan (touche U) : « Personnage » et « Arène ». |
| Mémoire | Niveaux et touches ne sont pas gardés d'une session à l'autre (Q52). |

## Comment c'est fait

- **Le monde** : pas de site (`ICEBREAKER_INSTANCE_SITE=Entrainement`), un plateau nivelé près du centre de la carte
  (`world/src/icebreaker/arene.rs`). L'arène est un disque au milieu, l'entrée est sur le plateau, hors du disque.
  Réglages : `assets/icebreaker/arene.ron` (`common/src/icebreaker_arene.rs`).
- **Le personnage** : à l'entrée en jeu, le serveur pose une fiche « tout connu » (`Acquis::complet`) ;
  `/entrainement` la modifie (`server/src/icebreaker_arene.rs`), avec les règles de la salle d'armes
  (`server/src/icebreaker_salle.rs`) sans les prix.
- **Les mannequins** du village (X.14) sont posés aussi, avec leur bilan de dégâts.
- Le mot « entraînement » était déjà pris par les mannequins (`icebreaker_entrainement.rs`,
  `entrainement.ron`) : le code de ce jalon s'appelle « arène » (`icebreaker_arene`, `arene.ron`).

## Tâches

- [ ] **O.1 Le lieu se lance : monde sans site, plateau, entrée, zone** — `icebreaker.ps1 entrainement`, le plateau
  nivelé, l'entrée comme point d'apparition, la zone fermée, l'entrée dans `lieux.ron` (pas encore au menu).
  *Fin* : le lieu démarre sans erreur ; au client texte, `!jouer` puis `!etat` donne une position à l'entrée, sur le
  plateau ; `!voisins` est vide.
- [ ] **O.2 Personnage tout débloqué à l'arrivée** — `Acquis::complet(classe, niveau)` ; à l'entrée en jeu, la classe
  de l'arme tenue (sinon celle par défaut), tous ses jeux d'armes, ultimes et passifs, l'armure de la classe.
  *Fin* : au client texte, `!competences` montre classe, jeux, ultimes, passifs ; `!touche 1` lance une compétence.
- [ ] **O.3 `/entrainement` : régler le personnage** — `classe <id>`, `arme <jeu>`, `placer <1|2> [id]`,
  `niveau <id|tout> <1-5>`, `ultime <id>`, `passif <id>`, `soin`, `entree` ; refusée hors de ce lieu.
  *Fin* : au client texte, `/entrainement classe mage`, `arme sceptre`, `niveau tout 5` se voient dans
  `!competences`, `!sac`, `!etat` ; la commande est refusée sur le serveur du village.
- [ ] **O.4 Ennemis, alliés, nettoyage, mannequins** — `ennemi <race> [role] [nombre]` (la fonction du donjon,
  `world::icebreaker::races::entity`), `allie <id> [nombre]` (liste dans `arene.ron`), `nettoyer`, les mannequins.
  *Fin* : au client texte, 3 orcs dans `!voisins` ; des alliés qui se battent contre eux ; `nettoyer` ne laisse que
  les mannequins ; frapper un mannequin donne son bilan.
- [ ] **O.5 Mort : réapparition à l'entrée** — un joueur mort réapparaît seul après `delai_reapparition`
  (`arene.ron`).
  *Fin* : au client texte, à terre puis mort ; après le délai, vivant, PV pleins, à l'entrée ; sac inchangé.
- [ ] **O.6 Panneaux « Personnage » et « Arène » sur le plan** — deux cases du plan ; Personnage : onglets Classe,
  Armes, Compétences, Ultimes, Passifs ; Arène : onglets Ennemis, Alliés, boutons Nettoyer, Soin, Retour à l'entrée.
  *Fin* : au client texte, `!batiments` liste les deux cases, `!panneau <sorte>` imprime les onglets, chaque commande
  imprimée marche.
- [ ] **O.7 Bouton au menu et essai dans le vrai client** — un lieu `sans_partie` se lance sans choix de partie.
  *Fin* : le bouton « Entraînement » lance le serveur et y connecte ; captures du plan et des deux panneaux.
- [ ] **O.8 Habillage « à terre » et mort dans le vrai client** — le bloc « À terre » et le décompte ne s'affichent
  que si le client a vu un registre : les activer aussi ici.
  *Fin* : capture du décompte « Mort dans N s » dans le lieu d'entraînement.

## Risques

- Un monde sans aucun site ni civilisation n'a jamais été lancé (repli : le village lancé avec une variable
  d'entraînement).
- Des alliés `Owned` (comme `/spawn pet`) doivent attaquer les ennemis : à voir en jeu (repli : `Npc` + garde).

## À vérifier en jeu

*(rempli au fil des tâches ; repris dans « À vérifier en jeu » de `PLAN.md`)*
