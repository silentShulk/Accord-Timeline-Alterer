#! /bin/fish
# GO TO LINE 159 TO SEE WHAT THE SCRIPT DOES

function try
    $argv
    if test $status -ne 0
        echo "Error running: $argv"
        exit $status
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

function modding_requirements_installation
    printf "\nRunning script to install files needed to mod the game"
    ./install-prerequisites.fish $game_path
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
    printf "\nCreating data file in ~/.config\n"
    try touch $HOME/.config/ATA/data.json

    echo "{
        \"game_path\": \"$game_path\",
        \"mods\": []
    }" > $HOME/.config/ATA/data.json
end

function user_action
    printf "\n⚠️  Before continuing, please run steam's \"Integrity of game's files check\"."
    printf "\nHOWTO:
    - Open Automata's page from you steam library
    - Click the gear icon (⚙️) and select Properties
    - \"Installed files\" tab -> \"Verify integrity of game files\"
    - Let it run for however long it takes and then come back here"
    
    printf "\nType 'file check done' and press Enter when ready: "
    while true
        read -P "Type here> " -l user_input
        if test "$user_input" = "file check done"
            break
        else 
            printf "\nNot quite my \"file check done\""
        end
    end
end
    
function reshade_setup
    printf ""
    # Move ReShade dll into Proton prefix
    cd ../lib/
    try mkdir -p $HOME/.local/share/Steam/steamapps/compatdata/524220/pfx/drive_c/users/steamuser/Documents/My\ Mods/SpecialK/PlugIns/ThirdParty/Reshade/
    try cp ReShade64.dll $HOME/.local/share/Steam/steamapps/compatdata/524220/pfx/drive_c/users/steamuser/Documents/My\ Mods/SpecialK/PlugIns/ThirdParty/Reshade/
    
    # Creating ReShade folders
    try mkdir -p $HOME/.local/share/Steam/steamapps/compatdata/524220/pfx/drive_c/users/steamuser/Documents/My\ Mods/SpecialK/Global/ReShade/Textures
    try mkdir -p $HOME/.local/share/Steam/steamapps/compatdata/524220/pfx/drive_c/users/steamuser/Documents/My\ Mods/SpecialK/Global/ReShade/Shaders
    try cd $HOME/.local/share/Steam/steamapps/compatdata/524220/pfx/drive_c/users/steamuser/Documents/My\ Mods/SpecialK/Global/ReShade/
    
    # Copying ReShade default effects and textures from repo
    if test -d reshade-shaders
        rm -rf reshade-shaders
    end
    git clone https://github.com/crosire/reshade-shaders.git
    try cp reshade-shaders/Shaders/* Shaders/
    try cp reshade-shaders/Textures/* Textures/
    try rm -rf reshade-shaders
end

function setup_finalization
    printf "\nFinilazing installation...
    LET THE GAME START AND CLOSE IT FROM THE MAIN MENU"
    sleep 5
    
    # Launch game
    printf "\nLaunching the game\n"
    steam steam://rungameid/524220
    
    # Wait for the game process to start
    while not pgrep -f NieRAutomata.exe > /dev/null
        printf "Waiting for game to start...\n"
        sleep 3
    end
    
    # Now wait for the game process to end
    while pgrep -f NieRAutomata.exe > /dev/null
        printf "Game started! You should see _wax loaded_ in the loading screen
        Close it from the main menu\n"
        sleep 3
    end
    
    printf "\nGame closed"
    sleep 2
    
    printf "\nRemoving framerate cap
    It is recommended to set a custom one using tools like MangoHud\n"
    sed -i 's/"uncap_fps": false/"uncap_fps": true/' $game_path/wax/config.json
end
    
    



# SCRIPT STARTS HERE
printf "Please go read the documentation if you haven't already\n"
sleep 2

# Check if the game's installation path was passed correctly
argument_check $argv

# Installing dependencies (fzf, 7z)
dependencies_installation

# Installing required modding files (WAX dll, MCppBT)
modding_requirements_installation

# Setupping folders and copying files
ATA_setup

# Let user run file check
user_action

# Creating ReShade files and folders, cloning default effects/textures
# reshade_setup

# Starting the game to let WAX create files
setup_finalization



printf "\nCheck you game dir, there should now be:
- d3d11.dll
- data
- installation_os_check.vdf (REMOVE THIS)
- logs/
- NieRAutomata.exe
- NieRAutomataCompat.exe (REMOVE THIS)
- steam_api64.dll
- Wallpaper
- win8_7_setup.bat (REMOVE THIS)
- win10setup.bat (REMOVE THIS)
"



printf "\n\nInstallation complete, make sure ~/.local/bin is in your PATH
Do not touch the ATA folders inside ~/.config, ~/.local/share and ~/.local/bin\n"