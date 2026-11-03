# Icebreaker : lance un serveur pour un lieu de la campagne.
#
#   .\icebreaker.ps1 campagne                (port 14100 ; la campagne en un seul serveur, décision 0016 : le
#                                             village et des emplacements de donjon, assets\icebreaker\campagne.ron)
#   .\icebreaker.ps1 donjon -Site Gnarling   (port 14110)
#   .\icebreaker.ps1 donjon -Site Genere     (port 14110 ; donjon de notre générateur, jalon G ; mission :
#                                             -Taille -Objectif -Region -Saison -Menace -Ennemis -Faction)
#   .\icebreaker.ps1 route                   (port 14120, pas encore fait : P.1)
#   .\icebreaker.ps1 village                 (port 14100 ; -Site Town, CliffTown, DesertCity,
#                                             SavannahTown ou CoastalTown, Town par défaut)
#   .\icebreaker.ps1 mod                     (construit le mod dans assets\plugins ; le serveur
#                                             doit être compilé avec --features plugins)
#
# Chaque lieu génère son propre petit monde à chaque lancement (graine au hasard, ou -Seed).
# Les personnages sont communs à tous les lieux : ils vivent dans userdata-icebreaker\campagne.
# Dans le jeu : Multijoueur, serveur « localhost:<port> », n'importe quel pseudo.
param(
    [Parameter(Mandatory = $true, Position = 0)]
    [ValidateSet("campagne", "village", "donjon", "route", "mod")]
    [string]$Lieu,
    # Site autour duquel le lieu est généré (par défaut : Gnarling au donjon, Town au village).
    [string]$Site,
    # Taille de la ville du village (Town seulement), de 0.03 (hameau) à 1.0 (grande ville).
    [double]$TailleVille = 1.0,
    # Taille du monde (2^Lg tronçons de côté) : 7 par défaut pour la campagne, 6 pour un lieu seul.
    [int]$Lg = 0,
    [int]$Seed = (Get-Random -Maximum 1000000),
    # Durée d'une journée en minutes réelles (1 min réelle = 30 min de carte).
    [double]$DayLength = 48.0,
    # Port du serveur, à la place de celui du lieu (pour lancer deux serveurs du même lieu).
    [int]$Port = 0,
    # Lieu « Genere » : une seule salle à regarder (nom de fichier de assets\icebreaker\donjons\salles).
    [string]$Salle = "",
    # Lieu « Genere » : la mission (assets\icebreaker\donjons\generation.ron ; vide : valeur par défaut).
    [ValidateSet("", "small", "medium", "large")] [string]$Taille = "",
    [ValidateSet("", "killChief", "rescue", "loot", "clear", "explore", "bossLair")] [string]$Objectif = "",
    [ValidateSet("", "forest", "mountain", "swamp")] [string]$Region = "",
    [ValidateSet("", "spring", "summer", "autumn", "winter")] [string]$Saison = "",
    [ValidateSet("", "1", "2", "3", "4", "5")] [string]$Menace = "",
    [ValidateSet("", "low", "medium", "high")] [string]$Ennemis = "",
    [ValidateSet("", "gnarling", "adlet", "cultist")] [string]$Faction = ""
)

$repo = $PSScriptRoot

# Le mod (docs/icebreaker/PLAN-MOD.md) : plugin WASM empaqueté comme Veloren l'attend, plugin.toml
# et module à la racine de l'archive.
if ($Lieu -eq "mod") {
    $src = "$repo\plugin\icebreaker"
    cargo build --release --target wasm32-wasip2 --manifest-path "$src\Cargo.toml"
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    $pack = "$src\target\pack"
    New-Item -ItemType Directory -Force $pack, "$repo\assets\plugins" | Out-Null
    Copy-Item "$src\target\wasm32-wasip2\release\icebreaker.wasm", "$src\plugin.toml" $pack
    tar -cf "$repo\assets\plugins\icebreaker.plugin.tar" -C $pack icebreaker.wasm plugin.toml
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    Write-Host "Icebreaker : mod pose dans assets\plugins\icebreaker.plugin.tar"
    exit 0
}

$ports = @{ campagne = 14100; village = 14100; donjon = 14110; route = 14120 }
if ($Lieu -eq "route") {
    Write-Host "Le lieu « $Lieu » n'est pas encore fait (voir docs/icebreaker/PLAN.md)."
    exit 1
}
if (-not $Site) { $Site = @{ campagne = "Town"; village = "Town"; donjon = "Gnarling" }[$Lieu] }
if (-not $Lg) { $Lg = if ($Lieu -eq "campagne") { 7 } else { 6 } }

$root = "$repo\userdata-icebreaker"
# Le monde de la campagne a son dossier à lui, à côté de « campagne » (personnages, caisse, entrepôt…).
$ud = if ($Lieu -eq "campagne") { "$root\monde" } else { "$root\$Lieu" }
$port = if ($Port) { $Port } else { $ports[$Lieu] }

# Monde neuf à chaque lancement ; les personnages restent dans la campagne.
Remove-Item -Recurse -Force "$ud\server\rtsim" -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force "$ud\server\server_config", "$ud\server-cli", "$root\campagne" | Out-Null

# Le village (seul ou dans la campagne) garde sa graine pour toute la campagne : ses maisons (bâties ou à bâtir)
# restent à leur place.
$graineVillage = "$root\campagne\$Lieu-graine.txt"
if ($Lieu -in "village", "campagne" -and -not $PSBoundParameters.ContainsKey("Seed")) {
    if (Test-Path $graineVillage) { $Seed = [int](Get-Content $graineVillage -TotalCount 1) }
    else { Set-Content -Path $graineVillage -Value $Seed -Encoding ascii }
}

# Les lecteurs RON refusent le BOM : UTF-8 sans BOM. Les champs absents prennent leur valeur par défaut.
$utf8 = New-Object Text.UTF8Encoding($false)
[IO.File]::WriteAllText("$ud\server\server_config\settings.ron", @"
(
    gameserver_protocols: [Tcp(address: "0.0.0.0:$port")],
    auth_server_address: None,
    query_address: None,
    world_seed: $Seed,
    server_name: "Icebreaker - $Lieu",
    day_length: $DayLength,
    map_file: Some(Generate((x_lg: $Lg, y_lg: $Lg))),
)
"@, $utf8)
[IO.File]::WriteAllText("$ud\server-cli\settings.ron", @"
(
    web_address: "127.0.0.1:$($port + 1)",
)
"@, $utf8)

$env:VELOREN_USERDATA = $ud
$env:ICEBREAKER_CAMPAIGN_DIR = "$root\campagne"
$env:ICEBREAKER_INSTANCE_SITE = $Site
$env:ICEBREAKER_CAMPAGNE = if ($Lieu -eq "campagne") { "1" } else { $null }
# Charrette et registre commun (jalon R) : on arrive en fantôme au village ; un donjon seul garde l'écran de Veloren.
$env:ICEBREAKER_REGISTRE = if ($Lieu -in "campagne", "village") { "1" } else { $null }
$env:ICEBREAKER_TOWN_SIZE = $TailleVille.ToString([Globalization.CultureInfo]::InvariantCulture)
$env:ICEBREAKER_DONJON_SALLE = $Salle
$env:ICEBREAKER_DONJON_TAILLE = $Taille
$env:ICEBREAKER_DONJON_OBJECTIF = $Objectif
$env:ICEBREAKER_DONJON_REGION = $Region
$env:ICEBREAKER_DONJON_SAISON = $Saison
$env:ICEBREAKER_DONJON_MENACE = $Menace
$env:ICEBREAKER_DONJON_ENNEMIS = $Ennemis
$env:ICEBREAKER_DONJON_FACTION = $Faction
Write-Host "Icebreaker : $Lieu ($Site), graine $Seed, port $port"
Set-Location $repo
& ".\target\debug\veloren-server-cli.exe" --no-auth
