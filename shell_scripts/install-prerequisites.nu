#!/usr/bin/env nu

# 'def main' is the nushell entry point for scripts with arguments (fish: $argv).
# The argument is declared as a typed parameter in the signature instead of
# being extracted from a positional array — this is idiomatic nushell and also
# gives you free type checking and --help generation.
def main [game_dir: string] {

    # CHECK FOR ARGUMENT
    # Nushell enforces this automatically via the typed signature above —
    # running the script without game_dir will print a usage error and exit.
    # The explicit check below is kept for a matching error message to the original.
    if ($game_dir | str length) == 0 {
        print "\nREQUIRED ARGUMENT NOT FOUND
    Run the installer again and pass the path to Automata's folder
    (the one containing the exe)"
        exit 1
    }



    # Installing Microsoft C++ tools
    # '> /dev/null 2>&1' becomes 'o+e>/dev/null' in nushell — the o+e> redirect
    # merges stdout and stderr and discards them (fish: > /dev/null 2>&1).
    ^wine ../bin/VC_redist.x64.exe o+e>/dev/null    # 64 bits
    ^wine ../bin/VC_redist.x86.exe o+e>/dev/null    # 32 bits



    # Copying modded files in game directory
    print "\nCopying modded files into game's directory\n"
    cp ../lib/d3d11.dll $game_dir     # Put WAX dll in game directory
    rm $"($game_dir)/NieRAutomata.exe"     # Remove original exe (will be readded by steam)
}
