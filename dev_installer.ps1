# ATA dev installer — Windows

$dataDir     = [Environment]::GetFolderPath("ApplicationData")       # %APPDATA%  (Roaming)
$localData   = [Environment]::GetFolderPath("LocalApplicationData")  # %LOCALAPPDATA%

# ATA dirs
New-Item -ItemType Directory -Force -Path "$localData\ATA\UIs"   | Out-Null
New-Item -ItemType Directory -Force -Path "$localData\ATA\Apps"  | Out-Null
New-Item -ItemType Directory -Force -Path "$localData\Programs\ATA" | Out-Null
New-Item -ItemType Directory -Force -Path "$dataDir\ATA"         | Out-Null
New-Item -ItemType Directory -Force -Path "$localData\ATA"       | Out-Null

# data.json
@'
{
    "mods": []
}
'@ | Set-Content -Path "$dataDir\ATA\data.json" -Encoding UTF8

# settings.json
@'
{
  "style": "SilentShulk",
  "palette": "Automata",
  "sortingOrder": "ModType",
  "filesConflictResolution": "Ask",
  "keepExtractedFolders": true,
  "extractedFoldersLocation": "",
  "gamePath": "",
  "discordRichPresence": "Altering NieRAutomata's timelines"
}
'@ | Set-Content -Path "$localData\ATA\settings.json" -Encoding UTF8

Write-Host "ATA dev environment ready."
