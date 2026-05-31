#Requires -Version 5.1

$steamAppsPath = "$env:USERPROFILE\AppData\Roaming\Steam\steamapps\common\NieRAutomata"
$configPath    = "$env:APPDATA\ATA"
$dataPath      = "$env:LOCALAPPDATA\ATA"

$utf8NoBom = [System.Text.UTF8Encoding]::new($false)

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
$data = [ordered]@{ mods = @() }
[System.IO.File]::WriteAllText(
    "$dataPath\data.json",
    ($data | ConvertTo-Json -Depth 10),
    $utf8NoBom
)

# settings.json
$settings = [ordered]@{
    style                    = "SilentShulk"
    palette                  = "Replicant"
    sortingOrder             = "ModType"
    filesConflictResolution  = "Ask"
    keepExtractedFolders     = $true
    extractedFoldersLocation = "$env:USERPROFILE\Downloads\"
    gamePath                 = "$steamAppsPath\"
    discordRichPresence      = "Altering NieRAutomata's timelines"
}
[System.IO.File]::WriteAllText(
    "$configPath\settings.json",
    ($settings | ConvertTo-Json -Depth 10),
    $utf8NoBom
)

Write-Host "Done." -ForegroundColor Green
