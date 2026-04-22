#! /bin/fish
# GO TO LINE 159 TO SEE WHAT THE SCRIPT DOES

function try
    set -l output (eval $argv 2>&1)
    set -l code $status

    if test $code -ne 0
        echo "Error running: $argv"
        echo "  $output"
        exit $code
    end
end

function argument_check
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
    
    set -g game_path $argv[1]
end

function dependencies_installation
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
        
    # Checking if 7zip is installed
    if ! type -q 7z
        printf "\n7zip required but not installed"
        
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
                sudo pacman -S --noconfirm p7zip
            case apt
                sudo apt install -y p7zip-full
            case dnf
                sudo dnf install -y p7zip
            case brew
                brew install p7zip
        end
    end
end

function ATA_setup 
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
end

function modding_requirements_installation
    printf "\nRunning script to install files needed to mod the game"
    ./install-prerequisites.fish $game_path
end
    
function specialk_auto_setup
    printf "LET THE GAME START AND CLOSE IT FROM THE MAIN MENU"
    sleep 5
    
    # Launch game
    printf "\nLaunching the game\n"
    steam steam://rungameid/524220
    
    # Wait for the game process to start
    printf "Waiting for game to start...\n"
    while not pgrep -x "steampp_524200" > /dev/null
        sleep 1
    end
    
    # Now wait for the game process to end
    printf "Game started! Close it from the main menu when ready.\n"
    while pgrep -x "steampp_524200" > /dev/null
        sleep 1
    end
    
    printf "Game closed, continuing installation...\n"
end

function reshade_setup
    # Extracting dll from installer exe
    7z e ../bin/ReShade_Setup_6.7.3_Addon.exe ReShade64.dll
    
    # Move into Proton prefix
    try mv ReShade64.dll $HOME/.local/share/Steam/steamapps/compatdata/524220/pfx/drive_c/users/steamuser/Documents/My\ Mods/SpecialK/PlugIns/ThirdParty/Reshade/
    
    # Creating ReShade folders
    try mkdir -p $HOME/.local/share/Steam/steamapps/compatdata/524220/pfx/drive_c/users/steamuser/Documents/My\ Mods/SpecialK/Global/ReShade/Textures
    try mkdir -p $HOME/.local/share/Steam/steamapps/compatdata/524220/pfx/drive_c/users/steamuser/Documents/My\ Mods/SpecialK/Global/ReShade/Shaders
    try cd $HOME/.local/share/Steam/steamapps/compatdata/524220/pfx/drive_c/users/steamuser/Documents/My\ Mods/SpecialK/Global/ReShade/
    
    # Copying ReShade default effects and textures from repo
    git clone https://github.com/crosire/reshade-shaders.git
    try cp reshade-shaders/Shaders/* Shaders/
    try cp reshade-shaders/Textures/* Textures/
    try rm -rf reshade-shaders
end





# SCRIPT STARTS HERE
printf "Please go read the documentation if you haven't already\n"

# Check if the game's installation path was passed correctly
argument_check $argv

# Installing dependencies (fzf, 7z)
dependencies_installation

# Setupping folders and copying files
ATA_setup

# Installing required modding files (SpecialK dll, MC++BT, Wolf's patched exe)
modding_requirements_installation

# Starting the game to let specialk create its folders/files
specialk_auto_setup

# Creating ReShade files and folders, cloning default effects/textures
reshade_setup



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
