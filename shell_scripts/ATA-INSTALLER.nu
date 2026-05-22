#!/usr/bin/env nu
# GO TO LINE 159 TO SEE WHAT THE SCRIPT DOES

# IMPORTANT: 'try' is a reserved keyword in nushell (its built-in try/catch mechanism),
# so the helper is named 'must' instead — same semantics: run or die.
# 'run-external' is used to call external OS commands by dynamic name.
# '| complete' captures the exit code without nushell throwing its own error first,
# giving us control over the error message and exit code ourselves.
def must [...args: string] {
    let result = (do { run-external $args.0 ...($args | skip 1) } | complete)
    if $result.exit_code != 0 {
        print $"Error running: ($args | str join ' ')"
        exit $result.exit_code
    }
}

# Nushell functions declare their parameters in the signature (fish uses $argv implicitly).
# The function returns a value with the last expression — here it returns the validated path,
# which the caller assigns with 'let game_path = (argument_check ...)'.
# '$"(...)"' is nushell string interpolation (fish: "$variable" or (command)).
# 'path exists' is the nushell built-in to check for file/dir existence (fish: test -f).
def argument_check [args: list<string>]: nothing -> string {
    if ($args | length) == 0 {
        print "REQUIRED ARGUMENT NOT FOUND
        Run the installer again and pass the path to Automata's folder
        (the one containing the exe)"
        exit 1
    }

    # CHECK IF GIVEN PATH IS ACTUALLY GAME PATH
    if not ($"($args.0)/NieRAutomata.exe" | path exists) {
        print "GIVEN PATH ISN'T GAME PATH
        It does not contain the NieRAutomata.exe
        Run the installer again and pass the path to Automata's folder"
        exit 1
    }

    $args.0
}

# 'which cmd' returns a table of results — 'is-empty' checks if nothing was found,
# equivalent to fish's 'type -q cmd'.
# 'mut' declares a mutable variable (nushell variables are immutable by default with 'let').
# 'match' is the nushell equivalent of fish's switch/case/end.
# '^command' explicitly runs an external command (the ^ prevents shadowing by nushell built-ins).
def dependencies_installation [] {
    # Checking if FZF is installed
    if (which fzf | is-empty) {
        print "\nFZF required but not installed"

        mut package_manager = "pacman"
        for pm in [apt dnf pacman brew] {
            if not (which $pm | is-empty) {
                $package_manager = $pm
                print $"($pm) found as default package manager"
                break
            }
        }

        match $package_manager {
            "pacman" => { ^sudo pacman -S --noconfirm fzf }
            "apt"    => { ^sudo apt install -y fzf }
            "dnf"    => { ^sudo dnf install -y fzf }
            "brew"   => { ^brew install fzf }
        }
    }

    # Checking if 7zip is installed
    if (which 7z | is-empty) {
        print "\n7zip required but not installed"

        mut package_manager = "pacman"
        for pm in [apt dnf pacman brew] {
            if not (which $pm | is-empty) {
                $package_manager = $pm
                print $"($pm) found as default package manager"
                break
            }
        }

        match $package_manager {
            "pacman" => { ^sudo pacman -S --noconfirm p7zip }
            "apt"    => { ^sudo apt install -y p7zip-full }
            "dnf"    => { ^sudo dnf install -y p7zip }
            "brew"   => { ^brew install p7zip }
        }
    }
}

# game_path is passed as an explicit parameter instead of being a global variable.
# Nushell does not have mutable global variables — state is passed through function arguments.
def modding_requirements_installation [game_path: string] {
    print "\nRunning script to install files needed to mod the game"
    ^./install-prerequisites.nu $game_path
}

# '$env.HOME' is the nushell equivalent of $HOME (fish: $HOME).
# 'mkdir', 'cp', 'touch', 'rm' are nushell built-ins — they throw structured errors
# on failure automatically, so 'must' is not needed for them here.
# 'save -f' writes (and overwrites) a file from a pipeline (fish: echo "..." > file).
def ATA_setup [game_path: string] {
    # Creating directories
    print "\nCreating directories in ~/.local and ~/.config"
    mkdir $"($env.HOME)/.local/share/ATA"
    mkdir $"($env.HOME)/.local/bin"
    mkdir $"($env.HOME)/.config/ATA"

    # Copying files into the newly created directories
    print "\nCopying ATA files into ~/.local/share/ and ~/.local/bin"
    cp ./install-prerequisites.nu $"($env.HOME)/.local/share/ATA"
    cp ../target/release/ATA $"($env.HOME)/.local/bin"

    # Creating default data file
    print "\nCreating data file in ~/.config\n"
    touch $"($env.HOME)/.config/ATA/data.json"

    $"{
        \"game_path\": \"($game_path)\",
        \"mods\": []
    }" | save -f $"($env.HOME)/.config/ATA/data.json"
}

# 'input "prompt"' reads a line from stdin and returns it as a string (fish: read -P -l).
# The while loop condition uses a mutable variable since nushell requires explicit mutability.
def user_action [] {
    print "\n⚠️  Before continuing, please run steam's \"Integrity of game's files check\"."
    print "\nHOWTO:
    - Open Automata's page from you steam library
    - Click the gear icon (⚙️) and select Properties
    - \"Installed files\" tab -> \"Verify integrity of game files\"
    - Let it run for however long it takes and then come back here"

    print "\nType 'file check done' and press Enter when ready: "
    mut user_input = ""
    while $user_input != "file check done" {
        $user_input = (input "Type here> ")
        if $user_input != "file check done" {
            print "\nNot quite my \"file check done\""
        }
    }
}

# NOTE: In nushell, 'cd' inside a function only changes the working directory for
# that function's scope — it does NOT affect the caller, unlike bash/fish.
# This is by design in nushell (lexical scoping for $env.PWD).
def reshade_setup [] {
    print ""
    # Move ReShade dll into Proton prefix
    cd ../lib/
    mkdir $"($env.HOME)/.local/share/Steam/steamapps/compatdata/524220/pfx/drive_c/users/steamuser/Documents/My Mods/SpecialK/PlugIns/ThirdParty/Reshade/"
    cp ReShade64.dll $"($env.HOME)/.local/share/Steam/steamapps/compatdata/524220/pfx/drive_c/users/steamuser/Documents/My Mods/SpecialK/PlugIns/ThirdParty/Reshade/"

    # Creating ReShade folders
    mkdir $"($env.HOME)/.local/share/Steam/steamapps/compatdata/524220/pfx/drive_c/users/steamuser/Documents/My Mods/SpecialK/Global/ReShade/Textures"
    mkdir $"($env.HOME)/.local/share/Steam/steamapps/compatdata/524220/pfx/drive_c/users/steamuser/Documents/My Mods/SpecialK/Global/ReShade/Shaders"
    cd $"($env.HOME)/.local/share/Steam/steamapps/compatdata/524220/pfx/drive_c/users/steamuser/Documents/My Mods/SpecialK/Global/ReShade/"

    # Copying ReShade default effects and textures from repo
    if ("reshade-shaders" | path exists) {
        rm -rf reshade-shaders
    }
    ^git clone https://github.com/crosire/reshade-shaders.git
    cp reshade-shaders/Shaders/* Shaders/
    cp reshade-shaders/Textures/* Textures/
    rm -rf reshade-shaders
}

# 'sleep 5sec' — nushell requires a duration unit suffix (fish: sleep 5).
# 'do { ^pgrep ... } | complete' captures the exit code without nushell erroring out,
# equivalent to fish's 'pgrep ... > /dev/null' (exit code check only).
def setup_finalization [game_path: string] {
    print "\nFinilazing installation...
    LET THE GAME START AND CLOSE IT FROM THE MAIN MENU"
    sleep 5sec

    # Launch game
    print "\nLaunching the game\n"
    ^steam steam://rungameid/524220

    # Wait for the game process to start
    while (do { ^pgrep -f NieRAutomata.exe } | complete).exit_code != 0 {
        print "Waiting for game to start...\n"
        sleep 3sec
    }

    # Now wait for the game process to end
    while (do { ^pgrep -f NieRAutomata.exe } | complete).exit_code == 0 {
        print "Game started! You should see _wax loaded_ in the loading screen
        Close it from the main menu\n"
        sleep 3sec
    }

    print "\nGame closed"
    sleep 2sec

    print "\nRemoving framerate cap
    It is recommended to set a custom one using tools like MangoHud\n"
    ^sed -i 's/"uncap_fps": false/"uncap_fps": true/' $"($game_path)/wax/config.json"
}




# SCRIPT STARTS HERE
# 'def main' is the nushell-idiomatic entry point for scripts that accept arguments.
# It replaces the implicit top-level $argv of fish. Nushell calls main() automatically
# when the script is run, passing CLI arguments as typed parameters.
def main [game_path: string] {
    print "Please go read the documentation if you haven't already\n"
    sleep 2sec

    # Check if the game's installation path was passed correctly
    let validated_path = (argument_check [$game_path])

    # Installing dependencies (fzf, 7z)
    dependencies_installation

    # Installing required modding files (WAX dll, MCppBT)
    modding_requirements_installation $validated_path

    # Setupping folders and copying files
    ATA_setup $validated_path

    # Let user run file check
    user_action

    # Creating ReShade files and folders, cloning default effects/textures
    # reshade_setup

    # Starting the game to let WAX create files
    setup_finalization $validated_path



    print "\nCheck you game dir, there should now be:
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



    print "\n\nInstallation complete, make sure ~/.local/bin is in your PATH
Do not touch the ATA folders inside ~/.config, ~/.local/share and ~/.local/bin\n"
}
