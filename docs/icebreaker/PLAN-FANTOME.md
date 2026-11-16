# Jalon FT — Le fantôme du village a un corps

*Découpage proposé le 2026-11-16, à valider par Flo.* Worktree `Veloren-fantome`, branche `icebreaker-fantome`.
Décision : `decisions/0021-fantome-avec-un-corps.md`.

## Pourquoi

Au village, un joueur sans personnage est une caméra libre invisible. On veut un fantôme bleu un peu transparent,
de la taille d'un humain, que les autres voient, qui marche et plane (pas de vol) et qui interagit avec les PNJ.
L'expédition ne change pas.

## Règles

| Sujet | Règle |
|---|---|
| Où | Au village seulement (registre actif). En donjon, le mort regarde un allié comme aujourd'hui. |
| Apparence | Drapé bleu pâle, sans jambes, deux yeux, environ 2 m ; on voit le décor au travers (tramage). |
| Déplacement | Marche, saut haut, chute lente et plafonnée, pas de dégâts de chute, pas de vol. |
| Ce qu'il peut | Approcher et utiliser PNJ et bâtiments comme un joueur ; engager ou jouer à la charrette. |
| Ce qu'il ne peut pas | Attaquer, porter, déposer, partir en expédition (refus inchangés). Invulnérable. |
| Sauvegarde | Rien : le fantôme n'existe que le temps d'être en jeu. |
| Réglages | `assets/icebreaker/fantome.ron` (opacité, gravité, chute maximale, saut, vitesse). |

## Tâches

- [ ] **FT.1 Le modèle et l'espèce** — espèce `Fantome` dans `biped_small` (taille, masse, nom fr/en, manifestes),
  `.vox` écrits par un petit programme gardé dans le dépôt. Fin : `/spawn` au lieu d'entraînement le fait
  apparaître, il marche sans erreur au journal ; une capture.
- [ ] **FT.2 Transparence** — drapeau « fantôme » dans les données de la figure, tramage en damier dans
  `figure-frag.glsl`, opacité lue dans `fantome.ron`. Fin : capture où le décor se voit au travers ; les autres
  créatures sont inchangées.
- [ ] **FT.3 Planer** — pour ce corps : gravité réduite, chute plafonnée, saut plus haut, pas de dégâts de chute,
  valeurs dans `fantome.ron`. Fin : test ciblé sur la vitesse de chute ; essai avec `/body` en admin.
- [ ] **FT.4 Entrer au village en fantôme avec un corps** — `PresenceKind::IcebreakerFantome` ; avec le registre,
  l'entrée en spectateur pose le corps et ses composants sans base de données ; `is_ghost` couvre les deux sortes ;
  rien n'est sauvé à la sortie ; invulnérable, sans sac, sans attaque. Fin : tests des panneaux verts, client texte
  (`!fantome` puis `!voisins` vu par un second joueur), vrai client sans clic jusqu'à la place.
- [ ] **FT.5 Fantôme ↔ personnage** — engager, jouer, rendre, retour d'expédition et mort ramènent au bon état ;
  interface allégée (pas de vie ni de compétences, bandeau du fantôme, point du joueur sur le plan). Fin : la
  suite « fantôme → engager → rendre → fantôme » au client texte et en captures.
- [ ] **FT.6 Interactions et docs** — touche d'interaction sur PNJ et bâtiments comme un joueur, refus en place
  pour ce qui demande un personnage ; fiches wiki `charrette` et `mort`, ligne du jalon dans `README.md`. Fin :
  capture d'un panneau ouvert en s'approchant à pied.
