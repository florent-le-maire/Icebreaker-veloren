# Plan — Une IA à nous pour les six espèces (jalon IA)

Décision : `decisions/0024-ia-des-ennemis.md`. Règles : `design/creatures.md` (section « Pour Veloren ») et
`design/menace.md`. Questions : `QUESTIONS.md`, Q69 à Q73 (et Q50 pour les identités de race). Worktree
`Veloren-ia`, branche `icebreaker-ia`.

But : des combats intéressants et des expéditions vivantes, pour nos six espèces humanoïdes (humain, orc, nain, elfe,
draugr, danari). Les autres créatures de Veloren ne sont pas concernées.

## Les pièces

```
icebreaker-ia/            crate pur (vek, serde, ron, rand), sans Veloren
  perception.rs           cône, dos, soupçon → repéré, ouïe, dernière position connue
  cerveau.rs              états Repos / Enquête / Combat / Fuite + réaction en cours ; tick(obs, profil, dt, rng)
  evenements.rs           VoitIntrus, EntendBruit, EntendCri, Touché, AlliéTombe, PvSous(x), CiblePerdue,
                          Coincé, MessageReçu, OrdreReçu, CibleÀTerre, ChefMort
  reactions/              un fichier par réaction (combattre, enqueter, fuir, aller_prevenir…)
  tirage.rs               tirage pondéré à l'événement
  cible.rs                rang, score, acheve, reciblage (venus de common/src/icebreaker_races.rs)
  combat.rs               habileté et tables de compétences par arme
  profil.rs               Profil = commun + race + rôle + cran
  tests/simulation.rs     mini-monde 2D pour jouer des scènes sans le jeu

assets/icebreaker/ia/                     profils.ron, bruits.ron, scenes.ron
common/src/icebreaker_ia.rs               charge les données
common/src/comp/agent.rs                  champ `icebreaker_ia`
server/src/sys/agent/icebreaker_ia.rs     adaptateur : observation, exécution de l'intention
server/src/sys/agent/behavior_tree/mod.rs une ligne dans root()
server/src/sys/icebreaker_ia_monde.rs     bruits, cris, messages, ordres, mémoire du lieu
```

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
  « ? » puis « ! ». Pas vu : un vrai meurtre hors de vue, le réveil d'un dormeur, le vrai client.
- [x] **IA.4 Prévenir** — tirage pondéré à l'événement (`reactions` du profil : `VoitIntrus`, `AllieTombe`),
  réaction `reactions/prevenir.rs`, cri porteur d'un signalement, profil = commun + race + rôle + cran (le chef ne
  part pas), liste des nôtres tenue par `icebreaker_ia_monde`. Testé ; client texte, scène `massacre` (poids forcés) :
  « Je vais chercher les autres ! », « Des intrus ! », le groupe du nord arrive (« ? » puis « ! »). Pas vu : le vrai
  client, un donjon (chemin du messager), le tirage aux vrais poids (1 sur 6 à la vue, 1 sur 3 à une mort).
- [x] **IA.5 Habileté** — `crans` de `ia/profils.ron` (mêmes clés que `difficulte.ron`) : délai de réaction, temps
  mort après une attaque, enchaînement borné, roulade et parade filtrées (le cerveau retire des entrées à ce que
  `attack()` de Veloren propose), sens un peu plus fins ; classeur relancé (colonne Habileté). Corrige IA.3 : un coup
  ne faisait pas de bruit (daté du tick précédent). Testé ; client texte, un orc sur un joueur immobile : 0,55 coup
  par seconde en difficulté 1 (1,15 sans les réglages du cran), 2,8 en difficulté 6. Pas vu : le vrai client, la
  roulade et la parade, l'archer et le mage.
- [x] **IA.6 Le mage** — table de sorts du bâton (`sorts` du rôle mage, `combat.rs`) : distance, cible qui fonce,
  recharge, poids ; les sorts dont il dispose viennent du rang de son cran. `handle_staff_attack` garde le
  déplacement et la visée, son choix de sort est retiré et remplacé par celui du cerveau. Scène `mage`. Testé ;
  client texte : boule de feu à 25 blocs, lance-flammes ou boule à 10, onde de feu à 3, napalm en difficulté 6. Pas
  vu : le vrai client (visée de chaque sort, ruée, cape), une cible qui fonce.
- [x] **IA.7 Défense réactive** — un adversaire à moins de `garde` blocs arme un coup de mêlée vers lui
  (`CoupArme`) : il tire une fois rien, esquiver ou parer (`reactions/defense.rs`), et fait le geste après son délai
  de réaction ; un coup qui tombe avant passe. Poids par cran (base : rien ; aguerri : pare 1 fois sur 3 ; vétéran :
  pare ou roule 4 fois sur 5). Scène `duel`. Testé ; client texte, difficulté 6 : « esquive ! » deux fois sur quatre
  coups du joueur. Pas vu : la parade, l'effet réel (le coup est-il évité ?), le vrai client ; les tirs ne sont pas
  esquivés.
- [x] **IA.8 Fuite et coincé** — `fuite_memoire` (il fuit 10 s une menace qu'il ne voit plus, puis se calme sur
  place) ; coincé (1 s sans s'éloigner, menace à 8 blocs) il tire `SeTapir` (frappé, il détale à travers son
  agresseur) ou `Forcer` (4 s au plus vers un point derrière la menace), `reactions/coince.rs` ; le chef ne fuit pas.
  Scène `fuyard` (un danari à 25 % de ses PV). Testé ; client texte : « fuite ! », il court à 36 blocs, « repos »
  10 s plus tard, sans revenir. Pas vu : coincé (l'arène n'a pas de mur), se cacher derrière un angle (pas codé :
  il se tapit sur place), le vrai client.
- [x] **IA.9 Appel à l'aide et déroute** (ex-G.12, G.13) — archer et mage frappés sous 75 % de leurs PV crient
  « À l'aide ! » (30 blocs, toutes les 6 s au plus) ; guerriers et chef qui l'entendent prennent l'agresseur pour
  cible 6 s (sans le voir, ils courent où le cri le dit), les autres viennent voir ; à la mort du chef (24 blocs) :
  « Fuite ! », guerriers 4 s, archers et mages 8 s, puis ils reviennent voir. Scènes `appel` et `garde`. Testé ;
  client texte : « À l'aide ! » puis le guerrier éloigné arrive ; chef tué, trois « Fuite ! » puis retour. Pas vu :
  le vrai client, le guerrier qui quitte un autre combat pour venir.
- [ ] **IA.10 Ordres** — boîte d'ordres ; les rondes humaines (Y.3) passent dessus.
  *Fin* : la ronde tourne comme avant ; une ronde qui entend un combat enquête, puis reprend.
- [ ] **IA.11 Une race en plus** — Y.4, orcs en rage : une réaction et trois lignes de profil, tronc intact.
  *Fin* : un orc sous la moitié de ses PV s'enrage, une fois.
- [ ] **IA.12 Mémoire du lieu et niveau d'alerte** (ex-G.15) — à redécouper.
- [ ] **IA.13 Salles exportées** — `MissionTargets.salles` ; embuscade (ex-G.16), nains qui tiennent leur salle (Y.5).
- [ ] **IA.14 Guerrier et archer en table** — seulement si IA.5 et IA.7 ne suffisent pas.
- [ ] **IA.15 Wiki** (ex-Y.8) — après la réponse à Q50.

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
- Un handler de Veloren enveloppé peut changer sans prévenir après une mise à jour : rejouer les scènes.
- IA.1, IA.2 et IA.6 dépassent « quelques heures ».
- Toute identité de race codée avant la réponse à Q50 peut être jetée : seule IA.11 en code une.
