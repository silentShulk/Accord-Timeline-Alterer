#!/bin/bash

# ---------------------------------- #
#   INSTALLING MICROSOFT C++ TOOLS   #
# ---------------------------------- #

# Install 64 bit
./bin/VC_redist.x64.exe

#Install 32 bit
./bin/VC_redist.x86.exe



# ---------------------------- #
#   Preparing game directory   #
# ---------------------------- #

# Default game directory for most distros
game_dir="$HOME/.local/share/Steam/steamapps/common/NieRAutomata"

# Reassigning default game directory to the first argument
if [ "$#" -gt 0 ]; then
  game_dir="$1"
fi

# Stopping early if no game directory is found
if [ ! -d "$game_dir" ]; then
  echo "Game directory not found in $game_dir"
  exit 1
fi



# -------------------------------- #
#   CHANGING THE GAME EXECUTABLE   #
# -------------------------------- #

# Change the name of the default exe
mv "$game_dir/NieRAutomata.exe" "$game_dir/NieRAutomata(original).exe"

# Put the WolfLimitBreaker exe in the game directory
cp ./bin/NieRAutomata.exe "$game_dir"



# ------------- #
#   SPECIAL K   #
# ------------- #

# Put the dll in the games's directory
cp ./lib/d3d11.dll "$game_dir"



# --------------- #
#   LAUNCH GAME   #
# --------------- #

# Launch game
steam steam://rungameid/524220
