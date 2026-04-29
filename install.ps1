# Install sgit on Windows via PowerShell
# Run: iwr -useb https://raw.githubusercontent.com/karandevhub/sgit/master/install.ps1 | iex

param([string]$Version = "latest")

$ErrorActionPreference = "Stop"
$Repo   = "karandevhub/sgit"
$Binary = "sgit"
$ApiUrl = "https://api.github.com/repos/$Repo/releases"

function Write-Info  { Write-Host "  -> $args" -ForegroundColor Cyan   }
function Write-Ok    { Write-Host "  v  $args" -ForegroundColor Green  }
function Write-Fail  { Write-Host "  x  $args" -ForegroundColor Red; exit 1 }

Write-Host ""
Write-Host "  sgit installer for Windows" -ForegroundColor White
Write-Host ""

# Resolve version
if ($Version -eq "latest") {
    Write-Info "Resolving latest version..."
    $rel     = Invoke-RestMethod "$ApiUrl/latest"
    $Version = $rel.tag_name -replace '^v',''
}
Write-Info "Version: v$Version"

# Platform detection (Windows = x86_64 only for now)
$platform = "windows-x86_64"
Write-Info "Platform: $platform"

$BaseUrl  = "https://github.com/$Repo/releases/download/v$Version"
$ZipName  = "$Binary-$platform.zip"
$SumName  = "$Binary-$platform.sha256"
$TmpDir   = Join-Path $env:TEMP "sgit-install-$(Get-Random)"
New-Item -ItemType Directory -Path $TmpDir | Out-Null

try {
    # Download
    Write-Info "Downloading $ZipName..."
    Invoke-WebRequest "$BaseUrl/$ZipName" -OutFile "$TmpDir\$ZipName" -UseBasicParsing
    Invoke-WebRequest "$BaseUrl/$SumName" -OutFile "$TmpDir\$SumName" -UseBasicParsing

    # Verify checksum
    Write-Info "Verifying checksum..."
    $Expected = (Get-Content "$TmpDir\$SumName" -Raw).Split(" ")[0].Trim()
    $Actual   = (Get-FileHash "$TmpDir\$ZipName" -Algorithm SHA256).Hash.ToLower()
    if ($Actual -ne $Expected) { Write-Fail "Checksum mismatch! Expected=$Expected Got=$Actual" }
    Write-Ok "Checksum verified"

    # Extract
    Expand-Archive "$TmpDir\$ZipName" -DestinationPath $TmpDir -Force

    # Install to %LOCALAPPDATA%\sgit\bin
    $InstallDir = "$env:LOCALAPPDATA\sgit\bin"
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    Copy-Item "$TmpDir\$Binary.exe" "$InstallDir\$Binary.exe" -Force

    # Add to PATH (user scope — no admin required)
    $CurrentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($CurrentPath -notlike "*$InstallDir*") {
        [Environment]::SetEnvironmentVariable("PATH", "$InstallDir;$CurrentPath", "User")
        Write-Info "Added $InstallDir to your PATH (restart your terminal to apply)"
    }

    Write-Host ""
    Write-Host "  v  sgit v$Version installed!" -ForegroundColor Green
    Write-Host ""
    Write-Host "  Quick start:"
    Write-Host "    sgit index             # build the search index" -ForegroundColor Cyan
    Write-Host "    sgit log `"auth bug`"    # semantic search"        -ForegroundColor Cyan
    Write-Host "    sgit log --help        # all options"             -ForegroundColor Cyan
    Write-Host ""
} finally {
    Remove-Item -Recurse -Force $TmpDir -ErrorAction SilentlyContinue
}
