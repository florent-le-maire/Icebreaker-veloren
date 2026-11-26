# Game design — Créatures

Comportement de base, habitants (piste H) et fiches des créatures.
Index des règles : [`docs/GAME_DESIGN.md`](../GAME_DESIGN.md).

## Pour Veloren : l'IA de nos six espèces (jalon IA, 2026-11-26)

Le reste de ce fichier a été écrit pour le prototype Godot. Sur Veloren, ces règles valent pour nos six espèces
(humain, orc, nain, elfe, draugr, danari), dans les rôles guerrier, archer, mage et chef. Décision : `0024` ; tâches :
`PLAN-IA.md` ; chiffres : `assets/icebreaker/ia/` (ceux cités ici sont des valeurs de départ, à régler en jeu).

**Ce qu'on reprend**

- **Quatre états** : repos, enquête (l'« alerte » du texte d'origine), combat, fuite.
- **Les créatures ne trichent pas** (`menace.md`, principe 1) : un ennemi ne connaît que ce qu'il voit, entend ou
  reçoit d'un allié. Il ne connaît jamais la position d'un joueur qu'il ne perçoit pas.
- **Vue** : un cône devant lui, sans mur entre les deux ; tout près, il perçoit aussi dans son dos. Un joueur loin ou
  discret est d'abord **soupçonné** (l'ennemi se tourne et regarde), puis **repéré** s'il reste visible.
- **Ouïe** : un combat, une mort, un cri font un bruit qui porte à un rayon donné, multiplié par la finesse d'ouïe.
  Entendre un bruit → enquête : il va voir l'endroit, fouille un moment, puis rentre à son poste. Il entend aussi
  pendant un combat.
- **Combat** : il garde sa cible tant qu'il la perçoit. Perdue de vue, il la poursuit quelques secondes vers sa
  **dernière position connue**, puis enquête là. Un ennemi touché sait d'où vient le coup, même de dos.
- **Choix de cible** (Y.2, Y.9) : inchangé (debout avant à terre, répartition, danari qui achèvent à plusieurs).
- **Fuite** : sous un seuil de PV selon le profil ; le fuyard n'oublie pas (il ne revient pas voir) ; coincé, il tire
  une réaction.
- **Appel à l'aide, déroute à la mort du chef** : repris, avec des poids au lieu de rôles.
- **Après un combat ou une enquête**, il rentre à son poste.

**Ce qui change**

- **Pas de rôle fixe dans un groupe.** Il n'y a plus de guetteur : à chaque événement (il voit un intrus, un allié
  tombe, il est touché, ses PV passent un seuil, il reçoit un message…), chaque ennemi tire sa réaction, avec des
  poids par race, rôle et cran. Les poids tiennent compte de ce qu'il voit faire ses voisins (pas deux messagers).
- **Chaque décision tirée s'annonce** (`menace.md`, principe 2) : un cri, une bulle au-dessus de lui.
- **Prévenir** : celui qui tire « aller prévenir » court vers l'allié non alerté le plus proche (il connaît les postes
  des siens) et crie ; le cri porte un signalement (où il a vu les intrus, quand). Ceux qui l'entendent vont voir à
  cet endroit ; ils attaquent s'ils voient un joueur, sinon ils fouillent.
- **Trois crans d'habileté** (`design/difficulte.md`) règlent à quel point l'ennemi sait se battre : délai de
  réaction, roulade, parade, temps mort entre deux attaques, compétences utilisées. Les sens s'aiguisent un peu avec
  le cran et restent bornés.

**Ce qui ne se transpose pas**

- Les rôles brute, guetteur, archer, chef et leurs fiches (bas de ce fichier) : nos ennemis sont dans `ennemis/`.
- L'armement annoncé, `turnRate`, `trackLock`, la trajectoire de la lame : Veloren a ses propres temps de préparation.
- « ! » et « ? » au-dessus des têtes : des bulles de texte pour l'instant (Q73).
- Le bruit divisé par plancher : atténué selon l'écart de hauteur ; les murs n'arrêtent aucun son tant que les salles
  ne sont pas connues du serveur (IA.13).
- Faire le mort, se fondre dans l'ombre : demandent une animation et de la transparence côté client.
- L'encerclement : Veloren écarte déjà les ennemis les uns des autres, on n'y ajoute rien.
- La difficulté selon le nombre de joueurs, la provocation (Q50), les distances en mètres.

## Créatures : comportement de base

Chaque créature a un état : **repos**, **alerte**, **combat** ou **fuite** (réglages par créature ; valeurs par défaut) :
- **Vue** : cône de 120° sur 12 m, sans mur entre les deux ; à moins de 2 m elle perçoit aussi dans son dos.
- **Ouïe** : un bruit (combat, cri d'alarme) porte à un rayon donné, multiplié par la finesse d'ouïe de la créature.
  Entendre un bruit → alerte : elle va voir l'endroit pendant 6 s, puis revient au repos.
- **Combat** : elle garde sa cible tant qu'elle la perçoit, sinon prend l'ennemi le plus proche. Une cible perdue de vue
  est poursuivie 3 s vers sa dernière position connue, puis la créature passe en alerte à cet endroit.
- Une créature touchée sait d'où vient le coup, même de dos.
- **Fuite** : selon la créature, sous un seuil de PV ou quand un ennemi s'approche trop ; elle fuit tant qu'elle voit
  des ennemis (et encore 3 s après), puis passe en alerte.
- **Provocation** : prime sur tout, même sur la fuite : la créature attaque le lanceur, même sans le voir.
- **Armement** : les coups des créatures sont annoncés (« ! » rouge, animation, zone de la frappe en arc) pendant leur
  temps d'armement, puis partent vers l'endroit visé au début : on peut s'écarter.
  Coups au corps à corps suivis (`turnRate`, piste F) : pendant l'armement, la créature suit sa cible à vitesse
  limitée (gobelin 240°/s, brute, chef, guetteur et chef orque 120°/s), puis se fige `trackLock` s avant de frapper
  (0,15 à 0,2 s) ; pas de zone au sol pour ces coups, on lit le geste. Gobelin : armement 0,5 s, récupération 0,35 s.
  C'est leur arme qui touche (`blade` de `creatures.json`, trajectoire du geste : voir `combat.md`, frappes à la lame) :
  on l'esquive en sortant de sa trajectoire. Portées réelles : gobelin 0,9 m, guetteur 0,9, brute 1,3, chef orque 1,4,
  chef 1,8 (au bord du corps de la cible).
- Au-dessus d'une créature : « ! » quand elle passe en combat, « ? » quand elle cherche.
- Hors combat, une créature regarde où elle va ; en combat, elle fixe sa cible (et marche moins vite à reculons).
- Les créatures ne se blessent pas entre elles (`combat.creatureFriendlyFire`, désactivé).
- **Étages** : une créature voit à travers les vides mais pas à travers les planchers ; un bruit porte moitié moins
  loin par plancher traversé (`creatures.floorNoiseFactor`). Elle va chercher sa cible à un autre étage (escalier,
  échelle, fosse). La brute ne frappe et ne charge qu'au même niveau ; l'archer préfère les hauteurs et ne descend pas
  chercher une cible en contrebas ; le guetteur monte ou descend rejoindre ses renforts.
- **Difficulté selon le nombre de joueurs connectés** (`creatures.healthByPlayers`, `creatures.countByPlayers`) : PV des créatures
  ×1 (1 joueur), ×1,25 (2), ×1,5 (3)… jusqu'à ×3,25 (10) ; marqueurs occupés : 40 % de chaque type à 1 joueur, 60 % à 2, 80 % à 3,
  tous à partir de 4 (au moins une créature par type ; les chefs, objectif de la mission, sont toujours tous là), répartis dans tout le donjon. Quand un
  joueur arrive, des renforts apparaissent sur les marqueurs libres ; quand il part, les PV baissent mais personne ne disparaît.

### Sommeil et attaque surprise (tâche H.1)

- Une créature peut commencer **endormie** : lettre majuscule dans le plan (`B`, `A`, `G`, `C`) ; le générateur endort
  celles des salles `awake: false` (caserne, nid). Elle est couchée, des « z » montent au-dessus d'elle.
- Endormie, elle **ne voit rien** et n'entend que les bruits forts : portée des bruits × 0,3 (`brain.sleepHearingFactor`),
  soit un combat à moins de ~3,6 m. Elle se réveille aussi au premier coup, à une provocation, ou si un ennemi la frôle
  (à moins de 1 m, `brain.sleepSenseRange`, même dans son dos).
- Au réveil, elle reste **hébétée** 1,2 s (`brain.wakeDuration`) : elle se relève en se tournant vers ce qui l'a
  réveillée, sans bouger ni frapper.
- **Attaque surprise** : un coup sur une créature qui n'a rien remarqué (endormie, ou au repos) fait ×1,5 dégâts
  (`creatures.surpriseDamageMultiplier`) ; « Surprise ! » s'affiche. Approcher de dos ou à pas de loup paie.

### Vie au repos (tâche H.2)

- **Brute** : flâne jusqu'à 5 m de son point de départ (`wanderRadius`), à 40 % de sa vitesse (`wanderPace`), avec des
  pauses d'environ 5 s (`wanderPause`, ± 50 %). **Chef** : 3 m, pauses de 6 s.
- **Archer** et **guetteur** gardent leur poste (`wanderRadius` 0) mais tournent la tête à chaque pause
  (`lookAround`) : leur cône de vue balaie la salle ; on passe pendant qu'ils regardent ailleurs.
- Après un combat ou une alerte, la créature rentre d'abord chez elle (à pleine vitesse), puis reprend sa ronde.

### Appel à l'aide (tâche H.3)

- Les fragiles crient « À l'aide ! » quand on les frappe : l'**archer** sous 75 % de ses PV (`brain.helpHealthRatio`),
  le **guetteur** dès le premier coup. Le cri porte à 24 m (`brain.helpRadius`), au plus une fois toutes les 6 s
  (`brain.helpCooldown`).
- Les **brutes** et le **chef** défendent (`brain.defends`) : s'ils entendent l'appel, ils prennent l'agresseur pour
  cible (même s'ils se battaient contre un autre joueur) et savent où il est pendant 6 s (`brain.defendDuration`),
  même sans le voir. Les autres créatures qui l'entendent viennent seulement voir.
- Conséquence : on règle son compte à l'archer d'un coup, ou loin des brutes ; s'acharner sur lui attire la brute.

### Encerclement (tâche H.4)

- Quand une brute (ou le chef) arrive sur une cible déjà au contact d'autres créatures, elle ne fait pas la queue :
  elle contourne la cible pour l'attaquer du côté libre (deux brutes face à face, trois à 120°), en la regardant.
- Conséquence : à deux brutes contre un joueur, on est pris en tenaille ; il faut reculer contre un mur ou dans un
  couloir pour n'en avoir qu'une devant soi.

### Moral : la mort du chef (tâche H.5)

- Quand le chef tombe, son dernier cri porte à 24 m (`deathRoutRadius`) : « Le chef est tombé ! ». Les créatures qui
  l'entendent sont **en déroute** : brute 4 s, archer 8 s, guetteur 12 s (`brain.routDuration` ; 0 = tient bon).
- En déroute, une créature fuit l'ennemi le plus proche (ou l'endroit où le chef est tombé), même si on la frappe ;
  seule la provocation l'arrête. Ensuite elle se reprend et va voir où était la menace.
- « Fuite ! » s'affiche au-dessus d'une créature qui se met à fuir (déroute) ; « Des intrus ! » au-dessus d'un guetteur
  qui vient de repérer quelqu'un et court chercher de l'aide.

### Le fuyard n'oublie pas (tâche H.7)

- Une créature qui fuit (guetteur, PV bas) continue de fuir une menace qu'elle ne voit plus pendant `brain.fleeMemory`
  (10 s ; guetteur 20 s), puis **se calme là où elle est** : c'est sa nouvelle place (près des alliés qu'elle a
  rejoints). Elle ne revient jamais voir où était la menace (avant : 3 pas, elle perdait le joueur de vue, revenait
  le voir, refuyait… sans fin). La déroute (H.5) garde son « on se reprend et on va voir ».
- Le guetteur va prévenir l'allié **combattant** le plus proche, où qu'il soit dans le donjon (pas un autre guetteur).

### Coincé (tâches H.8 à H.11)

- Une créature qui fuit est **coincée** quand elle essaie d'avancer depuis 1 s (`brain.corneredTime`) sans
  s'éloigner (sur place au fond d'un cul-de-sac, ou son chemin la ramène vers la menace : la seule issue passe par
  les joueurs), menace à moins de 8 m (`brain.corneredRange`).
- Elle tire alors au sort une réaction selon son espèce (`brain.cornered`, des poids) ; sans réaction, elle continue
  de fuir comme elle peut. Guetteur : se cacher 3, forcer le passage 2, faire le mort 1, se fondre dans l'ombre 1.
  Pas de nouvelle réaction avant 3 s (`brain.corneredCooldown`).
- Cachée (ou faisant la morte), elle guette ; frappée, elle détale à travers son agresseur ; sans voir personne
  pendant 20 s (`brain.hideDuration`), elle se calme sur place. Elle n'appelle pas à l'aide (elle se trahirait).
- **Se cacher** (H.9) : elle file vers le point de sol le plus proche (2 à 8 m autour d'elle) qu'aucun ennemi en vue
  ne voit (derrière un pilier, un coin de porte), à plus de 2,5 m d'eux, et s'y tapit accroupie face à la menace.
  Aucun angle mort : elle se tapit sur place.
- **Embuscade** (H.9) : cachée (ou faisant la morte, ou dans l'ombre), un ennemi qu'elle voit passe à moins de
  2,5 m (`brain.ambushRange`) : elle frappe (`ambushAbility` ; guetteur : coup de dague, 12 dégâts, étourdit 1,2 s,
  armement 0,3 s, recharge 4 s), puis 0,8 s plus tard (`brain.ambushStrikeTime`) détale **à travers** sa victime.
  On peut parer ou bloquer la dague ; un coup qui la fait vaciller annule la sienne.
- **Forcer le passage** (H.10) : pendant 4 s au plus, elle fonce vers l'endroit où était la menace quand la fuite a
  commencé (par là qu'on est arrivé, donc la sortie) ; si elle y est déjà, ou que la menace est à moins de 3 m, vers
  un point 6 m derrière la menace, 1,5 m sur le côté (gauche ou droite au hasard : elle la contourne en la frôlant). Arrivée à la sortie, elle reprend sa fuite (le guetteur, vers ses alliés). Un joueur qui lui barre vraiment la route
  la bloque (collision) : coincée de nouveau, elle retire une réaction 3 s plus tard. C'est aussi ainsi qu'elle
  sort d'une cachette (frappée, ou après son coup d'embuscade).
- **Faire le mort** (H.11) : elle tombe sur place comme un cadavre (même animation qu'une vraie mort), sans nom ni
  barre de vie ; elle se relève pour son embuscade, ou détale si on la frappe.
- **Se fondre dans l'ombre** (H.11) : accroupie sur place, elle devient presque transparente (85 %) en 0,6 s, sans
  nom ni barre de vie ; elle réapparaît pour son embuscade. (Partout pour l'instant, pas seulement dans le noir.)
- Conséquence : tuer le chef au milieu de ses troupes offre un répit pour souffler, achever les fuyards ou partir.

Données modifiables : `game/data/creatures/creatures.json` (et leurs capacités dans `combat/abilities.json`).

| Créature | PV | Vitesse | Comportement |
|---|---|---|---|
| Brute (`b`) | 220 | 3,6 m/s, se retourne à 90°/s | Vue 16 m sur 110°. Coup de hache (32 dégâts, arc de 100° sur 2,6 m, armement 0,7 s, recharge 2,2 s) si la cible est devant elle ; charge (ruée de 9 m, 22 dégâts + étourdi 1 s, armement 0,9 s, recharge 7 s) si la cible est entre 4 et 9 m et bien en face. Lente à se retourner : on peut la contourner. |
| Guetteur (`g`) | 50 | 5,5 m/s | Vue 18 m sur 160°, oreille fine (×1,5). Ne se bat pas : dès qu'il voit quelqu'un, il fuit vers la créature alliée la plus proche et, à 6 m d'elle, crie l'alarme (armement 0,4 s, recharge 8 s) : un cri de 24 m qui envoie ceux qui l'entendent là où il a vu les intrus. Il reste ensuite auprès de ses alliés. Sans allié, il crie sur place puis s'enfuit. On a intérêt à l'abattre avant qu'il arrive ! |
| Archer (`a`) | 80 | 4,2 m/s (75 % à reculons), se retourne à 270°/s | Vue 22 m sur 140°. Carreau (14 dégâts, portée 24 m, armement 0,6 s, recharge 1,6 s) s'il voit sa cible et lui fait face. Recule face à elle si elle approche à moins de 7 m ; rattrapé (moins de 3,5 m), il tire à bout portant. S'approche jusqu'à 14 m ; cible hors de vue : va voir où il l'a vue. |

