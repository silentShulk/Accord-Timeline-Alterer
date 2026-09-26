# ATA dev installer — Windows
#
# Prepares everything ATA needs to be developed/tested:
#   - ATA's folders, a default data.json and settings.json (only if missing)
#   - the latest built ATA executable (if `cargo build --release` was run)
#   - a fake game folder when NieR:Automata isn't installed
#
# Usage: .\dev_installer.ps1 [-Reset]
#   -Reset  also wipes data.json, settings.json and the mod folders of the game
#           (DESTRUCTIVE: installed mods are forgotten and their files deleted)
#
# The game folder can be overridden with the ATA_GAME_PATH environment variable.

param([switch]$Reset)

$ErrorActionPreference = "Stop"

$game = if ($env:ATA_GAME_PATH) { $env:ATA_GAME_PATH } else { "C:\Program Files (x86)\Steam\steamapps\common\NieRAutomata" }

# Paths used by ATA (must match src/data/paths.rs)
$exeDir       = "$env:LOCALAPPDATA\Programs\ATA"
$dataDir      = "$env:LOCALAPPDATA\ATA"
$settingsDir  = "$env:APPDATA\ATA"
$dataFile     = "$dataDir\data.json"
$settingsFile = "$settingsDir\settings.json"

if ($Reset) {
    Write-Host "Resetting ATA data and the game's mod folders"
    $toRemove = @(
        $dataFile, $settingsFile,
        "$game\data\pl", "$game\data\wp", "$game\data\bg", "$game\data\misctex",
        "$game\wax\mods", "$game\.ata-backup"
    )
    foreach ($path in $toRemove) {
        if (Test-Path $path) { Remove-Item -Recurse -Force $path }
    }
}

# ATA's folders, and the folders needed to test mod installation even without the game installed
foreach ($dir in $exeDir, "$dataDir\UIs", "$dataDir\Apps", $settingsDir, "$game\data", "$game\wax\mods") {
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
}
if (-not (Test-Path "$game\NieRAutomata.exe")) {
    Write-Host "NieR:Automata not found in $game, creating a fake game executable for testing"
    New-Item -ItemType File -Path "$game\NieRAutomata.exe" | Out-Null
}

# Latest release build of the backend
$built = Join-Path $PSScriptRoot "target\release\ATA.exe"
if (Test-Path $built) { Copy-Item -Force $built "$exeDir\ATA.exe" }

# Default data and settings, never overwriting existing ones
$utf8 = [System.Text.UTF8Encoding]::new($false)

if (-not (Test-Path $dataFile)) {
    [System.IO.File]::WriteAllText($dataFile, "{`n  `"mods`": []`n}`n", $utf8)
}

if (-not (Test-Path $settingsFile)) {
    $settings = [ordered]@{
        style                    = "ShellUI"
        palette                  = "Automata"
        sortingOrder             = "ModType"
        filesConflictResolution  = "Warn"
        keepExtractedFolders     = $false
        extractedFoldersLocation = ""
        gamePath                 = $game
        discordRichPresence      = "Altering NieRAutomata's timelines"
    }
    [System.IO.File]::WriteAllText($settingsFile, ($settings | ConvertTo-Json), $utf8)
}

Write-Host "ATA dev environment ready." -ForegroundColor Green
