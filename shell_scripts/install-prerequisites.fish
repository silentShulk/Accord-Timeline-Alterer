#! /bin/fish

# CHECK FOR ARGUMENT
if test (count $argv) -eq 0
    printf "\nREQUIRED ARGUMENT NOT FOUND
    Run the installer again and pass the path to Automata's folder
    (the one containing the exe)"
    exit 1
end

set game_dir "$argv[1]"



# Installing Microsoft C++ tools
wine ../bin/VC_redist.x64.exe > /dev/null 2>&1    # 64 bits
wine ../bin/VC_redist.x86.exe > /dev/null 2>&1    # 32 bits



# Copying modded files in game directory
printf "\nCopying modded files into game's directory\n"
cp ../lib/d3d11.dll "$game_dir"     # Put WAX dll in game directory
rm "$game_dir/NieRAutomata.exe"     # Remove original exe (will be readded by steam)