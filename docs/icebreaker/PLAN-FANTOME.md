# Jalon FT — Le fantôme du village a un corps

*Découpage validé par Flo le 2026-11-16 ; FT.1 à FT.5 codés le même jour.* Worktree `Veloren-fantome`, branche
`icebreaker-fantome`. Décision : `decisions/0021-fantome-avec-un-corps.md`.

## Pourquoi

Au village, un joueur sans personnage était une caméra libre invisible. On veut un fantôme bleu un peu transparent,
de la taille d'un humain, que les autres voient, qui marche et plane (pas de vol) et qui interagit avec les PNJ.
L'expédition ne change pas.

## Règles

| Sujet | Règle |
|---|---|
| Où | Au village seulement (registre actif). En donjon, le mort regarde un allié comme avant. |
| Apparence | Drapé bleu, sans jambes, deux yeux, environ 2 m ; on voit le décor au travers (tramage). |
| Déplacement | Glisse en flottant, saut haut, chute lente et plafonnée, pas de vol. |
| Ce qu'il peut | Approcher et utiliser PNJ et animaux comme un joueur ; engager ou jouer à la charrette. |
| Ce qu'il ne peut pas | Attaquer, porter, déposer, partir en expédition (refus inchangés). Rien ne le blesse. |
| Sauvegarde | Rien : le fantôme n'existe que le temps d'être en jeu. |
| Réglages | `assets/icebreaker/fantome.ron` (opacité, gravité, chute maximale, saut, accélération). |

## Comment c'est fait

- **Le corps** : `object::Body::IcebreakerFantome`, un modèle et un os. Le `.vox` est écrit par
  `cargo run -p veloren-common --example icebreaker_fantome_vox` (relancer écrase une retouche faite à la main).
  L'animation d'attente des objets le fait flotter et onduler.
- **La transparence** : une opacité dans les données de la figure, un tramage 4x4 dans `figure-frag.glsl`.
- **Planer** : `common/src/icebreaker_fantome.rs`, appelé par le pas de physique et par les règles de déplacement.
- **L'entrée** : `PresenceKind::IcebreakerFantome` ; `server/src/icebreaker_fantome.rs` pose le corps quand un joueur
  demande à entrer en spectateur et que le registre est actif ; le client change sa présence et sa caméra à la
  réponse du serveur.

## Tâches

- [x] **FT.1 Le modèle et le corps** — corps de la famille « objet » (et non petit bipède : note 0021), modèle écrit
  par programme, flotte et ondule. Vu au lieu d'entraînement. — 9203f7e395
- [x] **FT.2 Transparence** — tramage sur le seul fantôme, opacité dans `fantome.ron`. Vu en jeu : le décor se voit
  au travers. — f7b307b2b6
- [x] **FT.3 Planer** — gravité réduite, chute plafonnée, saut, accélération. Test ciblé vert ; chute lente vue après
  `/jump`. Marche et saut au clavier pas essayés. — f7b307b2b6
- [x] **FT.4 Entrer au village en fantôme avec un corps** — présence `IcebreakerFantome`, rien en base, `is_ghost`
  couvre les deux sortes. Vu en jeu : arrivée, bandeau, un second fantôme visible. Le pseudo au-dessus du fantôme
  n'apparaît pas dans les captures. — 528e55abe4
- [x] **FT.5 Fantôme ↔ personnage** — engager puis rendre ramène au fantôme (vu en jeu) ; pas de barre de vie ni de
  compétences pour le fantôme. Mort et retour d'expédition pas essayés. — 528e55abe4
- [ ] **FT.6 Interactions et docs** — l'invite d'interaction d'un animal s'affiche pour le fantôme (vu) ; reste à
  essayer à la main : parler à un PNJ, vérifier qu'il ne peut ni attaquer ni ramasser ; afficher le pseudo
  au-dessus du fantôme ; une capture pour la fiche `charrette` du wiki.
