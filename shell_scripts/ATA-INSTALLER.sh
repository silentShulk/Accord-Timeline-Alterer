#! /bin/bash

function try {
    local output
    output=$(eval "$@" 2>&1)
    local code=$?
    if [ $code -ne 0 ]; then
        echo "Error running: $@"
        echo "  $output"
        exit $code
    fi
}



printf "Please go read the documentation if you haven't already\n"

# CHECK FOR ARGUMENT
if [ $# -eq 0 ]; then
    echo "REQUIRED ARGUMENT NOT FOUND
    Run the installer again and pass the path to Automata's folder
    (the one containing the exe)"
    exit 1
fi

# CHECK IF GIVEN PATH IS ACTUALLY GAME PATH
if [ ! -f "$1/NieRAutomata.exe" ]; then
    echo "GIVEN PATH ISN'T GAME PATH
    It does not contain the NieRAutomata.exe
    Run the installer again and pass the path to Automata's folder"
    exit 1
fi

game_path=$1



# Checking if FZF is installed
if ! command -v fzf &>/dev/null; then
    printf "\nFZF required but not installed"
    
    package_manager="pacman"
    for pm in apt dnf pacman brew; do
        if command -v $pm &>/dev/null; then
            package_manager=$pm
            break
        fi
    done
    
    case $package_manager in
        pacman)
            sudo pacman -S --noconfirm fzf ;;
        apt)
            sudo apt install -y fzf ;;
        dnf)
            sudo dnf install -y fzf ;;
        brew)
            brew install fzf ;;
    esac
fi



# Creating directories
printf "\nCreating directories in ~/.local and ~/.config"
try mkdir -p $HOME/.local/share/ATA
try mkdir -p $HOME/.local/bin
try mkdir -p $HOME/.config/ATA



# Copying files into the newly created directories
printf "\nCopying ATA files into ~/.local/share/ and ~/.local/bin"
try cp ./install-prerequisites.sh $HOME/.local/share/ATA
try cp ../target/release/ATA $HOME/.local/bin



# Creating default data file
printf "\nCreating data file in ~/.config"
try touch $HOME/.config/ATA/data.json

echo "{
  \"game_path\": \"$game_path\",
  \"mods\": []
}" > $HOME/.config/ATA/data.json



# Installing required modding files
printf "\nRunning script to install files needed to mod the game"
./install-prerequisites.sh $game_path



printf "\nCheck you game dir, there should now be:
- d3d11.dll
- d3d11.ini
- data
- FAR.ini
- logs/
- NieRAutomata.exe
- NieRAutomata.exe(original)
- SK_Res
- steam_api64.dll
- Wallpaper"



printf "\n\nInstallation complete, make sure ~/.local/bin is in your PATH
Do not touch the ATA folders inside ~/.config, ~/.local/share and ~/.local/bin\n"