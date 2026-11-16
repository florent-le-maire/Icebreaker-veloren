# 0021 — Le fantôme du village a un corps

- **Date** : 2026-11-16
- **Statut** : choisi et validé par Flo le 2026-11-16. Remplace, pour le village, « le mode
  spectateur de Veloren comme fantôme » de la décision `0017`.
- **Tâches** : jalon FT, `PLAN-FANTOME.md`

## Problème

Au village, un joueur sans personnage est le spectateur de Veloren : pas de corps, une caméra libre qui vole et
traverse tout, invisible pour les autres. Il ne peut rien approcher « en vrai » (PNJ, bâtiments) et on ne voit pas
qui attend à la charrette.

Flo veut qu'au village le fantôme soit **un personnage qu'on voit** : un fantôme bleu un peu transparent, de la
taille d'un humain, qui marche, **plane** au lieu de voler, plus flottant qu'un joueur normal, et qui peut
interagir avec les PNJ comme un joueur. **En expédition, rien ne change.**

## Options

1. **Garder le spectateur et lui ajouter un corps.** Tout ce qui dit « fantôme = spectateur » reste vrai, mais le
   client suppose partout qu'un spectateur n'a pas de corps (caméra, touches, interface).
2. **Un vrai personnage temporaire.** Le fantôme est une entité jouée comme un personnage (corps, physique,
   contrôles, interactions), jamais sauvegardée. Il faut une nouvelle sorte de présence et revoir chaque endroit
   qui teste « spectateur ».

## Décision

**Option 2** (Flo, 2026-11-16).

- **Présence** : une nouvelle sorte `PresenceKind::IcebreakerFantome`, « contrôle un corps, rien n'est sauvé ».
  Avec le registre actif, la demande d'entrée en spectateur donne ce fantôme sur la place du village. Hors
  registre (modérateur qui observe), le spectateur de Veloren reste tel quel.
- **« Est un fantôme »** (`icebreaker_registre::is_ghost`) vaut pour les deux sortes : les panneaux, la charrette
  et les refus (« Un fantôme ne peut pas faire ça ») gardent leurs règles.
- **Corps** : un nouveau corps `IcebreakerFantome` dans la famille « objet » de Veloren (un modèle, un os), aux
  dimensions d'un humain. Les petits bipèdes tirent leur apparence de pièces d'armure portées : trop lourd pour un
  fantôme sans sac. Pas d'animation de marche : il glisse, flotte au-dessus du sol et ondule. Modèle `.vox` écrit
  par `common/examples/icebreaker_fantome_vox.rs`, puis retouchable dans MagicaVoxel : drapé bleu pâle, sans
  jambes, deux yeux.
- **Transparence** : le rendu des créatures est opaque et ne sait pas mélanger. On ajoute un drapeau « fantôme » aux
  données de la figure et le shader des créatures écarte un pixel sur deux en damier (tramage, déjà utilisé
  ailleurs dans ce shader). Ni tri ni nouvelle passe de rendu.
- **Planer** : pour ce corps seulement, gravité réduite, vitesse de chute plafonnée, saut plus haut, pas de dégâts
  de chute. Pas de vol.
- **Réglages** dans `assets/icebreaker/fantome.ron` : opacité, gravité, chute maximale, saut, vitesse.

Choix de l'agent, à corriger si besoin :

- Les autres joueurs voient le fantôme, avec le pseudo du joueur au-dessus.
- Il est invulnérable, n'attaque pas, n'a pas de sac, ne bloque pas les autres joueurs.
- Il reste au village comme aujourd'hui (pas d'expédition pour un fantôme).

## Hors périmètre

- L'expédition : le mort qui regarde un allié, la caméra du donjon et le retour au village ne changent pas.
- Des animations propres au fantôme (ondulation, traînée) : plus tard si le rendu de base ne suffit pas.
- Une vraie transparence mélangée (alpha) : seulement si le tramage est jugé laid en jeu.

## Conséquences

- `PresenceKind` gagne une variante : chaque `match` du serveur, du client et de la persistance doit la traiter
  (le compilateur les liste).
- Le client entre en jeu comme avec un personnage (caméra à la troisième personne, touches), avec une interface
  allégée : pas de barre de vie ni de compétences, le bandeau du fantôme reste.
- Le nouveau corps touche `common/src/comp/body/object.rs`, le manifeste des objets et l'animation des objets :
  changements marqués « Icebreaker ».
- Les fiches du wiki `charrette` et `mort` sont à mettre à jour.
