#!/bin/sh
# Icebreaker : envoie le serveur sur le VPS de test et relance les lieux (PLAN.md S.2).
#
#   icebreaker-serveur/deployer.sh root@<ip>      (Git Bash, depuis la racine du dépôt)
#
# Le binaire est compilé dans WSL pour le VPS (ARM64) :
#   cargo build --release --target aarch64-unknown-linux-gnu -p veloren-server-cli --features plugins
# avec CC_/CXX_/AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc/g++/ar, et un lien
# /usr/bin/aarch64-linux-gnu-ld.mold vers mold (le gcc croisé d'Ubuntu 22.04 le cherche sous ce nom).
# BINAIRE : son chemin, par défaut target/vps/veloren-server-cli, posé par compiler-vps.sh.
# LIEUX : lieux lancés (« campagne » par défaut, décision 0016 : village et donjons dans un seul serveur) ; leurs ports sont ouverts dans le pare-feu (ufw).
# Sur le VPS : /opt/icebreaker-veloren, utilisateur veloren ; les personnages (userdata-icebreaker) restent.
set -e
hote="$1"
[ -n "$hote" ] || { echo "usage : $0 <utilisateur@hote>" >&2; exit 1; }
binaire=${BINAIRE:-$(cd "$(dirname "$0")/.." && pwd)/target/vps/veloren-server-cli}
lieux=${LIEUX:-campagne}

repo=$(cd "$(dirname "$0")/.." && pwd)
paquet=$(mktemp -d)
trap 'rm -rf "$paquet"' EXIT
cp "$binaire" "$paquet/veloren-server-cli"
cp "$repo/icebreaker.sh" "$repo/icebreaker-serveur/icebreaker-veloren@.service" "$paquet/"
tar -cf "$paquet/assets.tar" -C "$repo" assets/common assets/server assets/world assets/icebreaker
tar -czf "$paquet.tgz" -C "$paquet" .
scp -q "$paquet.tgz" "$hote:/tmp/icebreaker-veloren.tgz"
rm -f "$paquet.tgz"

ssh "$hote" "LIEUX='$lieux' sh -s" <<'DISTANT'
set -e
d=/opt/icebreaker-veloren
id veloren >/dev/null 2>&1 || useradd --system --home-dir $d --shell /usr/sbin/nologin veloren
for l in $LIEUX; do systemctl stop icebreaker-veloren@$l 2>/dev/null || true; done
mkdir -p $d
rm -rf $d/assets
tar -xzf /tmp/icebreaker-veloren.tgz -C $d
tar -xf $d/assets.tar -C $d
rm -f $d/assets.tar /tmp/icebreaker-veloren.tgz
mv $d/icebreaker-veloren@.service /etc/systemd/system/
chmod +x $d/icebreaker.sh $d/veloren-server-cli
chown -R veloren:veloren $d
systemctl daemon-reload
for l in $LIEUX; do
    case $l in campagne|village) p=14100 ;; donjon) p=14110 ;; esac
    ufw allow $p/tcp comment "icebreaker-veloren $l" >/dev/null
    systemctl enable --now icebreaker-veloren@$l 2>/dev/null
    systemctl restart icebreaker-veloren@$l
done
sleep 20
# Icebreaker S.8 : un lieu qui ne tourne pas fait échouer le script (et le job « deployer » de GitHub Actions).
echec=0
for l in $LIEUX; do printf '%s : ' $l; systemctl is-active icebreaker-veloren@$l || echec=1; done
ss -tln | grep -E ':141[0-9]0 ' || true
[ $echec = 0 ] || { journalctl -u "icebreaker-veloren@*" -n 40 --no-pager; exit 1; }
DISTANT
