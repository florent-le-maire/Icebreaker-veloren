# Classes et combos

*Proposition du 2026-10-21 (Flo et l'agent), à valider avec l'équipe.* Décision : `docs/decisions/0010-classes-et-combos.md`.
Découpage : `docs/roadmap/piste-S-classes.md`. Les valeurs sont des valeurs de départ, à régler en jouant ; ce sont
les données (`game/data/combat/`) qui font foi.

## Intention

Neuf classes **vraiment différentes** : chacune a sa façon de jouer (son « verbe ») et son rôle dans les combos.
Elles coopèrent **façon Guild Wars 2** : certaines actions **posent une zone** au sol, d'autres la **déclenchent**
pour un effet en plus. Un joueur seul fait le travail ; deux joueurs qui se coordonnent font beaucoup plus.

Inspirations de Flo : Deep Rock Galactic, Rainbow Six Siege, Guild Wars 2, Elden Ring Seamless Coop. Retenu : les
combos de GW2, posés sur le combat façon Elden Ring qui existe déjà (endurance, esquive, équilibre, relever un allié).

## Règles fixées avec Flo

1. **Les combos sont un gros bonus, jamais un passage obligé.** Un groupe qui ne fait aucun combo (débutants,
   composition bancale) finit le donjon, plus difficilement. Aucune porte, aucun boss ne demande un combo précis.
2. **Pas de trinité imposée** (tank / soigneur / dégâts) : chaque classe a un rôle propre, tout le monde peut relever
   un allié.
3. **Neuf classes** : guerrier, berserker, mage, rôdeur, assassin, clerc, ingénieur, druide, alchimiste.
4. **Ordre** : le système de combos d'abord, avec le guerrier et le mage ; puis les classes deux par deux.
5. **Un seul test à l'équipe, à la fin de la piste** (demande de Flo) ; le jeu reste jouable à chaque tâche.
6. **On peut faire un combo tout seul**, sur ses propres zones, comme dans GW2 (le mage pose du feu puis l'explose).
   Réglage `combat.selfComboFactor` (1 = effet entier) pour le réduire si le mage ou l'alchimiste écrasent les
   autres en jouant.

## Le vocabulaire des combos

### Les zones (ce qu'on pose)

Une zone est un disque au sol : un type, un centre, un rayon, une durée, un lanceur. Elle agit sur ce qui est dedans
à chaque instant. Deux zones du même type ne se cumulent pas sur une même cible.

| Zone | Durée | Rayon | Effet sur ce qui est dedans | Qui la pose |
|---|---|---|---|---|
| Feu | 4 s | 3 m | brûle : 8 dégâts/s | mage (boule de feu), alchimiste |
| Glace | 5 s | 4 m | ralentit de 40 % | mage (nova de givre), alchimiste |
| Eau | 6 s | 4 m | **mouillé** 6 s (voir réactions) ; éteint le feu et les torches | druide (pluie), alchimiste |
| Poison | 6 s | 3 m | 5 dégâts/s, continue 3 s après la sortie | assassin, alchimiste |
| Lumière | 6 s | 4 m | soigne les alliés de 6 PV/s | clerc (consécration), alchimiste |
| Fumée | 8 s | 4 m | les créatures ne voient ni dedans ni à travers ; un joueur dedans est caché | assassin (fumigène), alchimiste |

Zones nuisibles (feu, poison) et alliés : avec le tir allié (`combat.friendlyFire`, activé), elles blessent aussi les
alliés, à moitié (`combat.zoneFriendlyFactor`, 0,5). *Choix de l'agent, question ouverte.*

### Les déclencheurs (ce qui réveille une zone)

Une action de déclenchement qui a lieu **dans** une zone (ou la traverse) produit un effet en plus. La zone reste,
sauf pour l'explosion qui la consomme.

| Déclencheur | Ce qui compte | Exemples |
|---|---|---|
| **Tourbillon** | une attaque autour de soi (forme `self`) lancée depuis l'intérieur de la zone | tourbillon (épée à deux mains), rage du berserker |
| **Traversée** | une ruée (forme `dash`) qui passe dans la zone ; l'esquive ne compte pas | charge du guerrier, bond de l'assassin, charge du berserker |
| **Projectile** | un projectile qui traverse la zone | flèches du rôdeur, carreaux des tourelles, fioles |
| **Explosion** | une explosion en zone dont le centre tombe dans la zone | boule de feu, bombe de l'ingénieur |

### La table des combos (zone × déclencheur)

| Zone | Tourbillon | Traversée (imprègne l'arme 5 s ou 3 coups) | Projectile | Explosion (consomme la zone) |
|---|---|---|---|---|
| Feu | anneau de flammes : 20 dégâts autour | arme enflammée : +8 dégâts, brûle 2 s | flèche de feu : +10, brûle 2 s | explosion : 30 dégâts sur 4 m |
| Glace | nova gelante : ralentit de 70 % 2 s | arme gelée : chaque coup ralentit | flèche gelée : ralentit | éclats : étourdit 1 s |
| Eau | vague : repousse de 3 m | se lave : retire ses effets (brûlure, poison) | flèche d'eau : mouille | vapeur : la zone devient fumée |
| Poison | la zone grossit de 50 % | lame empoisonnée : chaque coup empoisonne | flèche empoisonnée | nuage toxique : double dégâts 2 s, rayon × 1,5 |
| Lumière | soin de 25 PV aux alliés proches | chaque coup soigne le lanceur de 5 PV | flèche bénie : soigne de 20 PV l'allié touché | éclat aveuglant : étourdit les ennemis 1,5 s |
| Fumée | voile : le lanceur et ses alliés proches cachés 3 s | le lanceur caché 3 s | flèche fumigène : fumée au point d'impact | dispersion : rayon × 2 |

Précisions (choix de l'agent) :
- **Ce qui déclenche** : seules les capacités marquées dans les données (`finisher` dans `abilities.json` :
  `whirl`, `dash`, `projectile`, `blast`). Au départ : tourbillon, charge, boule de feu, flèches, carreaux, bombes.
  La nova de givre, la provocation et l'esquive ne déclenchent rien.
- **Plusieurs zones à la fois** : un projectile déclenche **la première** zone qu'il traverse ; un tourbillon, une
  traversée ou une explosion déclenchent **la plus récente** des zones où ils ont lieu. Un seul combo par action.
- **La boule de feu** tombée dans une zone la fait exploser (la zone disparaît), puis laisse sa propre zone de feu.
- **Imprégnation** (traversée) : elle renforce les coups au corps à corps (arc, ruée, autour de soi), pas les
  projectiles ; chaque cible touchée compte pour un coup (un balayage qui touche trois ennemis en use trois). La
  brûlure (« brûle 2 s ») fait 4 dégâts/s. Valeurs : `game/data/combat/combos.json`.
- **Qui est touché** : les effets qui gênent (ralenti, mouillé, gelé, étourdi) ne touchent que les ennemis ; les
  soins et le « caché » que les alliés ; seuls les **dégâts** suivent le tir allié. La flèche bénie soigne les alliés
  à moins de 2 m de son point d'impact (les projectiles ne touchent toujours pas les alliés).
- **Créatures** : dans cette piste, elles ne posent ni ne déclenchent de zones ; chaque zone garde le camp de son
  lanceur pour qu'elles puissent le faire plus tard.

À l'écran, un combo réussi s'annonce (nom du combo au-dessus du lanceur, couleur de la zone), pour que les joueurs
apprennent la table en jouant.

### Les réactions (zone + zone)

Deux zones qui se touchent réagissent. C'est le terrain de jeu du druide et de l'alchimiste (tâche S.11).

| Zones | Réaction |
|---|---|
| Eau + feu | le feu s'éteint, la zone devient **fumée** (vapeur) |
| Eau + glace | **verglas** : les mouillés dedans sont gelés (étourdis 1,5 s) |
| Feu + poison | **explosion** : 30 dégâts, les deux zones disparaissent |
| Lumière + poison | **purification** : le poison disparaît |
| Mouillé + glace (cible) | gelé : étourdi 1,5 s, il n'est plus mouillé ; pas de nouveau gel tant qu'il est étourdi (`freezeWet`) |

Valeurs : `game/data/combat/reactions.json` (et `freezeWet` de la glace dans `zones.json`). Ce que la réaction pose ou
blesse est au lanceur de la plus récente des deux zones. Choix de l'agent : eau + glace n'est pas une réaction de
zones (le gel vient de la cible mouillée qui entre dans la glace).

## Les classes

Chaque classe garde la base commune (endurance, esquive, course, saut, relever un allié) et les touches actuelles :
**F** compétence de classe, **Maj + clic droit** compétence de l'arme, **clic droit** objet de main gauche (bouclier,
torche… et désormais les **outils de classe** : fumigènes, fioles, bombes), **1 / 2** font défiler les mains.

| Classe | Son verbe | PV | Endurance | Équilibre | Armes | Rôle dans les combos | Modèle |
|---|---|---|---|---|---|---|---|
| Guerrier | tenir la ligne | 150 | 100 | 60 | épée, hache, épée à deux mains, lance ; bouclier, torche | déclenche (tourbillon, charge) | Warrior (fait) |
| Berserker | tout risquer | 130 | 110 | 70 | grosse hache, épée à deux mains ; pas de bouclier | déclenche et **absorbe** | Warrior recoloré |
| Mage | frapper de loin | 90 | 80 | 30 | bâton, baguette ; grimoire, torche | pose feu et glace, explose | Wizard (fait) |
| Rôdeur | marquer, tenir la distance | 100 | 100 | 35 | arc ; dague | déclenche (projectile) | Ranger |
| Assassin | ne pas être vu | 95 | 110 | 30 | dague ; fumigènes, fioles de poison | pose poison et fumée, traversée | Rogue |
| Clerc | garder le groupe debout | 120 | 90 | 50 | bâton, masse ; bouclier | pose la lumière | Cleric |
| Ingénieur | construire et casser | 110 | 90 | 45 | arbalète, marteau ; bombes, outils | explose, ses tourelles tirent dans les zones | à créer |
| Druide | façonner le terrain | 110 | 90 | 40 | bâton | pose eau et ronces, réactions | Monk recoloré |
| Alchimiste | changer les zones | 95 | 85 | 35 | dague ; fioles | pose tout, transforme les zones | à créer |

### Guerrier (existe)

Garde, parade, provocation (F). Il protège : il détourne les créatures pendant que les autres préparent leurs combos.
Ses déclencheurs : tourbillon (épée à deux mains), charge.

### Berserker

- **Rage** (barre propre, 0 à 100) : +4 par coup porté, +1 par 2 PV perdus, −5/s hors combat. Dégâts × (1 + rage/200)
  (jusqu'à +50 %), et **plus il a peu de PV, plus il fait mal** (+30 % sous 30 % de PV).
- **Pas de bouclier, pas de parade.** Il esquive ou il encaisse.
- **F — Furie** (rage ≥ 50, consomme toute la rage) : 8 s pendant lesquelles il ne vacille pas et regagne 20 % des
  dégâts qu'il fait.
- **Armes** : hache de guerre (gestes de l'épée à deux mains, coups lourds de hache) et épée à deux mains ; la
  rage s'affiche en rouge sous l'endurance.
- **Absorbe** : il entre dans une zone en marchant et s'en **imprègne** comme par une traversée, sans en subir les
  dégâts ; la zone disparaît. Feu → arme enflammée, lumière → soin à chaque coup, etc.
- Coopératif sans être un soutien : il a besoin du clerc pour survivre, du guerrier pour souffler, des zones des autres.

### Mage (existe)

La boule de feu (bâton) laisse une **zone de feu** à l'impact ; la nova de givre (grimoire) laisse une **zone de
glace**. Une boule de feu qui tombe dans une zone existante la fait **exploser**. Transfert (F) : inchangé.

### Rôdeur

- **Arc** : tir léger (clic gauche), tir chargé (maintenu, plus fort, plus loin). Fragile au contact : la dague en
  arme de secours.
- **F — Marque** : l'ennemi le plus proche du point visé (25 m) prend +20 % de dégâts de tous les coups pendant 8 s.
  Recharge 12 s. *Pas encore fait* : le voir à travers les murs (seul « Marqué » s'affiche au-dessus de lui).
- **Volée** (compétence de l'arc) : 5 flèches en éventail de 30° ; chaque flèche qui traverse une zone déclenche son
  combo (annoncé une fois par seconde, pas à chaque flèche).
- Ses flèches sont les déclencheurs « projectile » les plus faciles à placer.

### Assassin

- **F — Ombre** : caché 6 s (les créatures ne le voient plus) ou jusqu'à sa première attaque ; recharge 15 s ; sans
  bruit. *Pas encore fait* : le personnage caché ne devient pas translucide, seul « Caché » s'affiche.
- **Coup dans le dos** : × 2,5 dégâts sur une créature qui ne l'a pas repéré, ou frappée par derrière (arc de 120°,
  `combat.backstabAngle`) ; il remplace l'attaque surprise de tous (× 1,5).
- **Pas de l'ombre** (compétence de la dague) : ruée silencieuse de 5 m, 8 dégâts (déclencheur traversée).
- Main gauche : **fumigènes** (zone de fumée là où ils tombent, 3 par donjon) et **fioles de poison** (zone de
  poison, 3) ; le clic droit les lance, la case de la touche montre ce qu'il en reste (×3). On n'en regagne pas en bas
  (`charges` des capacités). La fumée coupe la vue des créatures comme un mur : ni dedans, ni à travers.
- La fumée sert à tout le groupe : passer sans être vu, couper la vue d'un archer, se relever à l'abri.

### Clerc

- **F — Consécration** : pose une zone de lumière de 4 m à ses pieds. Recharge 14 s.
- **Bénédiction** (compétence du bâton) : une bulle sur un allié visé qui absorbe 40 dégâts, 8 s.
- **Relève deux fois plus vite** que les autres (2 s au lieu de 4) et l'allié revient avec 75 % de ses PV.
- Bouclier possible : il peut tenir la garde en soignant, mais pas aussi bien que le guerrier (équilibre 50) ; avec
  le bouclier, la parade prend la place de la Bénédiction (choix de l'agent : bouclier ou bénédiction, il faut choisir).
- Bâton de clerc (une main) : 3 coups légers, 1 lourd (chargé possible), au contact. Données : `abilities.json`, section Clerc.

### Ingénieur

- **Arbalète** (deux mains) : carreaux lents et lourds, recul, tir chargé ; ses carreaux déclenchent les combos.
  *Pas fait* : percer une créature.
- **F — Tourelle** : posée vers le point visé (6 m, 2,5 m en deçà d'un ennemi visé), 20 s, une à la fois ; elle tire
  toutes les 1,2 s sur l'ennemi le plus proche qu'elle voit (14 m). Ses carreaux déclenchent les combos des zones
  qu'ils traversent. *Choix de l'agent* : elle ne se détruit pas (pas de PV) pour commencer ; formes simples.
- **Construire** (main gauche, avec la hachette en main droite ; 3 de chaque) : barricade de bois sur la case de sol
  libre devant (bloque un couloir, les armes la cassent), planche sur un trou (jusqu'à 2 cases devant). Pas de place :
  rien n'est dépensé. *Choix de l'agent* : l'arbalète est à deux mains, donc on construit hachette en main.
- **Bombes** : 3 de plus que les autres classes (5 au départ, `extraEquipment`) ; toute bombe (de n'importe qui) est un
  déclencheur explosion. *Pas fait* : lancées plus loin, foreuse deux fois plus rapide.

### Druide

- **F — Pluie** : pose une zone d'eau de 4 m au point visé (portée 12 m). Recharge 10 s.
- **Ronces** (compétence du bâton) : nouvelle zone (thorns) de 3 m 5 s au point visé (10 m) qui ralentit de 60 % et
  fait 4 dégâts/s ; elle a ses 4 combos (`combos.json`). *Pas fait* : enraciner ce qui y entre en courant.
- **Bâton de druide** (deux mains) : il frappe au contact, gestes du bâton de clerc.
- Il prépare les réactions : eau puis glace du mage = gel ; eau sur le feu = vapeur (fumée). *Pas fait* : l'eau qui
  éteint les torches du décor (créatures dans le noir).
- **Forme d'ours** : *plus tard* (il faut un modèle d'animal) ; question ouverte.

### Alchimiste

- **Fioles comptées** (main gauche, 2 fait défiler ; clic droit les lance) : six sortes (feu, glace, eau, poison,
  lumière, fumée), 3 de chaque par donjon. *Choix de l'agent* : la main gauche n'a que trois emplacements, il en
  emporte trois sortes, choisies à l'écran de choix (feu, glace, eau par défaut). *Pas fait* : en trouver dans les
  coffres, les préparer au village.
- **F — Catalyseur** : la zone au point visé (10 m) devient du type de la fiole en main, qui est dépensée (rien si la
  zone est déjà de ce type). Recharge 8 s. Dague en main droite (pas de l'ombre compris).
- Une fiole qui tombe sur une autre zone la touche : la réaction part tout de suite (vapeur, explosion toxique…).
- Le plus difficile à bien jouer ; le plus fort dans un groupe qui joue ensemble.

## Ce qui n'est pas décidé (voir « Questions ouvertes » de la piste)

Tir allié des zones, forme d'ours, fioles et bombes comptées, deux joueurs de la même classe, modèles de l'ingénieur
et de l'alchimiste, comportement des créatures face aux zones.
