# ATA dev installer — Windows

# Remove folders for mod files (will be recreated by ATA if necessary)
# This doesn't affect a working installation of the game
$modPaths = @(
    "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/data/pl",
    "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/data/wp",
    "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/data/bg",
    "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/data/wax"
)

foreach ($path in $modPaths) {
    if (Test-Path $path) {
        # This will only run if the path exists. Else error gets thrown polluting the output
        # If it fails due to permissions, it WILL throw a visible error.
        rm -r -f $path
    }
}



# Create folders strictly necessary for development testing
# With these development testing is possible even if the game isn't installed
mkdir "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/data" -Force | Out-Null
mkdir "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/wax/mods" -Force | Out-Null



# Directories used by ATA
$exe = "$HOME/.local/bin/ATA"
$data = "$HOME/.local/share/ATA"
$settings = "$HOME/.config/ATA"
$uis = "$HOME/.local/share/UIs"
$apps = "$HOME/.local/share/Apps"

mkdir $exe -Force | Out-Null
mkdir $data -Force | Out-Null
mkdir $settings -Force | Out-Null
mkdir $uis -Force | Out-Null
mkdir $apps -Force | Out-Null



# Insert default content inside data and settings

# data.json
@'
{
    "mods": []
}
'@ > "$data\data.json"

# settings.json
@"
{
  "style": "SilentShulk",
  "palette": "Automata",
  "sortingOrder": "ModType",
  "filesConflictResolution": "Ask",
  "keepExtractedFolders": true,
  "extractedFoldersLocation": "$HOME\\Downloads",
  "gamePath": "C:\\Program Files (x86)\\Steam\\steamapps\\common\\NieRAutomata",
  "discordRichPresence": "Altering NieRAutomata's timelines"
}
"@ > "$settings\settings.json"



Write-Host "ATA dev environment ready." -ForegroundColor Green
