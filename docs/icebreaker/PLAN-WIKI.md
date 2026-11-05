# Wiki de l'équipe — site généré depuis les docs (jalon W)

Flo et Sayanel veulent un site façon wiki pour concevoir le jeu : voir ce qui est fini, en cours ou à vérifier en
jeu, et comprendre chaque mécanique à deux niveaux — la règle du jeu pour le joueur, puis « sous le capot » sans
lire le code. Référence d'ambiance : le wiki de Baldur's Gate 3 (thème sombre, barre latérale, fiches avec encart de
chiffres, tableaux de stats). Lecteurs : l'équipe seulement ; le dépôt GitHub est privé, donc pas de GitHub Pages.

## Choix (Flo, 2026-11-05)

- **Une seule source de vérité** : le site est généré depuis `docs/icebreaker/` et `assets/icebreaker/`. On n'écrit
  jamais dans le site ; on écrit dans la doc. La vulgarisation va dans de nouveaux fichiers `docs/icebreaker/wiki/`.
- **Générateur maison en Rust** : crate `icebreaker-wiki/` à la racine, membre du workspace
  (`pulldown-cmark` pour le Markdown, `minijinja` pour les gabarits, `git log` pour les commits). Pas de Node ni de
  Python : l'équipe a déjà la chaîne Rust, et un vrai moteur de gabarits est nécessaire pour les fiches (encarts,
  tableau de bord). Écarté : MkDocs Material / mdBook (tableau de bord et encarts difficiles sans plugins).
- **Hébergement sur le VPS existant** : Caddy sert le dossier généré en HTTPS avec un mot de passe ; un script
  `icebreaker-serveur/deployer-wiki.sh` génère puis envoie le site. Pas de CI : on régénère et déploie à la main,
  comme le serveur. En local : `cargo run -p icebreaker-wiki` depuis la racine, puis ouvrir `site/index.html`.
- **Langue** : français partout.

## Contenu du site

1. **Accueil / tableau de bord** : tableau des jalons (lettre, titre, état, tâches cochées / total, dernier commit),
   compteur « À vérifier en jeu », dernières tâches cochées. Source : `README.md`, `PLAN.md`, `HISTORIQUE.md`.
2. **Fiches mécaniques**, une page par domaine (à terre et mort, combat, classes et compétences, village, âmes,
   donjons et génération, grotte, clés et serrures, campagne et lieux…), à trois étages :
   - *La règle* : texte de vulgarisation (`docs/icebreaker/wiki/<domaine>.md`), encart à droite avec les chiffres
     clés lus dans le `.ron` du domaine (ex. `mort.ron` : 25 %, 30 s, 3 s).
   - *Le détail* : le fichier `design/<domaine>.md` rendu tel quel, repliable.
   - *Sous le capot* : systèmes et fichiers concernés (listés dans l'en-tête de la fiche), réglages complets du
     `.ron` (clé, valeur, commentaire), tâches liées avec hash, résumé du commit et son paragraphe « Testé ».
   En-tête de liaison (TOML) : `design = "combat.md"`, `reglages = ["combat.ron"]`, `taches = ["T", "C.3"]`,
   `systemes = ["server/src/sys/icebreaker_mort.rs"]`.
3. **Avancement** : une page par jalon avec les tâches de `PLAN.md` et `HISTORIQUE.md` et leurs commits ; sections
   « À vérifier en jeu » et « Plus tard ».
4. **Décisions**, **Questions / Idées** : rendu des fichiers existants, avec badges d'état (tranché / ouvert).
5. **Vision** : `VISION.md`, `design/piliers.md`.
6. Recherche plein texte côté client (index JSON, petit script) et navigation latérale par rubrique.

Design : thème sombre (bleu nuit / glace, accent chaud), barre latérale fixe, largeur de lecture ~72 caractères,
encart flottant sur les fiches, tableaux à lignes alternées, badges d'état, hash cliquable vers GitHub. Une feuille
CSS maison, pas de framework ; fonctionne sans JS sauf recherche et repli.

## Tâches (worktree `Veloren-wiki`, branche `icebreaker-wiki`)

- [ ] **W.1 Squelette du générateur** — *en cours : Veloren-wiki, 2026-11-05* — crate `icebreaker-wiki`, lecture de `docs/icebreaker/**.md`, rendu Markdown
  dans un gabarit de base avec barre latérale ; `site/` ignoré par git ; test : chaque fichier produit une page.
- [ ] **W.2 Tâches et commits** — parseur des lignes `- [x] **X.n …** … <hash>` (PLAN, HISTORIQUE, PLAN-*),
  jonction avec `git log` (résumé, corps, paragraphe « Testé ») ; pages Avancement et tableau de bord.
- [ ] **W.3 Réglages** — lecture des `.ron` de `assets/icebreaker/` (clé, valeur, commentaire) → tableau et encart.
- [ ] **W.4 Fiches mécaniques** — gabarit à trois étages, en-tête de liaison, première fiche « À terre et mort »
  (jalon T), puis combat, village, classes, donjons, grotte (vulgarisation à relire par l'équipe).
- [ ] **W.5 Décisions, questions, idées, vision** — rendu et badges d'état.
- [ ] **W.6 Recherche et finitions** — index JSON, script de recherche, revue du CSS sur petit écran.
- [ ] **W.7 Déploiement VPS** — Caddy + mot de passe, `deployer-wiki.sh`, notes dans `README.md` et `CLAUDE.md`.
  Il faut l'adresse du VPS et un mot de passe.

Hors périmètre pour l'instant : accès public, édition dans le navigateur, mise à jour automatique à chaque push,
historique des valeurs des réglages dans le temps (`IDEES.md`).
