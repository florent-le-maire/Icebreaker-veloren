# Game design — Menace et intelligence du donjon

*Proposition du 2026-10-21, à valider par l'équipe.* Rien de ce fichier n'est encore codé. Quand une partie est
faite, ses règles chiffrées passent dans [`creatures.md`](creatures.md) et la proposition est marquée « faite » ici.
Index des règles : [`docs/GAME_DESIGN.md`](../GAME_DESIGN.md). Tâches : [`piste-H-habitants.md`](../roadmap/piste-H-habitants.md).

## Pourquoi

Aujourd'hui on avance de salle en salle et on tue tout. Chaque salle est un combat isolé : ce qu'on a fait avant
ne change rien à la suivante. On veut que le joueur soit **fébrile**, qu'il **hésite avant d'ouvrir une porte**,
et que « tout tuer » ne soit plus le seul plan, ni toujours le meilleur.

Ce qu'on vise : le donjon se comporte **comme un tout qui se défend**. Ses habitants se parlent, se souviennent,
se préparent. Ce qu'on fait dans une salle a des conséquences trois salles plus loin.

## Principes

1. **Les créatures ne trichent pas.** Elles ne savent que ce qu'elles ont vu ou entendu, ou ce qu'un autre habitant
   est venu leur dire. Pas de créature qui « sait » où sont les joueurs sans raison : c'est ce qui rend la
   discrétion possible et l'échec juste.
2. **Toujours un signe avant la conséquence.** Chaque réaction du donjon s'annonce : un cri au loin, un guetteur
   qui détale, des torches qui s'allument, une porte qu'on entend claquer. Le joueur doit pouvoir se dire « j'aurais
   dû l'abattre » plutôt que « c'est injuste ».
3. **Un choix, pas une punition.** Entrer en douce et entrer en force sont deux façons de jouer valables. Le bruit
   rend le donjon plus dur, pas impossible ; la discrétion rapporte quelque chose (surprise, salles endormies).
4. **Lisible en coop.** Les réactions doivent se raconter à voix haute entre amis (« il est parti par là ! »,
   « j'entends ronfler derrière »). Un comportement que personne ne remarque ne sert à rien.
5. **Réglable et testable.** Comme le reste de la piste H : règles en C# pur (`src/Gameplay/Creatures`), valeurs en
   données (`creatures.json`, `game.json`), une salle de test par comportement.

**Anti-buts** : des ennemis qui devinent tout (pénible), une jauge qui oblige à tout faire en silence (frustrant), un
donjon qui punit la lenteur au point d'interdire l'exploration, des réactions invisibles (inutiles).

## Le socle : la mémoire du donjon

Presque toutes les idées ci-dessous reposent sur une même brique, à faire en premier : **ce que les habitants
savent**, partagé à l'échelle du donjon (classe C# pure, côté serveur, sans Godot).

- **Les intrus signalés** : où on les a vus, quand, combien. Une entrée n'est ajoutée que par un habitant qui les a
  perçus (vue, ouïe, coup reçu) ; elle vieillit et s'efface.
- **Les événements** : un cri d'alarme, un cadavre d'habitant découvert, la mort du chef, un combat entendu.
- **Le niveau d'alerte** (voir plus bas), calculé à partir de ces événements.

L'information ne voyage pas toute seule : elle passe par les **cris** (portée déjà réglée : 24 m) et par les
**messagers** (le guetteur qui court prévenir). Une salle que personne n'a prévenue reste calme, même si le donjon
est en alerte ailleurs. *(Mon choix pour rester simple : un niveau d'alerte unique par donjon, mais chaque créature
ne réagit qu'aux intrus qu'elle connaît. À revoir si ça se révèle trop grossier.)*

## Les idées, par palier

### Palier 1 — à faire maintenant (suite de la piste H)

Ces trois idées prolongent directement ce qui existe (bruit, appel à l'aide, guetteur malin) et changent tout de
suite la façon de jouer.

#### 1. Le niveau d'alerte

Quatre niveaux pour tout le donjon :

| Niveau | Comment on y arrive (exemples) | Ce qui change |
|---|---|---|
| **Calme** | au départ | rondes normales, dormeurs endormis, surprise possible partout |
| **Inquiet** | combat entendu au loin, cadavre découvert | les créatures au repos tournent la tête plus souvent ; les dormeurs ont le sommeil léger (ouïe moins réduite) |
| **En alerte** | cri d'alarme d'un guetteur, intrus vus par plusieurs habitants | les dormeurs prévenus se lèvent ; les rondes s'allongent vers les portes ; les archers montent sur les hauteurs |
| **Mobilisé** | mort du chef de salle de garde, beaucoup de morts, alerte qui dure | des groupes partent à la recherche des intrus signalés ; le chef se retranche dans son antre avec sa garde |

- Le niveau **monte** par points (chaque événement en rapporte, réglable) et **redescend lentement** si plus rien ne
  se passe, sans jamais revenir à « calme » une fois des cadavres découverts.
- **Signes visibles** du passage d'un niveau à l'autre (principe 2), en attendant le son : cri qui s'affiche au-dessus
  d'une créature lointaine vue à travers une porte, torches qui s'allument dans les salles éteintes (lien avec la
  lumière, `donjon.md`), petit texte d'ambiance en haut de l'écran (« Des cris résonnent au loin… »).
- **Pas de jauge chiffrée à l'écran** : on la devine par les signes. *(Mon choix, voir questions ouvertes.)*

#### 2. Le fuyard prépare l'embuscade

Le guetteur qui s'échappe (H.7) ne se contente plus de prévenir : la salle où il arrive **se prépare**.

- Les créatures prévenues ne foncent pas toutes vers les intrus : elles **se postent** autour de la porte par
  laquelle les intrus arriveront (brutes de chaque côté, dos au mur, hors de vue depuis l'entrée ; archers au fond
  ou en hauteur), et attendent.
- Elles attendent un temps réglable (par exemple 40 s), puis, si personne ne vient, partent voir là où le guetteur a
  vu les intrus.
- Un intrus qui passe la porte déclenche l'embuscade : toutes attaquent ensemble, avec le bonus de surprise inversé
  (le joueur n'a pas vu venir ; à définir : dégâts, étourdissement bref, ou juste le nombre).
- **Signe** : le joueur a vu le guetteur partir, et derrière la porte c'est **trop calme** (aucune ronde visible, voir
  l'idée 3). Les joueurs malins contournent, attendent ou envoient quelqu'un ouvrir pendant que les autres sont prêts.

#### 3. Lire le donjon avant d'entrer (indices)

Devant une porte ou à l'entrée d'un couloir, des **indices tirés des vrais habitants** de la salle d'à côté :

| Ce qu'il y a derrière | Indice (visuel d'abord, sonore plus tard) |
|---|---|
| Créatures endormies | « z » ou ronflement qui filtre sous la porte |
| Brutes éveillées | grognements, ombres qui passent sous la porte |
| Cuisine | fumée, lueur de feu |
| Nid | ossements rongés devant la porte |
| Salle en embuscade (idée 2) | rien du tout : un silence anormal |
| Salle vide | rien non plus : au joueur de deviner |

- Un indice se perçoit à quelques mètres de la porte (réglable). Il **ne ment jamais** (les pièges viendront plus tard,
  palier 3) : il est seulement incomplet, et le silence est ambigu.
- On peut noter les indices sur la carte d'exploration (piste C) : *à discuter*.

### Palier 2 — juste après, une fois le socle validé

4. **Les cadavres parlent.** Une ronde qui tombe sur un habitant mort fait monter l'alerte et va enquêter autour.
   Plus tard : traîner un corps dans un coin pour qu'on ne le trouve pas.
5. **Le noir protège aussi les joueurs.** Les créatures voient moins loin dans le noir (sauf les espèces nocturnes) ;
   une torche allumée se voit de loin, une torche levée encore plus. Lien avec la question ouverte de la lumière
   (`donjon.md` : « les créatures voient aussi bien dans le noir »). Éteindre sa torche devient un vrai choix.
6. **Le moral au-delà du chef.** Un groupe qui perd la moitié des siens peut fuir, se replier vers une salle plus sûre
   ou se rendre (les fragiles d'abord). Le combat ne se termine plus forcément par « tous morts ».
7. **Le repli organisé.** En alerte « mobilisé », les survivants se regroupent dans la salle du chef et ferment les
   portes ; on affronte alors un dernier carré plutôt que des créatures éparpillées.

### Palier 3 — demande d'autres briques (son, nouvelles créatures, temps du monde)

8. **Le traqueur.** Une créature qui suit le groupe à travers le donjon sans jamais attaquer de face, laisse des
   traces, et frappe le traînard isolé. Pousse à rester groupé. Il faut d'abord du son et une nouvelle espèce.
9. **Le faux calme.** Coffre qui mord, cadavre de héros qui sert d'appât, faux indice. Seulement une fois que les
   indices vrais (idée 3) sont bien compris, sinon on perd la confiance du joueur.
10. **Factions rivales.** Deux peuples dans le même donjon (gobelins contre araignées) qui se détestent : on peut les
    monter les uns contre les autres, ou s'allier à l'un. Demande une deuxième famille de créatures.
11. **Le donjon vit avec l'horloge du monde.** Créatures qui dorment le jour, troupe de chasse qui rentre « au soir »,
    repas à la cuisine. Plus on traîne, plus ça bouge. Demande l'horloge du monde (phase 4).
12. **Le retour n'est pas gratuit.** Les salles nettoyées se repeuplent : charognards sur les cadavres, autres
    créatures qui prennent la place libre (rejoint « le donjon garde des traces » de la vision, phase 2).
13. **Parler avec les habitants.** Négocier un passage, racheter un prisonnier, payer un tribut. Lié aux factions du
    monde (phase 5).

## Ordre conseillé

Socle (mémoire du donjon) → niveau d'alerte → embuscade du fuyard → indices → session de test. Puis palier 2 selon
ce que la session aura montré. Le découpage en tâches est proposé dans
[`piste-H-habitants.md`](../roadmap/piste-H-habitants.md) (H.13 à H.18).

## Ce qui manque au jeu pour certaines idées

- **Le son** : beaucoup de signes seraient bien meilleurs entendus (cris lointains, ronflements, portes). En
  attendant, tout passe par des signes visuels et du texte.
- **Une marche discrète** pour les joueurs : aujourd'hui on ne peut pas avancer à pas de loup ; la discrétion repose
  sur le fait de ne pas être vu. À proposer si le niveau d'alerte rend la discrétion intéressante.
- **Des portes qu'on ouvre et ferme** côté créatures (repli, barricade) : à vérifier avec la phase 2 (portes et clés).
