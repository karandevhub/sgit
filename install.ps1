# Install sgit on Windows

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

if ($Version -eq "latest") {
    Write-Info "Resolving latest version..."
    $rel     = Invoke-RestMethod "$ApiUrl/latest"
    $Version = $rel.tag_name -replace '^v',''
}
Write-Info "Version: v$Version"

$platform = "windows-x86_64"
Write-Info "Platform: $platform"

$BaseUrl  = "https://github.com/$Repo/releases/download/v$Version"
$ZipName  = "$Binary-$platform.zip"
$SumName  = "$ZipName.sha256"
$TmpDir   = Join-Path $env:TEMP "sgit-install-$(Get-Random)"
New-Item -ItemType Directory -Path $TmpDir | Out-Null

try {
    Write-Info "Downloading $ZipName..."
    Invoke-WebRequest "$BaseUrl/$ZipName" -OutFile "$TmpDir\$ZipName" -UseBasicParsing
    Invoke-WebRequest "$BaseUrl/$SumName" -OutFile "$TmpDir\$SumName" -UseBasicParsing

    Write-Info "Verifying checksum..."
    $Expected = (Get-Content "$TmpDir\$SumName" -Raw).Split(" ")[0].Trim()
    $Actual   = (Get-FileHash "$TmpDir\$ZipName" -Algorithm SHA256).Hash.ToLower()
    if ($Actual -ne $Expected) { Write-Fail "Checksum mismatch! Expected=$Expected Got=$Actual" }
    Write-Ok "Checksum verified"

    Expand-Archive "$TmpDir\$ZipName" -DestinationPath $TmpDir -Force

    $InstallDir = "$env:LOCALAPPDATA\sgit\bin"
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    Copy-Item "$TmpDir\$Binary.exe" "$InstallDir\$Binary.exe" -Force

    $CurrentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($CurrentPath -notlike "*$InstallDir*") {
        [Environment]::SetEnvironmentVariable("PATH", "$InstallDir;$CurrentPath", "User")
        Write-Info "Added $InstallDir to your PATH"
    }
    
    if ($env:PATH -notlike "*$InstallDir*") {
        $env:PATH = "$InstallDir;$env:PATH"
    }
    Write-Host ""
    Write-Host "  v  sgit v$Version installed!" -ForegroundColor Green
    Write-Host ""
    Write-Host "  Quick start:"
    Write-Host "    sgit index             # build the search index" -ForegroundColor Cyan
    Write-Host "    sgit log `"auth bug`"    # semantic search"        -ForegroundColor Cyan
    Write-Host "    sgit log --help        # all options"             -ForegroundColor Cyan
    Write-Host ""

    try {
        $null = Invoke-WebRequest -Uri "https://hits.seeyoufarm.com/api/count/incr/badge.svg?url=https%3A%2F%2Fgithub.com%2Fkarandevhub%2Fsgit%2Fdownload&count_bg=%230099CC&title_bg=%23555555&title=downloads&edge_flat=false" -UseBasicParsing -ErrorAction SilentlyContinue
    } catch {}

} finally {
    Remove-Item -Recurse -Force $TmpDir -ErrorAction SilentlyContinue
}
