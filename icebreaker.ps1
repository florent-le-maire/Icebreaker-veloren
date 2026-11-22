# Icebreaker : lance un serveur pour un lieu de la campagne.
#
#   .\icebreaker.ps1 campagne                (port 14100 ; la campagne en un seul serveur, décision 0016 : le
#                                             village et des emplacements de donjon, assets\icebreaker\campagne.ron)
#   .\icebreaker.ps1 donjon -Site Gnarling   (port 14110)
#   .\icebreaker.ps1 donjon -Site Genere     (port 14110 ; donjon de notre générateur, jalon G ; mission :
#                                             -Taille -Objectif -Region -Saison -Menace -Ennemis -Faction -Race)
#   .\icebreaker.ps1 donjon -Site Grotte     (port 14140 ; grotte façon Deep Rock Galactic, jalon K)
#   .\icebreaker.ps1 route                   (port 14120, pas encore fait : P.1)
#   .\icebreaker.ps1 village                 (port 14100 ; -Site Town, CliffTown, DesertCity,
#                                             SavannahTown ou CoastalTown, Town par défaut)
#   .\icebreaker.ps1 entrainement            (port 14150 ; lieu d'entraînement, jalon O : un plateau, sa propre
#                                             partie, tout se règle par /entrainement)
#   .\icebreaker.ps1 mod                     (construit le mod dans assets\plugins ; le serveur
#                                             doit être compilé avec --features plugins)
#   .\icebreaker.ps1 caverne                 (atelier des grottes, jalon U : compile le générateur pour le
#                                             navigateur et sert l'atelier sur http://localhost:8096, où
#                                             l'on enregistre des presets dans formes.ron)
#   .\icebreaker.ps1 salles                  (atelier des salles, F.15 : http://localhost:8097, où l'on meuble
#                                             les salles du donjon au bloc près avec les objets de Veloren)
#
# Chaque lieu génère son propre petit monde à chaque lancement (graine au hasard, ou -Seed).
# Les personnages sont communs à tous les lieux : ils vivent dans la partie, userdata-icebreaker\parties\<nom>
# (-Partie <nom>, la dernière jouée par défaut).
# Dans le jeu : Multijoueur, serveur « localhost:<port> », n'importe quel pseudo.
param(
    [Parameter(Mandatory = $true, Position = 0)]
    [ValidateSet("campagne", "village", "donjon", "route", "entrainement", "mod", "caverne", "salles")]
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
    [ValidateSet("", "races", "gnarling", "adlet", "cultist")] [string]$Faction = "",
    # Race des ennemis (H.2, donjon généré et grotte) : au hasard par la graine sinon.
    [ValidateSet("", "human", "orc", "dwarf", "elf", "draugr", "danari")] [string]$Race = "",
    # Ambiance de la grotte (K.3) : champignons, cristaux ou ossuaire ; au hasard sinon.
    [string]$Ambiance = "",
    # Forme de la grotte (U.4) : un profil de assets\icebreaker\grottes\formes.ron (gouffre, plaine, chaos,
    # mixte) ; au hasard par la graine sinon.
    [string]$Profil = "",
    # Icebreaker X.10 : la partie (dossier userdata-icebreaker\parties\<nom>) ; par défaut ICEBREAKER_PARTIE
    # (hérité d'un serveur qui en lance un autre), sinon la dernière jouée, sinon « Partie 1 ».
    [string]$Partie = ""
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

# L'atelier des grottes (docs/icebreaker/PLAN-CAVERNE.md) : le générateur en WebAssembly, écrit en base64 dans un
# script à côté de la page (elle s'ouvre aussi d'un double-clic, sans pouvoir enregistrer). Le serveur de l'atelier
# (icebreaker-caverne, bin atelier) sert la page et écrit les presets dans formes.ron, avec un commit (U.10).
if ($Lieu -eq "caverne") {
    cargo build -p icebreaker-caverne --release --lib --target wasm32-unknown-unknown
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Icebreaker : si la cible manque, lancer : rustup target add wasm32-unknown-unknown"
        exit $LASTEXITCODE
    }
    $wasm = [IO.File]::ReadAllBytes("$repo\target\wasm32-unknown-unknown\release\icebreaker_caverne.wasm")
    $page = "$repo\icebreaker-caverne\web"
    [IO.File]::WriteAllText("$page\caverne-wasm.js", "window.CAVERNE_WASM = '" + [Convert]::ToBase64String($wasm) + "';")
    cargo build -p icebreaker-caverne --release --bin atelier
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    # Si un atelier tourne déjà, celui-ci s'arrête (port pris) et la page s'ouvre sur l'autre.
    $atelier = Start-Process "$repo\target\release\atelier.exe" -WorkingDirectory $repo -NoNewWindow -PassThru
    Start-Sleep -Seconds 1
    Start-Process "http://localhost:8096"
    $atelier | Wait-Process
    exit 0
}

# L'atelier des salles (docs/icebreaker/PLAN-PLAYTEST.md, F.15 à F.17) : le serveur sert la page, les objets de
# Veloren et les salles, et enregistre une salle (contrôle du jeu, fichier, commit).
if ($Lieu -eq "salles") {
    cargo build -p icebreaker-salles --bin atelier-salles
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    $atelier = Start-Process "$repo\target\debug\atelier-salles.exe" -WorkingDirectory $repo -NoNewWindow -PassThru
    Start-Sleep -Seconds 2
    Start-Process "http://localhost:8097"
    $atelier | Wait-Process
    exit 0
}

$ports = @{ campagne = 14100; village = 14100; donjon = 14110; route = 14120; entrainement = 14150 }
if ($Lieu -eq "route") {
    Write-Host "Le lieu « $Lieu » n'est pas encore fait (voir docs/icebreaker/PLAN.md)."
    exit 1
}
if (-not $Site) { $Site = @{ campagne = "Town"; village = "Town"; donjon = "Gnarling"; entrainement = "Entrainement" }[$Lieu] }
if (-not $Lg) { $Lg = if ($Lieu -eq "campagne") { 7 } else { 6 } }

$root = "$repo\userdata-icebreaker"
# Le monde de la campagne a son dossier à lui, à côté des parties (personnages, caisse, entrepôt…).
$ud = if ($Lieu -eq "campagne") { "$root\monde" } else { "$root\$Lieu" }
$port = if ($Port) { $Port } else { $ports[$Lieu] }

# Icebreaker X.10 : une partie = un dossier de userdata-icebreaker\parties. L'ancienne campagne unique
# (userdata-icebreaker\campagne) devient « Partie 1 », une seule fois.
$parties = "$root\parties"
if (-not (Test-Path $parties) -and (Test-Path "$root\campagne")) {
    New-Item -ItemType Directory -Force $parties | Out-Null
    Move-Item "$root\campagne" "$parties\Partie 1"
    Write-Host "Icebreaker : l'ancienne campagne devient la partie « Partie 1 »"
}
if (-not $Partie) { $Partie = $env:ICEBREAKER_PARTIE }
if (-not $Partie -and (Test-Path "$parties\derniere.txt")) {
    $Partie = (Get-Content "$parties\derniere.txt" -TotalCount 1 -Encoding utf8).Trim()
}
if (-not $Partie) { $Partie = "Partie 1" }
$campagne = "$parties\$Partie"
# Icebreaker O : le lieu d'entraînement a sa partie à lui, hors des parties de la campagne.
$entrainement = $Lieu -eq "entrainement"
if ($entrainement) { $Partie = "Entrainement"; $campagne = "$ud\partie" }

# Monde neuf à chaque lancement ; les personnages restent dans la partie.
Remove-Item -Recurse -Force "$ud\server\rtsim" -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force "$ud\server\server_config", "$ud\server-cli", $campagne | Out-Null

# Les lecteurs RON refusent le BOM : UTF-8 sans BOM. Les champs absents prennent leur valeur par défaut.
$utf8 = New-Object Text.UTF8Encoding($false)
# La dernière partie jouée, et celle de ce serveur (le menu refuse d'y envoyer pour une autre partie).
if (-not $entrainement) { [IO.File]::WriteAllText("$parties\derniere.txt", $Partie, $utf8) }
[IO.File]::WriteAllText("$ud\partie.txt", $Partie, $utf8)

# Le village (seul ou dans la campagne) garde sa graine pour toute la partie : ses maisons (bâties ou à bâtir)
# restent à leur place.
$graineVillage = "$campagne\$Lieu-graine.txt"
if ($Lieu -in "village", "campagne", "entrainement" -and -not $PSBoundParameters.ContainsKey("Seed")) {
    if (Test-Path $graineVillage) { $Seed = [int](Get-Content $graineVillage -TotalCount 1) }
    else { Set-Content -Path $graineVillage -Value $Seed -Encoding ascii }
}

# Icebreaker : le terrain du village (graine gardée par la partie) est enregistré dans maps\ au premier
# lancement, puis relu : le serveur démarre bien plus vite.
$carte = if ($Lieu -eq "donjon") { "Generate((x_lg: $Lg, y_lg: $Lg))" }
else { "LoadOrGenerate(name: `"$Lieu-$Seed-lg$Lg`", opts: (x_lg: $Lg, y_lg: $Lg))" }

[IO.File]::WriteAllText("$ud\server\server_config\settings.ron", @"
(
    gameserver_protocols: [Tcp(address: "0.0.0.0:$port")],
    auth_server_address: None,
    query_address: None,
    world_seed: $Seed,
    server_name: "Icebreaker - $Lieu",
    day_length: $DayLength,
    map_file: Some($carte),
)
"@, $utf8)
[IO.File]::WriteAllText("$ud\server-cli\settings.ron", @"
(
    web_address: "127.0.0.1:$($port + 1)",
)
"@, $utf8)

$env:VELOREN_USERDATA = $ud
$env:ICEBREAKER_CAMPAIGN_DIR = $campagne
$env:ICEBREAKER_PARTIE = $Partie
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
$env:ICEBREAKER_DONJON_RACE = $Race
$env:ICEBREAKER_GROTTE_AMBIANCE = $Ambiance
$env:ICEBREAKER_GROTTE_PROFIL = $Profil
# Icebreaker F.33 : la forme du donjon généré (donjons/formes.ron), même paramètre.
$env:ICEBREAKER_DONJON_PROFIL = $Profil
Write-Host "Icebreaker : $Lieu ($Site), partie « $Partie », graine $Seed, port $port"
Set-Location $repo
& ".\target\debug\veloren-server-cli.exe" --no-auth
