#! /bin/bash

# CHECK FOR ARGUMENT
if [ $# -eq 0 ]; then
  echo "REQUIRED ARGUMENT NOT FOUND.
  Run the installer again and pass the path to Automata's folder
  (the one containing the exe)";
  exit 1
fi



# Creating directories
mkdir -p $HOME/.config/ATA/
mkdir -p $HOME/.local/share/ATA
mkdir -p $HOME/.local/bin



# Copying files into the newly created directories
echo "Copying ATA files into ~/.local/share/ and ~/.local/bin"
cp ./install-prerequisites.sh $HOME/.local/share/ATA
cp ./target/release/ATA $HOME/.local/bin



# Creating default data file
echo "Creating default data file in ~/.config/"
touch $HOME/.config/ATA/data.json

game_path=$1

cat <<EOF > "$HOME/.config/ATA/data.json"
{
  "game_path": "$game_path",
  "mods": []
}
EOF



echo ""
echo "Installation complete, make sure ~/.local/bin is in your PATH
Do not touch the ATA folders in ~/.config, ~/.local/share and ~/.local/bin"
