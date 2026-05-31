#! /bin/bash
rm -rf ~/.local/share/Steam/steamapps/common/NieRAutomata/data/pl
rm -rf ~/.local/share/Steam/steamapps/common/NieRAutomata/data/wp
rm -rf ~/.local/share/Steam/steamapps/common/NieRAutomata/data/bg

mkdir -p ~/.local/share/Steam/steamapps/common/NieRAutomata/data
mkdir -p ~/.local/share/Steam/steamapps/common/NieRAutomata/wax/mods



mkdir -p ~/.config/ATA/ && mkdir -p ~/.local/share/ATA
touch ~/.local/share/ATA/data.json ~/.config/ATA/settings.json



cat << 'EOF' > ~/.local/share/ATA/data.json
{
    "mods": []
}
EOF

cat << 'EOF' > ~/.config/ATA/settings.json
{
  "style": "SilentShulk",
  "palette": "Replicant",
  "sortingOrder": "ModType",
  "filesConflictResolution": "Ask",
  "keepExtractedFolders": true,
  "extractedFoldersLocation": "$HOME/Downloads/",
  "gamePath": "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/",
  "discordRichPresence": "Altering NieRAutomata's timelines"
}
EOF
