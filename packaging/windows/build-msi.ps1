# Build Windows MSI using WiX (latest, e.g. 6.x). Run from repo root.
# Requires: $env:SourceDir, $env:Version. Optional: AppName, Manufacturer, UpgradeCode, LicenseRtfPath, IconsDir.
# If packaging/icons/react.ico is missing, generates it from packaging/icons/react.png (requires System.Drawing).
param()

$ErrorActionPreference = 'Stop'
$RepoRoot = if ($env:REPO_ROOT) { $env:REPO_ROOT } else { (Get-Location).Path }
$SourceDir = $env:SourceDir
$Version = $env:Version
$AppName = $env:APP_NAME ?? 'Desktop Runtime'
$Manufacturer = $env:MANUFACTURER ?? 'Desktop Runtime'
$UpgradeCode = $env:UPGRADE_CODE ?? 'B7A1A2D3-E4F5-6789-0ABC-DEF123456789'
$IconsDir = $env:IconsDir ?? (Join-Path $RepoRoot 'packaging\icons')
$LicenseRtfPath = $env:LicenseRtfPath ?? (Resolve-Path (Join-Path $RepoRoot 'packaging\windows\License.rtf')).Path

if (-not $SourceDir) { Write-Error 'SourceDir (env) is required (path to directory containing desktop-runtime-core.exe)' }
if (-not $Version) { Write-Error 'Version (env) is required' }
$SourceDir = (Resolve-Path $SourceDir).Path

$PngPath = Join-Path $IconsDir 'react.png'
$IcoPath = Join-Path $IconsDir 'react.ico'

# Generate react.ico from react.png if missing
if (-not (Test-Path $IcoPath)) {
  if (Test-Path $PngPath) {
    Write-Host '>> Generating react.ico from react.png'
    try {
      Add-Type -AssemblyName System.Drawing
      $img = [System.Drawing.Image]::FromFile($PngPath)
      $size = [Math]::Min(256, [Math]::Min($img.Width, $img.Height))
      $bmp = New-Object System.Drawing.Bitmap($size, $size)
      $g = [System.Drawing.Graphics]::FromImage($bmp)
      $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
      $g.DrawImage($img, 0, 0, $size, $size)
      $g.Dispose()
      $img.Dispose()
      $stream = [System.IO.File]::Create($IcoPath)
      $icon = [System.Drawing.Icon]::FromHandle($bmp.GetHicon())
      $icon.Save($stream)
      $stream.Close()
      $icon.Dispose()
      $bmp.Dispose()
      Write-Host '>> Created react.ico'
    } catch {
      Write-Error "Failed to generate .ico from PNG: $_"
    }
  } else {
    Write-Error "react.ico not found at $IcoPath and react.png not found at $PngPath"
  }
}

$IconsDir = (Resolve-Path $IconsDir).Path
$OutDir = Join-Path $RepoRoot 'target\wix'
$OutMsi = Join-Path $OutDir "desktop-runtime-$Version-x86_64-pc-windows-msvc.msi"
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

# Ensure UI extension is in the wix cache (required for -ext WixToolset.UI.wixext).
# Pin to 6.x so we don't pull a WiX 7 prerelease when using wix 6.
Write-Host '>> Adding WixToolset.UI.wixext 6.0.2 to wix extension cache'
& wix extension add WixToolset.UI.wixext/6.0.2
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host ">> Building MSI for version $Version"
$wixArgs = @(
  'build',
  '-ext', 'WixToolset.UI.wixext/6.0.2',
  '-d', "SourceDir=$SourceDir",
  '-d', "Version=$Version",
  '-d', "AppName=$AppName",
  '-d', "Manufacturer=$Manufacturer",
  '-d', "UpgradeCode=$UpgradeCode",
  '-d', "LicenseRtfPath=$LicenseRtfPath",
  '-d', "IconsDir=$IconsDir",
  '-o', $OutMsi,
  (Join-Path $RepoRoot 'packaging\windows\main.wxs')
)
& wix @wixArgs
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host ">> Produced: $OutMsi"
