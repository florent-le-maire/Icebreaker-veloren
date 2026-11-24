#!/bin/sh
# Icebreaker : zip du client à envoyer aux potes (PLAN.md S.3, S.5), sous Windows (Git Bash) ou sur Mac.
#
#   icebreaker-serveur/paquet-client.sh [zip]      (depuis la racine du dépôt, après
#                                                   cargo build --release -p veloren-voxygen)
#
# Zip par défaut : target/Icebreaker-windows.zip ou target/Icebreaker-mac.zip. Aussi lancé par la Release GitHub
# (.github/workflows/release-client.yml).
# Le zip contient le jeu, ses assets et un lanceur (« Icebreaker.bat » sous Windows, « Icebreaker.command » sur
# Mac). Au menu, seul le bouton « En ligne » (serveur du VPS, adresse dans assets/icebreaker/lieux.ron) est
# proposé : le pote n'a pas de serveur sur son PC. Ses réglages et journaux vont dans le dossier userdata à côté
# du jeu.
set -e
case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*) systeme=windows; binaire=veloren-voxygen.exe ;;
    Darwin) systeme=mac; binaire=veloren-voxygen ;;
    *) echo "paquet-client.sh : Windows (Git Bash) ou Mac seulement" >&2; exit 1 ;;
esac
repo=$(cd "$(dirname "$0")/.." && pwd)
zip=${1:-$repo/target/Icebreaker-$systeme.zip}
case "$zip" in /*|[A-Za-z]:*) ;; *) zip="$PWD/$zip" ;; esac
paquet=$(mktemp -d)
trap 'rm -rf "$paquet"' EXIT
d="$paquet/Icebreaker"
mkdir -p "$d"
echo "Icebreaker : copie du jeu et des assets..."
cp "$repo/target/release/$binaire" "$d/"
tar -cf - -C "$repo" --exclude=assets/test assets | tar -xf - -C "$d"
# Au menu : les lieux distants seulement (les autres lanceraient icebreaker.ps1, absent du zip). Un lieu tient
# sur plusieurs lignes (description) : découpe aux « (nom: », on retire « au_menu: true » des lieux sans adresse.
perl -0pi -e '$_ = join "(nom:", map { /adresse:/ ? $_ : s/,\s*au_menu: true//r } split /\(nom:/, $_, -1' \
    "$d/assets/icebreaker/lieux.ron"
grep -q 'adresse: Some' "$d/assets/icebreaker/lieux.ron" || { echo "pas de lieu distant dans lieux.ron" >&2; exit 1; }
rm -f "$zip"
# Lanceur : réglages et journaux à côté du jeu (sinon le jeu les cherche dans le dossier où il a été compilé).
if [ "$systeme" = windows ]; then
    printf '@echo off\r\ncd /d "%%~dp0"\r\nset "VELOREN_USERDATA=%%~dp0userdata"\r\nset "VELOREN_ASSETS=%%~dp0."\r\nstart "" veloren-voxygen.exe\r\n' > "$d/Icebreaker.bat"
    echo "Icebreaker : compression du zip (2 à 3 minutes, rien ne s'affiche)..."
    # Compression par PowerShell (Windows n'a pas toujours zip).
    powershell -NoProfile -Command "Compress-Archive -Path '$(cygpath -w "$d")' -DestinationPath '$(cygpath -w "$zip")'"
else
    printf '#!/bin/sh\ncd "$(dirname "$0")"\nexport VELOREN_USERDATA="$PWD/userdata" VELOREN_ASSETS="$PWD"\nexec ./veloren-voxygen\n' > "$d/Icebreaker.command"
    chmod +x "$d/Icebreaker.command" "$d/$binaire"
    echo "Icebreaker : compression du zip..."
    # ditto garde les droits d'exécution du jeu et du lanceur.
    ditto -c -k --keepParent "$d" "$zip"
fi
ls -la "$zip"
echo "Icebreaker : zip prêt, $zip"
