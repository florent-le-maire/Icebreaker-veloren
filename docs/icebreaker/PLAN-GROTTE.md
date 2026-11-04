# La grotte — un lieu façon Deep Rock Galactic (jalon K)

*Découpage du 2026-11-03, validé par Flo.* Un **troisième lieu**, lancé depuis le chef d'expédition comme « Donjon
généré », avec son emplacement dans la campagne et sa zone fermée. Dans cette première version on y descend pour
**tuer des ennemis** et le chef au fond ; le contenu (ressources du donjon, foreuse, trous à sens unique, cordes,
contrats) vient de `PLAN-DONJON.md` (E.4 à E.8) et `PLAN-BOUCLES.md` (B.4, B.5, B.9). À faire **après** le jalon F
de `PLAN.md`. Worktree `Veloren-grotte` (`git worktree add ../Veloren-grotte -b icebreaker-grotte master`).

## Forme visée

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
  et lumière sans lanterne. — (hash)
- [ ] **K.4 Filons** — minerais en sprites minables contre les parois (bloc d'air voisin d'une roche), paliers par
  profondeur (`Copper`, `Tin`, `Iron` en haut ; `Coal`, `Cobalt`, `Silver` au milieu ; `Gold`, gemmes au fond),
  densités dans `grotte.ron`. Une pioche de Veloren dans le kit de chaque classe (`assets/icebreaker/classes/`,
  décision de Flo ; la foreuse E.6 viendra en plus).
  *Fin* : à la pioche, un filon du fond donne du minerai plus rare qu'un filon du haut (test de densité par palier).
- [ ] **K.5 Habitants et victoire** — créatures par ambiance (gobelins, truffler, fungome / dodarock, cave_spider,
  cave_troll / basilisk, blue_oni…) dans les poches et au bout des culs-de-sac, plus nombreuses en profondeur ;
  un chef par ambiance (cave_troll, basilisk, ogre…) dans la poche la plus profonde ; `dungeon_mission` étendu au
  plot pour que G.7 annonce « Mission accomplie » ; traces (tués, coffres) ; quelques coffres dans les culs-de-sac.
  *Fin* : au client texte, `/goto` chez le chef puis `/explosion` → victoire ; 25 à 40 créatures par grotte (test
  sur 20 graines).

Plus tard : trous à sens unique et cordes (E.7), filons comme carburant de la foreuse (E.4, E.6), ressources à
ramener (B.5, B.9), patrouilles (B.4), essaim qui sort des trous (E.15).

## Décisions (Flo, 2026-11-03)

1. La grotte se gagne comme le donjon : tuer le chef au fond.
2. Chaque classe part avec une pioche de Veloren (K.4).

## Questions pour l'équipe

1. **Ambiances** : trois dès K.3 (proposé, en données), ou une seule pour la démo ?
2. **Lumière** : grotte sombre où la lanterne compte (`design/donjon.md`, torche), ou assez de roche luisante pour
  jouer sans ? Proposé : lisible sans lanterne pour la première version.
