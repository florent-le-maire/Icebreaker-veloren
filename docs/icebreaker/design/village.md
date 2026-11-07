# Game design — Village

*Proposition du 2026-10-22 (phase 3) ; quatre points tranchés par Flo le même jour (voir « Décisions de
l'équipe »), le reste est à discuter.* Base commune entre les expéditions : on y fait les
stocks de consommables, on y soigne et améliore les personnages, on y recrute. Inspiration : le hameau de
*Darkest Dungeon*, adapté à un jeu en coop, en temps réel, sur une carte qui ne s'arrête jamais.
Index des règles : [`docs/GAME_DESIGN.md`](../GAME_DESIGN.md). Piliers : [`piliers.md`](piliers.md).

## Intention

> On part avec ce qu'on a préparé, on revient plus riche ou plus abîmé, et le village transforme ce retour en
> force pour la prochaine sortie.

- **Toute la progression se fait ici** (pilier : « dans un donjon, on ne devient pas plus fort »). Le donjon
  rapporte la matière (or, matériaux, reliques, expérience, blessures) ; le village la transforme.
- **Le village coûte et prend du temps** : chaque soin, chaque amélioration, chaque construction occupe du temps de
  carte (1 min réelle = 30 min de carte). Un personnage à l'infirmerie n'est pas disponible pour la sortie suivante.
- **On ne peut pas tout faire** : l'or manque toujours un peu (comme dans *Darkest Dungeon*), il faut choisir entre
  soigner le vétéran, améliorer l'arme ou agrandir l'entrepôt.

## Ce qu'on reprend de Darkest Dungeon (et ce qu'on change)

| Darkest Dungeon | Chez nous | Pourquoi |
|---|---|---|
| Diligence : nouvelles recrues chaque semaine | **Relais** : recrues tirées au hasard, renouvelées chaque jour de carte | Même tension « je prends qui ? » |
| Une équipe de héros parmi un registre | **Registre commun** : chaque joueur prend un personnage du registre avant de partir | Permet de changer de perso pendant qu'un autre se soigne |
| Fournisseur : on achète les vivres avant chaque sortie, invendus perdus | **Intendance + entrepôt** : les stocks restent d'une sortie à l'autre | Demande de l'équipe : faire des stocks |
| Forge : améliorer armes et armures | **Forge** : améliorer et réparer l'équipement | Identique |
| Guilde : améliorer les compétences | **Salle d'armes** : apprendre et améliorer les capacités de classe | Identique |
| Taverne / abbaye : soulager le stress | **Taverne** : repos et rumeurs, **pas de stress** (décision du 2026-10-22) | Le stress est lourd en temps réel |
| Sanatorium : soigner maladies, retirer ou verrouiller les bizarreries | **Infirmerie** : soigner blessures et maladies, traiter les traits | Identique, mais les blessures remplacent le stress |
| Roulotte nomade : babioles | **Marchand ambulant** : objets rares, stock qui tourne | Identique ; plus tard, il vient de la carte (phase 5) |
| Cimetière / mémorial | **Cimetière** : les corps ramenés, la résurrection, la liste des morts | Lien avec nos trois options après la mort |
| Héritages (bustes, portraits, actes, blasons) pour améliorer les bâtiments | **Reliques** trouvées seulement en donjon | Oblige à repartir pour faire grandir le village |
| Niveau de résolution : un héros de niveau 1 ne peut pas avoir une arme niveau 5 | **Rang** du personnage, gagné en expédition, qui plafonne les améliorations | Une recrue riche ne devient pas un vétéran en une soirée |
| Événements du hameau | **Événements du village** (une fois par jour de carte) | Plus tard ; puis branchés sur le monde (phase 5) |

## Les ressources

| Ressource | D'où elle vient | À quoi elle sert | À qui |
|---|---|---|---|
| **Or** | Butin revendu, coffres, missions | Acheter, soigner, recruter, améliorer | À chaque joueur (décision : chacun garde son butin) |
| **Matériaux** (bois, pierre, fer…) | Murs détruits, coffres, créatures | Construire et améliorer les bâtiments, la forge | Au village (on les dépose à l'entrepôt) |
| **Ingrédients** (herbes, venin, poudre…) | Créatures, salles du donjon | Préparer des consommables (apothicaire) | Au village |
| **Reliques** (4 sortes, comme les héritages de DD) | Seulement en donjon, surtout chez les chefs et dans les salles cachées | Monter les bâtiments de niveau | Au village |
| **Expérience** | Une sortie réussie (ou survécue) | Faire monter le rang du personnage | Au personnage |

Proposition : l'or reste personnel (règle de butin existante), mais chacun peut **verser** de l'or dans la caisse du
village pour financer une construction commune. Matériaux, ingrédients et reliques vont directement au village.

## Les bâtiments

Chaque bâtiment a des **niveaux** (1 à 3 pour commencer) ; monter de niveau coûte or + matériaux + reliques et prend
du temps de carte (6 h à 3 jours de carte, soit 12 min à 2 h 30 réelles, `docs/VISION.md`). Pendant les travaux, le
bâtiment reste utilisable au niveau d'avant.

| Bâtiment | Ce qu'on y fait | Ce que les niveaux apportent |
|---|---|---|
| **Relais** | Recruter (recrues tirées au hasard, 1 ou 2 traits) | Plus de recrues proposées, recrues de rang 1 ou 2, plus de places au registre |
| **Entrepôt** | Stocker consommables, matériaux, ingrédients, objets | Plus de place ; plus tard, une partie protégée des pillages (phase 5) |
| **Intendance** | Acheter les consommables de base ; préparer le **paquetage** de chacun | Prix plus bas, plus de choix |
| **Apothicaire** | Préparer potions, fioles d'alchimiste, antidotes, bombes à partir d'ingrédients | Nouvelles recettes, plusieurs préparations en même temps |
| **Forge** | Améliorer armes et armures (paliers), réparer | Paliers plus hauts, réparation moins chère |
| **Salle d'armes** | Apprendre une capacité de classe, améliorer celles qu'on a, changer de classe ? | Paliers plus hauts, entraînement plus court |
| **Infirmerie** | Soigner blessures et maladies ; retirer un trait négatif ; **verrouiller** un trait positif | Soins plus rapides, plusieurs lits |
| **Taverne** | Se reposer (moral), entendre des rumeurs sur les donjons proches | Rumeurs plus précises (région, menace, butin) |
| **Cimetière** | Déposer un corps, ressusciter (avec séquelle), mémorial des morts | Résurrection moins chère, séquelle moins lourde |
| **Marchand ambulant** | Acheter objets rares et babioles, vendre le butin | Plus d'objets, passe plus souvent |

Plus tard : **tour du nécromancien** (deuxième option après la mort), **palissade** et défenses (le monde peut
attaquer le village, phase 5), **écurie** (chevaux pour la carte, phase 4).

## Stocks et consommables

C'est le cœur de la préparation : on ne trouve pas tout en donjon, il faut **prévoir**.

- **Paquetage** : avant de partir, chaque personnage prend dans l'entrepôt ce qu'il emporte, dans un nombre limité
  de places (par exemple 6, réglable). Les touches R, 3 et 4 sont libres pour les objets (`piste-E-equipement.md`).
- **Ce qui n'est pas utilisé revient à l'entrepôt** (contrairement à *Darkest Dungeon*, où les invendus sont perdus).
  Ce qui est perdu reste sur le cadavre, en bas.
- Consommables de départ proposés :

| Objet | Effet | Vient de |
|---|---|---|
| Torche | Éclaire, fait peur à certaines créatures (existe déjà en main gauche) | Intendance |
| Potion de soin | Rend des PV d'un coup | Apothicaire |
| Bandage | Soigne lentement, arrête un saignement ; ne marche pas en combat | Intendance |
| Antidote | Enlève poison et maladie de donjon | Apothicaire |
| Fioles (feu, glace, eau, poison, lumière, fumée) | Munitions de l'alchimiste (aujourd'hui 3 de chaque, gratuites) | Apothicaire |
| Bombe | Fait céder un mur (`destruction.md`) | Apothicaire ou forge |
| Charge de foreuse | Munitions de l'ingénieur | Forge |
| Rations | Repos au camp en donjon (plus tard, comme le feu de camp de DD) | Intendance |

Aujourd'hui les fioles et bombes sont gratuites et comptées par donjon ; avec le village, elles viennent du stock.

## Les personnages

- **Recrue** : faible, aléatoire, 1 ou 2 traits (peureux, robuste, cleptomane…), une classe. Rang 0.
- **Rang** (0 à 5) : monte avec l'expérience des sorties. Il plafonne la forge et la salle d'armes (palier ≤ rang),
  comme le niveau de résolution de *Darkest Dungeon*. Une sortie trop facile pour le rang rapporte peu.
- **Traits** : positifs ou négatifs, gagnés aussi en donjon (« a vu un ami mourir : peureux face aux orques »).
  L'infirmerie retire un négatif ou verrouille un positif (pour qu'il ne disparaisse plus) ; maximum 5 de chaque.
- **Blessures** : un personnage revenu à terre ou très abîmé garde une blessure (moins de PV max, moins
  d'endurance…) jusqu'à ce qu'il soit soigné. Soigner prend du temps de carte : le joueur prend un autre personnage
  du registre en attendant.
- **Mort** : le corps reste en bas. Le ramener au cimetière ouvre la résurrection (chère, avec une séquelle). Sinon :
  nécromancien (plus tard) ou un autre personnage du registre.
- **Registre** : **commun à tout le monde** ; n'importe quel joueur peut prendre n'importe quel personnage disponible.
  Nombre de places limité (le relais l'agrandit). On peut renvoyer un personnage pour faire de la place. Plus tard,
  peut-être un système de partage (personnage attitré, prêté, réservé) : à prévoir sans le faire tout de suite.

## La boucle d'une soirée

1. **Retour** : écran de bilan (butin de chacun, qui est blessé, qui est resté en bas, expérience gagnée) ; les
   matériaux, ingrédients et reliques partent à l'entrepôt.
2. **Au village** : on vend, on soigne, on améliore, on lance une construction ou une préparation.
3. **Choix de l'équipe** : chacun prend un personnage disponible et remplit son paquetage.
4. **Départ** vers un donjon (aujourd'hui depuis un menu ; en phase 4 par la carte).

Avec la carte (phase 4), un joueur peut rester **intendant** au village pendant que les autres sont en donjon : il
lance les constructions, prépare les potions, accueille les blessés (`docs/VISION.md`, « Jouer ensemble »).

## Proposition pour la phase 3 (le plus simple qui marche)

- Le village est **une scène 2D** : une vue dessinée du village où chaque bâtiment est cliquable et ouvre son menu
  (pas de déplacement de personnage, pas de construction libre). Les bâtiments changent d'aspect avec leur niveau
  et les travaux en cours se voient.
- Plus tard, on veut **voir les personnages bouger dans le village** (le blessé à l'infirmerie, le forgeron au
  travail) : la scène 2D est pensée pour accueillir ces silhouettes sans tout refaire.
- Bâtiments du premier lot : **relais, entrepôt, intendance, forge, infirmerie, cimetière**. Ensuite : apothicaire,
  salle d'armes, taverne, marchand ambulant.
- Toutes les valeurs (prix, durées, places, niveaux, recettes) dans `game/data/village/*.json`.
- Règles dans `src/Gameplay/Village/` (C# pur, testé avec xUnit), sauvegarde dans SQLite (`src/Persistence`).

## Règles en jeu (phase 3)

Ce qui est fait et réglé dans les données (`game/data/village/`, `game.json` → `village`). Les valeurs sont des
réglages de départ, à ajuster en jouant.

- **Fondation** : 4 recrues gratuites au registre ; chaque joueur arrive avec 250 or (retrouvé par son pseudo).
- **Relais** : recrues du jour (3 au niveau 1), renouvelées à chaque nouveau jour de carte ; engager coûte 40 or au
  joueur qui engage ; registre de 6 places au niveau 1 (les morts ne comptent pas). On ne reste jamais sans personne à
  envoyer : si plus personne n'est disponible ni en route et que le relais est vide, des recrues arrivent tout de suite,
  et tant que personne ne peut partir, elles sont gratuites.
- **Départ** (porte du village) : on s'inscrit, on prend un personnage libre (un seul joueur par personnage), le premier
  inscrit choisit la mission ; l'expédition part 5 s après que tous les inscrits sont prêts. Ceux qui ne s'inscrivent
  pas restent au village et peuvent continuer d'y agir.
- **Retour** : un membre de l'expédition donne le signal (écran de fin ou menu pause « Rentrer au village »), sinon
  retour automatique 45 s après la fin de la mission, ou tout de suite si plus aucun membre n'est connecté.
  Chacun garde ce que son personnage a ramassé : l'or va à sa bourse, les objets à ses trésors (vendus à l'intendance).
  Un personnage à terre au retour est **mort** ; son corps et ce qu'il porte reviennent si au moins un membre est
  encore debout, sinon tout reste en bas.
- **Chroniques** : le bilan de chaque expédition rentrée (heure de carte, mission, ce que chacun rapporte ou ce qui lui
  est arrivé) est gardé au village et sauvegardé ; on garde les `historySize` dernières (30, `village.json`).
- **Paquetage** : 6 places, une pile d'un même objet par place (`items.json` → `stack`) ; on le remplit à la porte du
  village depuis l'entrepôt. Il **reste sur le personnage** d'une expédition à l'autre (pas besoin de le refaire) ; ce
  qui n'a pas servi y est encore au retour. Celui d'un mort revient à l'entrepôt avec son corps, ou reste en bas.
- **Outils et munitions** : venu du village, un personnage n'a que les bombes, charges de foreuse, fioles et fumigènes
  de son paquetage (plus de dotation gratuite par donjon) ; ce qu'il n'a pas employé, et les outils trouvés en bas,
  reviennent dans son paquetage.
- **Intendance** : on y achète (de sa bourse) pour l'entrepôt commun ; on y vend ses trésors.
- **Ce que rapporte le donjon** (`loot/loot.json`) : brutes et archers lâchent du bois, du fer, des herbes ; les coffres
  des matériaux, ingrédients et parfois une relique ; chaque chef a une chance sur quatre de laisser chaque sorte de
  relique. Au retour, tout objet du village va à l'entrepôt (même plein : on ne jette pas ce qu'on a rapporté).
  Sur Veloren : `assets/icebreaker/butin.ron`, coffres et créatures par palier de profondeur (B.5, B.14, `PLAN-BOUCLES.md`).
- **Rang** (0 à 5) : chaque expédition dont on revient vivant rapporte de l'expérience (réussie 3, échouée ou
  écourtée 1, moitié si la menace de la mission est sous le rang du personnage) ; seuils 3, 8, 15, 25, 40.
- **Traits** (`village/traits.json`) : une recrue arrive avec 1 ou 2 traits (qualités : Robuste, Vif, Infatigable,
  Costaud ; défauts : Fragile, Maladroit, Lourdaud, Essoufflé). Au retour, on peut en attraper : Endurci ou Craintif
  (a été à terre), Ébranlé (un membre est mort), Aguerri (mission réussie). Au plus 5 qualités et 5 défauts ; au-delà,
  le plus ancien non verrouillé est remplacé. Ils changent PV max, endurance, vitesse et dégâts en donjon.
- **Blessures et infirmerie** : revenir après avoir été à terre (puis relevé), ou avec moins de 30 % de ses PV, laisse
  une blessure (Bras cassé, Côtes fêlées, Jambe blessée, Commotion). L'infirmerie les soigne (60 or et 8 h de carte par
  blessure), retire un défaut (150 or, 12 h) ou verrouille une qualité (200 or) ; payé de sa bourse ; le personnage est
  indisponible pendant les soins ; un lit au niveau 1 (prix et durées baissent avec le niveau).
- **Forge** (à construire) : chaque palier d'armes donne +10 % de dégâts, chaque palier d'armure +8 % de PV ; prix
  de 80 or + 1 fer (palier 1) à 800 or + 10 fer + 4 pierre (palier 5), payés de la bourse ; un personnage ne dépasse
  pas son rang + 1, ni ce que la forge sait faire à son niveau (paliers 2, 4, 5). Pas encore d'armures à porter : le
  palier d'armure représente l'équipement du personnage.
- **Cimetière** (à construire) : un mort dont le corps a été ramené peut être relevé (400 or de la bourse, 2 jours de
  carte) ; il revient avec une **séquelle** pour toujours (Froid de la tombe, Regard vide, Mains tremblantes). On peut
  aussi l'enterrer : il quitte le registre et son nom va au mémorial. Un corps resté en bas ne se relève pas (on ira
  peut-être le chercher plus tard). Le nécromancien viendra avec le second lot de bâtiments.
- **Bâtiments** : construire ou agrandir se paie avec l'or de la **caisse commune** et les matériaux et reliques de
  l'entrepôt, et prend des heures de carte (6 h à 2 jours) ; pendant les travaux, le bâtiment sert à son niveau d'avant.
  Au départ, relais, entrepôt, intendance et infirmerie sont au niveau 1 ; forge et cimetière sont à construire, et
  le village de départ ne peut payer que l'un des deux (pierre). Monter au niveau 2 ou 3 demande des reliques. Un
  relais agrandi attire des recrues déjà aguerries (rang 1, puis 2).
- Un **mode essai** (sans village) reste disponible pour tester un donjon : chacun choisit sa classe, rien n'est gardé.

## Décisions et questions

Les quatre décisions de Flo du 2026-10-22 (un seul village commun, pas de stress, registre commun, scène 2D à menus)
sont dans `../QUESTIONS.md`, § Tranché ; les questions restantes (changer de classe, paquetage, village attaqué)
sont Q31 à Q33 du même fichier.
