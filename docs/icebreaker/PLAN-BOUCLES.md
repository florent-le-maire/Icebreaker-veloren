# Boucles de jeu — donjon et village

*Proposition du 2026-11-02, à valider par l'équipe (Flo, Sayanel).* Ce plan dit **ce qui rend intéressant**
l'enchaînement village → donjon → village, avec des règles chiffrées et des tâches. Il ne dépend **pas** de la
structure actuelle des donjons (salles, graphe, Gnarling) : il ne s'appuie que sur des notions que tout donjon aura —
une **entrée**, une **profondeur** (distance à l'entrée), des **créatures**, des **coffres**, un **chef**.
Piliers : `design/piliers.md` ; village visé : `design/village.md` ; menace : `design/menace.md`.

## Le constat

Aujourd'hui la boucle tourne mais rien ne pèse :

- **Aucun risque** : mourir renvoie au point de réapparition de Veloren, sans rien perdre ; `/rentrer` marche de
  partout ; les PV remontent seuls.
- **Aucune direction** : on part dans un donjon « parce qu'il est là » ; rien ne dit pourquoi celui-ci plutôt qu'un autre.
- **Aucune dépense utile** : l'or va à la caisse, la caisse bâtit des maisons… qui n'apportent rien.
- **Aucun lien** : ce qu'on trouve en bas ne correspond à aucun besoin en haut.

## Les trois boucles

| Boucle | Durée réelle | La question que se pose le joueur | Ce qui la rend intéressante |
|---|---|---|---|
| **Rencontre** (une salle, un combat) | 30 s à 3 min | « On y va comment ? » | IA lisible (G.11 à G.16), surprise, alerte — **hors de ce plan** |
| **Expédition** | 30 à 60 min | « **On pousse ou on rentre ?** » | Butin qui grossit, ressources qui fondent, sortie loin, mort qui coûte |
| **Village** | 5 à 15 min entre deux sorties | « **Qu'est-ce qu'on paie d'abord ?** » | Trop de besoins pour l'or disponible, des besoins qui ne se remplissent qu'en bas |
| **Campagne** (plusieurs soirées) | 3 à 4 expéditions par soirée | « Où en est notre village ? » | Bâtiments qui débloquent, contrats plus durs, le temps de carte qui avance |

Le fil qui relie tout : **le village a des besoins précis (or, matériaux, reliques), les contrats disent où les
trouver, le donjon les donne au prix d'un risque, et ce qu'on bâtit rend la sortie suivante possible.**

## Boucle d'expédition : « on pousse ou on rentre ? »

Le cœur du jeu (façon Darkest Dungeon). Pour que la question existe, il faut **trois courbes** qui se croisent :
la valeur du sac **monte**, les ressources **baissent**, le danger **monte**.

### 1. La valeur monte avec la profondeur

- **Profondeur** d'un coffre ou d'une créature = son palier, calculé par le lieu (distance de chemin depuis l'entrée
  rapportée à la plus grande ; 1 = premier tiers, 2 = deuxième, 3 = dernier tiers). Tout générateur sait donner ça.
- Tables de butin par palier (`assets/icebreaker/butin.ron`) :

| Palier | Or d'un coffre | Matériaux | Chance de relique |
|---|---|---|---|
| 1 | 10 à 30 | pierre, bois | 0 % |
| 2 | 30 à 80 | fer, cuir | 5 % |
| 3 | 80 à 200 | acier, cristal | 15 % |
| Chef | 150 à 300 | 2 au palier 3 | 50 % |

- **Reliques** (4 sortes, comme les héritages de Darkest Dungeon : *Sceau*, *Idole*, *Grimoire*, *Bannière*) : objets
  nouveaux, invendables, qu'on ne trouve **qu'en donjon**. Elles servent à monter les bâtiments au niveau 2 et 3.

### 2. Les ressources baissent

- **Pas de soin gratuit en expédition** : régénération naturelle coupée hors du village (réglage). On se soigne
  avec ce qu'on a apporté : potions (intendance), bandages, **feu de camp** (voir plus bas).
- **Les PV suivent le personnage** : on revient au village comme on est sorti du donjon (même serveur depuis `0016`,
  ce qui règle la question ouverte 1 du plan). L'infirmerie devient utile.
- **Usure** : Veloren abîme déjà l'équipement à la mort ; on ajoute une usure par expédition (réglage, 5 %). La forge répare.
- **Feu de camp** : une *ration* (intendance, 20 or) pose un feu de camp de Veloren (il soigne déjà ceux assis
  autour). 60 s assis remontent ~50 % des PV, mais **le feu fait monter l'alerte** (point 3) : se reposer a un prix.

### 3. Le danger monte

- **Le temps joue contre nous** : toutes les `minutes_renfort` (12 min réelles, réglage) passées dans le donjon, une
  **patrouille** (3 créatures de la faction du lieu) entre par l'entrée et remonte vers les joueurs. Signe avant :
  « Des pas résonnent vers l'entrée… » (pilier « toujours un signe avant la conséquence »). Plus on traîne, plus le
  **retour** est dur — et le retour est obligatoire (point 4).
- Le bruit (combats, feu de camp, mort du chef) rapproche la patrouille suivante de 3 min. Se branche plus tard sur le
  niveau d'alerte (G.15) sans l'attendre.

### 4. Sortir coûte

- **`/rentrer` (et son bouton) ne marche qu'à l'entrée** (rayon réglable, 8 blocs) ; ailleurs : « Il faut regagner
  la sortie. » Le chemin du retour fait partie de l'expédition.
- **Parchemin de retour** (intendance, 150 or, niveau 2) : sortie immédiate de tout le groupe — l'assurance qu'on
  achète, ou pas.
- **Le groupe sort ensemble** : un membre resté en bas garde l'emplacement occupé (déjà le cas, I.8).

### 5. Mourir coûte

- **Mort en expédition** : le personnage laisse un **sac** à l'endroit de sa mort avec tout son inventaire **hors
  équipement porté** (or compris), revient au village avec 1 PV et une **blessure** (point village).
- **Un allié peut rapporter le sac** (on le ramasse comme un objet) : ce qu'il contient revient à son propriétaire au
  village. Sac non ramassé = perdu à la remise à neuf (I.9).
- Ça rend les potions, le parchemin et le fait de rester groupés **précieux**, sans « game over ».

### 6. Une raison d'y aller : le contrat

Chaque expédition part d'un **contrat** (voir village). Le contrat fixe l'objectif ; l'objectif change la façon
de jouer sans dépendre de la forme du donjon :

| Objectif | Réussi quand | Ce que ça change |
|---|---|---|
| **Tuer le chef** | le chef meurt (G.7) | aller au fond, puis revenir avec la patrouille sur le dos |
| **Piller** | rapporter le **trésor** à l'entrée | le trésor pèse : son porteur va à 70 % de sa vitesse et ne peut pas rouler ; il faut l'escorter |
| **Récupérer** | rapporter N matériaux précis (ex. 10 fer) | on fouille les coffres plutôt que tuer ; mission courte |
| **Sauver** | escorter un prisonnier jusqu'à l'entrée | un PNJ fragile qui suit (Veloren sait faire suivre un compagnon) |

Réussir paie **la caisse** (récompense du contrat) ; chacun garde en plus ce qu'il a ramassé (décision 3).

## Boucle du village : « qu'est-ce qu'on paie d'abord ? »

### Les ressources

| Ressource | Vient de | Va où | Sert à |
|---|---|---|---|
| **Or personnel** | coffres, créatures, vente | bourse du joueur | potions, soins, réparations, forge |
| **Caisse** | versements, **récompense des contrats** | trésorier | bâtiments |
| **Matériaux** | coffres, créatures | entrepôt (commun) | bâtiments, forge |
| **Reliques** | donjon seulement (palier 3, chef) | entrepôt | niveaux 2 et 3 des bâtiments |
| **Points de compétence** | expérience de Veloren gagnée en bas | personnage | dépensés **à la salle d'armes** seulement |

### Les bâtiments : chacun ouvre quelque chose

On remplace les « maisons » de C.3 par des **bâtiments nommés à niveaux**, posés sur les terrains à bâtir. Au départ :
entrepôt, caisse, intendance, infirmerie (niveau 1), départ. Le reste est à construire. Tout en données
(`assets/icebreaker/batiments.ron`) ; prix et durées de départ à ajuster en jouant.

| Bâtiment | Niveau 1 | Niveau 2 | Niveau 3 |
|---|---|---|---|
| **Intendance** | potions faibles, rations, torches | potions moyennes, **parchemin de retour** | potions fortes, bombes |
| **Infirmerie** | soigne une blessure (60 or, 4 h de carte) | 2 lits, durée −30 % | retire une séquelle |
| **Entrepôt** | 36 places | 54 places | 72 places |
| **Forge** *(à bâtir)* | réparer ; recettes Veloren cuivre/bronze | recettes fer | recettes acier |
| **Salle d'armes** *(à bâtir)* | dépenser ses points de compétence | +1 point par expédition réussie | — |
| **Taverne** *(à bâtir)* | rumeurs : faction des contrats | menace exacte, présence de relique | un 4e contrat, mieux payé |

Coûts : **niveau 1** = 300 or de caisse + 10 pierre, 6 h de carte ; **niveau 2** = 1 000 or + 20 fer + 1 relique,
12 h ; **niveau 3** = 3 000 or + 10 acier + 3 reliques, 1 jour. Pendant les travaux, le bâtiment sert à son niveau
d'avant. Le chantier avance **pendant l'expédition** (temps de carte) : on lance la forge, on part, elle est prête au
retour.

La forge et la salle d'armes reprennent ce que Veloren a déjà (recettes près d'une station, arbre de compétences) :
on n'ajoute que **la condition** « au village, dans le bâtiment, au bon niveau ». C'est le pilier « on progresse au
village, pas en bas ».

### Le tableau des contrats (chef d'expédition)

- **3 contrats** proposés, renouvelés à chaque jour de carte (48 min réelles). Un contrat = lieu, objectif, menace
  (1 à 5, paramètres G.6), faction, **récompense à la caisse**, et ce que la taverne en révèle.
- Récompense = 150 × menace (réglage) ; menace 4–5 : une relique garantie au chef.
- Choisir un contrat remplace le choix de destination (V.7, V.11) ; l'emplacement de donjon est généré avec ses
  paramètres.
- Un contrat raté ou abandonné disparaît (on ne le retente pas en boucle).

### Blessures

- Revenir **sous 30 % de ses PV** ou **mort** donne une blessure (un effet de Veloren qui dure) : *Bras cassé*
  (−20 % dégâts), *Côtes fêlées* (−20 % PV max), *Jambe blessée* (−15 % vitesse), *Commotion* (−20 % endurance).
- Elle reste jusqu'à l'infirmerie. On peut repartir blessé : c'est un choix.
- Plus tard, avec le registre commun, un personnage à l'infirmerie n'est pas disponible (`design/village.md`).

### Le bilan au retour

À la sortie, à tous : contrat réussi ou non, récompense versée, ce que chacun rapporte (V.5), morts et blessures,
reliques trouvées. Gardé dans une **chronique** (`campagne/chronique.ron`, 30 dernières) consultable dans la vue U.

## Une soirée type (cible de la démo D.1)

1. **Arrivée** (village, 100 or chacun, caisse vide). Taverne pas bâtie : 3 contrats à l'aveugle. On prend
   « Récupérer 10 pierre, menace 1, 150 or ». Potions à 30 or.
2. **Expédition 1** (~25 min) : courte, on tâtonne, un joueur meurt au palier 2, l'autre rapporte son sac. Retour :
   150 or à la caisse, ~200 or chacun, 14 pierre, une blessure.
3. **Village** : infirmerie (60 or) ; on verse à la caisse ; **forge niveau 1** lancée (300 or + 10 pierre).
4. **Expédition 2** (menace 2, tuer le chef, ~40 min) : armes réparées, une patrouille dans le dos au retour, une
   relique au chef. 300 or à la caisse.
5. **Village** : taverne niveau 1 ; recettes de bronze à la forge ; la première relique attend l'intendance niveau 2.
6. **Expédition 3** : on choisit le contrat grâce à la rumeur… et on vise les reliques pour la suite.

## Tâches (jalon B, à faire dans cet ordre)

Chaque tâche = quelques heures, un critère de fin, le jeu reste lançable. Les plus utiles à la démo d'abord.

**Bloc 1 — que l'expédition pèse** (sans rien toucher au générateur)
- [ ] **B.1 PV gardés, pas de soin gratuit** — régénération naturelle coupée hors village (`assets/icebreaker/
  boucles.ron`) ; PV gardés au retour.
  *Fin* : on revient au village avec les PV qu'on avait en bas.
- [ ] **B.2 Sortie par l'entrée** — `/rentrer` et le bouton refusés loin de l'entrée de l'emplacement.
  *Fin* : refus au fond du donjon, retour à l'entrée.
- [ ] **B.3 Le sac du mort** — mort en expédition : sac au sol (inventaire hors équipement, or compris), retour au
  village à 1 PV ; un allié qui ramasse le sac le rend à son propriétaire au retour.
  *Fin* : un joueur meurt, l'autre rapporte son or.
- [ ] **B.4 Patrouilles** — toutes les `minutes_renfort`, une patrouille entre par l'entrée et cherche les joueurs ;
  annonce avant ; le bruit l'avance.
  *Fin* : rester 12 min fait arriver une patrouille annoncée.

**Bloc 2 — que le butin ait un sens**
- [ ] **B.5 Butin par profondeur** — palier 1 à 3 selon la distance à l'entrée (fonction du lieu, pas du générateur) ;
  tables `butin.ron`.
  *Fin* : coffre du fond nettement plus riche que celui de l'entrée (test sur 3 graines).
- [ ] **B.6 Reliques** — 4 objets nouveaux, invendables, dans les tables palier 3 et chef.
  *Fin* : un chef en laisse une sur deux (test).
- [ ] **B.7 Consommables** — ration → feu de camp (soigne, avance la patrouille), parchemin de retour, potions par
  niveau d'intendance.
  *Fin* : le parchemin fait sortir le groupe du fond du donjon.

**Bloc 3 — que le village ait des besoins**
- [ ] **B.8 Bâtiments à niveaux** — `batiments.ron` (coût or + matériaux + reliques, durée, effets) ; un terrain à
  bâtir propose les bâtiments manquants ; monter de niveau depuis le panneau.
  *Fin* : bâtir une forge en payant caisse + pierre de l'entrepôt.
- [ ] **B.9 Tableau des contrats** — 3 contrats par jour de carte chez le chef d'expédition ; le contrat donne ses
  paramètres à l'emplacement ; réussite = récompense à la caisse.
  *Fin* : réussir un contrat « tuer le chef » remplit la caisse.
- [ ] **B.10 Blessures et infirmerie** — blessure au retour (< 30 % PV ou mort), soignée à l'infirmerie (or + heures de carte).
  *Fin* : un blessé a moins de PV max jusqu'à l'infirmerie.
- [ ] **B.11 Forge** — réparer ; recettes de Veloren permises selon le niveau de la forge.
  *Fin* : forge niveau 1 : on fabrique du bronze, pas du fer.
- [ ] **B.12 Salle d'armes** — arbre de compétences verrouillé hors de la salle d'armes.
  *Fin* : impossible de dépenser un point en donjon, possible au village.
- [ ] **B.13 Taverne et bilan** — rumeurs sur les contrats selon le niveau ; bilan complet au retour et chronique.
  *Fin* : la taverne niveau 2 affiche la menace exacte d'un contrat.

Bloc 1 et bloc 2 peuvent avancer en parallèle (worktree `Veloren-boucles` et `Veloren-donjon`) ; le bloc 3 dépend de
B.6 pour les reliques. Entrepôt à 54/72 places et objectif « piller » / « sauver » : après une première soirée.

## Hors de ce plan

Idées pas encore retenues (relever un allié, curiosités, trophées de chef, tribut…) : `IDEES.md`.

Registre commun et recrues, classes, carte du monde, village attaqué, structure des donjons (formes, lieux avancés),
IA fine (G.11 à G.16 restent dans `PLAN.md`).

## Questions pour l'équipe

1. **Mort** : sac au sol + blessure (proposé), ou plus dur (le personnage reste mort jusqu'à ce qu'on rapporte son
   corps, comme `design/village.md`) ? Le plus dur attend le registre commun.
2. **Or des contrats** : à la caisse (proposé, pour pousser à bâtir ensemble) ou partagé entre les membres ?
3. **Entretien** : le village coûte-t-il de l'or chaque jour de carte (pression permanente, façon Darkest Dungeon) ?
   Proposé : non pour la démo, à essayer après.
4. **Patrouilles** toutes les 12 min : trop punitif pour une première soirée ? On peut commencer à 20 min.
5. **Salle d'armes** : bloquer l'arbre de compétences de Veloren hors village, ou le laisser libre pour la démo ?
