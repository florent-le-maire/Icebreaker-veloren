# Icebreaker

Prototype du jeu **Icebreaker**, construit sur une copie modifiée de
[Veloren](https://github.com/veloren/veloren).

Icebreaker est un jeu de donjon en coop et en temps réel : chaque joueur a une classe, on part en expédition depuis
un village qui grandit avec nous, on en revient plus riche ou plus abîmé, on améliore le village et on repart plus
fort. Les donjons, grottes et forêts sont générés, leurs habitants ont leur propre logique, et ils ne sont jamais
deux fois les mêmes.

## Ce qu'on a ajouté à Veloren

- Une **campagne** : le village et les expéditions dans un seul serveur, lancée par le bouton « Campagne » du menu.
- Un **village** fixe, avec sa caisse commune, sa charrette et ses bâtiments à améliorer.
- Des **lieux générés** : donjon sous le sol, grottes organiques, forêt à deux étages, avec une difficulté de 1 à 6.
- Des **classes**, jeux d'armes et compétences, un combat létal (à terre, mort, sac du mort) et une IA propre à nos
  espèces d'ennemis.
- Des **ateliers web** pour régler les générateurs (grottes, salles, donjon, forêts), un classeur d'équilibrage et
  un wiki pour l'équipe.

Nos changements sont marqués par un commentaire « Icebreaker : … » pour pouvoir suivre les mises à jour de Veloren ;
le code qui nous est propre est surtout dans `common/src/icebreaker_*.rs`, les crates `icebreaker-*`, le mod
`plugin/icebreaker` et les données `assets/icebreaker/`.

## Jouer

Le client Windows et Mac se télécharge dans les
[Releases](https://github.com/florent-le-maire/Icebreaker-veloren/releases) de ce dépôt.

Pour compiler et lancer soi-même (depuis la racine du dépôt) :

```powershell
cargo build -p veloren-server-cli --features plugins   # le serveur
cargo run --release -p veloren-voxygen                  # le client, puis bouton « Campagne »
```

Le détail (serveur seul, lieux de test, paquet du client, déploiement) est dans
[docs/icebreaker/MAINTENANCE.md](docs/icebreaker/MAINTENANCE.md).

## Documentation

Tout ce qui concerne Icebreaker est dans [docs/icebreaker/](docs/icebreaker/) : commencer par
[README.md](docs/icebreaker/README.md) (fichiers, jalons, conventions), puis [PLAN.md](docs/icebreaker/PLAN.md)
(état et tâches) et [VISION.md](docs/icebreaker/VISION.md) (la vision du jeu).

## Crédits et licence

Icebreaker repose entièrement sur **Veloren**, un RPG d'action-aventure libre en voxels, développé par sa communauté :

- Code source : <https://github.com/veloren/veloren> (miroir GitHub ; le dépôt principal est sur
  [GitLab](https://gitlab.com/veloren/veloren))
- Site : <https://veloren.net>

Un grand merci aux développeurs, artistes, compositeurs, traducteurs et à toute la communauté de Veloren, sans qui
ce prototype n'existerait pas : le moteur, les modèles, les sons et les musiques viennent de leur travail. Les
auteurs des assets repris de Veloren sont listés dans [assets/credits.ron](assets/credits.ron).

Comme Veloren, ce projet est distribué sous la **[GNU General Public License v3.0](LICENSE)** : il reste libre,
modifiable et redistribuable.
