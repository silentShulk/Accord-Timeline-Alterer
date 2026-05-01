#!/bin/bash
# GO TO LINE 159 TO SEE WHAT THE SCRIPT DOES

# In bash, 'try' is not a reserved keyword (unlike nushell), so the name is kept.
# "$@" expands all positional arguments passed to the function, equivalent to fish's $argv.
# $? holds the exit status of the last command — captured immediately to avoid it being
# overwritten by the 'if' or 'echo' calls that follow.
function try {
    "$@"
    local status=$?
    if [ $status -ne 0 ]; then
        echo "Error running: $@"
        exit $status
    fi
}

# $# holds the count of arguments passed to the function (fish: count $argv).
# [ ! -f path ] checks for file non-existence (fish: ! test -f path).
# Variables set inside a function without 'local' are global in bash,
# so game_path="$1" is visible to the rest of the script.
function argument_check {
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

    game_path="$1"
}

# 'command -v cmd &>/dev/null' is the POSIX-portable way to check if a command
# exists, equivalent to fish's 'type -q cmd'. 'which' is not portable across distros.
# The for loop syntax is: for var in list; do ... done (fish: for var in list; end).
# case/esac replaces fish's switch/case/end.
function dependencies_installation {
    # Checking if FZF is installed
    if ! command -v fzf &>/dev/null; then
        printf "\nFZF required but not installed"

        local package_manager=pacman
        for pm in apt dnf pacman brew; do
            if command -v $pm &>/dev/null; then
                package_manager=$pm
                printf "$pm found as default package manager"
                break
            fi
        done

        case $package_manager in
            pacman) sudo pacman -S --noconfirm fzf ;;
            apt)    sudo apt install -y fzf ;;
            dnf)    sudo dnf install -y fzf ;;
            brew)   brew install fzf ;;
        esac
    fi

    # Checking if 7zip is installed
    if ! command -v 7z &>/dev/null; then
        printf "\n7zip required but not installed"

        local package_manager=pacman
        for pm in apt dnf pacman brew; do
            if command -v $pm &>/dev/null; then
                package_manager=$pm
                printf "$pm found as default package manager"
                break
            fi
        done

        case $package_manager in
            pacman) sudo pacman -S --noconfirm p7zip ;;
            apt)    sudo apt install -y p7zip-full ;;
            dnf)    sudo dnf install -y p7zip ;;
            brew)   brew install p7zip ;;
        esac
    fi
}

function modding_requirements_installation {
    printf "\nRunning script to install files needed to mod the game"
    ./install-prerequisites.bash "$game_path"
}

function ATA_setup {
    # Creating directories
    printf "\nCreating directories in ~/.local and ~/.config"
    try mkdir -p "$HOME/.local/share/ATA"
    try mkdir -p "$HOME/.local/bin"
    try mkdir -p "$HOME/.config/ATA"

    # Copying files into the newly created directories
    printf "\nCopying ATA files into ~/.local/share/ and ~/.local/bin"
    try cp ./install-prerequisites.sh "$HOME/.local/share/ATA"
    try cp ../target/release/ATA "$HOME/.local/bin"

    # Creating default data file
    printf "\nCreating data file in ~/.config\n"
    try touch "$HOME/.config/ATA/data.json"

    # Bash requires escaping double-quotes inside a double-quoted string with \".
    # The heredoc alternative (cat << EOF) exists but this mirrors the original closely.
    echo "{
        \"game_path\": \"$game_path\",
        \"mods\": []
    }" > "$HOME/.config/ATA/data.json"
}

# 'read -p "prompt" var' is the bash equivalent of fish's 'read -P "prompt" -l var'.
# The while true / break pattern is identical in both shells.
function user_action {
    printf "\n⚠️  Before continuing, please run steam's \"Integrity of game's files check\"."
    printf "\nHOWTO:
    - Open Automata's page from you steam library
    - Click the gear icon (⚙️) and select Properties
    - \"Installed files\" tab -> \"Verify integrity of game files\"
    - Let it run for however long it takes and then come back here"

    printf "\nType 'file check done' and press Enter when ready: "
    while true; do
        read -p "Type here> " user_input
        if [ "$user_input" = "file check done" ]; then
            break
        else
            printf "\nNot quite my \"file check done\""
        fi
    done
}

function reshade_setup {
    printf ""
    # Move ReShade dll into Proton prefix
    cd ../lib/
    try mkdir -p "$HOME/.local/share/Steam/steamapps/compatdata/524220/pfx/drive_c/users/steamuser/Documents/My Mods/SpecialK/PlugIns/ThirdParty/Reshade/"
    try cp ReShade64.dll "$HOME/.local/share/Steam/steamapps/compatdata/524220/pfx/drive_c/users/steamuser/Documents/My Mods/SpecialK/PlugIns/ThirdParty/Reshade/"

    # Creating ReShade folders
    try mkdir -p "$HOME/.local/share/Steam/steamapps/compatdata/524220/pfx/drive_c/users/steamuser/Documents/My Mods/SpecialK/Global/ReShade/Textures"
    try mkdir -p "$HOME/.local/share/Steam/steamapps/compatdata/524220/pfx/drive_c/users/steamuser/Documents/My Mods/SpecialK/Global/ReShade/Shaders"
    try cd "$HOME/.local/share/Steam/steamapps/compatdata/524220/pfx/drive_c/users/steamuser/Documents/My Mods/SpecialK/Global/ReShade/"

    # Copying ReShade default effects and textures from repo
    if [ -d reshade-shaders ]; then
        rm -rf reshade-shaders
    fi
    git clone https://github.com/crosire/reshade-shaders.git
    try cp reshade-shaders/Shaders/* Shaders/
    try cp reshade-shaders/Textures/* Textures/
    try rm -rf reshade-shaders
}

# 'while ! pgrep ...' is the direct bash equivalent of fish's 'while not pgrep ...'.
# pgrep exit codes: 0 = found, 1 = not found — same behavior across both shells.
function setup_finalization {
    printf "\nFinilazing installation...
    LET THE GAME START AND CLOSE IT FROM THE MAIN MENU"
    sleep 5

    # Launch game
    printf "\nLaunching the game\n"
    steam steam://rungameid/524220

    # Wait for the game process to start
    while ! pgrep -f NieRAutomata.exe > /dev/null; do
        printf "Waiting for game to start...\n"
        sleep 3
    done

    # Now wait for the game process to end
    while pgrep -f NieRAutomata.exe > /dev/null; do
        printf "Game started! You should see _wax loaded_ in the loading screen
        Close it from the main menu\n"
        sleep 3
    done

    printf "\nGame closed"
    sleep 2

    printf "\nRemoving framerate cap
    It is recommended to set a custom one using tools like MangoHud\n"
    sed -i 's/"uncap_fps": false/"uncap_fps": true/' "$game_path/wax/config.json"
}




# SCRIPT STARTS HERE
printf "Please go read the documentation if you haven't already\n"
sleep 2

# Check if the game's installation path was passed correctly
# "$@" passes all script arguments to the function (fish: $argv)
argument_check "$@"

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
