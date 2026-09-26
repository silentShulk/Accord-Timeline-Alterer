#!/bin/bash
# ATA dev installer — Linux
#
# Prepares everything ATA needs to be developed/tested:
#   - ATA's folders, a default data.json and settings.json (only if missing)
#   - the latest built ATA executable (if `cargo build --release` was run)
#   - a fake game folder when NieR:Automata isn't installed
#
# Usage: ./dev_installer.sh [--reset]
#   --reset  also wipes data.json, settings.json and the mod folders of the game
#            (DESTRUCTIVE: installed mods are forgotten and their files deleted)
#
# The game folder can be overridden with the ATA_GAME_PATH environment variable.

set -euo pipefail

reset=false
[ "${1:-}" = "--reset" ] && reset=true

game="${ATA_GAME_PATH:-$HOME/.local/share/Steam/steamapps/common/NieRAutomata}"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Paths used by ATA (must match src/data/paths.rs)
exe_dir="$HOME/.local/bin/ATA"
data_dir="$HOME/.local/share/ATA"
settings_dir="$HOME/.config/ATA"
data_file="$data_dir/data.json"
settings_file="$settings_dir/settings.json"

if $reset; then
    echo "Resetting ATA data and the game's mod folders"
    rm -f "$data_file" "$settings_file"
    rm -rf "$game/data/pl" "$game/data/wp" "$game/data/bg" "$game/data/misctex" \
           "$game/wax/mods" "$game/.ata-backup"
fi

# ATA's folders
mkdir -p "$exe_dir" "$data_dir/UIs" "$data_dir/Apps" "$settings_dir"

# Folders needed to test mod installation, even without the game installed
mkdir -p "$game/data" "$game/wax/mods"
if [ ! -f "$game/NieRAutomata.exe" ]; then
    echo "NieR:Automata not found in $game, creating a fake game executable for testing"
    touch "$game/NieRAutomata.exe"
fi

# Latest release build of the backend
if [ -f "$script_dir/target/release/ATA" ]; then
    install -m 755 "$script_dir/target/release/ATA" "$exe_dir/ATA"
fi

# Default data and settings, never overwriting existing ones
if [ ! -f "$data_file" ]; then
    printf '{\n  "mods": []\n}\n' > "$data_file"
fi

if [ ! -f "$settings_file" ]; then
    cat << JSON > "$settings_file"
{
  "style": "ShellUI",
  "palette": "Automata",
  "sortingOrder": "ModType",
  "filesConflictResolution": "Warn",
  "keepExtractedFolders": false,
  "extractedFoldersLocation": "",
  "gamePath": "$game",
  "discordRichPresence": "Altering NieRAutomata's timelines"
}
JSON
fi

echo "ATA dev environment ready."
