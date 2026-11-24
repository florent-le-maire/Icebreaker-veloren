# Jalon FO — La région Forêt et son atelier de réglage

*Plan validé par Flo le 2026-11-24.* Worktree `Veloren-foret`, branche `icebreaker-foret`.

## Pourquoi

L'équipe règle déjà les grottes et le donjon dans un atelier à curseurs, puis les joue en expédition. Flo veut la
même chose pour une **forêt**, qui devient une troisième région d'expédition : on règle la densité des arbres, la
part de ponts entre les arbres, la hauteur des plateformes… on enregistre un preset, on le joue.

Décidé avec Flo (2026-11-24) :

- **Deux étages** : clairières et sentiers au sol, plateformes et ponts en hauteur, des montées entre les deux.
- **Tomber n'est pas mourir** : dégâts de chute normaux, le sol est jouable, on peut toujours remonter.
- **Nos propres arbres** (tronc, racines, feuillage en boules), pas ceux de Veloren : leurs branches partent au hasard
  et couperaient ponts et plateformes, et la page ne montrerait pas au bloc près ce qu'on joue. Les arbres sont moins
  beaux au début ; habiller le feuillage se fera une fois la forme jouée.

## Comment c'est fait

Un seul générateur, `icebreaker-foret/` : un crate sans dépendance à Veloren, sur le modèle de `icebreaker-caverne`
(dont il reprend le bruit et l'écriture des presets), compilé en WebAssembly pour la page. Ce qu'on voit dans
l'atelier est la forêt que le jeu posera pour la même graine et les mêmes réglages (sur terrain plat).

1. **Clairières** : celle d'arrivée au bord de la forêt, les autres n'importe où, à distance les unes des autres. La
   plus loin de l'arrivée par les sentiers est le **fond** (le chef) ; les autres sont des camps ou des nids.
2. **Sentiers** : l'arbre le plus court qui relie les clairières, plus quelques boucles ; ils serpentent selon
   « détours ».
3. **Arbres** : jetés dans la forêt, écartés selon la densité, jamais dans une clairière ni sur un sentier. Une part
   de très grands arbres.
4. **Étage haut** : une part des arbres porte des plateformes, sur un à trois niveaux. Deux plateformes du même
   niveau, assez proches et sans tronc entre elles, peuvent être reliées par un **pont** (le plus court d'abord, deux
   ponts ne se croisent jamais) ; le curseur « ponts » garde une part de ces ponts. Deux niveaux d'un même arbre sont
   toujours reliés.
5. **Montées** : escalier en spirale autour du tronc ou échelle. Chaque groupe de plateformes reliées par des ponts
   en a au moins une depuis le sol, et une sente y mène depuis les sentiers en contournant les troncs. Sans pont,
   chaque arbre à plateforme a donc sa montée (des postes de guet).
6. **Contrôle à pied** : depuis la clairière d'arrivée, on doit atteindre chaque clairière et chaque plateforme
   (marches d'un bloc, échelles). Sinon le générateur essaie la graine dérivée suivante (12 au plus).
7. **Grille** : un octet par bloc au-dessus du sol (tronc, racine, feuilles, planche, garde-corps, échelle, buisson),
   plus la hauteur et la nature du sol par colonne (herbe, sentier, clairière, sente).

Les réglages d'un profil sont tous des nombres bornés, déclarés une fois (`forme!` dans `icebreaker-foret/src/lib.rs`) :
l'atelier en tire ses curseurs, le jeu son fichier (`assets/icebreaker/forets/formes.ron` : `clairsemee`,
`passerelles`, `geante`).

**L'atelier** : `.\icebreaker.ps1 foret` compile le générateur (cible `wasm32-unknown-unknown`) et sert la page sur
http://localhost:8099. Graine, preset, un curseur par réglage, vue 3D (perspective, dessus, côté), case « Feuillage »
pour voir l'étage haut, coupe en hauteur, « Enregistrer le preset » (écrit `formes.ron` et committe).
`?profil=passerelles&graine=3&vue=dessus&feuilles=1` ouvre sur une forêt précise. Sans la page :
`cargo run --release -p icebreaker-foret --example coupes -- <profil> <graine> <dossier>` (PNG), ou `-- stats`.

## Tâches

- [x] **FO.0 Ouvrir la piste** — ce plan, la ligne du jalon dans `README.md`, la forêt à deux étages dans
  `design/generation.md`. — 1d3827eef1
- [x] **FO.1 Le sol** — crate `icebreaker-foret` : réglages, clairières, sentiers, arbres, buissons, exemple `coupes`.
  Testé : même graine = même forêt, réglages aller-retour, toutes les clairières atteintes. — 0f571283a5
- [x] **FO.2 L'étage haut** — plateformes, ponts, garde-corps, escaliers, échelles, sentes, contrôle à pied. Testé :
  7 tests (dont sol en pente, sans arbre porteur, sans pont, tous les ponts), `stats` : 120 forêts sur 120
  praticables au premier essai, 8 à 27 ms par forêt. Codé avec FO.1, même commit. — 0f571283a5
- [x] **FO.3 L'atelier** — page à curseurs en 3D, serveur `atelier-foret` (port 8099), `.\icebreaker.ps1 foret`, trois
  presets de départ. Essayé : page servie et capturée (Edge sans fenêtre), case « Feuillage », enregistrement et
  suppression d'un preset par la page (commits faits puis retirés). Pas essayé : la commande `foret` elle-même
  (compilation et serveur lancés à la main), les curseurs à la souris. — 0f571283a5
- [x] **FO.4 Le lieu jouable** — `donjon -Site Foret -Profil <preset>` (port 14110) : site `IcebreakerForet`, sol
  nivelé, blocs de la grille posés tels quels, échelles en sprites `Ladder`, arrivée dans la clairière du bord ;
  couleurs et emprise dans `forets/foret.ron`. Vu dans le vrai client (preset `passerelles`) : arrivée dans la
  clairière, troncs, buissons, une plateforme et ses garde-corps. Pas vérifié : monter un escalier et une échelle
  (l'endurance suffit-elle ?), traverser un pont, tomber puis remonter ; de jour. Pas encore dans `lieux.ron`
  (FO.6). — 864f44132d
- [ ] **FO.5 Habitants et butin** — `assets/icebreaker/forets/foret.ron` : nids au sol, archers sur les plateformes,
  chef dans la clairière du fond, coffres (certains en hauteur). Attend la réponse à `QUESTIONS.md`, Q66.
- [ ] **FO.6 En campagne** — `FormeLieu::Foret`, emplacement dans `campagne.ron`, `/expedition <n> <preset>`.
- [ ] **FO.7 Wiki et doc** — `docs/icebreaker/wiki/foret.md`, captures.

## À savoir pour la suite

- L'arrivée est au **bord** de la forêt (clairière 0), pas au centre : `Foret::depart`.
- En jeu, une échelle se grimpe comme un mur (endurance) : à essayer sur 16 blocs et plus ; `echelles: 0` donne des
  escaliers partout.
- Un garde-corps est un bloc d'un de haut : il évite la chute par mégarde, pas le saut.
- Les buissons font un ou deux blocs : on passe par-dessus ceux d'un bloc.
- Le serveur de l'atelier est la troisième copie du petit serveur HTTP des ateliers (grottes, donjon) : à mettre en
  commun si un quatrième atelier arrive.
- Le générateur compilé pour la page embarque aussi les exports de `icebreaker-caverne` (d'où le préfixe `foret_`).
