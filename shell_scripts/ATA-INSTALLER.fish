#! /bin/fish

function try
    set -l output (eval $argv 2>&1)
    set -l code $status

    if test $code -ne 0
        echo "Error running: $argv"
        echo "  $output"
        exit $code
    end
end



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



# Checking if FZF is installed
if ! type -q fzf
    printf "\nFZF required but not installed"
    
    set package_manager pacman
    for pm in apt dnf pacman brew
        if type -q $pm
            set package_manager $pm 
            printf $pm "found as default package manager"
            break
        end
    end
    
    switch $package_manager
        case pacman
            sudo pacman -S --noconfirm fzf
        case apt
            sudo apt install -y fzf
        case dnf
            sudo dnf install -y fzf
        case brew
            brew install fzf
    end
end
    


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
./install-prerequisites.fish $game_path



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