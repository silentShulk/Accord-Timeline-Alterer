#! /bin/bash
# GO TO LINE 159 TO SEE WHAT THE SCRIPT DOES

function try {
    local output
    output=$(eval "$@" 2>&1)
    local code=$?

    if [ $code -ne 0 ]; then
        echo "\nError running: $@"
        echo "  $output"
        exit $code
    fi
}

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

function dependencies_installation {
    # Checking if FZF is installed
    if ! command -v fzf &> /dev/null; then
        printf "\nFZF required but not installed"

        local package_manager="pacman"
        for pm in apt dnf pacman brew; do
            if command -v "$pm" &> /dev/null; then
                package_manager=$pm
                printf "%s found as default package manager" "$pm"
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
    if ! command -v 7z &> /dev/null; then
        printf "\n7z required but not installed"

        local package_manager="pacman"
        for pm in apt dnf pacman brew; do
            if command -v "$pm" &> /dev/null; then
                package_manager=$pm
                printf "%s found as default package manager" "$pm"
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
    printf "\nCreating data file in ~/.config"
    try touch "$HOME/.config/ATA/data.json"

    echo "{
        \"game_path\": \"$game_path\",
        \"mods\": []
    }" > "$HOME/.config/ATA/data.json"
}

function modding_requirements_installation {
    printf "\nRunning script to install files needed to mod the game"
    ./install-prerequisites.sh "$game_path"
}

function specialk_auto_setup {
    printf "LET THE GAME START AND CLOSE IT FROM THE MAIN MENU"
    sleep 5

    # Launch game
    printf "\nLaunching the game\n"
    steam steam://rungameid/524220

    # Wait for the game process to start
    printf "Waiting for game to start...\n"
    while ! pgrep -x "steampp_524200" > /dev/null; do
        sleep 1
    done

    # Now wait for the game process to end
    printf "Game started! Close it from the main menu when ready.\n"
    while pgrep -x "steampp_524200" > /dev/null; do
        sleep 1
    done

    printf "Game closed, continuing installation...\n"
}

function reshade_setup {
    # Extracting dll from installer exe
    7z e ../bin/ReShade_Setup_6.7.3_Addon.exe ReShade64.dll

    # Move into Proton prefix
    try mv ReShade64.dll "$HOME/.local/share/Steam/steamapps/compatdata/524220/pfx/drive_c/users/steamuser/Documents/My Mods/SpecialK/PlugIns/ThirdParty/Reshade/"

    # Creating ReShade folders
    try mkdir -p "$HOME/.local/share/Steam/steamapps/compatdata/524220/pfx/drive_c/users/steamuser/Documents/My Mods/SpecialK/Global/ReShade/Textures"
    try mkdir -p "$HOME/.local/share/Steam/steamapps/compatdata/524220/pfx/drive_c/users/steamuser/Documents/My Mods/SpecialK/Global/ReShade/Shaders"
    cd "$HOME/.local/share/Steam/steamapps/compatdata/524220/pfx/drive_c/users/steamuser/Documents/My Mods/SpecialK/Global/ReShade/" || exit 1

    # Copying ReShade default effects and textures from repo
    git clone https://github.com/crosire/reshade-shaders.git
    try cp reshade-shaders/Shaders/* Shaders/
    try cp reshade-shaders/Textures/* Textures/
    try rm -rf reshade-shaders
}




# SCRIPT STARTS HERE
printf "Please go read the documentation if you haven't already\n"

# Check if the game's installation path was passed correctly
argument_check "$@"

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
