#!/bin/sh
# Icebreaker : lance un serveur pour un lieu de la campagne, sous Linux (serveur de test, PLAN.md S.1).
# Même rôle que icebreaker.ps1, sans le mod ni les options du donjon généré.
#
#   ./icebreaker.sh village               (port 14100 ; ICEBREAKER_SITE : Town, CliffTown, DesertCity,
#                                          SavannahTown ou CoastalTown, Town par défaut)
#   ./icebreaker.sh donjon                (port 14110 ; ICEBREAKER_SITE : Gnarling par défaut, ou Genere)
#
# Réglages par variables d'environnement : ICEBREAKER_SITE, ICEBREAKER_PORT, ICEBREAKER_SEED,
# ICEBREAKER_LG (6), ICEBREAKER_DAY_LENGTH (48 minutes réelles), ICEBREAKER_TOWN_SIZE (1.0),
# ICEBREAKER_SERVER (binaire, par défaut target/release/veloren-server-cli).
#
# Chaque lieu génère son propre petit monde à chaque lancement ; les personnages sont communs à tous les
# lieux et vivent dans userdata-icebreaker/campagne.
set -e

lieu="$1"
case "$lieu" in
    village) port_lieu=14100; site_lieu=Town ;;
    donjon) port_lieu=14110; site_lieu=Gnarling ;;
    *) echo "usage : $0 village|donjon" >&2; exit 1 ;;
esac

repo=$(cd "$(dirname "$0")" && pwd)
port=${ICEBREAKER_PORT:-$port_lieu}
seed=${ICEBREAKER_SEED:-$(awk 'BEGIN { srand(); print int(rand() * 1000000) }')}
lg=${ICEBREAKER_LG:-6}
server=${ICEBREAKER_SERVER:-$repo/target/release/veloren-server-cli}

root="$repo/userdata-icebreaker"
ud="$root/$lieu"

# Monde neuf à chaque lancement ; les personnages restent dans la campagne.
rm -rf "$ud/server/rtsim"
mkdir -p "$ud/server/server_config" "$ud/server-cli" "$root/campagne"

# Les champs absents prennent leur valeur par défaut.
cat > "$ud/server/server_config/settings.ron" <<EOF
(
    gameserver_protocols: [Tcp(address: "0.0.0.0:$port")],
    auth_server_address: None,
    query_address: None,
    world_seed: $seed,
    server_name: "Icebreaker - $lieu",
    day_length: ${ICEBREAKER_DAY_LENGTH:-48.0},
    map_file: Some(Generate((x_lg: $lg, y_lg: $lg))),
)
EOF
cat > "$ud/server-cli/settings.ron" <<EOF
(
    web_address: "127.0.0.1:$((port + 1))",
)
EOF

export VELOREN_USERDATA="$ud"
export ICEBREAKER_CAMPAIGN_DIR="$root/campagne"
export ICEBREAKER_INSTANCE_SITE="${ICEBREAKER_SITE:-$site_lieu}"
export ICEBREAKER_TOWN_SIZE="${ICEBREAKER_TOWN_SIZE:-1.0}"
# Charrette et registre commun (jalon R) : on arrive en fantôme au village.
if [ "$lieu" = village ]; then export ICEBREAKER_REGISTRE=1; else unset ICEBREAKER_REGISTRE; fi
echo "Icebreaker : $lieu ($ICEBREAKER_INSTANCE_SITE), graine $seed, port $port"
cd "$repo"
# Sans terminal (service systemd), le serveur n'écoute pas l'entrée standard.
if [ -t 0 ]; then exec "$server" --no-auth; else exec "$server" --no-auth --non-interactive; fi
