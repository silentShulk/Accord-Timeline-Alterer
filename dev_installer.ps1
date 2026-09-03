# ATA dev installer — Windows
# Iterations over arrays of paths are done to chekc if thigs exist
# Without it the stderr would polluted with warnings 

# Remove folders for mod files (will be recreated by ATA if necessary)
# This doesn't affect a working installation of the game
$modPaths = @(
    "C:/Program Files (x86)/Steam/steamapps/common/NieRAutomata/data/pl",
    "C:/Program Files (x86)/Steam/steamapps/common/NieRAutomata/data/wp",
    "C:/Program Files (x86)/Steam/steamapps/common/NieRAutomata/data/bg",
    "C:/Program Files (x86)/Steam/steamapps/common/NieRAutomata/data/wax"
)

foreach ($path in $modPaths) {
    if (Test-Path $path) {
        rm -r -Force $path
    }
}



# Create folders strictly necessary for development testing
$required_mod_paths = "C:/Program Files (x86)/Steam/steamapps/common/NieRAutomata/data", "C:/Program Files (x86)/Steam/steamapps/common/NieRAutomata/wax/mods"
foreach ($rmp in $required_mod_paths) {
    if (-not(Test-Path -Path $rmp)) {
        mkdir $rmp -Force | Out-Null
    }
}



# Directories used by ATA
$exe      = "$env:LOCALAPPDATA\Programs\ATA"
$data     = "$env:LOCALAPPDATA\ATA"
$settings = "$env:APPDATA\ATA"
$uis      = "$env:LOCALAPPDATA\ATA\UIs"
$apps     = "$env:LOCALAPPDATA\ATA\Apps"

$ata_dirs = $exe, $data, $settings, $uis, $apps
foreach ($dir in $ata_dirs) {
    if (-not(Test-Path -Path $dir)) {
        mkdir $dir -Force | Out-Null
    }
}



# Insert default content inside data and settings

# data.json
[System.IO.File]::WriteAllText("$data\data.json", @'
{
    "mods": []
}
'@, [System.Text.UTF8Encoding]::new($false))
 
# settings.json
# Pre-escape the $HOME path so it retains double backslashes in the JSON
$escapedHome = $HOME -replace '\\', '\\\\'
 
[System.IO.File]::WriteAllText("$settings\settings.json", @"
{
  "style": "ShellUI",
  "palette": "Automata",
  "sortingOrder": "ModType",
  "filesConflictResolution": "Warn",
  "keepExtractedFolders": true,
  "extractedFoldersLocation": "${escapedHome}\\\\Downloads",
  "gamePath": "C:\\\\Program Files (x86)\\\\Steam\\\\steamapps\\\\common\\\\NieRAutomata",
  "discordRichPresence": "Altering NieRAutomata's timelines"
}
"@, [System.Text.UTF8Encoding]::new($false))


Write-Host "ATA dev environment ready." -ForegroundColor Green
