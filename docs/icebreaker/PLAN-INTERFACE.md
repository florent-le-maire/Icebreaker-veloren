# Interface Icebreaker — maquettes, puis refonte (jalon N)

Flo (2026-11-10) : refaire toute l'interface qu'Icebreaker a ajoutée à Veloren, pour qu'elle soit ergonomique, claire
et qu'elle réponde tout de suite. On ne touche pas à l'interface d'origine de Veloren (sac, carte, chat, barre de
vie). La décision `0018` avait gardé les cadres de Veloren et réparé défaut par défaut ; ce jalon reprend l'ensemble.
Décision : `decisions/0019-interface-icebreaker.md` (choisie par Flo le 2026-11-11, `QUESTIONS.md`, Q40).

## Choix de Flo (2026-11-11)

- **Style « proche de Veloren »** : les polices pixel du jeu et son esprit ; on ne change que les couleurs et la mise
  en page, pour que nos fenêtres ne se distinguent pas du reste. La charte du projet Godot (`theme.tres`, bronze et
  parchemin, Cinzel) n'est pas reprise.
- **« Lag » = un clic sans réponse** : au village, on clique un bouton et le panneau ne change qu'un peu plus tard, ou
  le cadre saute. Ce n'est pas une chute d'images par seconde.

## Les maquettes

Canvas privé de Flo sur claude.ai (à partager avec Sayanel depuis le menu Partager de la page) :
<https://claude.ai/artifact/KXR2tEFGXNNdTk8QJMUcdW>

Quatorze planches en 1280 × 720, rangées par moment du jeu, chacune avec une note « aujourd'hui / maquette / ce que ça
demande ». La planche « Village · plan et bâtiment » se joue (bouton Lecture) : les bâtiments se cliquent, les boutons
répondent. En pointillé sur les planches : ce qui reste à Veloren.

| Planche | Remplace aujourd'hui | Demande |
|---|---|---|
| Charte ; composants « HUD de Veloren » et « panneau de bâtiment » | — | — |
| Menu · bloc Icebreaker | titre et bloc « Campagne » (`menu/main/ui/login.rs`) | client |
| Menu · choix de la partie | 4 parties au plus, erreur dans la boîte de connexion | client |
| Menu · lancement et attente | écran vide, nouvel essai toutes les 5 s sans rien dire | client |
| Village · plan et bâtiment | plan U + panneau flottant à gauche (`hud/mod.rs`, « the village plan ») | client et protocole |
| Village · tous les panneaux | panneau générique : lignes de texte et colonne de boutons | client et protocole |
| Village · bandeaux et messages | accueil du fantôme, annonces, refus, mannequin dans le chat ; encadré d'expédition | client, messages typés |
| Donjon · verrouillage et balise | anneau seul ; panneau de couleurs sans pastille ; recharge qui cache la quantité | client |
| Donjon · à terre et sac du mort | texte empilé sur ceux de Veloren ; fouille sans progression | client ; temps restant envoyé par le serveur |
| Donjon · mort ; regarder un allié | deux actions sur une ligne ; allié regardé sans nom | client |
| Retour · bilan de l'expédition | victoire et bilan en lignes de chat | client et protocole |

## Charte

- Polices : `Alkhemikal` (titres), `EnterCommand` (texte) ; chiffres 32, texte 22, légendes 16 à 18 (pour 720 de haut).
- Couleurs : fond `#0C0E10`, bandeau de titre `#1E2426`, case `#14181A`, bordure `#5B666A`, trait discret `#2A3033`,
  texte `#FFFFFF`, texte discret `#9CB3B3` (le `UI_MAIN` de Veloren), bouton rouille `#5A2A1A` (survol `#7A3519`,
  appuyé `#40200F`), vert `#54A100`, rouge `#C9302B`, bleu `#4A9EBF`.
- Un seul accent à nous : l'or de l'anneau de verrouillage (`#FFD166`), pour l'action principale et la sélection.
- Tout se dessine avec des rectangles, des contours, du texte, des cercles et les icônes déjà dans le jeu : aucune
  nouvelle texture.

## Règles

Ergonomie :
- Trois zones fixes qui ne recouvrent ni le groupe, ni le chat, ni la mini-carte : bandeaux en haut au centre,
  fenêtre au centre, messages à droite sous la mini-carte. Une seule fenêtre Icebreaker à la fois.
- Toute fenêtre a un titre, une croix, Échap, et ses touches écrites dans la barre de titre.
- Une action impossible reste visible, grisée, avec sa raison écrite dessous.
- Les chiffres qui comptent (or, temps, places) sont en grand ; la couleur ne porte jamais l'information seule.
- La réponse à une action s'affiche là où on a cliqué (ligne de message du panneau) ; le chat garde la trace.

Fluidité :
- Le serveur envoie l'état d'un panneau quand il change, et juste après une action ; plus de demande chaque seconde
  (`session/mod.rs`, `update_icebreaker_top_view`).
- Le bouton passe à « appuyé » au clic, sans attendre le serveur.
- Tailles fixes : la hauteur d'un panneau ne dépend plus du texte de l'image précédente ; les listes défilent.
- Textes et listes préparés à la réception, pas à chaque image ; aucun parcours de toutes les entités à chaque image
  (verrouillage, allié à regarder) ; `mort.ron` lu une fois.

## Le panneau typé

Un seul gabarit pour tous les bâtiments, et pour la balise. Le serveur n'envoie plus des phrases mais des blocs :
en-tête (icône, titre, une ligne), onglets, chiffres (valeur, légende), jauge, objets (icône, quantité), lignes
(titre, sous-titre, pastille d'état, cadenas, un bouton), actions (libellé, principale ou non, permise ou non),
message (texte, sorte : info, réussite, refus). Le client range chaque bloc toujours au même endroit.

Une ligne de personnage (recrue, fiche du registre) porte l'**icône de sa classe** (Flo, 2026-11-11) : un champ
`icone` par classe dans `assets/icebreaker/classes.ron`, choisi parmi les icônes d'armes déjà dans le jeu
(`assets/voxygen/element/weapons/` : épées croisées pour le guerrier, bâton pour le mage, arc pour l'archer ; hache,
marteau, sceptre, dagues, épée et bouclier pour les classes à venir). Une classe sans icône garde l'icône générique.

## Tâches (worktree `Veloren-interface`, branche `icebreaker-interface`)

- [x] **N.1 Maquettes** — 14 planches sur le canvas ci-dessus, charte, règles, décision `0019` proposée. Le rendu
  n'a pas été relu par l'agent (le canvas ne se capture pas d'ici) : à relire par Flo. Aucun code du jeu touché.
  `91d04b1f1b`
- [x] **N.2 Lecture par l'équipe** — Flo, 2026-11-11 : « la maquette me va bien, tu peux implémenter ça en jeu ».
  `0019` choisie, Q40 tranchée. Sayanel n'a pas encore vu le canvas (à partager).

## Réalisation (accord de Flo le 2026-11-11 pour N.3 à N.9)

Les types partagés, dans `common/src/icebreaker_panneau.rs` (tous sérialisés, comparables) :

- `Panneau { icone, titre, description, onglets: Vec<Onglet>, onglet, chiffres: Vec<Chiffre>, jauge: Option<Jauge>,
  objets: Vec<(ItemKey, u32)>, lignes: Vec<Ligne>, actions: Vec<Action>, colonnes, message: Option<(Content, Sorte)>,
  libre }` ; `Onglet { libelle, lignes }` ; `Chiffre { valeur, legende, ton }` ; `Jauge { legende, texte, part, ton }` ;
  `Ligne { icone, titre, sous_titre, pastille: Option<(Content, Ton)>, cadenas: Option<(bool, Vec<String>)>,
  action: Option<Action>, choisie, estompee }` ; `Action { libelle, commande, principale, permise, couleur }` ;
  `Ton { Normal, Or, Bon, Mauvais, Info, Discret }` ; `Sorte { Info, Reussite, Refus }`.
- `Demande { Ouvrir, Batiment(Vec2<f32>), FermerPanneau, Fermer }` (le client dit ce qu'il fait dans le plan) ;
  `Batiment { nom, emprise, altitude, sorte, etat }` ; `Expedition { destination, depart_dans, membres }`.
- Messages : `ClientGeneral::IcebreakerPlan(Demande)` ; `ServerGeneral::IcebreakerPanel(Panneau)`,
  `IcebreakerBuildings(Vec<Batiment>)`, `IcebreakerExpedition(Option<Expedition>)`.

Les onglets se changent côté client (le serveur envoie les lignes de chaque onglet) : pas d'aller-retour. La réponse
à une action reste un message de chat du serveur ; le client l'affiche aussi dans la ligne de message du panneau
tant que la fenêtre est ouverte, et en message à droite sinon.

- [ ] **N.3 Protocole : panneau typé, envoyé au changement** — `server/src/icebreaker_panneaux.rs` (ressource
  `IcebreakerPanneaux` : qui a quoi d'ouvert, dernier envoi ; constructeurs purs `caisse`, `infirmerie`, `terrain`,
  `chantier`, `entrepot`, `intendance`), `server/src/sys/icebreaker_panneaux.rs` (à chaque tic : reconstruit pour
  les joueurs marqués ou toutes les 0,25 s, n'envoie que ce qui a changé), `icebreaker_registre::panneau_pour`,
  `icebreaker_expedition::panneau_pour`, `icebreaker_balise` (panneau `libre`), `cmd.rs` (une commande marque le
  panneau du joueur), `classes.ron` (`icone`). Client : `request_icebreaker_plan`, `chat_cli` (`!batiments`,
  `!panneau x y`, `!fermer`). Fin : tests des constructeurs (bouton grisé sans or, raison écrite) ; au client
  texte, `/caisse 10` puis le panneau reçu sans le redemander.
- [ ] **N.4 Fenêtre du village** — `voxygen/src/hud/icebreaker.rs` (nouveau : état, textes préparés à la
  réception, dessin de la fenêtre : plan à gauche, panneau typé à droite, tailles fixes, liste qui défile, bouton
  « en cours » au clic), `hud/mod.rs` (l'ancien dessin part), `session/mod.rs` (plus de demande chaque seconde ni de
  calcul du plan à chaque image ; survol et clic dans le HUD). Fin : tests de mise en page ; capture en jeu.
- [ ] **N.5 Bandeaux et messages** — bandeau du fantôme, bandeau d'expédition (envoyé au changement, décompte côté
  client), messages à droite (annonces, refus, infos) quand la fenêtre est fermée.
- [ ] **N.6 Menu** — bloc Icebreaker, liste des parties qui défile, erreur sous le champ, étapes du lancement.
- [ ] **N.7 Donjon** — rappel des touches au verrouillage, couleurs de la balise en pastilles, quantité et recharge
  ensemble, bloc « à terre », marqueur d'un allié à terre, écran de mort, bandeau de l'allié regardé.
- [ ] **N.8 Bilan de retour** — fenêtre au retour au village (créatures, coffres, or, ce que chacun rapporte).
- [ ] **N.9 Coûts par image restants** — cibles du verrouillage, `mort.ron`, alliés à regarder.
