# ATA dev installer — Windows

# Remove folders for mod files (will be recreated by ATA if necessary)
# This doesn't affect a working installation of the game
$modPaths = @(
    "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/data/pl",
    "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/data/wp",
    "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/data/bg",
    "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/data/wax"
)

foreach ($path in $modPaths)
{
    if (Test-Path $path)
    {
        rm -r -f $path
    }
}



# Create folders strictly necessary for development testing
mkdir "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/data" -Force | Out-Null
mkdir "$HOME/.local/share/Steam/steamapps/common/NieRAutomata/wax/mods" -Force | Out-Null



# Directories used by ATA
$exe      = "$env:LOCALAPPDATA\Programs\ATA"
$data     = "$env:LOCALAPPDATA\ATA"
$settings = "$env:APPDATA\ATA"
$uis      = "$env:LOCALAPPDATA\ATA\UIs"
$apps     = "$env:LOCALAPPDATA\ATA\Apps"

mkdir $exe -Force | Out-Null
mkdir $data -Force | Out-Null
mkdir $settings -Force | Out-Null
mkdir $uis -Force | Out-Null
mkdir $apps -Force | Out-Null



# Insert default content inside data and settings

# data.json
[System.IO.File]::WriteAllText("$data\data.json", @'
{
    "mods": []
}
'@, [System.Text.UTF8Encoding]::new($false))
 
# settings.json
# Pre-escape the $HOME path so it retains double backslashes in the JSON
$escapedHome = $HOME -replace '\\', '\\\\'
 
[System.IO.File]::WriteAllText("$settings\settings.json", @"
{
  "style": "SilentShulk",
  "palette": "Automata",
  "sortingOrder": "ModType",
  "filesConflictResolution": "Ask",
  "keepExtractedFolders": true,
  "extractedFoldersLocation": "${escapedHome}\\\\Downloads",
  "gamePath": "C:\\\\Program Files (x86)\\\\Steam\\\\steamapps\\\\common\\\\NieRAutomata",
  "discordRichPresence": "Altering NieRAutomata's timelines"
}
"@, [System.Text.UTF8Encoding]::new($false))


Write-Host "ATA dev environment ready." -ForegroundColor Green