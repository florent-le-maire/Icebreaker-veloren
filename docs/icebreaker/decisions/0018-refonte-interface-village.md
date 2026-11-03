# 0018 — Refonte de l'interface du village : robustesse d'abord, visuel de Veloren

- **Date** : 2026-11-03
- **Statut** : choisi par Flo le 2026-11-03, fait le soir même (jalon X de `PLAN.md`), vu en jeu
- **Règles** : `design/village.md` (vue 2D du village, décision 4 de l'équipe) ; tâches : jalon X de `PLAN.md`

## Problème

Les premières parties dans le vrai client ont montré une interface qui marche mais fragile et sommaire : la vue
de dessus (touche U) restait active après un départ d'expédition ou un retour, les zones cliquables ne tombaient
pas sur les maisons en pente, le panneau d'un bâtiment était un rectangle noir, les maisons se ressemblaient
toutes et leurs PNJ étaient dehors, l'anneau de verrouillage n'était pas centré, « Campagne » demandait encore des
clics, et la charrette ne montrait que des noms.

## Options

1. **Robustesse et cohérence** : reprendre chaque élément un par un (états propres, projection juste, cadres et
   icônes existants de Veloren), sans nouvelle texture.
2. **Refonte visuelle** : une charte à nous (panneaux, listes, icônes dessinées), avec les correctifs en plus.
3. **Les deux**, robustesse d'abord.

## Décision

- **Option 1** : le visuel reste celui de Veloren (cadres de dialogue, boutons, icônes du HUD, sprites du monde,
  police de titre) ; pas de nouvelle texture, pas de portrait 3D dans les panneaux.
- Huit tâches, une par défaut vu en jeu (X.1 à X.8), chacune avec ses réglages en données :
  `vue_village.ron` (vue U, apparences), `village.ron` (enseignes), `registre.ron` (postes des recrues),
  `verrouillage.ron` (anneau).
- Les recrues de la charrette deviennent des **PNJ debout devant la charrette**, équipés et nommés, plutôt qu'un
  portrait dans le panneau.
- Les bâtiments se reconnaissent par **couleur et icône** en vue U, et par des **enseignes** (caisses, tonneaux,
  lanternes…) de part et d'autre de leur porte dans le village.
- Les PNJ des bâtiments sont **à l'intérieur** de leur maison, sauf la charrette.
- `ICEBREAKER_AUTO_CAMPAGNE=1` entre dans la campagne sans clic, pour les tests.

## Conséquences

- Le protocole client-serveur d'Icebreaker change (sorte et altitude des bâtiments, lignes de panneau avec
  icônes) : client et serveur doivent être recompilés ensemble ; un vieux serveur fait tomber la connexion.
- Une enseigne nommée par bâtiment demanderait un asset à nous (le sprite `Sign` n'a pas de texte) : question
  ouverte 4 du plan. Hauteur des postes des recrues en pente : question 5.
- La refonte visuelle (option 2) reste possible plus tard, sur cette base propre.
