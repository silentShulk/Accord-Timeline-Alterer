#!/bin/bash
# ATA dev installer — Linux
# Iterations over arrays of paths are done to chekc if thigs exist
# Without it the stderr would polluted with warnings

# Remove folders for mod files (will be recreated by ATA if necessary)
# This doesn't affect a working installation of the game
rm -rf "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/data/pl"
rm -rf "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/data/wp"
rm -rf "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/data/bg"
rm -rf "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/data/wax"



# Create folders strictly necessary for development testing
# With these, development testing is possible even if the game isn't installed
for mod_path in "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/data" "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/wax/mods"; do
    if [ ! -d "$mod_path" ]; then
        mkdir -p "$mod_path"
    fi
done



# Directories used by ATA
exe="$HOME/.local/bin/ATA"
data="$HOME/.local/share/ATA"
settings="$HOME/.config/ATA"
uis="$HOME/.local/share/UIs"
apps="$HOME/.local/share/Apps"

ata_dirs=(
    "$exe" "$data" "$settings" "$uis" "$apps"
)
for dir in "${ata_dirs[@]}"; do
    if [ ! -d "$dir" ]; then
        mkdir -p "$dir"
    fi
done



# Insert default content inside data and settings

# data.json
cat << 'JSON' > "$data/data.json"
{
    "mods": []
}
JSON

# settings.json
cat << 'JSON' > "$settings/settings.json"
{
  "style": "SilentShulk",
  "palette": "Automata",
  "sortingOrder": "ModType",
  "filesConflictResolution": "Ask",
  "keepExtractedFolders": true,
  "extractedFoldersLocation": "$HOME/Downloads",
  "gamePath": "$HOME/.local/share/Steam/steamapps/common/NieRAutomata",
  "discordRichPresence": "Altering NieRAutomata's timelines"
}
JSON



echo "ATA dev environment ready."
