+++
ordre = 11
titre = "L'IA des ennemis"
icone = "voxygen/element/ui/map/buttons/character.png"
resume = "Nos six espèces voient, entendent, se préviennent, appellent à l'aide et fuient : elles ne savent que ce qu'elles ont perçu."
reglages = ["ia/profils.ron", "ia/bruits.ron", "ia/scenes.ron"]
taches = ["IA"]
design = ["creatures.md", "menace.md"]
systemes = [
  "icebreaker-ia/src/cerveau.rs",
  "icebreaker-ia/src/perception.rs",
  "icebreaker-ia/src/ouie.rs",
  "icebreaker-ia/src/combat.rs",
  "icebreaker-ia/src/reactions/mod.rs",
  "common/src/icebreaker_ia.rs",
  "server/src/sys/agent/icebreaker_ia.rs",
  "server/src/sys/icebreaker_ia_monde.rs",
  "server/src/sys/icebreaker_rondes.rs",
]
+++

## La règle

Les ennemis des expéditions (humains, orcs, nains, elfes, draugr, danari ; guerrier, archer, mage, chef) **ne
trichent pas** : un ennemi ne sait que ce qu'il a vu, entendu, ou ce qu'un des siens est venu lui crier. Il ne
connaît jamais la position d'un joueur qu'il ne perçoit pas. C'est ce qui rend la discrétion possible.

Chaque ennemi est dans un état, qu'il annonce au-dessus de lui :

| État | Ce qu'il fait | Ce qu'on voit |
|---|---|---|
| Repos | il flâne à son poste, ou fait sa ronde | rien |
| Enquête | il regarde un soupçon, ou va voir un endroit et y fouille | « ? » |
| Combat | il se bat contre celui qu'il perçoit | « ! » |
| Fuite | blessé, il s'éloigne ; il ne revient pas voir | il court |
| Il part prévenir | il court chercher un allié qui ne sait rien | « Je vais chercher les autres ! » |

## Voir

Un ennemi voit dans un **cône devant lui** (140°, 32 blocs), s'il n'y a pas de mur entre les deux. À moins de
3 blocs il sent aussi ce qui est dans son dos. Jusqu'à 18 blocs il reconnaît tout de suite ; au-delà, ou face à un
joueur accroupi, il a d'abord un **soupçon** : il s'arrête et regarde (« ? »). Si on reste en vue un peu plus d'une
seconde, il attaque (« ! ») ; si on disparaît, le soupçon repart de zéro.

Conséquences : on passe dans son dos, on passe pendant qu'il regarde ailleurs, et un mur coupe la poursuite.

## Perdre sa cible

Quand il ne voit plus sa cible, il ne garde que **le dernier endroit où il l'a vue**. Il y court trois secondes,
puis il y enquête : il fouille du regard quelques secondes, et rentre à son poste. Il ne suit personne à travers
un mur.

## Entendre

Un coup qui touche et une mort font du bruit (22 et 28 blocs). Celui qui l'entend **vient voir**, fouille, puis
rentre. En plein combat il garde sa cible, mais retient le bruit : s'il la perd, il va au plus récent de ce qu'il
sait. Un dormeur entend moitié moins loin, et se réveille. Les murs n'arrêtent pas encore le son.

## Se prévenir

Il n'y a pas de guetteur attitré. Quand un ennemi repère un intrus, ou entend tomber un des siens, il **tire sa
réaction** : le plus souvent il se bat ou va voir, parfois il part **prévenir** l'allié non alerté le plus proche.
Arrivé près de lui il crie « Des intrus ! » ; ceux qui l'entendent vont là où les intrus ont été vus, attaquent
s'ils voient quelqu'un, fouillent sinon. Il n'y a jamais deux messagers à la fois, et le chef ne part pas.

On a donc intérêt à abattre celui qui détale.

## Se battre selon la difficulté

Le cran d'habileté d'une difficulté (page « La difficulté ») règle sa façon de se battre :

| Cran | Avant son premier coup | Après chaque attaque | Défense |
|---|---|---|---|
| base (difficultés 1 à 3) | 0,8 s | 1,4 s de temps mort, enchaînements courts | aucune |
| aguerri (4 et 5) | 0,45 s | 0,7 s | pare parfois |
| vétéran (6) | 0,25 s | il enchaîne | pare ou roule souvent |

**Défense** : quand on arme un coup de mêlée contre lui, il tire une fois sa réponse et la fait après son délai de
réaction. Un coup lent est donc esquivé ou paré ; un coup rapide passe.

**Le mage** choisit son sort selon la distance : de loin la boule de feu (et, aux crans hauts, napalm, souffle,
pluie de braises), à mi-distance le lance-flammes, au contact l'onde de feu, la cape de flammes ou la ruée. Lui
foncer dessus appelle l'onde qui repousse.

## Appeler à l'aide, fuir, se disperser

- **À l'aide !** Un archer ou un mage frappé sous 75 % de ses PV crie. Les guerriers et le chef qui l'entendent
  prennent l'agresseur pour cible, même s'ils se battaient contre quelqu'un d'autre. On tue l'archer d'un coup, ou
  loin des guerriers.
- **Fuite.** Un danari sous 30 % de ses PV fuit, et continue de fuir dix secondes sans nous voir avant de se calmer
  plus loin. **Coincé** au fond d'une salle, il se tapit (« Pitié ! » ; le frapper le fait détaler à travers nous)
  ou force le passage (« Poussez-vous ! »).
- **Le chef est tombé.** Sa garde crie « Fuite ! » et se disperse : les guerriers quatre secondes, archers et
  mages huit. Puis ils se reprennent et reviennent voir. C'est un répit, pas une victoire.
- **La rage des orcs.** Sous la moitié de ses PV, un orc crie « RAAAH ! » : il frappe plus fort et plus vite
  pendant douze secondes, et encaisse moins bien. Une seule fois.

## Les rondes

Les humains font des rondes entre les salles. Une ronde est un **ordre** donné à ses membres : un combat ou un
bruit passe avant, puis ils reprennent leur ronde là où ils en étaient.

## L'essayer

Au lieu d'entraînement : `/entrainement ia` fait dire aux ennemis leur état, leurs sorts et leurs défenses ;
`/entrainement scene <nom>` monte une scène (`ecoute`, `massacre`, `duel`, `mage`, `fuyard`, `appel`, `garde`,
`rage`) ; `/entrainement bruit` fait du bruit là où on est ; `/entrainement difficulte <1-6>` règle le cran des
prochains ennemis.

## Pas encore

Les identités des autres races (nains, elfes, draugr : question Q50), la mémoire du lieu et le niveau d'alerte,
les murs qui arrêtent le son, l'embuscade préparée par un fuyard.
