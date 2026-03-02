# #! /bin/bash

# CHECK FOR ARGUMENT
if [ $# -eq 0 ]; then
  echo "REQUIRES ARGUMENT NOT FOUND.\n
  Run the installer again and pass the path to Automata's folder\n
  (the one containing the exe)";
  exit 1
fi



# Paths
$data_folder = $HOME/.config/ATA/
$files_folder = $HOME/.local/share/ATA
$bin_folder = /usr/bin



# Creating directories
mkdir -p $data_folder
mkdir -p $files_folder



# Copying files into the newly created directories
cp ./install-prerequisites.sh $files_folder
# Copy executable into $bin_folder



# Creating default data file

