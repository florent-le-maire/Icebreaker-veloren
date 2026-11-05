#!/bin/sh
# Icebreaker : zip du client Windows à envoyer aux potes (PLAN.md S.3).
#
#   icebreaker-serveur/paquet-client.sh [zip]      (Git Bash, depuis la racine du dépôt, après
#                                                   cargo build --release -p veloren-voxygen)
#
# Le zip contient le jeu (veloren-voxygen.exe), ses assets et un lanceur « Icebreaker.bat ». Au menu, seul le
# bouton « En ligne » (serveur du VPS, adresse dans assets/icebreaker/lieux.ron) est proposé : le pote n'a pas
# de serveur sur son PC. Ses réglages et journaux vont dans le dossier userdata à côté du jeu.
set -e
repo=$(cd "$(dirname "$0")/.." && pwd)
zip=${1:-$repo/target/Icebreaker-client.zip}
paquet=$(mktemp -d)
trap 'rm -rf "$paquet"' EXIT
d="$paquet/Icebreaker"
mkdir -p "$d"
cp "$repo/target/release/veloren-voxygen.exe" "$d/"
tar -cf - -C "$repo" --exclude=assets/test assets | tar -xf - -C "$d"
# Au menu : les lieux distants seulement (les autres lanceraient icebreaker.ps1, absent du zip).
perl -i -pe 's/, au_menu: true\)/)/ unless /adresse/ || /^\s*\/\//' "$d/assets/icebreaker/lieux.ron"
grep -q 'adresse: Some' "$d/assets/icebreaker/lieux.ron" || { echo "pas de lieu distant dans lieux.ron" >&2; exit 1; }
# Réglages et journaux à côté du jeu (sinon le jeu les cherche dans le dossier où il a été compilé).
printf '@echo off\r\ncd /d "%%~dp0"\r\nset "VELOREN_USERDATA=%%~dp0userdata"\r\nset "VELOREN_ASSETS=%%~dp0."\r\nstart "" veloren-voxygen.exe\r\n' > "$d/Icebreaker.bat"
rm -f "$zip"
# Compression par PowerShell (Windows n'a pas toujours zip).
powershell -NoProfile -Command "Compress-Archive -Path '$(cygpath -w "$d")' -DestinationPath '$(cygpath -w "$zip")'"
ls -la "$zip"
