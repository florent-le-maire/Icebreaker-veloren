#!/bin/sh
# Icebreaker : lance un serveur pour un lieu de la campagne, sous Linux (serveur de test, PLAN.md S.1).
# Même rôle que icebreaker.ps1, sans le mod ni les options du donjon généré.
#
#   ./icebreaker.sh campagne              (port 14100 ; la campagne en un seul serveur, décision 0016 : le
#                                          village et des emplacements de donjon)
#   ./icebreaker.sh village               (port 14100 ; ICEBREAKER_SITE : Town, CliffTown, DesertCity,
#                                          SavannahTown ou CoastalTown, Town par défaut)
#   ./icebreaker.sh donjon                (port 14110 ; ICEBREAKER_SITE : Gnarling par défaut, ou Genere)
#
# Réglages par variables d'environnement : ICEBREAKER_SITE, ICEBREAKER_PORT, ICEBREAKER_SEED,
# ICEBREAKER_LG (7 pour la campagne, 6 sinon), ICEBREAKER_DAY_LENGTH (48 minutes réelles), ICEBREAKER_TOWN_SIZE (1.0),
# ICEBREAKER_SERVER (binaire, par défaut target/release/veloren-server-cli).
#
# Chaque lieu génère son propre petit monde à chaque lancement ; les personnages sont communs à tous les
# lieux et vivent dans la partie, userdata-icebreaker/parties/<nom>.
set -e

lieu="$1"
case "$lieu" in
    campagne|village) port_lieu=14100; site_lieu=Town ;;
    donjon) port_lieu=14110; site_lieu=Gnarling ;;
    *) echo "usage : $0 campagne|village|donjon" >&2; exit 1 ;;
esac

repo=$(cd "$(dirname "$0")" && pwd)
port=${ICEBREAKER_PORT:-$port_lieu}
seed=${ICEBREAKER_SEED:-$(awk 'BEGIN { srand(); print int(rand() * 1000000) }')}
if [ "$lieu" = campagne ]; then lg_lieu=7; else lg_lieu=6; fi
lg=${ICEBREAKER_LG:-$lg_lieu}
server=${ICEBREAKER_SERVER:-$repo/target/release/veloren-server-cli}

root="$repo/userdata-icebreaker"
# Le monde de la campagne a son dossier à lui, à côté des parties.
if [ "$lieu" = campagne ]; then ud="$root/monde"; else ud="$root/$lieu"; fi

# Icebreaker X.10 : une partie = un dossier de userdata-icebreaker/parties (ICEBREAKER_PARTIE, sinon la
# dernière jouée, sinon « Partie 1 ») ; l'ancienne campagne unique devient « Partie 1 », une seule fois.
parties="$root/parties"
if [ ! -d "$parties" ] && [ -d "$root/campagne" ]; then
    mkdir -p "$parties" && mv "$root/campagne" "$parties/Partie 1"
fi
partie=${ICEBREAKER_PARTIE:-}
if [ -z "$partie" ] && [ -f "$parties/derniere.txt" ]; then partie=$(head -n 1 "$parties/derniere.txt"); fi
partie=${partie:-Partie 1}
campagne="$parties/$partie"

# Monde neuf à chaque lancement ; les personnages restent dans la partie.
rm -rf "$ud/server/rtsim"
mkdir -p "$ud/server/server_config" "$ud/server-cli" "$campagne"
printf '%s' "$partie" > "$parties/derniere.txt"
printf '%s' "$partie" > "$ud/partie.txt"

# Le village (seul ou dans la campagne) garde sa graine pour toute la partie : ses maisons restent à leur place.
graine_village="$campagne/$lieu-graine.txt"
if [ "$lieu" != donjon ] && [ -z "${ICEBREAKER_SEED:-}" ]; then
    if [ -f "$graine_village" ]; then seed=$(head -n 1 "$graine_village" | tr -d "\r ")
    else echo "$seed" > "$graine_village"; fi
fi

# Le terrain du village (graine gardée par la partie) est enregistré dans maps/ au premier lancement, puis relu.
if [ "$lieu" = donjon ]; then carte="Generate((x_lg: $lg, y_lg: $lg))"
else carte="LoadOrGenerate(name: \"$lieu-$seed-lg$lg\", opts: (x_lg: $lg, y_lg: $lg))"; fi
# Les champs absents prennent leur valeur par défaut.
cat > "$ud/server/server_config/settings.ron" <<EOF
(
    gameserver_protocols: [Tcp(address: "0.0.0.0:$port")],
    auth_server_address: None,
    query_address: None,
    world_seed: $seed,
    server_name: "Icebreaker - $lieu",
    day_length: ${ICEBREAKER_DAY_LENGTH:-48.0},
    map_file: Some($carte),
)
EOF
cat > "$ud/server-cli/settings.ron" <<EOF
(
    web_address: "127.0.0.1:$((port + 1))",
)
EOF

# Le serveur cherche assets/ à côté de son binaire : on lui donne la racine du dépôt.
export VELOREN_ASSETS="$repo"
export VELOREN_USERDATA="$ud"
export ICEBREAKER_CAMPAIGN_DIR="$campagne"
export ICEBREAKER_PARTIE="$partie"
export ICEBREAKER_INSTANCE_SITE="${ICEBREAKER_SITE:-$site_lieu}"
export ICEBREAKER_TOWN_SIZE="${ICEBREAKER_TOWN_SIZE:-1.0}"
if [ "$lieu" = campagne ]; then export ICEBREAKER_CAMPAGNE=1; else unset ICEBREAKER_CAMPAGNE; fi
# Charrette et registre commun (jalon R) : on arrive en fantôme au village.
if [ "$lieu" != donjon ]; then export ICEBREAKER_REGISTRE=1; else unset ICEBREAKER_REGISTRE; fi
echo "Icebreaker : $lieu ($ICEBREAKER_INSTANCE_SITE), graine $seed, port $port"
cd "$repo"
# Sans terminal (service systemd), le serveur n'écoute pas l'entrée standard.
if [ -t 0 ]; then exec "$server" --no-auth; else exec "$server" --no-auth --non-interactive; fi
