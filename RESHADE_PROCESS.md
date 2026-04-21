# How to install ReShade on Linux, assuming Steam installation and Proton

1. Get ReShade_Setup_6.7.3_Addon.exe (Download with curl??)
2. Use 7zip to extract ReShade64.dll - `7z e ReShade_Setup_6.7.3_Addon.exe ReShade64.dll`
3. Move it into Proton's prefix (/home/cmarco/.local/share/Steam/steamapps/compatdata/524220/pfx/drive_c/users/steamuser) inside *Documents/My Mods/SpecialK/PlugIns/ThirdParty/Reshade* (create folders from PluIns onwards)
    `mv ReShade64.dll /home/cmarco/.local/share/Steam/steamapps/compatdata/524220/pfx/drive_c/users/steamuser/Documents/My\ Mods/SpecialK/PlugIns/ThirdParty/Reshade/`
4. Create Reshade Shaders and Textures folders inside *drive_c/users/steamuser/Documents/My Mods/SpecialK/Global/ReShade* (Create Reshade folder)
    `mkdir -p /home/cmarco/.local/share/Steam/steamapps/compatdata/524220/pfx/drive_c/users/steamuser/Documents/My\ Mods/SpecialK/Global/ReShade/Textures`
    `mkdir -p /home/cmarco/.local/share/Steam/steamapps/compatdata/524220/pfx/drive_c/users/steamuser/Documents/My\ Mods/SpecialK/Global/ReShade/Shaders`
5. copy contents of https://github.com/crosire/reshade-shaders.git into respective folders
