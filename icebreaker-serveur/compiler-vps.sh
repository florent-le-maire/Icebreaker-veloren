#!/bin/sh
# Icebreaker : compile le serveur pour le VPS (ARM64) dans WSL, depuis le master du dossier Windows.
#
#   wsl -d Ubuntu-22.04 -u root -e sh icebreaker-serveur/compiler-vps.sh      (PowerShell, racine du dépôt)
#
# Puis icebreaker-serveur/deployer.sh root@<ip> (Git Bash). Le clone /root/veloren a pour origin le dossier
# Windows Veloren-test : il reprend son master. Garder les variables telles quelles, en changer une recompile tout.
set -e
export PATH="/root/.cargo/bin:$PATH"
export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++
export AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-B/usr/libexec/mold"
cd /root/veloren
git fetch -q
git reset -q --hard origin/master
echo "Icebreaker : compilation de $(git log --oneline -1)"
cargo build --release --target aarch64-unknown-linux-gnu -p veloren-server-cli --features plugins
echo "Icebreaker : serveur prêt, lancer icebreaker-serveur/deployer.sh root@<ip> depuis Git Bash"
