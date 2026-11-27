# Plan — Une IA à nous pour les six espèces (jalon IA)

Décision : `decisions/0024-ia-des-ennemis.md`. Règles : `design/creatures.md` (section « Pour Veloren ») et
`design/menace.md`. Questions : `QUESTIONS.md`, Q69 à Q73 (et Q50 pour les identités de race). Worktree
`Veloren-ia`, branche `icebreaker-ia`.

But : des combats intéressants et des expéditions vivantes, pour nos six espèces humanoïdes (humain, orc, nain, elfe,
draugr, danari). Les autres créatures de Veloren ne sont pas concernées.

## Les pièces

```
icebreaker-ia/            crate pur (vek, serde), sans Veloren ; tests dans chaque fichier
  perception.rs           la vue : cône, dos, soupçon ou net
  ouie.rs                 les bruits : portée par sorte, finesse d'ouïe, écart de hauteur
  cerveau.rs              le tronc : états, cible, enquête, retenue, ordres ; tick(observation, profil) → intention ;
                          entrées (entendre, recevoir, ordonner) et sorties (cris, effets, retenu, sort, geste)
  evenements.rs           VoitIntrus, AllieTombe, CoupArme, Coince, Touche, ChefTombe, PvBas ; les messages
  reactions/              prevenir.rs, defense.rs, coince.rs, rage.rs (les autres réactions sont dans le tronc)
  tirage.rs               tirage pondéré à l'événement
  cible.rs                rang, score, acheve, reciblage (venus de common/src/icebreaker_races.rs)
  combat.rs               la table de sorts du mage
  profil.rs               Profil = commun + race + rôle + cran (macro `profil!`)

assets/icebreaker/ia/                     profils.ron, bruits.ron, scenes.ron (lus au lancement du serveur)
common/src/icebreaker_ia.rs               charge les données ; scènes d'essai ; liste des nôtres
common/src/comp/agent.rs                  champ `icebreaker_ia`
common/src/generation.rs                  EntityInfo::icebreaker_ia (rôle et cran, posés par world/src/icebreaker/races.rs)
server/src/sys/agent/icebreaker_ia.rs     adaptateur : observation, exécution de l'intention, filtre des entrées
server/src/sys/agent/behavior_tree/mod.rs une ligne dans root()
server/agent/src/data.rs                  ReadData::icebreaker_allies
server/src/sys/icebreaker_ia_monde.rs     bruits, cris, mort du chef, effets (rage), liste des nôtres
server/src/sys/icebreaker_rondes.rs       les rondes, par la boîte d'ordres
server/src/icebreaker_arene.rs            /entrainement scene <nom>, bruit, ia
```

Écarts avec le plan du 2026-11-26 : pas de mini-monde de simulation (les scènes se jouent au lieu d'entraînement,
les règles se testent tick par tick) ; les tables de sorts sont au rôle, pas au cran (le rang du cran décide des
sorts disponibles) ; les crans sont dans `ia/profils.ron`, aux clés de `difficulte.ron`.

Règles de conduite :

- Le cerveau ne lit que son observation ; la position d'un joueur non perçu n'y est jamais.
- Une décision de groupe n'existe pas : chaque ennemi tire la sienne à l'événement, avec des poids en données qui
  tiennent compte de ce qu'il voit faire ses voisins. Chaque décision tirée s'annonce (cri, bulle).
- Un comportement de race = des lignes de profil, au pire un fichier dans `reactions/`. On ne touche pas le tronc.
- TDD sur le crate (décision `0023`) : chaque règle de `creatures.md` a son test qui la cite ; les scènes sont des
  tests de propriétés.
- Chaque tâche a sa scène au lieu d'entraînement (`/entrainement scene <nom>`, `/entrainement ia`).

## Tâches

- [x] **IA.0 Décision et plan** — décision `0024`, ce plan, section « Pour Veloren » de `design/creatures.md`,
  questions Q69 à Q73. Plan validé par Flo le 2026-11-26 ; Sayanel n'a pas relu. `2575e705`
- [x] **IA.1 Socle** — crate `icebreaker-ia`, cerveau (repos, combat, fuite), nœud, champ, `ia/profils.ron`, règles de
  cible migrées (patches Y retirés de Veloren), `/entrainement ia`. Testé ; client texte : orcs et danari se battent,
  achèvent, repassent au repos. Pas vu : vrai client, fuite des danari, répartition à plusieurs, rondes. `78b24298`
- [x] **IA.2 Voir et chercher** — vue à nous (cône, dos, soupçon « ? » puis repéré « ! »), dernière position connue,
  poursuite, fouille, retour au poste, `laisse` (réglage, aucune race ne s'en sert). Testé ; client texte au lieu
  d'entraînement : « ? », « ! », combat, puis le joueur téléporté au loin : l'orc fouille et rentre. Pas vu : derrière
  un vrai mur, les chemins longs en donjon, le vrai client. `2ec6141a`
- [x] **IA.3 Entendre** — un coup qui touche et une mort font un bruit (`icebreaker_ia_monde`, `ia/bruits.ron`) ;
  `ouie` par profil, écart de hauteur compté double ; celui qui entend va voir, fouille, rentre ; en combat il le
  retient pour quand il perd sa cible ; un dormeur qui entend se réveille. Scènes d'essai (`ia/scenes.ron`,
  `/entrainement scene ecoute`, `/entrainement bruit`). Testé ; client texte : dans le dos de l'orc rien, au bruit
  « ? » puis « ! ». Pas vu : un vrai meurtre hors de vue, le réveil d'un dormeur, le vrai client. `13c1c901`
- [x] **IA.4 Prévenir** — tirage pondéré à l'événement (`reactions` du profil : `VoitIntrus`, `AllieTombe`),
  réaction `reactions/prevenir.rs`, cri porteur d'un signalement, profil = commun + race + rôle + cran (le chef ne
  part pas), liste des nôtres tenue par `icebreaker_ia_monde`. Testé ; client texte, scène `massacre` (poids forcés) :
  « Je vais chercher les autres ! », « Des intrus ! », le groupe du nord arrive (« ? » puis « ! »). Pas vu : le vrai
  client, un donjon (chemin du messager), le tirage aux vrais poids (1 sur 6 à la vue, 1 sur 3 à une mort). `c6bec35c`
- [x] **IA.5 Habileté** — `crans` de `ia/profils.ron` (mêmes clés que `difficulte.ron`) : délai de réaction, temps
  mort après une attaque, enchaînement borné, roulade et parade filtrées (le cerveau retire des entrées à ce que
  `attack()` de Veloren propose), sens un peu plus fins ; classeur relancé (colonne Habileté). Corrige IA.3 : un coup
  ne faisait pas de bruit (daté du tick précédent). Testé ; client texte, un orc sur un joueur immobile : 0,55 coup
  par seconde en difficulté 1 (1,15 sans les réglages du cran), 2,8 en difficulté 6. Pas vu : le vrai client, la
  roulade et la parade, l'archer et le mage. `f09f41ef`
- [x] **IA.6 Le mage** — table de sorts du bâton (`sorts` du rôle mage, `combat.rs`) : distance, cible qui fonce,
  recharge, poids ; les sorts dont il dispose viennent du rang de son cran. `handle_staff_attack` garde le
  déplacement et la visée, son choix de sort est retiré et remplacé par celui du cerveau. Scène `mage`. Testé ;
  client texte : boule de feu à 25 blocs, lance-flammes ou boule à 10, onde de feu à 3, napalm en difficulté 6. Pas
  vu : le vrai client (visée de chaque sort, ruée, cape), une cible qui fonce. `a0be7c03`
- [x] **IA.7 Défense réactive** — un adversaire à moins de `garde` blocs arme un coup de mêlée vers lui
  (`CoupArme`) : il tire une fois rien, esquiver ou parer (`reactions/defense.rs`), et fait le geste après son délai
  de réaction ; un coup qui tombe avant passe. Poids par cran (base : rien ; aguerri : pare 1 fois sur 3 ; vétéran :
  pare ou roule 4 fois sur 5). Scène `duel`. Testé ; client texte, difficulté 6 : « esquive ! » deux fois sur quatre
  coups du joueur. Pas vu : la parade, l'effet réel (le coup est-il évité ?), le vrai client ; les tirs ne sont pas
  esquivés. `5023fea0`
- [x] **IA.8 Fuite et coincé** — `fuite_memoire` (il fuit 10 s une menace qu'il ne voit plus, puis se calme sur
  place) ; coincé (1 s sans s'éloigner, menace à 8 blocs) il tire `SeTapir` (frappé, il détale à travers son
  agresseur) ou `Forcer` (4 s au plus vers un point derrière la menace), `reactions/coince.rs` ; le chef ne fuit pas.
  Scène `fuyard` (un danari à 25 % de ses PV). Testé ; client texte : « fuite ! », il court à 36 blocs, « repos »
  10 s plus tard, sans revenir. Pas vu : coincé (l'arène n'a pas de mur), se cacher derrière un angle (pas codé :
  il se tapit sur place), le vrai client. `f1f0750a`
- [x] **IA.9 Appel à l'aide et déroute** (ex-G.12, G.13) — archer et mage frappés sous 75 % de leurs PV crient
  « À l'aide ! » (30 blocs, toutes les 6 s au plus) ; guerriers et chef qui l'entendent prennent l'agresseur pour
  cible 6 s (sans le voir, ils courent où le cri le dit), les autres viennent voir ; à la mort du chef (24 blocs) :
  « Fuite ! », guerriers 4 s, archers et mages 8 s, puis ils reviennent voir. Scènes `appel` et `garde`. Testé ;
  client texte : « À l'aide ! » puis le guerrier éloigné arrive ; chef tué, trois « Fuite ! » puis retour. Pas vu :
  le vrai client, le guerrier qui quitte un autre combat pour venir. `d822a3cb`
- [x] **IA.10 Ordres** — boîte d'ordres du cerveau (`Cerveau::ordonner`, `Ordre::Aller` : un lieu et une allure, ni
  joueur ni position de joueur) ; les rondes humaines (Y.3, `icebreaker_rondes`) y écrivent au lieu de passer par
  rtsim ; combat, enquête et fuite passent avant, puis il reprend son ordre sans rentrer à son poste. Testé ; client
  texte, donjon humain : deux rondes font 1, 2, 1, 0, 1, 2 dans le journal. Pas vu : une ronde qui entend un combat,
  enquête et reprend (testé dans le crate seulement), le vrai client. `03a57c73`
- [x] **IA.11 Une race en plus** — Y.4, orcs en rage : `reactions/rage.rs` et trois réglages de la race (`pv_bas`,
  `reactions: { PvBas: [(Rage, 1.0)] }`, `rage`) ; l'effet est le « Berserk » de Veloren, posé par
  `icebreaker_ia_monde` (+40 % de dégâts, plus vite, encaisse moins bien, 12 s). Le tronc gagne l'événement `PvBas` et
  la sortie `effets`, qui serviront aux prochaines races. Scène `rage`. Testé ; client texte : un orc à 55 % frappé une
  fois crie « RAAAH ! », une seule fois. Pas vu : l'effet au vrai client (frappe-t-il visiblement plus fort ?). `6154a1c7`
- [ ] **IA.12 Mémoire du lieu et niveau d'alerte** (ex-G.15) — à redécouper. *Découpage proposé le 2026-11-27, en
  attente de l'accord de l'équipe (Q74)* :
  - IA.12a Mémoire du lieu : une ressource par lieu (intrus signalés : où, quand ; événements : cri, mort, chef
    tombé), nourrie par ce qui passe déjà par `icebreaker_ia_monde` ; elle vieillit. Rien ne la lit encore.
  - IA.12b Niveau d'alerte : calme, inquiet, en alerte, mobilisé, calculé en points depuis ces événements, qui
    redescend lentement (`design/menace.md`) ; `/entrainement ia` l'affiche.
  - IA.12c Ce que le niveau change, un effet par tâche, par ordres et profils : dormeurs au sommeil léger, puis
    qui se lèvent ; rondes allongées ; groupes envoyés aux intrus signalés (boîte d'ordres, Q71).
  - IA.12d Les signes : texte d'ambiance, torches (côté client et monde, hors du crate).
- [ ] **IA.13 Salles exportées** — `MissionTargets.salles` ; embuscade (ex-G.16), nains qui tiennent leur salle (Y.5).
  *Pas commencé : trois tâches en une, dont deux attendent une réponse (Q75).* Proposé : IA.13a les salles du donjon
  généré connues du serveur (emprise, portes) et le bruit arrêté par les murs (même salle : portée pleine ; salle
  voisine : réduite ; au-delà : rien) — à faire avec la piste `Veloren-donjon`, qui tient le générateur ; IA.13b les
  nains tiennent leur salle (après Q50) ; IA.13c l'embuscade (après validation de `design/menace.md`, idée 2).
- [ ] **IA.14 Guerrier et archer en table** — seulement si IA.5 et IA.7 ne suffisent pas : à décider après l'essai
  en jeu de la scène `duel`.
- [ ] **IA.15 Wiki** (ex-Y.8) — après la réponse à Q50. *Fait en avance* : la fiche « L'IA des ennemis »
  (`wiki/ia.md`) dit ce qui est codé ; il reste le comportement de chaque race.

## Ce que deviennent les tâches de `PLAN.md`

| Tâche | Devient |
|---|---|
| G.11 Attaque surprise | reste dans `PLAN.md` ; s'appuie sur l'état « n'a rien remarqué » du cerveau (après IA.2) |
| G.12, G.13 | IA.9 |
| G.14 Guetteur messager | IA.4 (plus de guetteur : n'importe qui peut aller prévenir) |
| G.15 | IA.12 |
| G.16 | IA.13 |
| Y.4 | IA.11 |
| Y.5 Nains tenaces | laisse au poste : réglage de IA.2 ; tenir sa salle : IA.13 ; l'armure est dans la fiche |
| Y.6 Elfes tireurs | vue : une ligne de profil ; plus d'archers : la composition du lieu, hors IA |
| Y.7 Draugr dormeurs | part d'endormis : générateur ; lenteur : fiche ; réveil au bruit : IA.3 |
| Y.8 | IA.15 |

## Risques

- Chemins longs (prévenir, enquêter) avec le `Chaser` de Veloren à travers portes et escaliers : à vérifier dès IA.2
  sur un vrai donjon.
- Un rayon de ligne de vue par joueur à portée et par ennemi : à échelonner.
- Le bruit traverse les murs jusqu'à IA.13 : portées courtes au départ.
- L'essai au client texte ne dit pas si un geste réussit (une parade, une roulade, la visée d'un sort) : tout le
  jalon reste à voir dans le vrai client (`PLAN.md`, « À vérifier en jeu », IA.1 à IA.11).
- « Coincé » n'a pas pu être essayé au lieu d'entraînement (pas de mur) : à voir dans un donjon danari.
- Un handler de Veloren enveloppé peut changer sans prévenir après une mise à jour : rejouer les scènes.
- IA.1, IA.2 et IA.6 dépassent « quelques heures ».
- Toute identité de race codée avant la réponse à Q50 peut être jetée : seule IA.11 en code une.
