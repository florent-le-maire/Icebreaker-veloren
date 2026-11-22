---
name: piece-vivante
description: Créer ou refaire une pièce du donjon Icebreaker qui a de la vie (cuisine, forge, crypte, dortoir…) — nos propres modèles voxel générés par code (crate icebreaker-modeles), branchés comme sprites de Veloren, puis la salle meublée serrée et vérifiée dans l'atelier et le vrai client. À utiliser quand l'équipe trouve une salle vide ou demande une nouvelle pièce avec des objets que Veloren n'a pas.
---

# Une pièce vivante pour le donjon

Recette suivie pour la cuisine (F.23 à F.26, `docs/icebreaker/PLAN-PLAYTEST.md`, 2026-11-21), que Flo a jugée
réussie. Elle vaut pour n'importe quelle pièce : forge, crypte, dortoir, bibliothèque, armurerie… Exemple complet à
relire avant de commencer : `icebreaker-modeles/src/cuisine.rs` et `assets/icebreaker/donjons/salles/cuisine.txt`.

Le constat de départ : une case fait 7 blocs et Veloren a peu d'objets de détail ; une salle meublée avec ses seuls
sprites paraît vide. Ce qui donne la vie : **nos modèles** (détails à l'échelle du voxel), **des traces
d'activité** (travail en cours, désordre), **la densité et la hauteur** (des choses qui pendent, des murs pleins),
**une pièce plus petite**.

## 1. Cadrer avec l'équipe (avant de coder)

Poser les questions d'un coup (AskUserQuestion), en recommandant une réponse :
- **L'ambiance** : une image de référence est bienvenue (jeu, photo) ; on en prend l'ambiance, jamais les modèles ni
  le plan d'un jeu existant.
- **Ce qui fait vivre la pièce** (plusieurs choix) : traces d'activité, feu/lumière/fumée, densité et hauteur,
  habitants en situation.
- **La taille** : plus petite et pleine (recommandé), grande avec piliers, ou deux variantes.
- **Le lot** : ~6 modèles avant le premier essai en jeu (recommandé).

Écrire le plan (fichier de plan), puis annoncer les tâches dans `PLAN-PLAYTEST.md` sous un petit paragraphe
(numéros F suivants, `— *en cours : Veloren-<piste>, <date>*`). Travailler dans le worktree de la piste
(`Veloren-donjon`) ; copier `target/` du dossier principal si le worktree est neuf (`robocopy … /E /MT:32 /XD
incremental`, code de sortie 1 = réussi).

## 2. Faire l'inventaire

- `assets/icebreaker/donjons/pieces.ron` : l'idée de la pièce, ses `meubles` (sprites de Veloren qui servent déjà)
  et ses `manques` (modèles à faire, décrits dans `modeles`).
- Chercher ce que Veloren a déjà : `grep -oE "\b(Mot\w*)\s*=" common/src/terrain/sprite/mod.rs` (tonneaux, sacs,
  caisses, étagères, plans de travail `CounterWood*`, bancs `BenchWood2*`, bûches `LogsWood*`…).
- Choisir les 6 modèles du lot : **un point focal** (l'âtre, la forge, l'autel : gros, contre un mur, qui éclaire),
  **deux ou trois traces d'activité** (le travail laissé en plan), **un ou deux objets en hauteur** (pendus au mur,
  au plafond), **un objet à poser sur une table**.

## 3. Écrire les modèles (`icebreaker-modeles`)

Un fichier `icebreaker-modeles/src/<piece>.rs` (copier la forme de `cuisine.rs`), ajouté à `tous()` dans `lib.rs`.
Une fonction par modèle : `Modele::new(id, taille, ancre)`, puis `boite`, `creuser`, `cylindre`, `cylindre_axe`,
`percer`, `ellipsoide`, `ligne`, `mettre`, `voxel` (lire), et `bruit(x, y, z)` pour semer du désordre fixe.

**Échelle et repère**
- 11 voxels = 1 bloc ; un personnage fait ~2 blocs ; une table ou un plan de travail, 1 bloc (11 voxels).
- L'avant du modèle regarde **-y** (comme les meubles de Veloren ; l'exemple `cotes` le vérifie sur un `.vox`).
- Taille en multiples impairs de 11 dans les directions centrées (33, 55) pour tomber juste sur les blocs.
- L'**ancre** (offset du manifeste = son opposé) est le point posé au centre du bas du bloc du sprite :
  - au sol : bloc avant, au milieu → `(largeur/2, 5.5, 0)` ; le modèle s'étend vers l'arrière ;
  - contre un mur : dos du modèle collé au mur → profondeur p, ancre y = `p - 5.5` ;
  - sous le plafond : le haut du modèle au haut du bloc → ancre z = `hauteur - 11`, posé à z 5 ;
  - sur une table : le bloc au-dessus du plan de travail (posé à z 1), ancre z 0 ; ça tombe juste sur les
    `CounterWood*` (hauteur 1,0) mais pas sur les tables rondes (1,273).

**Palette** (`lib.rs`, `Matiere` et `couleur`) : indices 8 à 12 brillants (métal, vernis), 13 à 15 lumineux (braises,
flammes), **16 à 21 interdits** (creux, le test le refuse), nos matières mates à partir de 22. Ajouter une matière =
une entrée dans `Matiere`, ses indices dans `tons()`, ses couleurs dans `couleur()`.

**Ce qui rend vivant** : des choses entamées (pain coupé dont la tranche montre la mie, couperet planté, farine
renversée semée au `bruit`), des taches rares (`bruit % 11 == 0`, pas plus : à `% 6` c'était criard), des braises
clairsemées (`% 3 == 0`) plutôt qu'un tas orange, des intérieurs noircis (`Suie` autour des creux d'un foyer).

**Pièges vus** : les briques ont un joint aux z où `z % 3 == 2` ; un dessus à cette hauteur est tout gris → finir
les dessus à un autre z. Un petit objet de 1 voxel disparaît en jeu : grossir les détails (2 voxels minimum).

**Regarder** : `cargo run -p icebreaker-modeles <scratchpad>/modeles` écrit les `.vox` et une image par modèle
(trois quarts et face) ; la lire avec Read, corriger, relancer. `cargo test -p icebreaker-modeles` (taille, pas
d'indice creux, mêmes octets à chaque lancement, relecture).

## 4. Brancher les sprites dans le jeu

Fichiers de Veloren en CRLF : après une insertion par perl ou sed, renormaliser
(`perl -pi -e 's/(?<!\r)\n/\r\n/' <fichier>`). Chaque ajout porte un commentaire « Icebreaker : … ».
- `common/src/terrain/sprite/mod.rs` : les nouveaux noms `Icebreaker<Nom>` dans `Furniture`, à la suite du bloc
  Icebreaker (`0xD4` à `0xD9` pris par la cuisine ; places libres jusqu'à `0xFF`). Collision dans `solid_height`
  (avant `_ => return None`) : en blocs, au plus 3,0, et **seulement sur le bloc d'ancre** ; ne rien mettre pour ce
  qui pend ou se pose sur une table.
- Un objet qui brûle : `Block::get_glow` (`common/src/terrain/block.rs`, 20 comme `Ember`) et les étincelles dans
  `voxygen/src/scene/terrain/watcher.rs` (`fires.push(pos)` ; éviter `smokers` à l'intérieur).
- `assets/voxygen/voxel/sprite_manifest.ron` : une entrée par sprite en fin de fichier, `model:
  "voxygen.voxel.sprite.icebreaker.<id>"`, l'offset donné par `cargo run -p icebreaker-modeles`, `lod_axes: (0.0,
  0.0, 0.0)`, `custom_indices: { 14: (attr: Fire), 15: (attr: Fire) }` pour les flammes.
- `pieces.ron` : les modèles faits passent dans `meubles` (nom français + sprite), sortent de `manques` et de
  `modeles` (le contrôle refuse un modèle que personne ne demande).
- L'atelier des salles lit le manifeste : les nouveaux objets y sont d'office.

## 5. Meubler la salle

`assets/icebreaker/donjons/salles/<nom>.txt` : en-têtes `@roles`, `@fonction`, `@theme`, puis les `@meuble: <Sprite>
x y z ori [miroir]`, puis le plan par cases (`#` mur, `.` sol, `+` porte).
- **Taille** : plan murs compris ; dedans = (largeur − 2) × 7 blocs. La cuisine : 6×5 cases, 28×21 blocs dedans,
  86 meubles. Compter environ un meuble pour 7 blocs² de sol.
- **Coordonnées** en blocs depuis le coin du plan : dedans de x = 7 à 7 × (largeur − 1) − 1, de même en y ; z 0 au
  sol, 5 sous le plafond.
- **Sens (vérifié en jeu)** : dos au mur du haut (première ligne) **4**, mur est **6**, mur du bas **0**, mur ouest
  **2**. Un modèle large tourne avec : à 4 il reste le long du mur du haut.
- **Interdit** : un meuble dans la case (7×7) devant une porte, dans un mur, au plafond. Trois portes suffisent à une
  petite pièce.
- **Composition** : le point focal au milieu du mur du fond, encadré de rangement (étagères, tonneaux, bois) ; un
  îlot central de deux rangs (`CounterWood*`, bouts en `End` tournés vers l'extérieur) entre deux bancs, chargé des
  objets de table ; un mur latéral de travail ; les coins pour les provisions empilées (`CrateBlock` à z 0 et 1) ;
  des objets pendus au-dessus de chaque zone de travail.
- Écrire le fichier à la main (plus rapide que cliquer), puis contrôler : `cargo test -p veloren-world icebreaker`
  relit toutes les salles.

## 6. Vérifier

**Dans l'atelier** (vue 3D comme le jeu) : `cargo build -p icebreaker-salles --bin atelier-salles`, lancer
`./target/debug/atelier-salles.exe --port 8197` en tâche de fond, Edge sans fenêtre en tâche de fond :
`"/c/Program Files (x86)/Microsoft/Edge/Application/msedge.exe" --headless=new --remote-debugging-port=9433
"--user-data-dir=<scratchpad>/edge" --window-size=1600,950 --enable-unsafe-swiftshader --no-first-run
"http://localhost:8197/?salle=<nom>"`, puis `node capture.mjs 9433 <sortie.png> [js]` (scripts à côté de ce
fichier ; js = par exemple cliquer « Dessus » :
`[...document.querySelectorAll('button')].find(b=>b.textContent.trim()==='Dessus').click()`). Un nouveau port et un
nouveau dossier Edge pour chaque rechargement. Dans la vue, le haut du plan est en bas de l'image. Fermer Edge par
`node fermer.mjs <port>` (pas de taskkill).

**Dans le vrai client** (voir aussi la mémoire « essai client sans les mains ») :
1. `cargo build -p veloren-voxygen` (avec cmake, ninja et python dans le PATH) et `cargo build -p
   veloren-server-cli --features plugins`.
2. Serveur en tâche de fond : `icebreaker.ps1 donjon -Site Genere -Salle <nom> -Port 14170 -Seed 7 -Partie essai<nom>`.
   Le journal donne `viewpoint … corner=(cx, cy, z)` : le coin du plan est à `(cx − 3.5, cy − 3.5)`, donc un bloc
   (x, y) de la salle est en `(cx − 3.5 + x, cy − 3.5 + y)`, sol à z.
3. Admin pour `/goto` : serveur arrêté, `VELOREN_USERDATA=userdata-icebreaker/donjon
   ./target/debug/veloren-server-cli.exe --no-auth admin add Essai admin`, puis relancer.
4. `userdata-essai/voxygen/settings.ron` prêt avant (fenêtre 1600×900, muet, `screenshots_path` dans le
   scratchpad), puis en tâche de fond : `VELOREN_USERDATA=userdata-essai ICEBREAKER_AUTO_PERSONNAGE=1
   ICEBREAKER_AUTO_CAMPAGNE=1 ICEBREAKER_AUTO_ADRESSE=localhost:14170 ICEBREAKER_AUTO_PLAN="lanterne,fermer,/goto x y
   z,fermer,fermer,…" ./target/debug/veloren-voxygen.exe`. **`lanterne` d'abord** : une salle meublée seule n'a pas de
   torches, tout est noir. Deux `fermer` après chaque `/goto` (le premier part parfois avant le chargement). Le
   personnage garde son regard : choisir des points en recul, face au point focal.
5. Attendre le nombre de captures (`until [ $(ls <dossier> | wc -l) -ge N ]; do sleep 5; done` en tâche de fond),
   les lire, arrêter client et serveur (TaskStop), supprimer `userdata-essai` et `userdata-icebreaker` du worktree
   (chemins écrits en entier, pas de variable dans un `rm`). ~45 s avant de reconnecter le même pseudo.

## 7. Committer et rendre compte

Un commit par tâche (générateur ou recettes, sprites, salle), message en français avec ce qui a été vu et ce qui
ne l'a pas été ; cocher les tâches avec les hashes ; ajouter à « À vérifier en jeu » de `PLAN.md` ce qui reste
(la salle dans un donjon généré, les collisions au contact). `git rebase master` puis `git merge --ff-only` depuis
`Veloren-test`. Donner à l'équipe les chemins des captures et proposer le lot suivant.
