# Game design

Règles fixées et valeurs de référence, découpées par domaine dans `docs/design/`. La vision complète est dans
`docs/VISION.md`. Toutes les valeurs chiffrées sont des réglages des fichiers de `game/data/` (surtout
`game/data/config/game.json`) : en cas d'écart, ce sont les données qui font foi.

| Domaine | Fichier | Contenu |
|---|---|---|
| Piliers | [`design/piliers.md`](design/piliers.md) | *proposition* : principe directeur, question de contrôle pour toute nouvelle mécanique, sources de fun |
| Généralités | [`design/general.md`](design/general.md) | joueurs, temps, déplacement, style visuel, caméra et contrôles |
| Donjon | [`design/donjon.md`](design/donjon.md) | décisions de l'équipe, étages, gravité / saut / chutes, carte d'exploration, format des plans texte |
| Génération | [`design/generation.md`](design/generation.md) | graphe de mission, salles préfaites, assemblage, habitants crédibles, paramètres de mission |
| Habillage 2D | [`design/habillage.md`](design/habillage.md) | tuiles par lieu, mobilier par rôle de salle, traces des occupants |
| Combat | [`design/combat.md`](design/combat.md) | classes, armes et enchaînements, capacités, endurance, esquive, équilibre |
| Créatures | [`design/creatures.md`](design/creatures.md) | comportement de base, sommeil, errance, appel à l'aide, encerclement, déroute, fiches |
| Menace | [`design/menace.md`](design/menace.md) | *proposition* : intelligence du donjon, alerte, embuscades, indices, idées pour la suite |
| Mission | [`design/mission.md`](design/mission.md) | le chef et l'objectif, le butin, le HUD |
| Boss | [`design/boss.md`](design/boss.md) | séquence d'actions, seuils de PV et rage |
| Destruction | [`design/destruction.md`](design/destruction.md) | matières, points de vie, outils (arme, bombe, foreuses), ce qui cède |
| Classes | [`design/classes.md`](design/classes.md) | *proposition* : neuf classes, zones, déclencheurs, table des combos, réactions (3D) |
| Compétences 2D | [`design/competences-2d.md`](design/competences-2d.md) | kits d'AmbianceurZinzin en 2D : touches, recharges, compétences, ultimes, brise-glace |
| Village | [`design/village.md`](design/village.md) | *proposition* : bâtiments, ressources, stocks et paquetage, rang, traits, blessures, boucle entre deux sorties |

Une nouvelle règle va dans le fichier de son domaine ; un nouveau domaine (village, monde, boss…) = un nouveau fichier,
ajouté à ce tableau.
