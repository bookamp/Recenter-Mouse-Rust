param (
    [string]$Version = ""
)

$ErrorActionPreference = "Stop"

$ProjectRoot = Resolve-Path "$PSScriptRoot\.."
Set-Location $ProjectRoot

# Detect version from Cargo.toml if not specified
if (-not $Version) {
    $CargoContent = Get-Content "Cargo.toml" -Raw
    if ($CargoContent -match 'version\s*=\s*"([^"]+)"') {
        $Version = $matches[1]
    } else {
        $Version = "1.0.0"
    }
}

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host " Packaging Recenter Mouse v$Version" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# 1. Compile Release Binary
Write-Host "`n[1/3] Building release binary..." -ForegroundColor Green
cargo build --release
if ($LASTEXITCODE -ne 0) {
    throw "Cargo release build failed."
}

$ReleaseExe = "target\release\recenter-mouse.exe"
if (-not (Test-Path $ReleaseExe)) {
    throw "Binary $ReleaseExe not found."
}

# 2. Package Portable ZIP
Write-Host "`n[2/3] Creating portable ZIP package..." -ForegroundColor Green
$PortableDir = "target\portable-staging"
if (Test-Path $PortableDir) {
    Remove-Item $PortableDir -Recurse -Force
}
New-Item -ItemType Directory -Path $PortableDir | Out-Null

Copy-Item $ReleaseExe -Destination $PortableDir
Copy-Item "README.md" -Destination $PortableDir
if (Test-Path "LICENSE") {
    Copy-Item "LICENSE" -Destination $PortableDir
}

$ZipName = "recenter-mouse-v$Version-windows-x64-portable.zip"
$ZipPath = "target\release\$ZipName"
if (Test-Path $ZipPath) {
    Remove-Item $ZipPath -Force
}

Compress-Archive -Path "$PortableDir\*" -DestinationPath $ZipPath -Force
Remove-Item $PortableDir -Recurse -Force
Write-Host "  -> Created portable package: $ZipPath" -ForegroundColor Yellow

# 3. Package MSI Installer (via WiX Toolset)
Write-Host "`n[3/3] Building MSI installer..." -ForegroundColor Green
$MsiName = "recenter-mouse-v$Version-x64.msi"
$MsiPath = "target\release\$MsiName"

if (Get-Command wix -ErrorAction SilentlyContinue) {
    wix build -ext WixToolset.UI.wixext wix/main.wxs -o $MsiPath
    if ($LASTEXITCODE -eq 0 -and (Test-Path $MsiPath)) {
        Write-Host "  -> Created MSI package: $MsiPath" -ForegroundColor Yellow
    } else {
        Write-Warning "WiX build failed."
    }
} else {
    Write-Warning "WiX toolset command 'wix' not found on PATH. Skipping MSI build."
}

Write-Host "`nPackaging completed successfully!" -ForegroundColor Cyan
Write-Host "Artifacts:" -ForegroundColor Cyan
Get-Item $ZipPath, $MsiPath -ErrorAction SilentlyContinue | Select-Object Name, Length, LastWriteTime | Format-Table -AutoSize
