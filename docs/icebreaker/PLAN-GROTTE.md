# La grotte — un lieu façon Deep Rock Galactic (jalon K)

*Découpage du 2026-11-03, validé par Flo.* Un **troisième lieu**, lancé depuis le chef d'expédition comme « Donjon
généré », avec son emplacement dans la campagne et sa zone fermée. Dans cette première version on y descend pour
**tuer des ennemis** et le chef au fond ; le contenu (ressources du donjon, foreuse, trous à sens unique, cordes,
contrats) vient de `PLAN-DONJON.md` (E.4 à E.8) et `PLAN-BOUCLES.md` (B.4, B.5, B.9). À faire **après** le jalon F
de `PLAN.md`. Worktree `Veloren-grotte` (`git worktree add ../Veloren-grotte -b icebreaker-grotte master`).

## Forme visée

*Depuis le jalon U (2026-11-15, `PLAN-CAVERNE.md`), la forme décrite ici (poches en ellipsoïde, boyaux droits,
culs-de-sac) est remplacée par celle du générateur `icebreaker-caverne` : zones plates ou chaotiques, puits,
niches et sentier. Ambiances, filons, habitants, chef et butin restent ceux de K.3 à K.5 ; les réglages `entree`,
`poche`, `reseau` et `relief` de `grotte.ron` n'existent plus (voir `formes.ron`).*

Pas de cases ni de salles : le tracé est en blocs, rendu avec le `Painter` de Veloren.

- Une **entrée** en surface : tunnel en pente, comme le donjon généré ; apparition devant.
- Des **poches** (chambres) à plusieurs profondeurs, reliées par des **boyaux** (tubes de 3 à 5 de rayon, en
  bézier, en pente) ; des **culs-de-sac** étroits, les « trous » d'où sortent les bêtes.
- Sol et plafond **irréguliers** (bruit), stalagmites et stalactites (cônes), roche luisante, champignons et cristaux
  pour la lumière ; **filons** de minerai dans les parois, plus riches au fond ; un **chef** dans la poche la plus
  profonde.
- Trois **ambiances** en données (champignons, cristaux, ossuaire) : blocs, sprites et créatures.
- Verticalité pour l'instant : poches à des hauteurs différentes et boyaux en pente. Trous à sens unique et cordes
  arrivent avec E.7.

## Ce que Veloren fournit

- Le générateur de grottes de Veloren (`world/src/layer/cave.rs`) couvre tout le monde avec des graines fixes et ne
  se confine pas à un site : on ne s'en sert pas pour le tracé, mais on reprend son **décor** (`GlowMushroom`,
  `CaveMushroom`, `MycelBlue`, `CrystalHigh/Low`, `Bones`, `EnsnaringWeb`, blocs `GlowingRock`,
  `GlowingWeakRock`), ses **minerais** (sprites minables `Copper`, `Tin`, `Iron`, `Coal`, `Cobalt`, `Silver`,
  `Gold`, gemmes) et ses **tables de créatures** par biome (lignes 1647 à 1932).
- Le tracé s'inspire de `gnarling.rs` (arbre de tunnels en `cubic_bezier`, salles en `sphere`) et d'`adlet.rs`
  (caverne au bruit, stalagmites en `cone_with_radius`).
- Un lieu se déclare comme « Genere » : `SiteKind` (`world/src/site/mod.rs`), `PlotKind` (`world/src/site/plot/mod.rs`),
  `civ/mod.rs` (génération, `is_suitable_loc`, `is_dungeon`), `world/src/icebreaker.rs` (`site_kind`,
  `dungeon_entrance`, `dungeon_mission`), `rtsim/src/generate/site.rs` (liste des donjons, sinon les tués ne sont
  pas comptés), `assets/icebreaker/campagne.ron` (emplacement), `assets/icebreaker/lieux.ron` (bouton du chef
  d'expédition, port 14140).
- Attention : le bilan et le sommeil retrouvent les créatures par leur **position d'apparition exacte**
  (`server/src/icebreaker_mission.rs`) ; le plot doit connaître ses sols et ses positions dès sa création, donc un
  bruit **de la position seule** (`FastNoise(pos)`), pas `sample_with_column`.

## Tâches

Code dans `world/src/icebreaker/grotte/` (tracé, réglages) et `world/src/site/plot/icebreaker_grotte.rs` (rendu) ;
réglages dans `assets/icebreaker/grottes/grotte.ron` ; exemple de contrôle `world/examples/icebreaker_grotte.rs`.

- [x] **K.1 Lieu « Grotte » minimal** — `SiteKind::IcebreakerGrotte`, plot `icebreaker_grotte.rs`, tracé dans
  `world/src/icebreaker/grotte/` (`grotte.ron` : entrée, poche), `site_kind("Grotte")`, `is_dungeon` et liste rtsim,
  emplacement `("Grotte", 1)` dans `campagne.ron`, « Grotte » dans `lieux.ron` (port 14140), en-tête du lanceur ;
  couche grottes de Veloren coupée dans toutes les instances. Rendu : une poche (ellipsoïde) sous la bouche et le boyau
  en pente qui y descend ; arrivée devant la bouche. Testé : tests, exemple `icebreaker_grotte` sur 3 graines (poche
  et boyau creux), serveur lancé (apparition devant la bouche) ; pas encore descendu en jeu. — 619d257d25
- [x] **K.2 Réseau de poches et de boyaux** — tracé déterministe par graine (`grotte.ron` → `reseau` : 6 à 10 poches,
  3 niveaux, boyaux de rayon 3 à 5, 1 à 2 boucles entre les poches les plus proches, 1 à 2 culs-de-sac par poche) :
  chaque poche accrochée à une poche du niveau au-dessus, jamais deux qui se chevauchent ; poches en ellipsoïde,
  boyaux droits (`line`). Testé : tests (20 graines : liées, sur plusieurs niveaux, sans chevauchement), serveur
  lancé (graine 42 : 7 poches, 2 boucles). Le contrôle « marche depuis la bouche » de l'exemple était resté inactif
  par erreur (l'ancien contrôle tournait encore) : il n'a pris effet qu'en K.3, qui a corrigé ce qu'il a trouvé.
  Pas encore parcouru en jeu. — bca9fee91b
- [x] **K.3 Relief et décor** — parois au bruit, sol en remblai bosselé avec roche luisante, sol plat sous chaque poche
  pour que les boyaux y arrivent de plain-pied, stalagmites, stalactites et cristaux (cônes), sprites au sol et au
  plafond ; trois ambiances en données (`grotte.ron` → `ambiances` : champignons, cristaux, ossuaire ; `-Ambiance`,
  variable `ICEBREAKER_GROTTE_AMBIANCE`, au hasard sinon). Tracé revu pour que tout se marche : poches sous le terrain
  (entrée vers le terrain le plus haut, allongée jusqu'à une pente douce), boyaux de bord à bord au niveau du sol,
  pas de boyau qui en traverse un autre, sol des boyaux posé explicitement. Contrôle à pied de l'exemple remis en
  service (diagnostics `ICEBREAKER_GROTTE_COLONNE=x,y`, `ICEBREAKER_GROTTE_ATTEINT=x,y`). Testé : 4 tests, exemple
  sur 20 graines (0 problème), serveur lancé (graine 7, cristaux). Pas encore vu en jeu : lisibilité des ambiances
  et lumière sans lanterne. — e0fc89ce37
- [x] **K.4 Filons** — minerais de Veloren en sprites minables au pied des parois des poches (`grotte.ron` → `filons` :
  chance par bloc du bord, paliers par niveau : cuivre, étain, fer en haut ; charbon, cobalt, argent au milieu ; or et
  gemmes au fond), tirés par la position (`VeinSettings::ore`). Une pioche de pierre de Veloren en mains secondaires
  de chaque classe (`classes/*.ron`, `inactive_hands` : on change d'arme pour miner). L'exemple compte les filons par
  niveau (sprites minables sur lesquels on marche). Testé : test de paliers (haut et fond disjoints, or au fond), test
  des classes (pioche), exemple sur 20 graines (0 problème, 2 à 23 filons par niveau), serveur lancé (graine 3,
  ossuaire). Pas vérifié en jeu : miner un filon à la pioche, changement d'arme. — 2a1589751b
- [x] **K.5 Habitants et victoire** — créatures de Veloren par ambiance (`grotte.ron` → `population` et `creatures`,
  `chef` de chaque ambiance : gobelins, truffler, fungome et troll des cavernes ; dodarock, araignée, salamandre et
  basilic ; araignée, onis, chauve-souris et ogre) : 2 à 3 par poche plus 1 par niveau de profondeur, une au bout
  d'un cul-de-sac sur deux, le chef au milieu de la poche la plus profonde ; coffres (`DungeonChest2`) au bout de
  certains culs-de-sac ; `dungeon_mission` étendu au plot (chef, créatures, coffres, personne ne dort). L'exemple
  vérifie que chaque habitant apparaît une fois, debout à un bloc près, et que les coffres sont posés. Testé : test
  (20 graines : un chef au fond, 25 à 40 habitants en moyenne, entités connues), exemple sur 20 graines (0 problème,
  25 à 45 habitants), serveur graine 7 (42 créatures, 2 coffres), client texte admin : `!jouer`, `/goto` chez le
  chef, `/explosion 500` → « Mission accomplie : le chef du donjon est tombé ! Bilan : 16 créature(s) tuée(s) sur
  42, 0 coffre(s) ouvert(s) sur 2. ». Pas encore joué à pied ; les culs-de-sac posent parfois la créature un bloc
  trop haut ou trop bas (elle tombe ou ressort). — 4d75e94a76

Plus tard : trous à sens unique et cordes (E.7), filons comme carburant de la foreuse (E.4, E.6), ressources à
ramener (B.5, B.9), patrouilles (B.4), essaim qui sort des trous (E.15).

## Décisions et questions

Décisions de Flo du 2026-11-03 (victoire en tuant le chef, pioche des classes) : `QUESTIONS.md`, § Tranché.
Questions ouvertes : `QUESTIONS.md`, § 3 « Donjon généré et grotte » (Q21 à Q25).

## Bilan de session (2026-11-04)

Fait : K.3 (relief, décor, trois ambiances, tracé revu avec le contrôle à pied), K.4 (filons par profondeur, pioche
des classes), K.5 (habitants, chef, coffres, victoire G.7 vérifiée au client texte). Le jalon K est complet pour la
première version : on lance « Grotte », on descend, on tue des ennemis et le chef au fond.

Reste : parcourir une grotte dans le vrai client (ambiances, lumière, caméra, filons à la pioche, changement d'arme),
puis le contenu prévu plus tard (E.4 à E.8, B.4, B.5, B.9) selon les retours.
