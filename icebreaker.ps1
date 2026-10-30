# Icebreaker : lance un serveur pour un lieu de la campagne.
#
#   .\icebreaker.ps1 donjon -Site Gnarling   (port 14110)
#   .\icebreaker.ps1 route                   (port 14120, pas encore fait : P.1)
#   .\icebreaker.ps1 village                 (port 14100 ; -Site Town, CliffTown, DesertCity,
#                                             SavannahTown ou CoastalTown, Town par défaut)
#
# Chaque lieu génère son propre petit monde à chaque lancement (graine au hasard, ou -Seed).
# Les personnages sont communs à tous les lieux : ils vivent dans userdata-icebreaker\campagne.
# Dans le jeu : Multijoueur, serveur « localhost:<port> », n'importe quel pseudo.
param(
    [Parameter(Mandatory = $true, Position = 0)]
    [ValidateSet("village", "donjon", "route")]
    [string]$Lieu,
    # Site autour duquel le lieu est généré (par défaut : Gnarling au donjon, Town au village).
    [string]$Site,
    # Taille de la ville du village (Town seulement), de 0.03 (hameau) à 1.0 (grande ville).
    [double]$TailleVille = 1.0,
    [int]$Lg = 6,
    [int]$Seed = (Get-Random -Maximum 1000000),
    # Durée d'une journée en minutes réelles (1 min réelle = 30 min de carte).
    [double]$DayLength = 48.0
)

$ports = @{ village = 14100; donjon = 14110; route = 14120 }
if ($Lieu -eq "route") {
    Write-Host "Le lieu « $Lieu » n'est pas encore fait (voir docs/icebreaker/PLAN.md)."
    exit 1
}
if (-not $Site) { $Site = @{ village = "Town"; donjon = "Gnarling" }[$Lieu] }

$repo = $PSScriptRoot
$root = "$repo\userdata-icebreaker"
$ud = "$root\$Lieu"
$port = $ports[$Lieu]

# Monde neuf à chaque lancement ; les personnages restent dans la campagne.
Remove-Item -Recurse -Force "$ud\server\rtsim" -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force "$ud\server\server_config", "$ud\server-cli", "$root\campagne" | Out-Null

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
$env:ICEBREAKER_TOWN_SIZE = $TailleVille.ToString([Globalization.CultureInfo]::InvariantCulture)
Write-Host "Icebreaker : $Lieu ($Site), graine $Seed, port $port"
Set-Location $repo
& ".\target\debug\veloren-server-cli.exe" --no-auth
