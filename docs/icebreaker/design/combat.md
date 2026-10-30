# Game design — Combat

Classes, armes, capacités, endurance, esquive, équilibre (valeurs de départ, à ajuster en jouant).
Index des règles : [`docs/GAME_DESIGN.md`](../GAME_DESIGN.md).

## Combat (valeurs de départ, à ajuster en jouant)

Données modifiables : `game/data/combat/classes.json`, `weapons.json` et `abilities.json` (durées en s, distances en m).

**Armes** (piste A) : chaque classe a une liste de types d'armes permis. Une arme se porte à une main (la main gauche
peut alors tenir un bouclier, un grimoire, une torche), en main gauche, ou à deux mains (main gauche rangée). La **lance**
(piste W) change de prise : **à deux mains si la main gauche est vide** (estocs en légère, balayages en lourde), **à une
main sinon** (tous ses coups sont des estocs) ; `withOffHand` de `weapons.json` donne les suites de la prise à une main.
**Touches** (piste E, façon Elden Ring ; toutes changeables dans Options, Maj + touche compris). **Manette** (piste K,
disposition d'Elden Ring, fixe ; `GamepadLayout`, réglages dans `camera.gamepad`) : RB légère, RT lourde (maintenue :
chargée), LB main gauche, LT compétence d'arme, X compétence de classe, B roulade / course, A saut, Y interagir, L3
s'accroupir, R3 verrouiller, croix droite / gauche arme / objet suivant, Affichage carte (croix haut / bas : feuilles),
Menu pause. La lourde en courant est donc RT en tenant B.

| Touche | Action |
|---|---|
| Clic gauche | **attaque légère** (main droite) : sa suite légère, un clic par coup |
| Maj + clic gauche | **attaque lourde** (main droite) : sa suite lourde ; maintenue jusqu'à la fin de l'armement = **coup chargé** ; **en courant : attaque en course** (`run` de l'arme : épée, hache, épée à deux mains), qui garde l'élan (pas en avant de 2,2 à 2,5 m) |
| Clic droit | **main gauche** : **garde** au bouclier (maintenue), **lever la torche** (maintenu). Main gauche libre, grimoire ou arme à deux mains : l'attaque lourde, comme Maj + clic gauche |
| Maj + clic droit | **compétence d'arme** ; celle de la main gauche passe avant celle de l'arme principale (bouclier : **parade**) |
| F | compétence de classe |
| 1 / 2 | arme suivante en main droite / objet suivant en main gauche |
| Maj | appui : roulade (sans direction : en arrière) ; maintenue : course. En courant, un clic est donc un « Maj + clic » |
| R / 3 | employer l'outil de destruction (bombe, foreuse) / outil suivant |
| 4 / 5 | **employer le consommable** choisi du paquetage (potion : +60 PV d'un coup ; bandage : +45 PV en 6 s, pas à moins de 5 s d'un coup reçu, un coup l'arrête ; antidote : retire le poison) / consommable suivant (phase 3, `village/items.json`) |
| C / X | **s'accroupir** / se lever : moitié de la vitesse, les créatures ne le voient qu'à la moitié de leur portée de vue (`player.crouch`) ; courir, rouler, sauter, prendre une échelle relèvent. Pas d'animation accroupie dans les modèles : le personnage se tasse en attendant |

**Mains gauche et droite** (piste E) : à l'écran de choix de classe, on remplit **3 emplacements par main** parmi ce
que la classe peut porter : armes en main droite (au moins une), bouclier / grimoire / **torche** en main gauche. En
jeu, **1** passe à l'arme suivante de la main droite (les emplacements vides sont sautés), **2** à l'objet suivant de
la main gauche (un emplacement vide = **main libre**, une seule fois). Pas en plein coup, ni pendant une parade, une
ruée, étourdi ; 0,4 s entre deux changements (`combat.weaponSwapCooldown`, commun aux deux mains). Une arme à deux mains
**range la main gauche** (son objet reste dans l'emplacement, grisé au HUD) ; on peut quand même faire défiler la main
gauche. Les recharges des compétences continuent de tourner. Départ (`classes.json`) : guerrier épée / épée à deux
mains / hache + bouclier / torche / main libre ; mage bâton / baguette + grimoire / torche. Pas de double arme (épée
en main gauche) pour l'instant.

**Arsenal** (A.10) : chaque arme a son rythme, ses suites légère et lourde et sa compétence. Durée d'une suite =
armements + récupérations de tous ses coups, enchaînés au plus vite.

| Classe | Arme | Ressenti | Suite légère (clic gauche) | Suite lourde (Maj + clic gauche) | Chargé (maintenu) | Compétence (Maj + clic droit) |
|---|---|---|---|---|---|---|
| Guerrier | Épée | rapide, précise | 4 coups en 2,7 s : 18, 18, 20 (estoc), 30 | 2 coups : 38, 42 (60–65 d'équilibre) | 60, 65 (95–100 d'équilibre), +0,45 s | Charge |
| Guerrier | Hache | plus lente, déséquilibre (« perce la garde ») | 3 coups en 3 s : 27, 30, 40 (150 d'équilibre en tout) | 2 coups : 48, 50 (80–85) | 75, 75 (130), +0,5 s | Brise-garde |
| Guerrier | Épée ou hache + bouclier | la suite légère de l'arme, et le bouclier | comme l'arme | comme l'arme ; clic droit : garde (maintenue) | comme l'arme | Parade |
| Guerrier | Épée à deux mains | lente, large (160°), gros dégâts | 2 coups en 2,35 s : 30, 38 | 2 coups : 60 (écrasant), 64 (estoc 3,6 m) | 90, 95 (150), +0,6 s | Tourbillon |
| Guerrier | Lance à deux mains (main gauche vide) | portée la plus longue (lame 1,5 m), étroite (30°) | 3 estocs : 18, 20, 26 | 2 balayages (170°) : 30, 34 | grand estoc 60 (120), repousse 2,5 m | Charge |
| Guerrier | Lance à une main (bouclier, torche) | rien que des estocs (25°) | 2 piques : 15, 17 | 1 pique lourde : 32 | 50 (90), repousse 1,5 m | celle de la main gauche (parade) sinon Charge |
| Mage | Baguette + grimoire | rapide, à distance | 4 traits en 1,5 s : 8, 8, 8, 16 | 2 orbes en zone de 1,5 m : 20, 22 | 40, 42 en zone de 2,5 m, +0,45 s | Nova de givre (grimoire) |
| Mage | Bâton | lent, zones et contrôle | 3 projectiles de givre en 1,85 s : 12, 12, 22 (ralentissent) | 2 éclats de glace en zone de 2 m : 26, 28 | 45, 48 en zone de 3 m, +0,5 s | Boule de feu |

Endurance : les coups légers des armes coûtent 12 à 26, les lourds 26 à 36, les chargés 36 à 46 ; les sorts légers
sont gratuits, les sorts lourds coûtent 12 à 15 et les chargés 22 à 25. Tout est dans `abilities.json` (une ligne
par coup).

| Compétence | Effet |
|---|---|
| Charge (épée) | ruée de 8 m, 15 dégâts + étourdi 0,8 s, recharge 8 s |
| Brise-garde (hache) | 22 dégâts, 100 d'équilibre (un guerrier vacille d'un coup), recharge 8 s |
| Tourbillon (épée à deux mains) | 28 dégâts autour de soi sur 3,2 m, recharge 7 s |
| Nova de givre (grimoire) | 10 dégâts autour de soi sur 4 m, −50 % de vitesse 2,5 s, recharge 10 s |
| Boule de feu (bâton) | 35 dégâts en zone de 3 m, recharge 6 s |

| Classe | PV | Vitesse | Compétence de classe (F) |
|---|---|---|---|
| Guerrier | 150 | 5 m/s | Provocation (zone de 8 m, 4 s, recharge 12 s) |
| Mage | 90 | 5 m/s | Transfert (téléportation de 7 m, recharge 8 s) |

Règles :
- Un mort ne prend plus de dégâts ni d'effets, et ses effets disparaissent. On ne soigne pas un mort : il faut le relever.
- Un même effet ne se cumule pas : on garde la durée et l'intensité les plus fortes ; la dernière provocation l'emporte.
- Étourdi : ni déplacement ni capacité. Ralenti : vitesse × (1 − intensité).
- Une capacité hors de portée ou en recharge est refusée (la recharge ne démarre pas). On ne se touche jamais soi-même ;
  les alliés seulement si `combat.friendlyFire` est activé.
- Toutes les capacités demandent un appui à chaque fois (plus de tir continu en maintenant la touche).
- **Attaques en trois temps** (piste A) : armement (`windup`) → coup → récupération (`recovery`). Pendant l'armement et
  la récupération, on est engagé : on ne garde que `moveFactor` de sa vitesse (0,25 par défaut), on ne saute pas et on
  ne lance rien d'autre. Un appui pendant le coup (armement ou récupération) n'est pas perdu : il est gardé (le
  dernier compte) et part juste après la récupération. Plusieurs clics pendant un coup ne donnent qu'un coup suivant.
- **Engagement au corps à corps** (piste F, façon Elden Ring) : pendant un coup au corps à corps (armement et
  récupération), on ne se dirige plus (`moveFactor` ne compte plus que pour les sorts) : c'est le coup qui fait avancer
  d'un pas (`lunge` : 0,8 m pour les coups légers, 0,6 à la dague, 1 pour le dernier d'une suite, 1,2 pour les lourds,
  1,5 pour les chargés ; piste G : plus long qu'avant, on ne le sentait pas), droit devant. Un corps au contact
  l'arrête : collé à l'ennemi, on n'avance plus. Le personnage fait face au point visé au lancement et garde cette
  direction jusqu'à la fin du geste (armement, charge et récupération), au moins `player.attackCommitTime`. Le pas en
  avant est décidé par le serveur, au rythme de l'animation (elle aussi décidée par le serveur) : chez le client, le
  personnage glisse vers sa position, comme pour un recul.
- **Suites légère et lourde** : chaque arme principale donne le premier coup de sa suite légère (`light` dans
  `weapons.json`) et de sa suite lourde (`heavy`) ; les coups suivants sont les `combo` de `abilities.json`, le
  nombre de coups est propre à chaque arme, et le dernier est plus fort. Rappuyer sur la même attaque pendant le coup
  ou dans les 0,6 s qui suivent sa récupération (0,7 s pour une suite lourde ; `comboWindow`) porte le coup suivant ;
  après le dernier, on repart du premier. Une pause
  plus longue, ou l'autre attaque, repart du premier coup : léger 1 → léger 2 → lourd donne le **premier** coup lourd.
- **Gestes d'un enchaînement** (piste G) : plus une arme est rapide, plus sa suite légère est longue (dague 4 coups,
  épée et bâton du clerc 3, hache 3, grandes armes 2 ; lance à deux mains 3 estocs). D'un coup au suivant, l'arme
  change de sens (vu de dos : gauche → droite, puis droite → gauche, puis de nouveau) et de hauteur ; la hache part
  toujours du même côté (gauche → droite) mais change de hauteur. Les estocs et le coup vertical final cassent le
  rythme. Sens mesuré sur la trajectoire de la lame (`SwingShape`), vérifié par les tests `ComboGrammarTests` et
  `ComboLengthTests`.

  | Arme | Suite légère | Suite lourde |
  |---|---|---|
  | Épée | diagonale G→D, revers D→G, coup vertical | diagonale G→D, revers D→G |
  | Dague | taille G→D, revers D→G, taille basse G→D, estoc | estoc |
  | Hache | diagonale G→D, taille G→D, grande diagonale G→D | grande diagonale G→D, balayage G→D |
  | Épée à deux mains | taille G→D, revers D→G | grande diagonale G→D, estoc |
  | Hache de bataille | taille G→D, revers D→G | grande diagonale G→D, balayage G→D |
  | Lance (deux mains) | trois estocs | balayage G→D, revers D→G |
  | Lance (une main, avec bouclier ou torche) | trois piques | pique lourde |
  | Bâton du clerc, du druide | diagonale G→D, revers remontant D→G, grande diagonale G→D | grande diagonale G→D |

  Le tour complet (gros marteau) viendra avec le marteau.
- **Coup chargé** : garder la touche d'attaque lourde (Maj + clic gauche, ou clic droit main gauche libre) enfoncée jusqu'à la fin de l'armement d'un coup lourd le change en sa version
  chargée (`charged`), qui s'arme encore un moment ; ça marche à n'importe quel coup de la suite lourde, et la suite
  continue ensuite comme après le coup normal. À l'écran (A.11), l'arme d'un coup lourd reste levée à la fin de
  l'armement ; chargé, elle reprend de l'élan et le personnage s'illumine de plus en plus jusqu'au coup, pour que
  tout le monde le voie venir.
- **Endurance** (barre verte sous les PV) : les attaques (`staminaCost`) et l'esquive en dépensent. Une action part
  tant qu'il en reste un peu, même si elle coûte plus ; à 0, plus d'attaque payante ni d'esquive. Elle remonte
  (`staminaRegen` par seconde) après 0,8 s sans dépense (`combat.staminaRegenDelay`). Guerrier 100 (+40/s), mage 80
  (+35/s). Coûts des attaques : voir l'arsenal ci-dessus ; charge 20, esquive 25.
- **Esquive** (appui bref sur Maj, part au relâchement) : roulade de 4,5 m dans la direction du déplacement (en
  arrière à l'arrêt), invulnérable les 0,3 premières secondes (`combat.dodge`). Au sol seulement ; impossible pendant
  un armement, mais elle coupe une récupération. Demandée pendant un armement, elle attend sa fin et part dès que le
  coup est porté (piste Q) ; chez le joueur, elle s'affiche tout de suite, même en plein coup au corps à corps.
  Geste et course propres à la classe (`dodge` de `classes.json`) : guerrier = roulade (animation « Roll » de
  Quaternius, 4,5 m en 0,41 s), mage = dash (penché, traînée bleue, 4 m en 0,2 s) ; endurance, invulnérabilité
  (0,3 s) et recharge restent communes.
- **Appui en attente** (`player.inputBuffer`, 0,15 s) : une esquive ou un saut demandés un peu trop tôt (pendant un
  armement, une ruée, la recharge de l'esquive, juste avant d'atterrir) partent dès que possible au lieu d'être oubliés.
- **Course** (Maj maintenue plus de 0,25 s, `player.sprint`) : vitesse × 1,6, 15 d'endurance par seconde (elle ne
  remonte pas en courant). Seulement en se déplaçant, sans attaque en cours, bouclier baissé, carte rangée, hors
  échelle. À bout d'endurance, on marche jusqu'à relâcher Maj.
- **Équilibre** (`poise`) : chaque coup reçu en retire (`poiseDamage`, autant que les dégâts par défaut). À 0, on
  vacille 0,8 s (`combat.staggerDuration`) : le coup qu'on armait est perdu ; l'équilibre repart plein, et revient plein
  après 3 s sans coup (`combat.poiseResetDelay`). Guerrier 60, mage 30 ; brute 130, chef 220, archer 25, guetteur 15.
- **Recul selon le poids** (`docs/decisions/0007`) : chaque coup a une force (`impulse`), chaque corps un poids (`mass`,
  kg). La cible recule de `impulse / mass` m, rien sous 0,3 m (`combat.knockbackMin`), au plus 3 m
  (`combat.knockbackMax`), à 10 m/s ; le plus grand avec le recul fixe des coups de bouclier (`push`). Poids : guerrier
  90, mage 70 ; gobelin 40, kamikaze 35, guetteur 45, archer 60, chef 150, brute 220, chef orque 260. Épée légère 50
  (gobelin : 1,25 m ; brute : rien), coups lourds 100 à 230 ; gobelin 30 (un guerrier recule à peine : 0,33 m), brute 130 (1,4 m).
  Un coup chargé de hache ou d'épée à deux mains fait vaciller une brute d'un coup ; deux coups chargés le chef.
- **Bouclier** (A.13–A.16, façon Elden Ring ; réglages dans `guard` de `weapons.json`) : il prend le clic droit
  (la suite lourde reste sur Maj + clic gauche) et Maj + clic droit (la compétence de l'arme n'est plus accessible).
  - **Garde** (clic droit maintenu) : bouclier levé, on garde 50 % de sa vitesse ; un coup reçu de face (120°,
    projectiles compris) perd 80 % de ses dégâts et de son équilibre, et l'endurance paie les dégâts arrêtés (1 par
    point). À 0 d'endurance, **garde brisée** : on vacille 1 s. Le bouclier se baisse pendant une attaque.
  - **Coup de bouclier** (clic gauche bouclier levé) : 20 dégâts, 35 d'équilibre, repousse de 2 m, 12 d'endurance ;
    on revient en garde si le clic droit est toujours enfoncé.
  - **Parade** (A, 15 d'endurance) : pendant 0,25 s, un coup de mêlée (arc, ruée) reçu de face est annulé et
    l'attaquant vacille 1,5 s. Les projectiles et les zones ne se parent pas. Ratée : 0,5 s sans pouvoir agir.
  - **Riposte** : dans la seconde qui suit une parade réussie, le clic gauche donne un coup de bouclier puissant
    (45 dégâts, 80 d'équilibre, repousse de 3 m, gratuit).
  - Un corps repoussé recule tout droit à 10 m/s (`combat.pushSpeed`) ; une créature en pleine ruée ne l'est pas.
- Charge : ruée vers le point visé (jusqu’à lui, au plus 8 m) ; elle s’arrête contre un mur ou sur la première cible
  percutée, qui prend les dégâts et l’étourdissement. Pendant la ruée, les touches de déplacement sont ignorées.
- Provocation : touche les ennemis dans la zone autour du lanceur (jamais les alliés, même avec le tir allié) ; les créatures
  provoquées attaqueront le lanceur (tâches 1.10 et suivantes).
- Transfert (mage) : téléportation vers le point visé, au plus 7 m, sans traverser les murs (arrêt à 0,7 m avant) ni
  finir dans un autre corps ; refusé sans recharge s’il n’y a pas la place de bouger.
- Les effets s’affichent au-dessus de la cible : « Étourdi », « Ralenti », « Provoqué ».
- **Frappes à la lame** (piste F, `docs/decisions/0007`) : un coup au corps à corps (forme `arc`) ne touche plus une zone
  au sol : c'est la lame (`blade` de l'arme ou de la créature : longueur, épaisseur) qui touche, en suivant la trajectoire
  du geste joué (`combat/swings.json`, relevée par `scripts/animation-lab.ps1 bake`), pendant sa fenêtre active (la
  partie rapide du geste autour de l'impact). Une taille verticale rate une cible sur le côté, une horizontale touche
  tout ce qu'elle traverse ; chaque corps une fois par coup ; un mur (entre la poitrine de l'attaquant et la cible)
  ou un bouclier qui bloque ou pare arrêtent la lame ; l'attaquant abattu ou qui vacille arrête son coup. Le serveur
  tranche avec ce qu'il voit, sans compensation de latence. `range` des coups de mêlée = portée réelle de la lame
  (épée 1,3 m, hache 1,25, épée à deux mains 1,4, bouclier 1, gobelin 0,9, brute 1,3, chef 1,8) : elle ne sert plus
  qu'aux décisions des créatures et des bots. Sans trajectoire pour un geste, le coup touche encore en arc.
- Projectile : touche le premier
  corps ou mur rencontré ; avec un rayon (boule de feu), il explose et touche tout ce qui est dans la zone, alliés compris.
- Les corps à terre ne bloquent ni les projectiles ni les déplacements.
- **À terre** (0 PV) : le personnage devient un cadavre au sol ; il ne bouge plus et n'agit plus. Un **allié debout** qui
  reste à `combat.reviveRadius` (2 m) pendant `combat.reviveDuration` (4 s) le relève avec `combat.reviveHealthRatio`
  (50 %) de ses PV maximum et ses recharges à zéro ; s'il s'éloigne, tout est à refaire. Pendant ce temps le joueur à
  terre est spectateur : clic gauche pour regarder ses alliés à tour de rôle. Au-dessus du cadavre : « À relever » et
  l'avancement du relevé.
- **Déconnexion** : le personnage reste au sol en cadavre (« Déconnecté »), qu'on ne peut pas relever ; il garde son
  butin et ne compte plus pour l'échec de la mission.
- Mannequin d’entraînement (`m` dans les plans) : `combat.dummyHealth` PV (300), se relève après
  `combat.dummyRespawnDelay` s (3).
- Choix de classe : à la connexion, un écran présente chaque classe de `classes.json` (PV, vitesse, armes de départ,
  capacités et touches) ; le personnage apparaît une fois la classe choisie. On ne change pas de classe en cours de donjon.

## Version 2D (piste 2D, tâche 2D.7)

Mêmes règles et mêmes valeurs que la 3D (`CombatRules`, `abilities.json`, cerveaux des créatures), vues de dessus :

- Distances en mètres comme en 3D : une case du plan fait `topDown.metersPerCell` m (2), un étage 4 m de haut. On ne
  frappe et on ne voit qu'à son étage ; un mur, un pilier ou la roche coupent la vue.
- Le joueur frappe avec l'attaque légère de la première arme de sa main droite : à la souris, vers le pointeur (il se
  tourne de ce côté et le reste pendant le coup) ; au clavier ou à la manette, dans la direction où il marche (ou a
  marché en dernier).
- Vue de dessus, les portées de la 3D paraissent courtes : en 2D, la portée de tous les coups (joueurs et créatures)
  est multipliée par `topDown.attackReach` (2 au départ), sauf celle des tirs, déjà longue.
- Tout va plus vite qu'en 3D : vitesse de déplacement multipliée par `topDown.moveSpeed` (1,5), coups plus vifs
  (`topDown.actionSpeed`, 1,5 : armement, récupération, recharge et fenêtre d'enchaînement divisés d'autant), pour les
  joueurs comme pour les créatures.
- Aucune zone n'annonce un coup (2D.20, Flo) : on lit l'attaque sur le corps. Pendant l'armement, celui qui frappe
  lève son arme (début du geste LPC de l'arme, 2D.34) et une créature clignote un bref instant ; quand le coup part,
  le geste va au bout et une traînée (~0,15 s, blanche pour un joueur, rouge pour une créature) balaie l'arc frappé,
  dans un sens puis dans l'autre. Bouton tenu = la suite de coups enchaîne (`combo`, `comboWindow`) ; on ralentit en
  frappant (`moveFactor`).
- Animations (2D.34) : le sprite n'est jamais écrasé, étiré ni tourné (pixel art) ; vaciller le fait trembler d'un
  pixel, la marche s'anime à la vitesse où il avance (`topDown.walkStride`), tailles des peuples coupées
  (`topDown.peopleSizes`). Un corps touché blanchit 0,09 s et jette quelques étincelles orangées.
- Style « Tiny » (2D.39, décision 0013, `topDown.sprites`) : sprites fixes de 16 px agrandis ×2 ; liseré extérieur à la
  couleur du camp (son personnage cyan, autres joueurs et prisonnier verts, créatures rouges), vêtements à la couleur
  du rôle (`assets/tiny/looks.json`) ; il regarde à gauche ou à droite, sautille d'un pixel par pas, l'arme recule
  de 2 pixels à l'armement et part de 4 au coup ; mort, il est couché.
- Réactivité (2D.22 à 2D.24) : l'hôte voit le monde à chaque image ; un client bouge son personnage dès l'appui
  (prédiction, recalée sur le serveur) et prend son élan dès le clic. Recul, pas en avant et vacillement restent
  décidés par le serveur et se voient avec le retard du réseau.
- Coups qui existent en 2D : en arc, autour de soi (onde de choc du chef) et les tirs. Une classe dont l'attaque
  n'est ni l'un ni l'autre ne frappe pas ; seules les créatures qui se battent ainsi (brute, archer, chef) apparaissent
  (pas de charge, pas encore de guetteur ni de boss).
- Tirs (2D.26) : un coup `projectile` lâche, quand il part, `count` projectiles en éventail (`spread` degrés) qui
  volent en ligne droite à leur étage, à `speed × topDown.projectileSpeed` m/s, sur `range` m. Un tir touche le premier
  corps du camp d'en face (il traverse ceux de son camp) et s'arrête sur un mur, un pilier, la roche ou un meuble qui bloque (on s'abrite derrière). Il part vers
  là où était la cible : un pas de côté l'esquive. Rôdeur, mage et ingénieur tirent avec leur attaque légère ; les
  archers du plan tirent leurs carreaux (`archer_bolt`). À l'écran : un trait clair (joueur) ou rouge (créature),
  une gerbe d'éclats là où il s'arrête. Pas encore : explosion en zone (`radius`), zones posées, effets de combo.
- Chaque coup fait du bruit (`creatures.combatNoiseRadius`) : les créatures qui l'entendent viennent voir, même de
  dos ou endormies. Celui d'un joueur s'entend quand il part, pas pendant l'élan : un dormeur à portée est frappé
  avant d'entendre (2D.27).
- Sommeil, surprise, vie au repos (2D.27) : règles H.1 et H.2 de `creatures.md`, comme en 3D. Une créature endormie
  est plus sombre, des « z » montent au-dessus d'elle ; « ? » quand elle cherche, « ! » un instant quand elle se
  réveille ou passe en combat ; « Surprise ! » au coup (frappe ou tir) qui la prend sans qu'elle ait rien remarqué
  (×`creatures.surpriseDamageMultiplier`). Au repos, brutes et chef flânent, archers et guetteurs tournent la tête.
- Guetteur, alarme, moral (2D.28) : règles H.3, H.5 et H.7 de `creatures.md`, comme en 3D. Le guetteur fuit vers
  l'allié combattant le plus proche et crie « Alerte ! » (24 m, réveille un dormeur à moins de 7,2 m) : ceux qui
  l'entendent vont où il a vu les intrus. « À l'aide ! » d'un archer blessé : les défenseurs prennent l'agresseur pour
  cible. « Le chef est tombé ! » : ceux qui l'entendent fuient. Une créature qui fuit a un « !! » bleu qui tremble.
- Les corps debout ne se traversent pas. Un joueur à 0 PV reste à terre (pas encore de relevé en 2D).
- Impact (2D.9) : tout coup qui touche fait reculer sa cible de `impulse × topDown.knockback / mass` m (entre
  `combat.knockbackMin` et `knockbackMax`), à `combat.pushSpeed` m/s ; un mur arrête le recul. Une frappe en arc
  envoie un gobelin (40 kg) à plus d'une case, une brute (220 kg) bouge à peine. Il retire aussi
  `poiseDamage × topDown.poiseDamage` d'équilibre : à 0, la cible vacille `combat.staggerDuration` s (coup en cours
  perdu, immobile), en jaunâtre et chancelante à l'écran. Valable dans les deux sens (joueurs et créatures).
- Pas en avant (2D.10) : le coup d'un joueur le fait avancer de son `lunge` vers où il frappe (distance ×
  `topDown.attackReach`, au rythme de `topDown.actionSpeed`) ; vaciller l'arrête.
- Course (2D.11) : Maj tenue (clic du stick gauche) en se déplaçant, hors des coups : vitesse ×
  `player.sprint.speedFactor`, `player.sprint.staminaPerSecond` d'endurance par seconde. À bout d'endurance, on marche
  jusqu'à relâcher Maj. Frapper en courant lance l'attaque en course de l'arme (`run` dans `weapons.json`), seule,
  sans suite. Barre d'endurance jaune sous les PV d'un joueur qui en a dépensé.
- Accroupi (2D.32) : C (B à la manette) pour s'accroupir, C pour se lever ; courir relève. Vitesse ×
  `player.crouch.speedFactor` (0,5), vu à `player.crouch.visibility` (0,5) fois la portée de vue des créatures ; le
  personnage s'assombrit à l'écran (pas d'animation LPC accroupie ; le tasser abîmait les pixels, 2D.34).
- Bruits de pas (2D.32) : un joueur qui avance s'entend à `player.footsteps.walkRadius` m (6), × `runFactor` (2) en
  courant, × `crouchFactor` (0,2) accroupi ; contre un mur, pas de pas. Comme tout bruit : finesse d'ouïe de la
  créature, moitié de portée par plancher, à travers les murs ; endormie, elle n'entend qu'à 0,3 fois la portée (une
  course la réveille à moins de 3,6 m). Une créature qui entend vient voir (« ? »).

### Mission en 2D (tâche 2D.14)

- Objectif « tuer le chef » sur les créatures présentes en 2D : les chefs s'il y en a, sinon toutes les créatures (un
  boss, pas encore en 2D, ne peut pas être la cible). Réussie quand la dernière cible tombe, échouée quand tous les
  joueurs présents sont à terre en même temps (`Gameplay.Missions.Mission`).
- À la fin, un écran de résultat (temps, créatures abattues) ; la mission suivante (autre graine) part après
  `topDown.newMissionDelay` secondes (20), ou tout de suite si l'hôte clique « Nouvelle mission ». Tout le monde y repart
  debout.
- En 2D, une créature garde tous ses coups, mais ceux qui n'existent pas encore (charge, tir) ne sont jamais prêts ;
  l'onde de choc du chef (coup tout autour de lui) existe.
- Relever un allié (tâche 2D.15) : un joueur à terre se relève quand un allié debout reste à côté de lui, au même étage
  (`combat.reviveRadius`, 2 m), pendant `combat.reviveDuration` secondes (4 ; plus vite avec la vitesse de relevé de la
  classe, `reviveSpeed`), avec `combat.reviveHealthRatio` de ses PV (la moitié). S'il s'éloigne, tout repart de zéro.
  Un anneau autour du corps montre où en est le relevé.
- Butin en 2D (tâche 2D.16) : chacun garde ce qu'il ramasse. Un coffre (`$` du plan) revient au premier joueur debout qui
  s'en approche à `loot.chestOpenRadius` m (table `chest` de `loot/loot.json`) ; une créature abattue lâche sa table (si
  elle en a une) en tas, ramassé en passant à `loot.pickupRadius` m. Le bandeau montre l'or et les objets ramassés ;
  tout repart de zéro à la mission suivante (pas encore de retour au village en 2D).
