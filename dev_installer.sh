#!/bin/bash
# ATA dev installer — Linux

# Game dirs (clean slate)
rm -rf "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/data/pl"
rm -rf "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/data/wp"
rm -rf "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/data/bg"

mkdir -p "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/data"
mkdir -p "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/wax/mods"

# Config/data dirs
mkdir -p "$HOME/.config/ATA"
mkdir -p "$HOME/.local/share/ATA"
mkdir -p "$HOME/.local/state/ATA/UIs"
mkdir -p "$HOME/.local/state/ATA/Apps"
mkdir -p "$HOME/.local/bin/ATA"

# data.json
cat << 'JSON' > "$HOME/.local/share/ATA/data.json"
{
    "mods": []
}
JSON

# settings.json
cat << 'JSON' > "$HOME/.config/ATA/settings.json"
{
  "style": "SilentShulk",
  "palette": "Automata",
  "sortingOrder": "ModType",
  "filesConflictResolution": "Ask",
  "keepExtractedFolders": true,
  "extractedFoldersLocation": "",
  "gamePath": "",
  "discordRichPresence": "Altering NieRAutomata's timelines"
}
JSON

echo "ATA dev environment ready."
