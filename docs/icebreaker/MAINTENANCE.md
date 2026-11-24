# Maintenance

Tout se lance **depuis la racine du dépôt** (le jeu y cherche `assets/`). `cargo` est dans `~/.cargo/bin`.

## 1. Lancer le jeu (en local)

```powershell
cargo build -p veloren-server-cli --features plugins   # le serveur, que le menu lance tout seul
cargo run --release -p veloren-voxygen                  # le client
```

Au menu, bouton **Campagne** : il lance le serveur local (`icebreaker.ps1 campagne`) et s'y connecte.

## 2. Lancer le serveur seul

```powershell
.\icebreaker.ps1 mod                        # une fois, puis à chaque changement du mod (plugin/icebreaker)
.\icebreaker.ps1 campagne                   # port 14100 ; -Partie <nom> pour choisir la partie
```

Lieu seul pour tester : `village`, `donjon -Site Gnarling` ou `-Site Genere` (port 14110).
Côté client : Multijoueur → `localhost:<port>`, n'importe quel pseudo.
Les parties (personnages, caisse…) sont dans `userdata-icebreaker\parties\<nom>`.

Un serveur qui tourne verrouille son `.exe` : le renommer avant de recompiler.

## 3. Créer le client pour les autres joueurs

**Par GitHub (Windows et Mac)** : changer le numéro de `assets/icebreaker/version.ron` (ex. `"0.2.0"`), committer
et pousser sur `master` : GitHub Actions compile et publie la Release `v0.2.0` (rien si elle existe déjà). Pour un
essai sans nouvelle version : onglet Actions → « Release du client » → « Run workflow » donne la pré-release
`dernier-build` (remplacée à chaque lancement). Si un système échoue (souvent : pas de Mac libre chez GitHub), la
Release sort avec l'autre zip ; « Re-run failed jobs » sur le run ajoute ensuite le zip manquant. Chacune porte `Icebreaker-windows.zip` et `Icebreaker-mac.zip` (Apple Silicon), à télécharger
depuis la page Releases du dépôt. Premier build long (sans doute plus d'une heure), plus court ensuite grâce au cache.
Sur Mac, le jeu n'est pas signé : `xattr -dr com.apple.quarantine Icebreaker` dans le Terminal, une fois, puis
double-clic sur `Icebreaker.command`.

**À la main (Windows)** :

```powershell
cargo build --release -p veloren-voxygen
```
```sh
icebreaker-serveur/paquet-client.sh        # Git Bash → target/Icebreaker-windows.zip (~400 Mo)
```

Le lancer dans un terminal Git Bash ouvert à la racine du dépôt, pas par double-clic. La compression dure
2 à 3 minutes sans rien afficher : attendre le message « zip prêt ».

Le zip contient le jeu, ses assets et `Icebreaker.bat`. Le joueur dézippe, lance `Icebreaker.bat`, clique
**En ligne** (seul bouton du menu) : il rejoint le VPS, dont l'adresse est dans `assets/icebreaker/lieux.ron`.

Client et serveur doivent venir du **même commit** : si le protocole a changé, redéployer le serveur (§ 4).

## 4. Déployer le serveur sur le VPS

VPS : `root@116.202.107.125`, ARM64 Debian 12. Le binaire se compile dans WSL (`Ubuntu-22.04`, clone `/root/veloren`).

**a. Compiler dans WSL** (PowerShell, racine du dépôt, après avoir committé sur master) :

```powershell
wsl -d Ubuntu-22.04 -u root -e sh icebreaker-serveur/compiler-vps.sh
```

Le script reprend le master du dossier Windows dans `/root/veloren` et compile en root. Ne pas compiler à la main
dans un shell WSL ouvert avec un autre utilisateur (cargo trop vieux, pas d'accès à `/root/veloren`).

**b. Envoyer et relancer** (Git Bash, sur Windows) :

```sh
icebreaker-serveur/deployer.sh root@116.202.107.125
```

Il envoie le binaire et les assets dans `/opt/icebreaker-veloren`, puis relance le service
`icebreaker-veloren@campagne` (port 14100). Les personnages du VPS sont conservés. Le serveur du VPS tourne
sans le mod.

**Sur le VPS** :

```sh
systemctl status icebreaker-veloren@campagne
journalctl -u icebreaker-veloren@campagne -f     # journal en direct
systemctl restart icebreaker-veloren@campagne
```

Ne pas toucher au service `icebreaker` (node, port 2567), qui tourne aussi sur ce VPS.

**Wiki** : `MOT_DE_PASSE=… icebreaker-serveur/deployer-wiki.sh root@116.202.107.125`.
