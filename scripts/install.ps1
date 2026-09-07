$ErrorActionPreference = "Stop"

$InstallDir = if ($env:JULIX_HOME) { $env:JULIX_HOME } else { "$env:USERPROFILE\.julix" }
$BinDir = "$InstallDir\bin"
$Version = if ($env:JULIX_VERSION) { $env:JULIX_VERSION } else { "latest" }
$Repo = "furimeo/julix"

$Platform = "windows"
$Arch = if ([System.Environment]::Is64BitOperatingSystem) { "x86_64" } else { "x86" }

Write-Host "Installing Julix $Version for $Arch-$Platform..."

New-Item -ItemType Directory -Force -Path $BinDir | Out-Null

if ($Version -eq "latest") {
    $Url = "https://github.com/$Repo/releases/latest/download/julix-$Arch-$Platform.zip"
} else {
    $Url = "https://github.com/$Repo/releases/download/$Version/julix-$Arch-$Platform.zip"
}

$Tmp = New-Item -ItemType Directory -Force -Path "$env:TEMP\julix-install"
Invoke-WebRequest -Uri $Url -OutFile "$Tmp\julix.zip"
Expand-Archive -Path "$Tmp\julix.zip" -DestinationPath $Tmp -Force
Copy-Item "$Tmp\julix.exe" "$BinDir\julix.exe" -Force

cmd /c mklink /H "$BinDir\lixvm.exe" "$BinDir\julix.exe"
cmd /c mklink /H "$BinDir\jujit.exe" "$BinDir\julix.exe"

Remove-Item -Recurse -Force $Tmp

$UserPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$BinDir*") {
    [System.Environment]::SetEnvironmentVariable("Path", "$UserPath;$BinDir", "User")
    Write-Host "Added $BinDir to user PATH. Restart your terminal."
}

Write-Host "Done. Julix installed to $BinDir\julix.exe"
& "$BinDir\julix.exe" --version
