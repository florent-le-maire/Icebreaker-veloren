# Jalon A — Armes, classes et compétences

*Plan validé par Flo le 2026-11-11.* Décision : `decisions/0020-armes-classes-competences.md`. Règles de jeu :
`design/classes.md` (section « Veloren »). Worktree `Veloren-classes`, branche `icebreaker-classes`. Questions pour
l'équipe : `QUESTIONS.md`, Q43 à Q46.

## Pourquoi

Avant ce jalon, une classe n'est qu'un équipement de départ et quelques compétences Veloren débloquées
(`server/src/icebreaker_classes.rs`, `assets/icebreaker/classes.ron`) : ni recharge, ni niveau, ni passif, aucune
restriction d'arme ; la fiche d'une recrue ne porte que son nom et sa classe ; l'arbre de compétences de Veloren est
libre partout. Améliorer un personnage n'a rien d'intéressant et les classes se ressemblent.

Flo veut : des classes à forte identité, un vrai « feeling » par arme façon Guild Wars 2, le format 2 compétences
d'arme + 1 ultime + 1 passif, des compétences à niveaux, et des recrues de la charrette qui ne se ressemblent pas.
Le jalon répond aussi à deux todos de `A-FAIRE.md` du 2026-11-11 (recharges, progression intéressante).

## Règles décidées avec Flo (2026-11-11)

| Sujet | Règle |
|---|---|
| Jeu d'armes | Ce qu'on tient (épée + bouclier, grande épée, marteau…). Il donne les attaques de base (clic gauche, clic droit, garde : celles de Veloren) et un pool de **compétences d'arme** ; on en place **2** (touches 1 et 2). |
| Classe | Elle donne un pool d'**ultimes** (1 placé, touche 3, longue recharge) et de **passifs** (1 actif). Ils ne changent pas avec l'arme et **n'ont pas de niveau** : on en apprend d'autres. |
| Progression | Jouer un jeu d'armes en expédition fait monter **sa maîtrise** et rapporte des **points d'arme**. Ils se dépensent **au village, à la salle d'armes** : débloquer une compétence de cette arme, ou monter d'un niveau une compétence connue. |
| Niveaux | 1 à 5. Niveaux 2 et 4 : des chiffres. Niveaux 3 et 5 : **la compétence se transforme** (effet nouveau). |
| Recharges | Toutes les compétences en ont une, visible dans la barre ; celle de l'ultime est longue. |
| Recrues | Tirage au hasard d'un petit lot : jeu d'armes maîtrisé, 2 compétences connues, un ultime, un passif, les armes qui vont avec. Ensuite **tout s'apprend** à la salle d'armes (or pour un jeu d'armes, un ultime, un passif). |
| En expédition | Un jeu d'armes en mains, changement libre par l'inventaire ; les compétences suivent ce qu'on tient. La pioche garde sa place. |
| Périmètre | Guerrier d'abord (système complet), puis mage et archer. |
| Interface | Maquettes sur le canvas avant de coder (comme le jalon N). |

Choix de l'agent, à confirmer (`QUESTIONS.md`, Q44) :
- Un jeu d'armes non appris (ou étranger à la classe) ne donne que les attaques de base : touches 1 et 2 vides et un
  mot dans la barre. Pas de blocage à l'équipement.
- La maîtrise se compte **par jeu d'armes** et vient de l'expérience des ennemis tués, jeu d'armes en mains, en
  expédition. Les mannequins ne meurent pas : ils n'en donnent pas.
- Chaque classe a aussi un **inné** non choisi (guerrier : PV et tenue ; mage : énergie ; archer : vitesse), posé avec
  le passif.
- L'arbre de compétences de Veloren est coupé pour les personnages du registre (ex-Q13, remplace B.12).
- La mort reste définitive (T.6) : maîtrise et niveaux partent avec le personnage.
- On réutilise les capacités et animations de Veloren ; une transformation est une copie modifiée du fichier de la
  capacité (plus de coups, angle plus large, saignement, étourdissement, protection pendant le geste…).
- Hors registre (donjon lancé seul), rien ne change. Les combos de zones de `decisions/0010` restent hors du jalon.

## Architecture

Une **couche Icebreaker à part**, branchée par un seul crochet dans le lancement des capacités (`decisions/0020`).

- **Données** (`assets/icebreaker/`), un fichier par jeu d'armes et par classe : `jeux_armes/<id>.ron` (classe, main
  droite type + mains, main gauche, équipement, et ses compétences : nom, `capacite` = id d'un fichier
  `CharacterAbility`, `animation`, `icone`, `recharge`, `niveaux`), `ultimes/<classe>.ron`, `passifs/<classe>.ron`
  (listes de `BuffEffect`), `salle_armes.ron` (seuils de points, coûts, prix) ; tirage dans `registre.ron` ; copies et
  variantes de capacités dans `assets/icebreaker/abilities/<classe>/`. Un niveau écrit ses valeurs en entier :
  `(puissance, portee, recharge, capacite: Some(variante))`.
- **Règles** : `common/src/icebreaker_competences.rs` — types, lecture, `jeu_en_main(classe, inventaire)`,
  `effectif(compétence, niveau)`, points depuis la maîtrise, `resoudre(data, touche)`.
- **Fiche** : `Fiche` et `Recrue` (`server/src/icebreaker_registre.rs`) gagnent `#[serde(default)] acquis` : par jeu
  d'armes (maîtrise, points dépensés, compétences et niveaux, 2 placées), ultimes et passifs connus, ceux choisis.
  Aucune migration SQL ; une ancienne fiche reçoit le lot par défaut de sa classe.
- **Composants** : `IcebreakerCompetences` (visible du seul joueur : acquis + fins de recharge) et `IcebreakerPassif`
  (visible de tous, sinon les PV max vus par les coéquipiers sont faux). Créés à l'entrée en jeu
  (`server/src/state_ext.rs`, `update_character_data`), réécrits dans la fiche par la salle d'armes, `/rentrer` et la
  sauvegarde du personnage.
- **Crochets dans Veloren** (courts, commentés « Icebreaker : ») :
  - `common/src/states/utils.rs`, `handle_ability` : `resoudre(...)` avant `activate_ability` ; les `EnterStance`
    ignorés (plus de postures) ; recharge armée par le serveur seul ;
  - `common/src/comp/ability.rs` : variante `Ability::Icebreaker(u16)` — l'animation est choisie par l'id texte de la
    capacité, d'où un alias `animation` par compétence ;
  - `common/src/states/behavior.rs`, `common/systems/src/character_behavior.rs` : le composant dans `JoinData` ;
  - `common/net/src/synced_components.rs`, `common/state/src/state.rs` : enregistrement des deux composants ;
  - `common/systems/src/buff.rs` (après `reset_temp_modifiers`) : effets du passif via `execute_effect` ;
  - `server/src/events/entity_manipulation.rs` (attribution d'expérience) : maîtrise du jeu en mains ;
  - `server/src/sys/msg/in_game.rs` : `UnlockSkill` refusé pour ces personnages.
- **Barre** : sur le modèle de la balise — `voxygen/src/icebreaker_competences.rs` (instantané par touche),
  `voxygen/src/hud/hotbar.rs`, `slots.rs`, `skillbar.rs` ; passif en 4e case.
- **Salle d'armes** : 7e bâtiment ajouté à la fin de `Building` (`server/src/icebreaker_buildings.rs`,
  `server/src/lib.rs`, `sys/icebreaker_panneaux.rs`, `village.ron`, `vue_village.ron`) ; panneau typé existant
  (`common/src/icebreaker_panneau.rs`) avec une sélection gardée côté serveur ; commande `/salle` ; or pris par
  `icebreaker_treasury::take_coins`, comme `/soigner`.

## Contenu de départ

À régler en jouant ; le test de A.1 vérifie que chaque fichier de capacité se charge.

| Classe | Jeu d'armes | Compétences d'arme (`common.abilities.…`) |
|---|---|---|
| Guerrier | Épée + bouclier | `sword.defensive_riposte`, `sword.heavy_pommel_strike`, `sword.basic_skewer`, `sword.defensive_disengage` |
| Guerrier | Grande épée | `sword.heavy_sweep`, `sword.cleaving_whirlwind_slice`, `sword.cleaving_earth_splitter`, `sword.basic_fell_strike` |
| Guerrier | Marteau à deux mains | `hammer.scornful_swipe`, `hammer.intercept`, `hammer.iron_tempest`, une 4e sans coût de combo |
| Guerrier | Hache à deux mains | `axe.brutal_swing`, `axe.rake`, `axe.skull_bash`, `axe.fierce_raze` |
| Mage | Bâton | `staff.fireshockwave`, `staff.napalm_strike`, `staff.fire_breath`, `staff.flame_cloak` |
| Mage | Sceptre | `sceptre.wardingaura` + 3 copies à écrire (soin, rayon, onde) |
| Archer | Arc | `bow.heavy_nock`, `bow.piercing_gale`, `bow.thorn_stake`, `bow.foothold` |

Ultimes : guerrier `axe.furor`, `hammer.tremor`, `hammer.rampart` ; mage `staff.pyroclasm`, `staff.napalm_strike`,
`sceptre.wardingaura` ; archer `bow.death_volley`, `bow.fusillade`, `bow.eagle_eye`. Passifs : 3 ou 4 par classe,
écrits en `BuffEffect` (PV max, réduction de dégâts, vitesse, énergie rendue, effet à l'attaque ou quand on est
touché). Plus tard, en données seulement : deux épées, hache + épée (la hache à une main n'existe qu'en arme forgée).

## Tâches

Un commit par tâche, fusion dans `master` à chaque tâche ; le jeu reste lançable après chacune.

- [ ] **A.0 Docs et piste** — ce plan, `decisions/0020`, `design/classes.md` (section Veloren), jalon dans `README.md`,
  Q13 tranchée et Q43 à Q46 dans `QUESTIONS.md`, les deux todos de `A-FAIRE.md` pris en compte.
- [ ] **A.1 Données et règles** — types, lecture, `jeu_en_main`, `effectif`, points ; guerrier : 4 jeux × 4
  compétences. *Fin* : chaque `capacite` se charge. `cargo test -p veloren-common icebreaker_competences`.
- [ ] **A.2 Maquettes** — canvas du jalon N : barre (2 + ultime + passif, recharge), fiche d'une recrue, salle
  d'armes. *Fin* : Flo valide. Sans code, en parallèle de A.3 à A.5.
- [ ] **A.3 Fiche et composant** — `acquis`, lot par défaut, composant à l'entrée en jeu, réécriture au retour.
  *Fin* : un ancien `registre.ron` se relit ; le client reçoit le composant. Tests du registre, client texte.
- [ ] **A.4 Touches 1 et 2** *(risquée, cœur du jalon)* — variante, crochet, postures neutralisées, cases de la
  barre, `!touche n` dans `chat_cli`. *Fin* : la touche 1 lance la compétence avec son animation.
  `cargo test -p veloren-common-systems icebreaker` (banc `common/systems/tests/character_state.rs`).
- [ ] **A.5 Recharges** — *Fin* : second appui refusé avant la fin de la recharge (même banc, deux tics).
- [ ] **A.6 Barre de compétences** — icônes, infobulles, secondes, passif, selon la maquette. Capture au vrai client.
- [ ] **A.7 Jeux d'armes** — équipement de chaque jeu, jeu non appris = touches vides et message, `UnlockSkill` refusé.
- [ ] **A.8 Tirage et fiche à la charrette** *(format du registre)* — lot tiré, armes du lot, recrue devant la
  charrette avec ses armes, lot lisible dans le panneau. *Fin* : deux guerriers diffèrent. Tests du registre.
- [ ] **A.9 Maîtrise d'arme** — expérience des ennemis tués en expédition → maîtrise du jeu en mains → points ; ligne
  au bilan de retour. *Fin* : des points après une expédition, aucun sur les mannequins.
- [ ] **A.10 Niveaux et paliers** — `effectif` branché ; grande épée : paliers 3 et 5 de ses 4 compétences.
  *Fin* : le niveau 3 se voit en jeu (autre geste ou effet), le niveau 2 frappe plus fort.
- [ ] **A.11 Salle d'armes : le bâtiment** — 7e bâtiment, panneau en lecture (compétences, niveaux, maîtrise, points).
- [ ] **A.12 Salle d'armes : agir** — placer, débloquer, monter (points) ; apprendre un jeu d'armes, un ultime, un
  passif (or). *Fin* : chaque bouton agit ou dit pourquoi non. Test du constructeur de panneau, client texte.
- [ ] **A.13 Passifs et inné** *(touche au système de buffs)* — *Fin* : mêmes PV max vus par un coéquipier.
- [ ] **A.14 Ultimes du guerrier** — touche 3, longue recharge, trois ultimes. À voir avec les 4 jeux d'armes en mains.
- [ ] **A.15 Paliers des trois autres jeux du guerrier** — contenu seul.
- [ ] **A.16 Mage** — bâton, sceptre (copies à écrire), ultimes, passifs, paliers.
- [ ] **A.17 Archer** — arc, ultimes, passifs, paliers.
- [ ] **A.18 Wiki et nettoyage** — `icebreaker-wiki/src/competences.rs`, fiche `wiki/`, retrait de `touches` et
  `competences`, tests de `icebreaker_classes.rs`, jalon dans `HISTORIQUE.md`.

Reportable : blocage dur des armes, bascule entre deux jeux d'armes (Q46), masquer le journal de Veloren dans le
client, recharges gardées à la reconnexion.

## Qui fait quoi

Un worktree = un seul `target/`, donc une seule compilation à la fois ; et le cœur (A.1, A.3, A.4, A.5) est une
chaîne où chaque tâche modifie les mêmes fichiers que la précédente.

| Qui | Tâches | Comment |
|---|---|---|
| Session principale | A.0, A.1, A.2, A.3, A.4, A.5, A.9, mécanisme de A.10, A.13 | En séquence : crochets dans Veloren, format des données, canvas. |
| Un agent moins coûteux à la fois | A.6, A.7, A.8, A.11, A.12, A.14, A.18 | Code qui suit un modèle déjà présent (balise, registre, bâtiments, panneaux). Cahier précis, puis relecture du diff et tests par la session principale avant le commit. |
| Agents moins coûteux en parallèle | contenu de A.10, A.15, A.16, A.17, passifs, pages du wiki | Fichiers de données seulement, un fichier par agent, sans compiler ; la session principale lance ensuite le test de chargement de A.1. |

Si une tâche confiée revient deux fois fausse, la session principale la reprend.

## À vérifier en jeu (reporté dans `PLAN.md` au fil des tâches)

Animation des ultimes avec une arme étrangère au geste (Q45) ; bouclier en main gauche (clic droit, garde) ;
lisibilité de la barre ; deux recrues de même classe côte à côte devant la charrette.
