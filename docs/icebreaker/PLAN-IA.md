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
  un vrai mur, les chemins longs en donjon, le vrai client. `HASH_IA2`
- [ ] **IA.3 Entendre** — bruits de combat et de mort (`icebreaker_ia_monde`, `ia/bruits.ron`), ouïe par profil,
  enquête sur un bruit, aussi en combat.
  *Fin* : on tue un ennemi hors de vue d'un autre : l'autre vient voir.
- [ ] **IA.4 Prévenir** — tirage à l'événement, `aller_prevenir`, cri porteur d'un signalement, scènes
  (`ia/scenes.ron`).
  *Fin* : scène `massacre` : on tue un groupe d'orcs, un survivant éloigné part (ou non) chercher des renforts, qui
  viennent sur le lieu du massacre ; ils attaquent s'ils nous voient, sinon ils fouillent.
- [ ] **IA.5 Habileté** — crans étendus (`difficulte.ron`) : délai de réaction, roulade, parade, pause entre deux
  attaques, sens ; classeur d'équilibrage relancé.
  *Fin* : le même ennemi en difficulté 1 et 6 : l'un frappe simple et laisse des temps morts, l'autre roule et enchaîne.
- [ ] **IA.6 Le mage** — table de sorts du bâton, par cran, à la place de `handle_staff_attack` pour nos mages.
  *Fin* : le mage choisit son sort selon la distance et la situation.
- [ ] **IA.7 Défense réactive** — « le joueur arme un coup » : esquive ou parade tirée après le délai de réaction.
  *Fin* : au cran haut, un coup lent est esquivé ou paré ; une compétence rapide passe.
- [ ] **IA.8 Fuite et coincé** — le fuyard n'oublie pas ; coincé, il tire : se cacher, forcer le passage.
  *Fin* : un danari blessé fuit ; coincé, il force le passage ou se tapit.
- [ ] **IA.9 Appel à l'aide et déroute** (ex-G.12, G.13).
  *Fin* : frapper l'archer fait venir le guerrier voisin ; tuer le chef disperse sa garde.
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
