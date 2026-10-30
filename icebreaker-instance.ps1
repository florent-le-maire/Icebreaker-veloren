# Icebreaker : lance une instance de rencontre (petit monde + un donjon au centre).
# Usage : .\icebreaker-instance.ps1 [-Site Haniwa] [-Lg 6] [-Seed 1]
# Puis dans le jeu : Multijoueur, serveur « localhost:14104 », n'importe quel pseudo.
param([string]$Site = "Haniwa", [int]$Lg = 6, [int]$Seed = 1)

$repo = $PSScriptRoot
$ud = "$repo\userdata-instance"
$cfg = "$ud\server\server_config\settings.ron"

# Instance neuve : pas de monde, de sauvegarde ni de données rtsim précédents.
Remove-Item -Recurse -Force "$ud\server\rtsim", "$ud\server\saves" -ErrorAction SilentlyContinue

$text = (Get-Content $cfg -Raw).TrimStart([char]0xFEFF)
$text = $text -replace 'world_seed: \d+', "world_seed: $Seed"
$text = $text -replace 'map_file: [^\r\n]*,', "map_file: Some(Generate((x_lg: $Lg, y_lg: $Lg))),"
$text = $text -replace '14004', '14104' -replace '14006', '14106'
# Le lecteur RON refuse le BOM : UTF-8 sans BOM.
[IO.File]::WriteAllText($cfg, $text, (New-Object Text.UTF8Encoding($false)))

$env:VELOREN_USERDATA = $ud
$env:ICEBREAKER_INSTANCE_SITE = $Site
Set-Location $repo
& ".\target\debug\veloren-server-cli.exe" --no-auth
