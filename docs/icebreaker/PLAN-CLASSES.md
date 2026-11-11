# Jalon A — Armes, classes et compétences

*Plan validé par Flo le 2026-11-11.* Décision : `decisions/0020-armes-classes-competences.md`. Règles de jeu :
`design/classes.md` (section « Veloren »). Worktree `Veloren-classes`, branche `icebreaker-classes`. Questions pour
l'équipe : `QUESTIONS.md`, Q43 à Q48.

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

À régler en jouant ; le test de A.1 vérifie que chaque fichier de capacité se charge. Ce tableau est le point de
départ du plan ; le contenu en place se lit dans `assets/icebreaker/jeux_armes/`, `ultimes/`, `passifs/` et sur la
page « Classes » du wiki.

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

- [x] **A.0 Docs et piste** — ce plan, `decisions/0020`, `design/classes.md` (section Veloren), jalon dans `README.md`,
  Q13 tranchée et Q43 à Q46 dans `QUESTIONS.md`, les deux todos de `A-FAIRE.md` pris en compte. — 2c06ecba34
- [x] **A.1 Données et règles** — `common/src/icebreaker_competences.rs` et données du guerrier (4 jeux d'armes × 4
  compétences, 3 ultimes, inné et 4 passifs, `salle_armes.ron`). Une classe sans jeu d'armes en données (mage, archer)
  garde les compétences de Veloren jusqu'à A.16 et A.17. Testé (5 tests ciblés) ; rien n'est encore branché en jeu.
  — dcc97f21ba
- [ ] **A.2 Maquettes** — *publiées le 2026-11-11, en attente de l'avis de Flo (`QUESTIONS.md`, Q47)* : rangée
  « 4 · Armes, classes et compétences » du canvas du jalon N (<https://claude.ai/artifact/KXR2tEFGXNNdTk8QJMUcdW>) :
  la barre (1 et 2 pour l'arme, 3 pour l'ultime, passif en case ronde, recharge comme la balise) et dix panneaux
  (lot des recrues à la charrette, salle d'armes en quatre onglets, maîtrise au retour), tous dans le gabarit de
  panneau existant. Les panneaux de la salle d'armes et la place du passif ont été redessinés comme ils sont codés
  et vus dans le vrai client (un bouton d'un mot par ligne, sous-titres sur plusieurs rangs, deux boutons de
  touche en bas ; capture dans la fiche « Classes » du wiki). *Fin* : Flo valide.
- [x] **A.3 Fiche et composant** — la fiche et la recrue portent `acquis` ; composant `IcebreakerCompetences` créé à
  l'entrée en jeu d'après la fiche, envoyé au seul joueur, réécrit dans la fiche à `/rentrer` et à la sauvegarde ;
  `!competences` au client texte. Testé (59 tests du serveur, village et client texte : guerrière avec son lot,
  archère sans composant, fiche réécrite) ; rien à voir dans le vrai client. Le protocole change. — a8c4a0690a
- [x] **A.4 Touches 1 à 3** — crochet `resoudre` dans `handle_ability`, variante `Ability::Icebreaker` (animation de
  la capacité de Veloren reprise), postures ignorées pour ces personnages, niveaux appliqués ; `!touche n` et `!etat`
  au client texte. Testé (7 + 2 tests ; village et client texte : balayage lourd, tourbillon, fureur, rien sur la 4) ;
  pas vu dans le vrai client, dont la barre montre encore les cases de Veloren jusqu'à A.6. — 4e8d8cfc69
- [x] **A.5 Recharges** — la compétence lancée part en recharge (armée par le serveur, lue par le client) ; une touche
  tenue ne relance plus. Testé (3 tests du banc ; village et client texte : refus 2 s après, relance 6 s après) ;
  l'affichage vient avec A.6. — 0d714e99f8
- [x] **A.6 Barre de compétences** *(agent, worktree `Veloren-classes-barre`)* — cases 1 et 2 pour l'arme en mains,
  case 3 pour l'ultime (bord doré), grisées en recharge avec les secondes au centre ou faute d'endurance, infobulle
  (nom, niveau, description, recharge) ; pas `touche n` de `ICEBREAKER_AUTO_PLAN`. Les recharges se lisent à l'heure
  du serveur (ressource `IcebreakerDecalageHorloge` : l'horloge d'un client peut avancer de quelques secondes).
  Le passif actif est dans une case ronde à gauche de la barre, avant le bouton du journal (une seule icône verte
  pour tous les passifs, nom et description en infobulle). Testé (7 tests du client, 6 du banc ; captures au vrai
  client : icônes sur 1, 2, 3, cases grisées « 4 s », « 8 s », ultime « 58 s », case ronde du passif d'une mage) ;
  pas vu : les infobulles, le grisé faute d'endurance. — 51cd4566d8, 37ee7c5f43, 5444039f7e
- [x] **A.7 Jeux d'armes** *(agent, worktree `Veloren-classes-charrette`)* — une recrue d'une classe décrite porte les
  armes de son jeu d'armes et n'a plus de compétences de Veloren ; `UnlockSkill` refusé ; message une fois quand
  l'arme en mains n'est pas apprise ou pas de la classe (système `sys/icebreaker_armes.rs`). Testé (tests du
  serveur) ; pas vu en jeu : le message, le refus de l'arbre. — c2b62ced30, 2be667b7a6
- [x] **A.8 Tirage et fiche à la charrette** *(même agent)* — lot tiré (`Acquis::tirer`, `competences_tirees` de
  `registre.ron`) : un jeu d'armes, deux compétences, un ultime, un passif ; armes du lot en mains et sur la recrue
  devant la charrette ; deux lignes par recrue dans le panneau. Testé (65 tests du serveur ; village et client
  texte : « Bertrand, guerrier · marteau à deux mains », « Moulinet, Interception, ultime Fureur · passif Sang
  chaud », marteau en mains) ; pas vu dans le vrai client. — 343fe52eac
- [x] **A.9 Maîtrise d'arme** — l'expérience des ennemis tués en expédition monte la maîtrise du jeu d'armes en
  mains ; un point gagné s'annonce ; lignes de maîtrise au bilan de retour ; fiche réécrite. Testé (campagne et
  client texte : un gnarling tué = 17 de maîtrise, bilan « Grande épée, +17 de maîtrise » ; avec un taux monté
  pour l'essai : « Arc : un point d'arme gagné. À dépenser à la salle d'armes. ») ; pas vu : l'annonce et le bilan
  dans le vrai client. — f602bdf929
- [x] **A.10 Niveaux et paliers** — niveaux 2 à 5 des seize compétences du guerrier (32 variantes sous
  `assets/icebreaker/abilities/guerrier/`, écrites par quatre agents) ; le niveau est appliqué au lancer (A.4).
  Testé (chaque capacité se charge à chaque niveau ; banc : le tourbillon frappe plus fort au niveau 2 et fait un
  tour de plus au niveau 3) ; pas vu en jeu : il faut la salle d'armes (A.12) pour monter une compétence.
  — 928bc52b1d, e4704e92c0
- [x] **A.11 Salle d'armes : le bâtiment** *(agent, worktree `Veloren-classes-charrette`)* — 7e bâtiment
  (`Building::SalleArmes`, réglage `salle_offset`, enseigne et case du plan), panneau en quatre onglets
  (Compétences, Armes, Ultimes, Passifs), points d'arme, or, jauge de maîtrise ; refus pour un fantôme. Testé
  (74 tests du serveur ; village et client texte : « Salle d'armes » au plan) ; vu dans le vrai client (case du
  plan, panneau). L'essai a montré qu'un clic du plan ouvrait le bâtiment voisin dans une ville sans maison
  (emprises qui se chevauchent sur la place) : corrigé. — 2a49c6e13d, 1202cae497
- [x] **A.12 Salle d'armes : agir** *(même agent)* — commande `/salle` (`server/src/icebreaker_salle.rs`) : débloquer
  et monter (points d'arme), apprendre un jeu d'armes, un ultime, un passif (or ; les armes vont dans le sac),
  activer un ultime ou un passif, placer (« Touche 1 : changer », « Touche 2 : changer », en bas, qui font défiler
  les compétences connues). Un bouton d'un mot par ligne, grisé si impossible ; le sous-titre dit le prix, ce que
  fait la compétence et ce qu'apporte le niveau suivant, et passe à la ligne (gabarit commun des panneaux) ; cadre
  doré pour une compétence placée ; points et or dans l'en-tête ; la maquette est mise à jour. Testé (82 tests du
  serveur, 48 du client ; village et client texte : déblocage, montées 2 et 3, refus faute de points ou d'or,
  placement, passif changé, hache apprise ; campagne, de bout en bout avec une archère : refus sans point, « La
  salle d'armes est au village » en expédition, point gagné, « 1 point à dépenser » au bilan, montée au niveau 2
  au retour ; vrai client sans les mains : les quatre onglets, boutons grisés puis dorés avec de l'or, refus écrit
  en bas du panneau) ; pas vu : un bouton cliqué à la souris, la liste à la molette, le sac plein.
  — f815cd0410, 67aeebe592
- [x] **A.13 Passifs et inné** — composant `IcebreakerPassif` (inné de la classe, puis passif actif), envoyé à tous,
  appliqué à chaque tic par le système des effets ; le personnage entre avec tous ses PV. Testé (9 + 4 tests ;
  village et client texte : guerrière à 143,75 / 143,75 PV) ; pas vu : les PV d'un coéquipier à deux clients, le
  changement de passif (A.12), l'icône du passif dans la barre. Le protocole change. — 3234aeed36
- [x] **A.14 Ultimes du guerrier** — fureur, séisme, rempart (données de A.1, touche 3 de A.4, recharge de A.5) :
  rien de plus à coder. Essai (village et client texte, huit recrues couvrant les quatre jeux d'armes) : chaque
  recrue porte les armes de son lot ; la fureur part avec épée et bouclier ou grande épée, le séisme avec épée et
  bouclier, marteau ou hache, le rempart avec le marteau ; 13 des 16 compétences d'arme vues partir. Pas vu : le
  geste des ultimes avec une arme qui n'est pas la leur (`QUESTIONS.md`, Q45), dans le vrai client. — 4e8d8cfc69,
  0d714e99f8
- [x] **A.15 Paliers des trois autres jeux du guerrier** — contenu seul, écrit avec celui de A.10. Testé au
  chargement ; équilibre et rendu à voir en jeu (vol de vie en mêlée, second coup des balayages, ralentissements).
  — 928bc52b1d
- [x] **A.16 Mage** *(agent, worktree `Veloren-classes-barre`)* — données seules : bâton (Onde de feu, Napalm,
  Anneau de feu, Manteau de flammes), sceptre de soutien (Aura de soin, Protection, Rayon de vie, Cadence), ultimes
  Pyroclasme, Fournaise, Bénédiction, inné (+30 % d'endurance, −10 % de PV) et quatre passifs, niveaux 2 à 5.
  Testé (chargement ; village et client texte : chaque compétence et chaque ultime part, niveaux 1 et 5) ; pas vu :
  les effets sur de vraies cibles (brûlure, soin, hâte), les gestes du bâton avec les copies du sceptre, l'équilibre.
  — 873b538873
- [x] **A.17 Archer** *(même agent)* — données seules : arc (Trait lourd, Vent perçant, Pieu d'épines, Bond
  arrière), ultimes Volée mortelle, Pluie de flèches, Œil d'aigle, inné (+10 % de vitesse, +15 % d'endurance) et
  quatre passifs, niveaux 2 à 5. Testé (chargement ; village et client texte : tout part) ; pas vu : les effets sur
  de vraies cibles (saignement, ralentissement), les gestes dans le vrai client, l'équilibre. — 83b3dbaf2d
- [x] **A.18 Wiki et nettoyage** *(deux agents, worktrees `Veloren-classes-wiki` et `Veloren-classes-charrette`)* —
  wiki : fiche « Classes, armes et compétences », page Classes et jeux d'armes lue dans nos données, salle d'armes
  au schéma du village ; nettoyage : `competences` et `touches` retirés de `classes.ron`, dossier
  `classes/competences/` et `starting_skill_set` / `starting_abilities` supprimés (une classe = un nom, une icône,
  un équipement de base). Testé (60 tests du wiki, 80 du serveur ; village et client texte : les trois classes
  s'engagent avec leur lot et lancent une compétence). Le jalon reste ici tant que A.2 attend ; il ira dans
  `HISTORIQUE.md` ensuite. — fe906d7a6d, 9e9c6905b1

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

En pratique (2026-11-11) : trois worktrees de plus, chacun avec son `target/`, pour que les agents compilent sans
bloquer la session principale : `Veloren-classes-barre` (client compilé : A.6, puis A.16 et A.17),
`Veloren-classes-charrette` (A.7, A.8, A.11, A.12, puis le nettoyage de A.18) et `Veloren-classes-wiki` (wiki de
A.18). Chaque agent committe sur sa branche ; la session principale relit, rebase sur `icebreaker-classes`, relance
les tests et fusionne.

## À vérifier en jeu (reporté dans `PLAN.md` au fil des tâches)

Animation des ultimes avec une arme étrangère au geste (Q45) ; bouclier en main gauche (clic droit, garde) ;
lisibilité de la barre ; deux recrues de même classe côte à côte devant la charrette ; la salle d'armes à la souris
(clics, molette) ; les effets du mage et de l'archer sur de vraies cibles.
