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

```powershell
cargo build --release -p veloren-voxygen
```
```sh
icebreaker-serveur/paquet-client.sh        # Git Bash → target/Icebreaker-client.zip (~400 Mo)
```

Le lancer dans un terminal Git Bash ouvert à la racine du dépôt, pas par double-clic. La compression dure
2 à 3 minutes sans rien afficher : attendre le message « zip prêt ».

Le zip contient le jeu, ses assets et `Icebreaker.bat`. Le joueur dézippe, lance `Icebreaker.bat`, clique
**En ligne** (seul bouton du menu) : il rejoint le VPS, dont l'adresse est dans `assets/icebreaker/lieux.ron`.

Client et serveur doivent venir du **même commit** : si le protocole a changé, redéployer le serveur (§ 4).

## 4. Déployer le serveur sur le VPS

VPS : `root@116.202.107.125`, ARM64 Debian 12. Le binaire se compile dans WSL (`Ubuntu-22.04`, clone `/root/veloren`).

**a. Compiler dans WSL** (`wsl -d Ubuntu-22.04 -u root`) ; à la première ouverture, WSL demande de créer un
utilisateur : en donner un autre que `root` (ex. `lemai`) avec un mot de passe, une seule fois.

```sh
cd /root/veloren && git fetch && git reset --hard origin/master   # origin = le dossier Windows Veloren-test
export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++ \
       AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar \
       CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-B/usr/libexec/mold"
cargo build --release --target aarch64-unknown-linux-gnu -p veloren-server-cli --features plugins
```

Garder ces variables telles quelles : en changer une recompile tout.

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
