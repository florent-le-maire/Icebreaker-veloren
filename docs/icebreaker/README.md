# Docs Icebreaker sur Veloren — point d'entrée

Prototype du jeu **Icebreaker** (Flo, Sayanel) sur une copie modifiée de Veloren. Le projet d'origine (Godot) et sa
vision sont dans `../Icebreaker` ; ce dossier contient ce qu'il faut pour travailler ici.

## Ordre de lecture pour un agent

1. `README.md` (ce fichier) : fichiers, jalons, conventions.
2. `PLAN.md` : état du projet, tâches restantes, ce qui est à vérifier en jeu.
3. `QUESTIONS.md` : questions ouvertes (avec la proposition en cours) et petites décisions tranchées. Avec
   `A-FAIRE.md`, c'est la boîte de l'équipe : réponses et todos saisis sur le wiki local, à traiter en premier.
4. Le `PLAN-*.md` de la piste sur laquelle on travaille.
5. Le fichier de `design/` du domaine utile seulement (voir la colonne « Pour Veloren » de `GAME_DESIGN.md`).

## Les fichiers

| Fichier | Rôle |
|---|---|
| `PLAN.md` | Plan principal : état, architecture, tâches restantes des jalons C, P, G, F, S, D, liste « à vérifier en jeu » |
| `HISTORIQUE.md` | Tâches faites de `PLAN.md`, avec hash et ce qui n'a pas été vérifié en jeu |
| `QUESTIONS.md` | Source unique des questions pour l'équipe et des petites décisions |
| `PLAN-BOUCLES.md` | Proposition : ce qui rend village → donjon → village intéressant (jalon B) |
| `PLAN-DONJON.md` | Proposition : ce qu'on fait en bas, ressources, foreuse, trous, combat, monstres à coop (jalon E) |
| `PLAN-GROTTE.md` | La grotte façon Deep Rock Galactic (jalon K, fait en première version) |
| `PLAN-MOD.md` | Proposition : passer d'une copie modifiée à un mod (jalon M) |
| `PLAN-WIKI.md` | Le wiki de l'équipe : site généré depuis ces docs et `assets/icebreaker/` (jalon W) |
| `PLAN-INTERFACE.md` | Refonte de l'interface ajoutée par Icebreaker : maquettes, charte, règles (jalon N) |
| `PLAN-CLASSES.md` | Armes, classes et compétences : jeux d'armes, ultime, passif, maîtrise, salle d'armes (jalon A) |
| `A-FAIRE.md` | Todos de l'équipe (wiki local) : « Ouvert » à traiter en début de session, puis « Pris en compte » |
| `MAINTENANCE.md` | Lancer le jeu et le serveur, faire le zip du client, déployer sur le VPS |
| `IDEES.md` | Boîte à idées non décidées, avec leur effort |
| `VISION.md` | Vision du jeu (copie du dépôt Godot ; ses « grandes phases » sont celles du projet Godot) |
| `GAME_DESIGN.md` | Index des règles de `design/`, avec ce qui vaut pour Veloren |
| `design/` | Règles par domaine, copiées du dépôt Godot le 2026-10-28 ; `ames.md` et `village.md` sont les plus vivants |
| `wiki/` | Fiches mécaniques du wiki (jalon W) : vulgarisation + en-tête qui relie design, réglages, tâches et code |
| `decisions/` | Notes de décision (problème, options, recommandation) ; `0001` à `0014` restent dans `../Icebreaker` |

## Les jalons et leurs lettres

| Lettre | Jalon | Plan | État (2026-11-04) | Worktree |
|---|---|---|---|---|
| I | Instances propres | `HISTORIQUE.md` | fait | — |
| V | Village, première version | `HISTORIQUE.md` | fait | — (supprimé) |
| C | Le village qui grandit | `PLAN.md` | C.1 à C.3 faits ; C.4 à C.6 restent | — (supprimé) |
| P | Pillage sur la route | `PLAN.md` | pas commencé | à créer |
| G | Donjon généré | `PLAN.md` | G.1 à G.10 faits ; IA G.11 à G.16 restent | `Veloren-donjon` |
| F | Forme et vie du donjon généré | `PLAN.md` | F.1 à F.6 faits ; F.7 si besoin | `Veloren-donjon` |
| L | Combat létal | `HISTORIQUE.md` | fait | — (supprimé) |
| R | Charrette et registre commun | `HISTORIQUE.md` | fait | — (supprimé) |
| X | Interface du village | `HISTORIQUE.md` | fait, vu en jeu | — (supprimé) |
| S, D | Démo à l'équipe (serveur, soirée) | `PLAN.md` | S.1 fait ; S.2, S.3, D.1 restent | `Veloren-serveur` |
| B | Boucles de jeu | `PLAN-BOUCLES.md` | proposition à valider ; B.5 et B.14 (butin) pris par Flo | `Veloren-butin` (butin), `Veloren-boucles` (à créer) |
| E | Le donjon à jouer | `PLAN-DONJON.md` | proposition à valider | `Veloren-donjon` |
| K | La grotte | `PLAN-GROTTE.md` | K.1 à K.5 faits, à voir en jeu | `Veloren-grotte` |
| M | Mod Veloren | `PLAN-MOD.md` | M.1 fait ; la suite après la démo | `Veloren-mod` |
| T | À terre, mort et sac du mort | `PLAN.md` | T.1 à T.6 faits, à voir en jeu | `Veloren-mort` |
| J | Balise lumineuse | `HISTORIQUE.md` | fait, vu en jeu | — (supprimé) |
| W | Wiki de l'équipe | `PLAN-WIKI.md` | W.1 à W.6 et W.8 à W.17 faits ; W.7 (VPS) à lancer | `Veloren-wiki` |
| N | Interface Icebreaker | `HISTORIQUE.md` (charte et règles : `PLAN-INTERFACE.md`) | fait, vu en jeu | — (supprimé) |
| A | Armes, classes et compétences | `PLAN-CLASSES.md` | plan validé le 2026-11-11 ; A.0 fait, A.1 à A.18 restent | `Veloren-classes` |

Les « tâches H », « piste Z », « 2D.x » qu'on croise dans `design/` sont celles du projet Godot, pas des nôtres.

## Conventions

- Une tâche = quelques heures, un critère de fin, un commit en français, cochée avec son hash dans son plan ; les
  tâches cochées de `PLAN.md` partent dans `HISTORIQUE.md` quand le jalon est fini (ou en fin de session).
- Ce qui n'a pas été vérifié dans le vrai client se dit dans la tâche cochée **et** dans « À vérifier en jeu » de
  `PLAN.md` ; quand c'est vu, on le barre là.
- Une question nouvelle va dans `QUESTIONS.md` (numéro Qn suivant, jamais réattribué) ; le plan concerné y renvoie.
  Une fois tranchée, elle passe dans « Tranché » avec la date et le fichier où la règle est écrite.
- Une décision importante (choix technique, changement de cap) = une note `decisions/00NN-<sujet>.md` (prochain
  numéro : 0021), au format des existantes (date, statut, problème, options, décision, conséquences).
- Une idée pas encore décidée va dans `IDEES.md` ; retenue, elle devient une tâche et sort de là.
- Un worktree par piste (`CLAUDE.md`) ; la colonne ci-dessus dit lequel.
