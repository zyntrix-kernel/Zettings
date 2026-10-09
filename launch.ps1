# Launches Zettings desktop application detached in the background
$releaseDir = Join-Path $PSScriptRoot "build\windows\x64\runner\Release"
Start-Process -FilePath (Join-Path $releaseDir "zettings.exe") -WorkingDirectory $releaseDir
