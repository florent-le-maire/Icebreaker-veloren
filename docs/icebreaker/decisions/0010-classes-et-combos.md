# 0010 — Neuf classes qui coopèrent par des combos (façon Guild Wars 2)

- **Date** : 2026-10-21
- **Statut** : orientation décidée avec Flo le 2026-10-21 ; détail des classes et valeurs à valider avec l'équipe
- **Règles** : `docs/design/classes.md` ; découpage : `docs/roadmap/piste-S-classes.md`

## Problème

Il y a deux classes (guerrier, mage). Rien ne pousse vraiment à jouer ensemble, alors que la coop par
complémentarité est un des piliers (`docs/design/piliers.md`). Flo veut plus de classes, chacune vraiment
différente des autres, avec une coopération importante.

## Options

1. **Outils de progression façon Deep Rock Galactic** : chaque classe a un outil pour avancer (creuser, poser une
   plateforme, éclairer). Forte dépendance, mais elle pèse sur l'exploration plus que sur le combat, et risque de
   bloquer un groupe sans la bonne classe.
2. **Gadgets façon Rainbow Six Siege** : information et destruction. Riche, mais demande un jeu très tactique.
3. **Combos façon Guild Wars 2** : des zones posées au sol, réveillées par les actions des autres. La coopération
   vit dans le combat en temps réel, et un groupe qui ne combine pas joue quand même.
4. **Trinité classique** (tank, soigneur, dégâts) : simple, mais rend les classes interchangeables dans leur rôle
   et oblige à une composition.

## Décision

**Option 3, choix de Flo.** Les combos sont un **bonus, jamais obligatoires** (un groupe sans combo finit le
donjon, plus difficilement). On garde la base façon Elden Ring déjà en place. Quelques idées des autres options
restent dans les classes (construction de l'ingénieur, fumée de l'assassin) sans devenir des passages obligés.

Neuf classes : guerrier, berserker, mage, rôdeur, assassin, clerc, ingénieur, druide, alchimiste.

Ordre : le système de combos d'abord avec guerrier et mage (on vérifie que c'est amusant avant d'investir), puis les
classes deux par deux, en commençant par celles qui ont un modèle Quaternius. Un seul test à l'équipe, à la fin.

## Conséquences

- Nouvelle logique en C# pur : zones au sol, table des combos, réactions, ressources propres (rage, fioles).
- Nouveaux fichiers de données : `combat/zones.json`, `combat/combos.json` ; `classes.json` passe à neuf classes.
- La main gauche porte aussi les outils de classe (fumigènes, fioles, bombes) ; pas de nouvelle touche.
- L'ingénieur dépend de la destruction (0009) pour construire et casser ; le rôdeur de l'arc de la piste W.
- Deux modèles à créer (ingénieur, alchimiste) ; en attendant, un modèle existant recoloré.
