#! /bin/fish

# CHECK FOR ARGUMENT
if test (count $argv) -eq  0 
    echo "REQUIRED ARGUMENT NOT FOUND
        Run the installer again and pass the path to Automata's folder
        (the one containing the exe)"
    exit 1
end



# Creating directories
mkdir -p $HOME/.config/ATA
mkdir -p $HOME/.local/share/ATA
mkdir -p $HOME/.local/bin



# Copying files into the newly created directories
echo "Copying ATA files into ~/.local/share/ and ~/.local/bin"
cp ./install-prerequisites.sh $HOME/.local/share/ATA
cp ./target/release/ATA $HOME/.local/bin



# Creating default data file
echo "Creating default data file in ~/.config"
touch $HOME/.config/ATA/data.json

set game_path $argv[1]

echo "{
  \"game_path\": \"$game_path\",
  \"mods\": []
}" > $HOME/.config/ATA/data.json



printf "\nInstallation complete, make sure ~/.local/bin is in your PATH
Do not touch the ATA folders inside ~/.config, ~/.local/share and ~/.local/bin\n"