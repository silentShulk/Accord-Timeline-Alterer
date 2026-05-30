#Requires -Version 5.1

$steamAppsPath = "$env:USERPROFILE\AppData\Roaming\Steam\steamapps\common\NieRAutomata"
$configPath    = "$env:APPDATA\ATA"
$dataPath      = "$env:LOCALAPPDATA\ATA"

# Remove old dirs
@("data\pl", "data\wp", "data\bg") | ForEach-Object {
    $target = Join-Path $steamAppsPath $_
    if (Test-Path $target) {
        Remove-Item -Recurse -Force $target
    }
}

# Create dirs
@(
    (Join-Path $steamAppsPath "data"),
    (Join-Path $steamAppsPath "wax\mods"),
    $configPath,
    $dataPath
) | ForEach-Object {
    New-Item -ItemType Directory -Force -Path $_ | Out-Null
}

# data.json
@'
{
    "mods": []
}
'@ | Set-Content -Path "$dataPath\data.json" -Encoding UTF8

# settings.json
@"
{
  "style": "SilentShulk",
  "palette": "Replicant",
  "sortingOrder": "ModType",
  "filesConflictResolution": "Ask",
  "keepExtractedFolders": true,
  "extractedFoldersLocation": "$env:USERPROFILE\\Downloads\\",
  "gamePath": "$($steamAppsPath.Replace('\','\\'))\\",
  "discordRichPresence": "Altering NieRAutomata's timelines"
}
"@ | Set-Content -Path "$configPath\settings.json" -Encoding UTF8

Write-Host "Done." -ForegroundColor Green
