# ATA dev installer — Windows

# Remove folders for mod files (will be recreated by ATA if necessary)
# This doesn't affect a working installation of the game
rm -r -f "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/data/pl"
rm -r -f "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/data/wp"
rm -r -f "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/data/bg"
rm -r -f "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/data/wax"



# Create folders strictly necessary for development testing
# With these development testing is possible even if the game isn't installed
mkdir "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/data" -Force
mkdir "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/wax/mods" -Force



# Directories used by ATA
$exe = "$HOME/.local/bin/ATA"
$data = "$HOME/.local/share/ATA"
$settings = "$HOME/.config/ATA"
$uis = "$HOME/.local/share/UIs"
$apps = "$HOME/.local/share/Apps"

mkdir $exe -Force
mkdir $data -Force
mkdir $settings -Force
mkdir $uis -Force
mkdir $apps -Force



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