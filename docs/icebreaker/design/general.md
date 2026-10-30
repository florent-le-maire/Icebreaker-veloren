# Game design — Généralités

Joueurs, temps, déplacement, style visuel, caméra et contrôles.
Index des règles : [`docs/GAME_DESIGN.md`](../GAME_DESIGN.md).

## Joueurs

- Jusqu'à **10 joueurs** sur le serveur, et jusqu'à **10 dans une même instance** de donjon.
- Plateforme : Windows.

## Temps

- 1 minute réelle = **30 minutes** sur la carte (`time.mapMinutesPerRealMinute`) → une journée = 48 min réelles.
- Le monde s'arrête quand tous les joueurs sont déconnectés (phase 4).

## Déplacement

- Vitesse : celle de la classe (`moveSpeed` dans `game/data/combat/classes.json`, 5 m/s pour l’instant), réduite par les ralentissements.
- Contrôles : ZQSD / WASD / flèches, relatifs à la caméra.

## Style visuel

- Formes primitives pour l'instant ; assets libres (CC0) plus tard.

## Caméra et contrôles

- Caméra à la troisième personne **façon Elden Ring** (décision d’équipe du 2026-10-19, voir `docs/decisions/0003`) :
  centrée derrière le personnage, qui apparaît sous le centre de l’écran. **La souris tourne la caméra** autour de lui
  (capturée pendant le jeu, horizontal et vertical entre `minPitch` et `maxPitch` : on peut regarder vers le haut,
  axe inversable : réglage du joueur `gameplay.invertY` dans `user://settings.json`) ; **molette** : distance ; **Échap** ouvre le menu pause (`PauseMenu`) : le jeu continue (multijoueur), mais le personnage s’arrête et n’attaque plus tant qu’il est ouvert ; « Se déconnecter » et « Quitter » laissent le personnage en cadavre (confirmation demandée).
- La caméra ne monte jamais à plus de `camera.maxHeight` au-dessus des pieds du personnage (sous le haut des murs de son
  étage) : on ne voit pas par-dessus les murs, même sur une mezzanine.
  Quand un mur ou le sol est derrière le personnage, elle se rapproche de lui ; plaquée contre lui, il est masqué.
- En regardant vers le haut, le point visé monte au-dessus de la tête : le personnage descend dans l’écran et ne cache
  pas une mezzanine ou un balcon. L’étiquette « Vous » disparaît quand la caméra est proche.
- **Le personnage se tourne vers où il marche** (`player.turnSpeed`), pas vers la caméra : on peut regarder autour de soi.
- **Attaques (visée hybride)** : au lancement, le personnage pivote d’un coup vers le point visé (petit point au centre
  de l’écran) ou vers la cible verrouillée, et l’attaque part là. Pendant `player.attackCommitTime` s, sa direction
  est figée : bouger la caméra ou marcher ne la change plus.
- **Verrouillage de cible** (clic molette) : la créature vivante et visible la plus proche du centre de l’écran
  (portée `camera.lockRange`, écart maximal `camera.lockMaxAngle`). Un point blanc la marque ; le personnage lui fait
  face, la caméra la garde dans l’axe et les attaques partent vers elle. Un geste franc de la souris à gauche ou à
  droite (`camera.lockSwitchPixels`) passe à la cible voisine. Elle est lâchée si elle meurt, s’éloigne trop ou reste
  cachée derrière un mur. Sans cible verrouillable, le clic molette replace la caméra derrière le personnage.
- **Capacités** : clic gauche, clic droit, **E** (compétences des armes puis de la classe, dans cet ordre).
- **Esquive** : **Maj**.
- **Changer d'armes** : **A** en AZERTY (Q en QWERTY).
- **Interagir** : **F** — prendre ou lâcher une échelle (voir `donjon.md`).
- Plafond partout (piste D) : une dalle en haut de chaque case ouverte, 25 cm sous le sol de l’étage du dessus ; rien
  au-dessus des escaliers, échelles et trous, par où l’on voit l’étage du dessus. Pas de plafond dans l’atelier (vue de dessus).
- Déplacement ZQSD relatif à la caméra.
- Réglages : `camera.*` et `player.*` dans `game.json`.
