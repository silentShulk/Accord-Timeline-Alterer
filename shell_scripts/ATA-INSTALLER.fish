#! /bin/fish

printf "Please go read the documentation if you haven't already\n"

# CHECK FOR ARGUMENT
if test (count $argv) -eq  0
    echo "REQUIRED ARGUMENT NOT FOUND
    Run the installer again and pass the path to Automata's folder
    (the one containing the exe)"
    exit 1
end

# CHECK IF GIVEN PATH IS ACTUALLY GAME PATH
if ! test -f "$argv[1]/NieRAutomata.exe"
	echo "GIVEN PATH ISN'T GAME PATH
	It does not contain the NieRAutomata.exe
	Run the installer again and pass the path to Automata's folder"
	exit 1
end

set game_path $argv[1]



# Creating directories
echo "Creating directories in ~/.config and ~/.local"
mkdir -p $HOME/.config/ATA
mkdir -p $HOME/.local/share/ATA
mkdir -p $HOME/.local/bin



# Copying files into the newly created directories
echo "Copying ATA files into ~/.local/share/ and ~/.local/bin"
cp ./install-prerequisites.sh $HOME/.local/share/ATA
cp ./target/release/ATA $HOME/.local/bin



# Creating default data file
echo "Creating data file in ~/.config"
touch $HOME/.config/ATA/data.json

echo "{
  \"game_path\": \"$game_path\",
  \"mods\": []
}" > $HOME/.config/ATA/data.json



# Installing required modding files 
echo "Running script to install files needed to mod the game"
./install-prerequisites.fish


printf "\nInstallation complete, make sure ~/.local/bin is in your PATH
Do not touch the ATA folders inside ~/.config, ~/.local/share and ~/.local/bin\n"