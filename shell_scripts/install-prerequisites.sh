#! /bin/bash

# CHECK FOR ARGUMENT
if [ $# -eq 0 ]; then
    printf "\nREQUIRED ARGUMENT NOT FOUND
    Run the installer again and pass the path to Automata's folder
    (the one containing the exe)"
    exit 1
fi

game_dir="$1"



# Installing Microsoft C++ tools
printf "\nInstalling files needed to mod the game"
wine ../bin/VC_redist.x64.exe     # 64 bits
wine ../bin/VC_redist.x86.exe     # 32 bits



# Copying modded files in game directory
printf "\nCopying modded files into game's directory"
mv "$game_dir/NieRAutomata.exe" "$game_dir/NieRAutomata(original).exe"   # Change the name of the default exe
cp ../bin/NieRAutomata.exe "$game_dir"                                   # Put the WolfFileSizeLimitBreaker exe in the game directory
cp ../lib/d3d11.dll "$game_dir"                                          # Put SpecialK dll in game directory



# Launch game
printf "\nLaunching the game"
steam steam://rungameid/524220



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
