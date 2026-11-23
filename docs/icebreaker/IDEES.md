# Boîte à idées

Idées de fonctions, pas encore décidées ni découpées. Quand l'équipe en retient une, elle passe en tâche dans le
plan concerné (`PLAN.md`, `PLAN-BOUCLES.md`) et sort d'ici. Effort sur Veloren : P petit, M moyen, G gros.

## Âmes des monstres (Sayanel, 2026-11-03)

Âme lâchée rarement par un monstre, inconnue (blanche) jusqu'à ce qu'on la lie à son personnage, puis reconnue
(en couleur) ; Codex au village. *Retenue le 2026-11-16* : règles dans `design/ames.md`, tâches AM.1 à AM.8 dans
`PLAN-PLAYTEST.md`.

## Boucles de jeu (lectures de game design, 2026-11-02)

Ce que les lectures apprennent :
- **Une boucle tient si on la rejoue et qu'elle reste intéressante** (Cook) : préférer des règles qui se croisent à du
  contenu écrit à la main, qui s'use.
- **La peur de perdre fait la tension** (jeux d'extraction) : elle monte avec ce qu'on porte, retombe à la sortie ;
  une perte qui ne compte pas rend le jeu plat.
- **Priver de soin entre deux combats est un choix assumé** (Darkest Dungeon) ; une astuce qui casse la tension se
  corrige par une règle cohérente avec l'univers.
- **Progresser par le matériel, pas par le niveau** (Valheim, Monster Hunter) : ce qu'on tue débloque ce qu'on fabrique.
- **Pression d'un quota, joueur qui guide depuis l'abri, relever un allié** (Lethal Company, Deep Rock Galactic).

### Expédition

1. *(Relever un allié à terre : retenue, devenue le jalon T de `PLAN.md`, 2026-11-04.)*
2. **Le noir paie** (M) — salles sombres : coffres plus riches, surprises plus fréquentes ; une torche levée se voit
   de loin. S'appuie sur l'éclairage G.10. *Jauge de lumière de Darkest Dungeon.*
3. **Curiosités** (P à M) — autel, fontaine, cadavre d'aventurier, caisse scellée : avec le bon objet apporté (eau
   bénite, pied-de-biche, antidote), un bonus ; sans, on tente sa chance (malédiction, poison). Relie l'intendance au
   donjon. *Darkest Dungeon.*
4. **Dépouilles de chef** (P) — chaque chef laisse une pièce de sa faction (croc d'adlet, totem gnarling). Donne une
   raison de choisir un contrat plutôt qu'un autre. *Monster Hunter, Valheim.*
5. **Butin qui pèse** (M) — au-delà d'un seuil, on ralentit et on ne roule plus ; il faut choisir quoi garder.
   *Jeux d'extraction.*
6. **Sorties secondaires** (M) — corde, puits, éboulis à dégager, à découvrir en plus de l'entrée ; le parchemin de
   retour reste l'assurance qu'on achète. *Hunt: Showdown.*

6 bis. **Formes du donjon, la suite** (P à M, 2026-11-23, après F.46 à F.49) — ce que donjon.bin.sh a et que nous
   n'avons pas repris : un plan **symétrique** (difficile avec un graphe de mission) ; une silhouette en plusieurs
   morceaux (château : enceinte, cour, donjon au milieu). Et de notre côté : un petit butin ou un piège au bout
   d'une impasse, une silhouette différente par étage.

### Village et campagne

7. **Bâtiments débloqués par un trophée** (P) — forge niveau 2 : un croc d'adlet ; taverne niveau 2 : un totem
   gnarling. Suite de l'idée 4. *Valheim.*
8. **Tribut du seigneur** (P) — tous les 3 jours de carte, la caisse doit avoir X or, sinon pénalité (bâtiment saisi,
   prix en hausse). Répond à la question 3 de `PLAN-BOUCLES.md` (entretien). *Quota de Lethal Company.*
9. **Le joueur qui guide depuis le village** (M) — en vue de dessus : PV de l'expédition, alertes, messages, et une
   fois par expédition un ravitaillement de l'entrepôt qui apparaît à l'entrée. Rôle d'intendant de la vision.
   *Lethal Company.*
10. **Mémorial et objets qui ont une histoire** (P) — noms des morts sur un monument ; l'arme d'un mort rapportée
    porte son nom (« Épée de Bertrand »). Des histoires sans rien écrire. *Darkest Dungeon.*
11. **Événement du jour** (P) — un par jour de carte : marchand de passage, fête (soins −50 %), pénurie (potions ×2),
    blessé à la porte. *Hameau de Darkest Dungeon.*
12. **Assurance** (P) — payer avant de partir pour récupérer son équipement en cas de mort. *Jeux d'extraction.*
13. **Le donjon garde ce qu'on y a fait** (G) — un contrat raté se reprend (chefs morts, portes ouvertes), mais les
    créatures se sont réorganisées ; demande de revoir la remise à neuf I.9. *Vision, `design/menace.md`.*
14. **Recrues fragiles à traits, registre commun** (G) — déjà décrit dans `design/village.md` ; après la démo.

Recommandé en premier : 1, 3, puis 4 + 7 ; ensuite 8 et 10 ; 9 à essayer si l'on joue souvent à trois ou plus.

Sources : [Loops and Arcs](https://lostgarden.com/2012/04/30/loops-and-arcs),
[interview Darkest Dungeon](https://gamesbeat.com/darkest-dungeon-designers-really-arent-sadists-were-realists/),
[Light Meter](https://darkestdungeon.wiki.gg/wiki/Light_Meter),
[boucles d'extraction](https://cogconnected.com/2026/08/how-extraction-loops-changed-reward-design/),
[Lethal Company](https://gamecritics.com/gc-staff/lethal-company-review/),
[Valheim](https://guildorder.com/es/games/valheim/wiki/boss-and-biome-progression),
[Monster Hunter: World](https://en.wikipedia.org/wiki/Monster_Hunter:_World).
