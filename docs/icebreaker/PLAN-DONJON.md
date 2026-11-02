# Le donjon à jouer — ce qui rend une descente intéressante

*Proposition du 2026-11-02, d'après les idées de Flo, à valider par l'équipe.* `PLAN-BOUCLES.md` dit **pourquoi** on
descend (contrats, besoins du village) ; ce plan dit **ce qu'on fait en bas**. Comme lui, il ne dépend pas de la
structure actuelle des donjons : il décrit des mécaniques et, à la fin, **ce qu'un générateur devra fournir** pour
qu'elles brillent. Règles déjà écrites pour Godot et reprises ici : `design/destruction.md`, `design/combat.md`,
`design/creatures.md`.

## L'idée d'ensemble

Six mécaniques qui **se répondent** ; chacune seule est banale, c'est leur croisement qui fait le jeu :

```
 inventaire limité ──► choisir : ressources du village (valeur)  OU  ressources du donjon (survie)
        ▲                                                              │
        │                                                              ▼
   le sac du mort                                    éclat → foreuse, huile → lumière, corde → trous
                                                                       │
 monstres à coop ◄── verticalité (trous, gouffres) ◄── terraformation ◄┘
        │
        ▼
 combat garde / esquive / parade : la base sous tout le reste
```

Ce que Veloren a déjà et qu'on règle plutôt que refaire : **garde avec fenêtre de parade** (`basic_block.rs`,
`parry_window`), **roulade**, **pioche et pelle** (blocs minables), **explosions qui cassent les blocs** (puissance
par bloc), **escalade des murs**, inventaire à cases, objets en données (`assets/common/items`).

## 1. Deux sortes de ressources

Tout ce qu'on trouve en bas est **soit pour le village, soit pour le donjon**, et les deux se disputent le même sac.

| Pour le village (on la ramène) | Pour le donjon (on l'emploie en bas) |
|---|---|
| Or, fer, pierre, cristal, cuir | **Éclat** (filons dans la roche) : carburant de la foreuse, mèche des bombes |
| Reliques, trophées de chef | **Huile** (tonneaux, cuisines) : recharge la lanterne ; lancée, flaque qui brûle |
| Trésors (à vendre) | **Corde** (réserves, cadavres d'aventuriers) : descendre et remonter un trou |
| | **Champignons lumineux** : +20 PV, ne se mangent qu'en bas |
| | **Planches** (caisses cassées) : un pont ou une barricade de 3 blocs |

- Les ressources du donjon sont **instables** : elles disparaissent à la sortie (proposé ; voir questions). On ne les
  stocke pas, on les trouve **sur place** : explorer nourrit l'exploration.
- On peut **en apporter** du village (corde, huile, charges à l'intendance), mais elles prennent la place du butin.
- Le choix devient physique : « je laisse ce lingot de fer pour garder 3 cordes ? ».

## 2. Un inventaire limité, avec des formes

Façon *Resident Evil 4* / *Backpack Hero* : un sac en **grille**, chaque objet a une **forme**.

| Objet | Forme |
|---|---|
| Potion, champignon, éclat, clé | 1 × 1 |
| Corde, huile, lingot | 1 × 2 |
| Arme à une main | 1 × 3 |
| Relique, planches | 2 × 2 |
| Trésor de contrat, arme à deux mains | 2 × 3 |

- **Sac de 6 × 4** au départ (24 cases), plus grand à l'intendance du village (7 × 5, puis 8 × 6). On peut tourner un objet.
- **Ceinture** : 4 cases 1 × 1 en accès rapide (potions, éclat), hors du sac.
- Ce qu'on **porte** (arme, armure, lanterne) ne compte pas.
- Ramasser quand c'est plein : il faut **lâcher** quelque chose (au sol, il reste : on revient le chercher… ou pas).
- Le sac du mort (`PLAN-BOUCLES.md`, B.3) garde la grille telle quelle.
- **En deux temps** : d'abord une **capacité en cases** (chaque objet a une taille, total ≤ 24, côté serveur, sans
  toucher à l'interface) pour sentir la contrainte ; ensuite la vraie grille à formes dans l'interface (gros travail
  dans Voxygen).

## 3. Terraformer : foreuse et bombes

- **Matières** (`assets/icebreaker/matieres.ron`) : terre (vite), pierre, roche dure (foreuse seulement), **pierre
  ancienne** indestructible (antre du chef, chambre forte, bord du donjon : on ne contourne pas tout). Valeurs de
  départ reprises de `design/destruction.md`.
- **Pioche** (Veloren) : lente, silencieuse, terre et pierre.
- **Foreuse** : outil tenu, creuse un tunnel de 3 × 3 blocs devant soi, **1 éclat par 2 m** ; bruit à 24 m (les
  créatures viennent voir). Pas d'éclat, pas de forage.
- **Bombe** : explosion de Veloren (déjà en place), 1 éclat + 1 huile ; bruit à 60 m, réveille la salle voisine.
- **À quoi ça sert** (pour qu'on veuille creuser) :
  - **raccourci** vers la salle d'à côté, ou **contourner** une porte verrouillée sans la clé ;
  - **filons** visibles dans les murs (éclat, fer, cristal) : on creuse pour les atteindre ;
  - **piège** : faire tomber un plafond ou un plancher sur des créatures ;
  - **sortie de secours** : creuser jusqu'à la surface ouvre une sortie (`PLAN-BOUCLES.md`, sortie par l'entrée) — chère en éclat ;
  - **se retrancher** : une planche bouche un couloir le temps de souffler.
- Ce qui ne tient plus tombe (plancher sans appui), comme `design/destruction.md`.

## 4. Des trous qui comptent (façon Deep Rock Galactic)

Le relief doit poser des **questions** : par où passer, qui descend en premier, comment on remonte.

- **Descendre est facile, remonter coûte.** Un trou est un raccourci **à sens unique** : on saute (dégâts au-delà de
  ~5 m) ou on pose une **corde** (elle reste et sert au retour). Sauter sans corde, c'est se couper la retraite.
- **Escalade de Veloren limitée en donjon** : on ne grimpe plus n'importe quel mur, seulement les **prises** (parois
  marquées, lierre, échelles). Sans ça, aucun trou ne pose de question.
- **Gouffre** : grand puits au centre du donjon, des corniches, des salles qui y donnent à plusieurs hauteurs ; les
  archers tirent d'en face, on voit le fond (et sa lueur) depuis le haut. Descendre le gouffre = aller vite au fond.
- **Caverne** : grande salle ouverte, sol inégal, piliers naturels, plusieurs entrées et hauteurs ; on s'y fait
  prendre à revers.
- **Planches** : un pont de 3 blocs au-dessus d'un trou, ou une marche contre une paroi.
- **Plancher fragile** : craque (signe : fissures, poussière) puis cède sous le poids ou une explosion : on se
  retrouve un étage plus bas, séparés.

## 5. Le combat : garder, esquiver, parer

Trois réponses à une attaque, **chacune meilleure contre un type d'attaque**, lisibles à l'armement de l'ennemi :

| Réponse | Règle | Bonne contre |
|---|---|---|
| **Garde** (maintenue) | −80 % des dégâts de face ; chaque coup reçu coûte de l'endurance ; endurance à 0 = **garde brisée**, étourdi 1 s | coups rapides, enchaînements |
| **Parade parfaite** | garde levée ≤ 0,2 s avant l'impact : 0 dégât, 0 endurance, l'attaquant **vacille 1,2 s** ; riposte ×2 pendant ce temps | coups forts annoncés |
| **Esquive** (roulade) | 0,25 s d'invulnérabilité au début, 20 d'endurance | attaques **imparables** (lueur rouge à l'armement), zones, saisies |

- Les ennemis **annoncent** leurs coups (armement visible, déjà le cas dans Veloren) ; les coups imparables ont un
  signe à part. Une erreur de lecture se paie (combat létal, L.1).
- **Endurance** au centre : attaquer, garder, rouler la consomment ; à sec, on est vulnérable.
- **Armure** : lourde = garde plus solide, roulade plus courte ; légère = l'inverse.
- Pour Veloren : régler la fenêtre de parade, l'invulnérabilité de la roulade et le coût de la garde (fichier
  `combat.ron`, déjà là pour L.1), ajouter l'effet « vacille » après une parade et le drapeau « imparable ».

## 6. Des monstres qui demandent de jouer ensemble

Chaque monstre pose **un problème qu'un joueur seul résout mal**. Un ou deux par donjon suffisent, mêlés aux
créatures ordinaires.

| Monstre | Ce qu'il fait | Comment on le bat à plusieurs |
|---|---|---|
| **Porte-bouclier** | bloque tout de face, avance en tenaille avec les autres | l'un le provoque ou le garde, l'autre frappe dans le dos |
| **Étreigneur** | saisit un joueur et l'entraîne (vers un trou, dans le noir) ; la victime ne peut que se débattre | un allié frappe l'étreigneur pour libérer ; seul, on meurt |
| **Chaman** | rend ses alliés proches insensibles aux coups tant qu'il vit ; se tient en hauteur ou derrière | se séparer : un groupe tient, un autre va le chercher (corde, foreuse) |
| **Ombre** | insensible sauf **éclairée** par une lanterne levée | l'un tient la lumière, l'autre frappe |
| **Jumeaux** | deux créatures liées : l'une se relève si l'autre vit encore 5 s après sa mort | les tuer ensemble, donc se coordonner |
| **Colosse** | armure de face, point faible dans le dos ; une **double parade** (deux joueurs à moins de 2 s) le fait tomber | un l'occupe, l'autre le contourne ; ou parer à deux |
| **Essaim** | sort des trous par vagues de petites créatures | tenir un couloir : un garde, les autres frappent ; boucher le trou (planches, explosion) |

Faisable dans Veloren en partant de créatures existantes : un comportement (agent) et des effets (buffs)
par monstre ; l'étreigneur demande le plus de travail (saisie et déplacement d'un joueur).

## Ce que le générateur devra fournir

Pour qu'un futur générateur (quel qu'il soit) serve ces mécaniques :
- des **matières** par zone (terre, pierre, roche, pierre ancienne autour de ce qui doit rester protégé) ;
- des **filons** visibles dans les murs, plus riches en profondeur ;
- de la **verticalité** : au moins un gouffre ou une caverne, des trous à sens unique, des parois à prises ;
- des **réserves** de ressources du donjon (tonneaux d'huile, cordes, champignons), assez pour ne pas devoir tout apporter ;
- des **emplacements** pour les monstres à coop (gouffre pour l'étreigneur, hauteur pour le chaman, trous pour l'essaim).

## Tâches (jalon E, à faire dans cet ordre)

Chaque tâche : quelques heures, un critère de fin, testable sur le donjon actuel même s'il change ensuite.

**Bloc 1 — le combat (fondation, surtout des réglages)**
- [ ] **E.1 Garde et parade parfaite** — fenêtre de parade (0,2 s), coût de la garde, garde brisée, l'attaquant vacille
  après une parade ; valeurs dans `combat.ron`.
  *Fin* : parer un gnarling le fait vaciller ; garder trop longtemps brise la garde.
- [ ] **E.2 Esquive et coups imparables** — invulnérabilité de la roulade, drapeau « imparable » et son signe sur
  quelques attaques ennemies.
  *Fin* : un coup imparable traverse la garde mais pas la roulade.

**Bloc 2 — le sac et les ressources**
- [ ] **E.3 Capacité du sac** — taille par objet (`assets/icebreaker/tailles.ron`), total ≤ 24 en expédition ; plein,
  on ne ramasse plus.
  *Fin* : un sac plein refuse un lingot, accepte une potion après avoir lâché un objet.
- [ ] **E.4 Ressources du donjon** — éclat, huile, corde, champignon, planches : objets nouveaux, trouvés en bas,
  disparus à la sortie.
  *Fin* : on remonte sans ses champignons, avec son fer.

**Bloc 3 — creuser et descendre**
- [ ] **E.5 Matières** — dureté par bloc et pierre ancienne indestructible en donjon.
  *Fin* : une bombe ouvre un mur de pierre, pas celui de l'antre.
- [ ] **E.6 Foreuse** — outil qui creuse 3 × 3 en consommant de l'éclat, avec bruit.
  *Fin* : un tunnel de 10 m coûte 5 éclats et réveille une salle voisine.
- [ ] **E.7 Escalade limitée et corde** — escalade de Veloren coupée en donjon sauf sur les prises ; la corde posée au
  bord d'un trou devient une échelle.
  *Fin* : sans corde, on ne remonte pas d'un trou de 8 m ; avec, si.
- [ ] **E.8 Planches et plancher fragile** — pont de 3 blocs ; plancher qui craque puis cède.
  *Fin* : un pont franchit un trou de 3 blocs ; un plancher fragile sépare le groupe.

**Bloc 4 — les monstres à coop** (un par tâche, dans cet ordre : du plus simple au plus lourd)
- [ ] **E.9 Porte-bouclier** · [ ] **E.10 Ombre** · [ ] **E.11 Chaman** · [ ] **E.12 Jumeaux** ·
  [ ] **E.13 Colosse** · [ ] **E.14 Étreigneur** · [ ] **E.15 Essaim**
  *Fin* (chacun) : seul, on peine ; à deux avec la bonne idée, ça passe (essai au client texte puis en jeu).

**Bloc 5 — l'inventaire à formes**
- [ ] **E.16 Grille et formes** — sac en grille dans l'interface, formes par objet, rotation, ceinture rapide.
  *Fin* : un trésor 2 × 3 ne rentre que si on range le reste.

Bloc 1 et bloc 2 peuvent avancer en parallèle (deux worktrees). Les monstres attendent le combat (bloc 1) ; l'étreigneur
et l'essaim attendent les trous (bloc 3).

## Questions pour l'équipe

1. **Ressources du donjon à la sortie** : perdues (proposé : on les trouve sur place), ou gardées à l'entrepôt ?
2. **Escalade de Veloren** : la couper en donjon (proposé, sinon les trous ne comptent pas) ? Et au village ?
3. **Inventaire** : commencer par la capacité simple (E.3) et ne faire la grille (E.16) que si la contrainte plaît ?
4. **Foreuse et classes** : outil pour tous, ou réservé à une classe (ingénieur, `decisions/0010`) ?
5. **Monstres à coop** : lesquels pour la démo ? Proposé : porte-bouclier, ombre, chaman (les moins chers).
