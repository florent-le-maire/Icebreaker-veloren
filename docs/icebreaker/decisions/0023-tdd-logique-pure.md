# 0023 — Du TDD pour la logique pure et les bugs, la vérification en jeu pour le reste

- **Date** : 2026-11-26
- **Statut** : choisi par Flo, validé par Sayanel le 2026-11-26. Précise la règle « Teste ce que tu écris » de
  `CLAUDE.md`.
- **Tâches** : aucune ; la règle s'applique dès la prochaine tâche.

## Problème

Jusqu'ici, les agents écrivaient les tests avec le code ou juste après (modules `#[cfg(test)]` dans
`common/src/icebreaker_*.rs`, `world/src/icebreaker/` et les crates `icebreaker-*`) : jamais de test vu en échec
d'abord. Une règle du design mal comprise ou un bug déjà corrigé peut donc revenir sans que rien ne le signale, et
plusieurs agents fusionnent sur `master` chaque jour.

## Options

1. **Ne rien changer** : des tests après coup, là où l'agent y pense.
2. **TDD partout** : coûteux et peu utile pour le ressenti en jeu, les chiffres qu'on règle sans cesse et le code
   mêlé à Veloren (compilation longue, monde à monter pour chaque test).
3. **TDD ciblé** : là où la règle est précise et vérifiable sans lancer le jeu.

## Décision

Option 3.

- **En TDD** (skill `superpowers:test-driven-development` : test écrit d'abord, vu en échec, puis le code) :
  - les règles de jeu écrites dans `docs/icebreaker/design/` (difficulté, économie, progression, butin, âmes,
    rituels…) : le test cite la règle. Une règle qu'on n'arrive pas à écrire en test est ambiguë et devient une
    question dans `QUESTIONS.md` ;
  - les générateurs (`icebreaker-caverne`, `icebreaker-foret`, le donjon généré, `icebreaker-modeles`) : on teste
    des propriétés (toutes les salles reliées, l'abri présent, même graine → même résultat), pas des formes exactes ;
  - chaque correction de bug : d'abord un test qui reproduit le bug.
- **Sans TDD** :
  - le ressenti, le rendu, la caméra, le son : on lance le lieu, et ce qui n'est pas vu va dans « À vérifier en
    jeu » de `PLAN.md` ;
  - les valeurs de `assets/` : on teste la règle (« la difficulté 6 fait plus mal que la 1 »), jamais le chiffre,
    qui se règle avec le classeur d'équilibrage ;
  - le code mêlé à Veloren dans `server/` ou `voxygen/` : on en sort la logique dans une fonction pure
    (`common/src/icebreaker_*.rs` ou un crate `icebreaker-*`), on la teste, et le branchement reste mince.

## Conséquences

- Nos règles se regroupent dans des fonctions pures testées, ce qui garde nos changements dans Veloren petits et
  faciles à retrouver.
- Une tâche de logique coûte un peu plus de temps au départ, mais une règle du design ou un bug corrigé ne peut plus
  revenir sans qu'un test échoue.
