# Game design

Règles fixées et valeurs de référence, découpées par domaine dans `design/`. Copie du dépôt Godot `../Icebreaker`
(2026-10-28) : les chemins `game/data/*.json`, les pistes (H, S, T, Z…) et les tâches « 2D.x » qu'on y lit sont ceux
de l'ancien projet. Dans le prototype Veloren, les réglages sont dans `assets/icebreaker/*.ron` et les tâches dans
`PLAN.md` ; les règles se lisent comme des intentions, et en cas d'écart ce sont les données du prototype qui font
foi. La vision complète est dans `VISION.md`. La colonne « Pour Veloren » dit ce qui vaut la peine d'être lu ici.

| Domaine | Fichier | Contenu | Pour Veloren |
|---|---|---|---|
| Piliers | [`design/piliers.md`](design/piliers.md) | *proposition* : principe directeur, question de contrôle pour toute nouvelle mécanique, sources de fun | **repris** : filtre de toute mécanique (à valider, `QUESTIONS.md` Q34) |
| Généralités | [`design/general.md`](design/general.md) | joueurs, temps, déplacement, style visuel, caméra et contrôles | Godot : caméra et touches de l'ancien projet, ne pas lire (le temps 1 min = 30 min est repris, I.5) |
| Donjon | [`design/donjon.md`](design/donjon.md) | décisions de l'équipe, étages, gravité / saut / chutes, carte d'exploration, format des plans texte | repris pour « Donjon (décisions) » ; étages, carte et plans texte sont Godot |
| Génération | [`design/generation.md`](design/generation.md) | graphe de mission, salles préfaites, assemblage, habitants crédibles, paramètres de mission | **repris** : traduit en Rust (G.2 à G.6) |
| Habillage 2D | [`design/habillage.md`](design/habillage.md) | tuiles par lieu, mobilier par rôle de salle, traces des occupants | Godot 2D, ne pas lire (F.6 fait son propre décor par rôle) |
| Combat | [`design/combat.md`](design/combat.md) | classes, armes et enchaînements, capacités, endurance, esquive, équilibre | référence seulement : L.1 et `PLAN-DONJON.md` (E.1, E.2) règlent le combat de Veloren |
| Créatures | [`design/creatures.md`](design/creatures.md) | comportement de base, sommeil, errance, appel à l'aide, encerclement, déroute, fiches | **repris** : G.9 fait, G.11 à G.16 à faire |
| Menace | [`design/menace.md`](design/menace.md) | *proposition* : intelligence du donjon, alerte, embuscades, indices, idées pour la suite | **repris** (proposition, Q35) : G.15 et G.16 |
| Mission | [`design/mission.md`](design/mission.md) | le chef et l'objectif, le butin, le HUD | repris pour les objectifs (missions B.16 à B.20 de `PLAN-PLAYTEST.md`, contrats B.9) ; HUD Godot |
| Boss | [`design/boss.md`](design/boss.md) | séquence d'actions, seuils de PV et rage | Godot (boss d'AmbianceurZinzin), ne pas lire ; les boss de Veloren servent tels quels |
| Destruction | [`design/destruction.md`](design/destruction.md) | matières, points de vie, outils (arme, bombe, foreuses), ce qui cède | repris comme base de `PLAN-DONJON.md` (E.5 à E.8) |
| Classes | [`design/classes.md`](design/classes.md) | *proposition* : neuf classes, zones, déclencheurs, table des combos, réactions (3D) | **section « Veloren » en tête** (jalon A, `decisions/0020`, `PLAN-CLASSES.md`) ; le reste est la référence Godot (`decisions/0010`, Q36) |
| Compétences 2D | [`design/competences-2d.md`](design/competences-2d.md) | kits d'AmbianceurZinzin en 2D : touches, recharges, compétences, ultimes, brise-glace | Godot 2D, ne pas lire |
| Village | [`design/village.md`](design/village.md) | *proposition* : bâtiments, ressources, stocks et paquetage, rang, traits, blessures, boucle entre deux sorties | **repris** : jalons C, R et `PLAN-BOUCLES.md` ; questions Q31 à Q33 |
| Âmes | [`design/ames.md`](design/ames.md) | idée de Sayanel : âmes de monstres, Codex, races et classes d'ennemis, stats et résistances de Veloren | **écrit pour Veloren** (2026-11-03, règles précisées le 2026-11-16) ; jalon AM de `PLAN-PLAYTEST.md` ; Sayanel conçoit (Q37, Q57, Q58) |

Une nouvelle règle pour le prototype va dans le fichier de son domaine, marquée « Veloren » ; un nouveau domaine = un
nouveau fichier, ajouté à ce tableau. `design/ames-idees.xlsx` : tableur de Sayanel pour les âmes.
