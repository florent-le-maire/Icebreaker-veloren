# Jalon O — Lieu d'entraînement et de playtest

*Plan validé par Flo le 2026-11-14 ; O.1 à O.8 codés le même jour.* Worktree `Veloren-entrainement`, branche `icebreaker-entrainement`. Questions
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

- [x] **O.1 Le lieu se lance : monde sans site, plateau, entrée, zone** — `icebreaker.ps1 entrainement`, le plateau
  nivelé, l'entrée comme point d'apparition, la zone fermée, l'entrée dans `lieux.ron` (pas encore au menu).
  *Fin* : le lieu démarre sans erreur ; au client texte, `!jouer` puis `!etat` donne une position à l'entrée, sur le
  plateau ; `!voisins` est vide.
  *Fait* : le lieu démarre sur un monde sans site (repli inutile) ; arrivée à l'entrée vue au client texte et dans le
  vrai client. `ebcd651e8c`
- [x] **O.2 Personnage tout débloqué à l'arrivée** — `Acquis::complet(classe, niveau)` ; à l'entrée en jeu, la classe
  de l'arme tenue (sinon celle par défaut), tous ses jeux d'armes, ultimes et passifs, l'armure de la classe.
  *Fin* : au client texte, `!competences` montre classe, jeux, ultimes, passifs ; `!touche 1` lance une compétence.
  *Fait* : guerrier à l'arrivée avec les quatre jeux d'armes, les ultimes et les passifs. Commit commun avec O.3.
  `e46b3c16e6`
- [x] **O.3 `/entrainement` : régler le personnage** — `classe <id>`, `arme <jeu>`, `placer <1|2> [id]`,
  `niveau <id|tout> <1-5>`, `ultime <id>`, `passif <id>`, `soin`, `entree` ; refusée hors de ce lieu.
  *Fin* : au client texte, `/entrainement classe mage`, `arme sceptre`, `niveau tout 5` se voient dans
  `!competences`, `!sac`, `!etat` ; la commande est refusée sur le serveur du village.
  *Fait* : essayé au client texte (mage, sceptre, niveau tout 5, placer, soin, entree). Le refus hors du lieu n'a pas
  été essayé sur un serveur de village. `e46b3c16e6`
- [x] **O.4 Ennemis, alliés, nettoyage, mannequins** — `ennemi <race> [role] [nombre]` (la fonction du donjon,
  `world::icebreaker::races::entity`), `allie <id> [nombre]` (liste dans `arene.ron`), `nettoyer`, les mannequins.
  *Fin* : au client texte, 3 orcs dans `!voisins` ; des alliés qui se battent contre eux ; `nettoyer` ne laisse que
  les mannequins ; frapper un mannequin donne son bilan.
  *Fait* : orcs et danari appelés, deux gardes tuent quatre orcs à mes côtés (vu aussi dans le vrai client),
  nettoyer ne laisse que les mannequins. Le bilan d'un mannequin n'a pas été essayé ici. Commit commun avec O.5.
  `458f633af6`, corrigé par `a11931f449` (camps de Veloren coupés, seuls ennemis et alliés comptent)
- [x] **O.5 Mort : réapparition à l'entrée** — un joueur mort réapparaît seul après `delai_reapparition`
  (`arene.ron`).
  *Fin* : au client texte, à terre puis mort ; après le délai, vivant, PV pleins, à l'entrée ; sac inchangé.
  *Fait* : mort par des danari puis réapparition à l'entrée, PV pleins, sac gardé (client texte). Ajout : les ennemis
  ne sortent pas du disque de l'arène, sinon ils tuaient à l'entrée. `458f633af6`
- [x] **O.6 Panneaux « Personnage » et « Arène » sur le plan** — deux cases du plan ; Personnage : onglets Classe,
  Armes, Compétences, Ultimes, Passifs ; Arène : onglets Ennemis, Alliés, boutons Nettoyer, Soin, Retour à l'entrée.
  *Fin* : au client texte, `!batiments` liste les deux cases, `!panneau <sorte>` imprime les onglets, chaque commande
  imprimée marche.
  *Fait* : vu au client texte et en captures du vrai client. L'onglet des compétences s'appelle « Sorts » (cinq
  onglets), le bouton des ennemis « Appeler ». `edc5ec387f`, `a11931f449`
- [x] **O.7 Bouton au menu et essai dans le vrai client** — un lieu `sans_partie` se lance sans choix de partie.
  *Fin* : le bouton « Entraînement » lance le serveur et y connecte ; captures du plan et des deux panneaux.
  *Fait* : bouton vu au menu (capture) ; en plus, l'entrée est directe : pas d'écran des personnages de Veloren,
  le premier personnage du compte est pris ou créé. Le clic sur le bouton n'a pas été essayé (il lance un serveur dans
  une console, port 14150) ; l'entrée directe l'a été par `ICEBREAKER_AUTO_PERSONNAGE=1`. `9d6f2e65e4`
- [x] **O.8 Habillage « à terre » et mort dans le vrai client** — le bloc « À terre » et le décompte ne s'affichent
  que si le client a vu un registre : les activer aussi ici.
  *Fin* : capture du décompte « Mort dans N s » dans le lieu d'entraînement.
  *Fait* : le bloc « À terre » s'affiche partout hors campagne (aussi dans un donjon lancé seul), à la place du
  message de Veloren ; capture « Mort dans 28 s ». L'écran de mort reste celui de Veloren. `9d6f2e65e4`

## Risques

- Un monde sans aucun site ni civilisation n'a jamais été lancé (repli : le village lancé avec une variable
  d'entraînement).
- Des alliés `Owned` (comme `/spawn pet`) doivent attaquer les ennemis : à voir en jeu (repli : `Npc` + garde).

## À vérifier en jeu

Repris dans « À vérifier en jeu » de `PLAN.md`.

- Le bouton « Entraînement » du menu : il lance le serveur et on arrive à l'entrée, sans écran des personnages.
- Tomber à terre sous les coups des ennemis, être relevé par un autre joueur, mourir, réapparaître à l'entrée.
- Les alliés : suivent-ils bien, frappent-ils le joueur par erreur ? Les tirs des archers ennemis vers l'entrée.
- Chaque compétence du mage et de l'archer sur de vrais ennemis, aux niveaux 1, 3 et 5.
- Le bilan des mannequins au bord du plateau ; le bord du plateau (falaise ou mur selon le relief).
- Le plan s'intitule « Village » et le fond est vide : à renommer ou à dessiner si cela gêne.
