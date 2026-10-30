# Compétences des classes en 2D

Règles de la piste Z (`docs/roadmap/piste-Z-competences.md`, décision `docs/decisions/0014`) : les kits des classes
d'AmbianceurZinzin sur nos classes, en 2D. Les valeurs sont dans `game2d/data/combat/kits.json` (elles font foi) ; ce
sont celles d'AZ converties (1 case = 50 px d'AZ, durées telles quelles), à régler en jouant. Le projet 3D garde ses
capacités (`docs/design/classes.md`).

## Socle (tâche Z.1)

- **Touches** (comme AZ) : clic gauche **attaque** (tenu : elle repart à chaque recharge), clic droit **compétence 1**,
  Espace **compétence 2**, E **ultime** quand la jauge est pleine. Manette : X attaque, RB compétence 1, A compétence 2,
  Y ultime. *Choix de l'agent, question ouverte.*
- **Visée** : à la souris, le personnage regarde le pointeur et vise le point sous la souris (ramené à la portée de la
  compétence) ; à la manette, droit devant, à la portée maximale.
- **Recharges** : une compétence refusée (rien à faire, pas de place) ne dépense pas sa recharge. Les recharges
  avancent même à terre ou étourdi. Un appui pendant qu'on vacille est perdu.
- **Bandeau** en bas de l'écran : les quatre touches, l'ombre de la recharge, la jauge d'ultime ; au survol, la
  description.
- Une classe sans kit (ou une partie lancée sans kits) frappe avec son arme comme avant.
- **Dégâts** des kits × `damageFactor` (2) : nos créatures ont environ 2,5 fois les PV de celles d'AZ. Les soins restent
  ceux d'AZ. **Équilibre** retiré : dégâts × `poiseFactor` (0,5).
- **Résistance aux contrôles** (`controlResist`, de 0 à 1) : gel, ralenti et immobilisation durent × (1 − résistance).
  Brute et chef 0,5, chef orque 0,6.

## États (tâche Z.2)

| État | Effet | À l'écran |
|---|---|---|
| Gelé | ne bouge plus, ne frappe plus, son coup armé s'annule ; un allié du mage brise la glace (Z.15) | bleu, figé sans chanceler |
| Ralenti | vitesse × (1 − force) ; le plus fort l'emporte, pas de cumul | bleuté |
| Immobilisé | ralenti à 100 % : il ne marche plus mais frappe ce qui est à portée | anneau brun aux pieds |
| Étourdi | ni pas ni coup | chancelle, « * » |
| Provoqué | ne voit plus que celui qui l'a provoqué | anneau rouge |
| Caché | les créatures ne le voient plus et l'oublient, même en plein combat | pâle |
| Intouchable | aucun coup ne le touche | pâle |

Gel, ralenti et immobilisation tiennent compte de la résistance aux contrôles. Le recul d'une compétence est en cases
(celui des armes reste `impulse / mass`).

## Jauge d'ultime (tâche Z.12)

Elle se remplit comme dans AZ, de 0 à 1 (`ultGain` de chaque classe dans `kits.json`) ; pleine, « ULTIME PRÊT ! (E) »
s'annonce au-dessus du joueur et E lance l'ultime, qui la vide.

| Classe | Ce qui la remplit |
|---|---|
| Guerrier | les coups encaissés, bouclier compris (400 dégâts) |
| Mage | ses créatures gelées ou ralenties (60 secondes cumulées) |
| Rôdeur | les dégâts infligés (1 200) |
| Druide | les soins (600 PV) |
| Berserker | les dégâts infligés (1 500) et encaissés (400), pas pendant la furie |
| Alchimiste | les soins (700 PV) |
| Ingénieur | les dégâts infligés (1 100), pas ceux du canon |
| Assassin | les dégâts infligés (1 000) |
| Clerc | les soins et les bulles (600) |

Les dégâts comptent à l'échelle d'AZ (avant `damageFactor`). Les coups d'un ultime ne remplissent pas la jauge.
Chaque **combo d'équipe** (nova blindée, flèche ronce, brise-glace) ajoute 5 % à la jauge de tous les joueurs.

## Les kits

| Classe | Attaque | Clic droit | Espace | Ultime (E) |
|---|---|---|---|---|
| Guerrier (Paladin d'AZ) | coup d'épée en arc qui repousse | lever le bouclier | provocation | Jugement céleste |
| Mage (Mage de givre) | éclat de givre | nova de glace | blizzard | Autoroute de glace |
| Rôdeur | arc bandé | ruée | roulade | Trait de lumière |
| Druide | lianes | cercle de vie | forme de rat | Sentier de renouveau |
| Berserker | hache, rage | bond | soif de sang | Furie |
| Alchimiste | fiole d'acide | élixir | mixture | Transmutation |
| Ingénieur | pistolet à clous | tourelle | piège à mâchoires | Canon automatique |
| Assassin | dague, coup dans le dos | ombre | pas de l'ombre | Danse des lames |
| Clerc | bâton | bénédiction | consécration | Sanctuaire |

Le guerrier a son bouclier au clic droit et sa provocation à Espace, comme le Paladin d'AZ.

### Guerrier (tâche Z.3)

- **Coup d'épée** (recharge 0,5 s) : arc de 109° sur 1,44 case, 16 dégâts, repousse de 0,36 case. Comme toutes les
  attaques en arc des kits, il touche aussi une créature au contact, quel que soit l'angle, et casse le mobilier.
- **Lever le bouclier** (clic droit tenu) : arrête tout coup venu de face (arc de 120° vers où il vise), tirs compris ;
  chaque point de dégât arrêté coûte 1,5 d'endurance, et le bouclier en use 8 par seconde. Il faut 30 d'endurance pour
  le lever. À zéro, la garde cède : bouclier baissé, étourdi 1,5 s. Bouclier levé, il va à 60 % de sa vitesse et ne
  court pas. *Choix de l'agent* : c'est l'endurance de la classe (celle de la course).
- **Charge au bouclier** (clic gauche bouclier levé, recharge 0,8 s) : 40 d'endurance (il en faut 10), 8 dégâts,
  repousse de 1,8 case dans l'arc du bouclier. Vider l'endurance ainsi n'étourdit pas : le bouclier retombe.
- **Provocation** (Espace, recharge 9 s) : les créatures à 6 cases le ciblent 4 s, même sans le voir, même endormies.

### Mage (tâche Z.4)

- **Éclat de givre** (recharge 0,4 s) : tir de 7 dégâts (12 cases/s, 13 cases), ralentit de 45 % pendant 2 s.
- **Nova de glace** (clic droit, recharge 8 s) : 5 dégâts et gel 2,5 s aux créatures à 3,2 cases de lui.
- **Nova blindée** (combo d'AZ) : si un guerrier bouclier levé est à 3 cases du mage, la nova est 1,5 fois plus large,
  fait 15 dégâts de plus et gèle 1 s de plus ; annoncée, elle compte comme un combo d'équipe (jauge d'ultime de tous).
- **Blizzard** (Espace, recharge 11 s) : zone de 2,6 cases au point visé (8 cases au plus), 4 s ; ralentit de 60 % et
  fait 6 dégâts/s à ce qui est dedans.
- Les coups des compétences profitent aussi de la surprise (créature endormie ou au repos).

### Rôdeur (tâche Z.5)

- **Arc bandé** : tenir le clic gauche bande l'arc (1,2 s pour la pleine charge, il va à 70 % de sa vitesse) ; le
  relâcher tire la **grosse flèche** : 12 à 60 dégâts et 6 à 18 cases selon la charge, 22 cases/s. Elle **traverse**
  les créatures (chacune touchée une fois) et se **plante** dans un mur, un pilier, un meuble ou au bout de sa course.
  Il n'en a qu'une : tant qu'elle n'est pas revenue, il ne tire plus.
- **Rappel** : flèche plantée, un nouvel appui la fait revenir à travers tout, de 6 à 28 cases/s ; elle blesse (25) ce
  qu'elle traverse. Pas rappelée, elle revient seule après 4 s, sans blesser. Sur un autre étage, il la rattrape tout de
  suite.
- **Ruée** (clic droit, recharge 8 s) : il file vers sa flèche plantée (22 cases/s), intouchable ; chaque créature
  traversée prend 20 dégâts et recule de 0,4 case ; un mur l'arrête ; arrivé à moins de 0,8 case, il ramasse sa flèche.
  Sans flèche plantée (ou sur un autre étage), rien ne part et la recharge n'est pas dépensée.
- **Roulade** (Espace, recharge 4 s) : 18 cases/s pendant 0,18 s vers où il marche (là où il vise s'il est immobile),
  intouchable.
- Les déplacements forcés (roulade, ruée, et plus tard bond et jugement) ignorent les touches de déplacement.

### Druide (tâche Z.6)

- **Lianes** (recharge 0,9 s) : tir court (5 cases, 11,2 cases/s, 11 dégâts) qui laisse des **ronces** sur tout son
  trajet, de son départ à son arrêt (créature touchée, mur ou bout de course) : 3 s au sol, 0,4 case de large. Une
  créature qui les touche est ralentie de 40 % et brûle à 4 dégâts/s, encore 1 s après les avoir quittées (une seule
  brûlure, même sur plusieurs ronces). 6 ronces au plus par druide : la plus ancienne disparaît.
- **Cercle de vie** (clic droit, recharge 11 s) : zone de 2,2 cases au point visé (8 cases au plus), 5 s ; les alliés
  debout dedans regagnent 14 PV/s, un allié à terre dedans se relève comme si un allié était à côté de lui.
- **Forme de rat** (Espace) : 5 s rapide (×1,3) et cachée (les créatures le perdent de vue), sans attaquer ni lancer
  le cercle ; une réserve de 30 PV prend les coups d'abord, vide il reprend forme. Espace le fait reprendre forme
  plus tôt. La recharge (14 s) part à la fin de la forme ; à terre, la forme tombe.
- Il **relève deux fois plus vite** (`reviveSpeed` du kit).
- **Flèche ronce** (combo d'AZ) : la grosse flèche du rôdeur qui traverse des ronces fait 1,5 fois plus mal et ralentit
  de 40 % 2 s jusqu'à la fin de son trajet aller ; annoncée, elle compte comme un combo d'équipe.

### Berserker (tâche Z.7)

- **Hache** (recharge 0,6 s) : arc de 150° sur 1,5 case, 18 dégâts, recul 0,16 case.
- **Rage** : ses dégâts sont multipliés par 1 + (1 − PV/PVmax) : × 1,5 à mi-PV, × 2 presque à terre (et au maximum
  pendant la furie, tâche Z.14). Avec un kit, la rage de la classe 3D (barre qui monte en frappant) ne compte plus.
- **Bond** (clic droit, recharge 7 s) : saut de 0,45 s vers le point visé (5,2 cases au plus), intouchable en l'air ;
  il ne passe pas les murs (il retombe contre) ; à l'atterrissage, 25 dégâts et 0,6 case de recul à 1,8 case autour.
- **Soif de sang** (Espace, recharge 14 s) : 6 s pendant lesquelles chaque coup lui rend 25 % de ses dégâts (comptés à
  l'échelle d'AZ) et sa hache repart 25 % plus vite.

### Alchimiste (tâche Z.8)

Dans AZ, l'élixir, la mixture et la transmutation se **dessinent** à la souris (cercle, carré, pentagramme) et le sort
est plus fort plus le dessin est petit. *Choix de l'agent, question ouverte* : ici, ils partent au point visé, d'une
taille fixe (rayon 2 cases, celui du dessin de référence d'AZ).

- **Fiole d'acide** (recharge 0,7 s) : lancée vers le point visé (7,6 cases au plus, 10,4 cases/s), elle éclate là, ou
  sur la première créature qu'elle touche, ou contre un mur : 14 dégâts à 1,1 case autour.
- **Élixir** (clic droit, recharge 9 s) : soigne d'un coup de 35 PV les alliés debout à 2 cases du point visé (8 cases au
  plus), l'alchimiste compris.
- **Mixture** (Espace, recharge 10 s) : zone de 2 cases au point visé, 6 s ; les alliés dedans vont 1,4 fois plus vite
  (encore 0,2 s après en être sortis) et gagnent une **barrière** de 10 par seconde, jusqu'à 80 ; elle prend les coups
  avant les PV. Une seule mixture par alchimiste : la nouvelle remplace l'ancienne.

### Ingénieur (tâche Z.9)

- **Pistolet à clous** (recharge 0,3 s) : tir de 7 dégâts, 14 cases/s, 11 cases.
- **Tourelle** (clic droit, recharge 8 s) : posée au point visé (4 cases au plus ; à ses pieds si le point est dans un
  mur), 15 s ; toutes les 0,6 s, un clou de 8 dégâts sur la créature visible la plus proche à 6,4 cases. 2 au plus :
  la plus ancienne disparaît. Elle a 45 PV : les coups des créatures qui la touchent (en arc ou autour d'elles) la
  cassent. *Choix de l'agent* : les créatures ne la visent pas exprès.
- **Piège à mâchoires** (Espace, recharge 6 s) : posé au point visé (3 cases au plus), armé après 0,5 s, 30 s ; la
  première créature dessus prend 25 dégâts et est immobilisée 2 s (« Clac ! ») ; le piège est consommé. 3 au plus.

### Assassin (tâche Z.10, absent d'AZ)

Kit de `docs/design/classes.md` ramené à une attaque et deux compétences.

- **Dague** (recharge 0,35 s) : arc de 110° sur 1,2 case, 12 dégâts. **Coup dans le dos** : × 2,5 sur une créature qui
  ne l'a pas remarqué (endormie, au repos) ou frappée par derrière (dans l'arc de 120° derrière elle) ; il remplace
  l'attaque surprise de tous. « Dans le dos ! » s'affiche.
- **Ombre** (clic droit, recharge 15 s) : caché 6 s (les créatures le perdent de vue), jusqu'à sa première attaque.
- **Pas de l'ombre** (Espace, recharge 6 s) : file vers le point visé (4 cases au plus, 20 cases/s), intouchable et
  **sans bruit** ; chaque créature traversée prend 15 dégâts.

### Clerc (tâche Z.10, absent d'AZ)

- **Bâton** (recharge 0,55 s) : arc de 100° sur 1,4 case, 14 dégâts, recul 0,3 case.
- **Bénédiction** (clic droit, recharge 12 s) : une bulle qui absorbe 40 dégâts, 8 s, sur l'allié debout le plus proche
  du point visé (à 8 cases de lui au plus), lui-même s'il n'y en a pas.
- **Consécration** (Espace, recharge 14 s) : zone de lumière de 2 cases à ses pieds, 6 s ; 10 PV/s aux alliés debout
  dedans.
- Il **relève deux fois plus vite** (sa classe, `classes.json`).

Une ruée, une roulade, un bond ou un pas de l'ombre passent au travers des corps (ils ne poussent personne).

## Ultimes (tâches Z.13 et Z.14)

- **Guerrier, Jugement céleste** : il s'envole vers le point visé (7 cases au plus, 0,8 s, intouchable ; les murs
  l'arrêtent) et retombe : 70 dégâts et 0,8 case de recul à 3,2 cases ; les alliés à terre dans ce rayon se relèvent
  avec la moitié de leurs PV.
- **Mage, Autoroute de glace** : une bande de glace de lui au point visé (12 cases au plus, 1,2 case de large, 8 s) :
  les créatures dessus sont ralenties de 60 %, les joueurs vont 1,5 fois plus vite.
- **Rôdeur, Trait de lumière** : il bande l'arc 0,6 s, immobile (la charge se voit à ses pieds), puis un rayon part
  droit devant : 120 dégâts à tout ce qu'il traverse, 0,8 case de large. *Choix de l'agent* : dans AZ il traverse toute
  l'arène ; ici un mur l'arrête (30 cases au plus), pour ne pas toucher les salles voisines qu'on ne voit pas.
- **Druide, Sentier de renouveau** : 6 s, il sème une flaque tous les 0,6 case (0,8 case de rayon, 3 s), aussi en rat ;
  dessus, les créatures prennent 15 dégâts/s, les alliés regagnent 25 PV/s, un allié à terre se relève deux fois plus
  vite. Une seule flaque compte à la fois.
- **Berserker, Furie** : 8 s pendant lesquelles il ne tombe pas sous 1 PV, ne vacille plus et sa rage est au maximum
  (× 2) ; sa jauge ne se remplit pas pendant ce temps.
- **Alchimiste, Transmutation** : les créatures à 2 cases du point visé (8 cases au plus) deviennent des **poules** 5 s :
  elles ne frappent plus (leur coup armé s'annule), vont à 40 % de leur vitesse et prennent 1,5 fois plus de dégâts.
  Les boss résistent (« Résiste »). Pas de dessin à la souris (voir l'alchimiste).
- **Ingénieur, Canon automatique** : posé comme une tourelle, 10 s, un clou de 10 dégâts toutes les 0,15 s sur la
  créature visible la plus proche à 9 cases ; 2 au plus. Ses dégâts ne remplissent pas la jauge.
- **Assassin, Danse des lames** (*inventé par l'agent, AZ n'a pas d'assassin*) : il surgit dans le dos des créatures à
  6 cases (5 au plus, les plus proches d'abord), l'une après l'autre toutes les 0,15 s, intouchable, et frappe chacune
  d'un coup dans le dos (30 × 2,5). Sans créature à portée, rien ne part et la jauge reste pleine.
- **Clerc, Sanctuaire** (*inventé par l'agent, AZ n'a pas de clerc*) : les alliés à terre à 4 cases de lui se relèvent
  avec la moitié de leurs PV, les autres regagnent 40 PV, et tous sont intouchables 2 s.

## Brise-glace (tâche Z.15)

Un allié du mage qui frappe une créature **gelée** (nova de glace) brise la glace : ses dégâts sont multipliés par 3
(par 4 pour le rôdeur), avec un effet propre à sa classe, annoncé au-dessus de la créature ; c'est un combo d'équipe
(+5 % à la jauge de tous). La créature reste gelée : chaque coup la brise de nouveau.

| Classe | Nom | Effet en plus |
|---|---|---|
| Rôdeur | BRISE-GLACE ×4 | — |
| Guerrier | ÉCLATS ×3 | les créatures à 1,6 case prennent 12 dégâts |
| Druide | SÈVE GLACÉE ×3 | les alliés à 2,8 cases regagnent 15 PV |
| Berserker | FRACAS ×3 | il regagne 15 PV |
| Alchimiste | GIVRE ACIDE ×3 | les créatures à 1,6 case sont ralenties de 40 % 2 s |
| Ingénieur | SURCHARGE ×3 | sa tourelle est tout de suite prête |
| Assassin, clerc | BRISE-GLACE ×3 | — |

Ne brisent pas la glace : le mage lui-même, les zones (blizzard, ronces), le sentier de renouveau, les éclats.
