# ATA dev installer — Windows

$dataDir     = [Environment]::GetFolderPath("ApplicationData")       # %APPDATA%  (Roaming)
$localData   = [Environment]::GetFolderPath("LocalApplicationData")  # %LOCALAPPDATA%

# ATA dirs
New-Item -ItemType Directory -Force -Path "$localData\ATA\UIs"   | Out-Null
New-Item -ItemType Directory -Force -Path "$localData\ATA\Apps"  | Out-Null
New-Item -ItemType Directory -Force -Path "$localData\Programs\ATA" | Out-Null
New-Item -ItemType Directory -Force -Path "$dataDir\ATA"         | Out-Null
New-Item -ItemType Directory -Force -Path "$localData\ATA"       | Out-Null

# data.json  (WriteAllText = UTF-8 no BOM, serde_json-safe)
[System.IO.File]::WriteAllText("$dataDir\ATA\data.json", "{`n    `"mods`": []`n}`n")

# settings.json
$settings = @'
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
'@
[System.IO.File]::WriteAllText("$localData\ATA\settings.json", $settings)

Write-Host "ATA dev environment ready."
