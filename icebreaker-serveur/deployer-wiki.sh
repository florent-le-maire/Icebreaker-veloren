#!/bin/sh
# Icebreaker : génère le wiki de l'équipe et l'envoie sur le VPS, où Caddy le sert derrière un mot de passe
# (docs/icebreaker/PLAN-WIKI.md, W.7). Le dépôt est privé : le wiki l'est aussi.
#
#   MOT_DE_PASSE=... icebreaker-serveur/deployer-wiki.sh root@<ip>      (Git Bash, depuis la racine du dépôt)
#
# ADRESSE      : ce que Caddy sert ; par défaut http://<ip>:8090. Avec un nom de domaine qui pointe sur le VPS,
#                ADRESSE=wiki.exemple.fr donne du HTTPS automatique (ports 80 et 443).
# UTILISATEUR  : nom demandé par le navigateur (défaut : icebreaker).
# MOT_DE_PASSE : obligatoire la première fois ; haché sur le VPS (`caddy hash-password`), le hachage est gardé
#                dans /etc/caddy/icebreaker-wiki.hash et réutilisé ensuite si la variable est absente.
# Sur le VPS : /opt/icebreaker-wiki (les pages), /etc/caddy/icebreaker-wiki.caddy (le site), importé par
# /etc/caddy/Caddyfile ; Caddy installé avec apt s'il manque ; port ouvert dans ufw.
set -e
hote="$1"
[ -n "$hote" ] || { echo "usage : MOT_DE_PASSE=... $0 <utilisateur@hote>" >&2; exit 1; }
ip=${hote#*@}
adresse=${ADRESSE:-http://$ip:8090}
utilisateur=${UTILISATEUR:-icebreaker}

repo=$(cd "$(dirname "$0")/.." && pwd)
cd "$repo"
cargo run -q -p icebreaker-wiki
paquet=$(mktemp -d)
trap 'rm -rf "$paquet"' EXIT
tar -czf "$paquet/site.tgz" -C "$repo" site
scp -q "$paquet/site.tgz" "$hote:/tmp/icebreaker-wiki.tgz"

ssh "$hote" "ADRESSE='$adresse' UTILISATEUR='$utilisateur' MOT_DE_PASSE='$MOT_DE_PASSE' sh -s" <<'DISTANT'
set -e
d=/opt/icebreaker-wiki
hash_fichier=/etc/caddy/icebreaker-wiki.hash
command -v caddy >/dev/null 2>&1 || { apt-get update -q && apt-get install -y -q caddy; }
mkdir -p "$d.nouveau"
tar -xzf /tmp/icebreaker-wiki.tgz -C "$d.nouveau" --strip-components=1
rm -f /tmp/icebreaker-wiki.tgz
rm -rf "$d"
mv "$d.nouveau" "$d"
chown -R caddy:caddy "$d" 2>/dev/null || true
if [ -n "$MOT_DE_PASSE" ]; then
    caddy hash-password --plaintext "$MOT_DE_PASSE" > "$hash_fichier"
    chmod 600 "$hash_fichier"
fi
[ -s "$hash_fichier" ] || { echo "MOT_DE_PASSE manquant et pas de hachage gardé sur le VPS" >&2; exit 1; }
hash=$(cat "$hash_fichier")
cat > /etc/caddy/icebreaker-wiki.caddy <<FIN
# Icebreaker : le wiki de l'équipe (deployer-wiki.sh).
$ADRESSE {
    root * $d
    basic_auth {
        $UTILISATEUR $hash
    }
    encode gzip
    file_server
}
FIN
grep -q 'import /etc/caddy/icebreaker-wiki.caddy' /etc/caddy/Caddyfile 2>/dev/null \
    || printf '\nimport /etc/caddy/icebreaker-wiki.caddy\n' >> /etc/caddy/Caddyfile
case "$ADRESSE" in
    http://*) reste=${ADRESSE#http://}; port=${reste##*:}
              [ "$port" != "$reste" ] || port=80
              ufw allow "$port/tcp" comment "icebreaker-wiki" >/dev/null 2>&1 || true ;;
    *)        ufw allow 80/tcp comment "icebreaker-wiki" >/dev/null 2>&1 || true
              ufw allow 443/tcp comment "icebreaker-wiki" >/dev/null 2>&1 || true ;;
esac
caddy validate --config /etc/caddy/Caddyfile >/dev/null
systemctl enable --now caddy >/dev/null 2>&1 || true
systemctl reload caddy || systemctl restart caddy
sleep 1
systemctl is-active caddy
echo "wiki servi sur $ADRESSE (utilisateur $UTILISATEUR)"
DISTANT
