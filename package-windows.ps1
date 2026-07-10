# Builds a distributable Windows bundle for voxelcraft.
#
# Usage:   powershell -ExecutionPolicy Bypass -File package-windows.ps1
# Output:  dist/voxelcraft-windows/            (the runnable folder)
#          dist/voxelcraft-windows-x64.zip     (zipped for sharing)
#
# voxelcraft has no external asset files (all textures are generated in code),
# so the bundle is just the release .exe plus a short readme. Saves are written
# next to the .exe as world_N.sav.

$ErrorActionPreference = "Stop"
$root = $PSScriptRoot
$out  = Join-Path $root "dist\voxelcraft-windows"

Write-Host "Building release binary..." -ForegroundColor Cyan
cargo build --release
if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }

$exe = Join-Path $root "target\release\voxelcraft.exe"
if (-not (Test-Path $exe)) { throw "expected binary not found: $exe" }

Write-Host "Staging bundle at $out" -ForegroundColor Cyan
if (Test-Path $out) { Remove-Item -Recurse -Force $out }
New-Item -ItemType Directory -Force -Path $out | Out-Null

Copy-Item $exe $out
Copy-Item (Join-Path $root "README.md") $out

@"
voxelcraft — Windows build
==========================

To play: double-click voxelcraft.exe

Your world is saved next to the .exe as world_1.sav .. world_3.sav.
See README.md for full controls.

Requires a 64-bit Windows PC with a GPU that supports Vulkan or DirectX 12
(any reasonably modern machine). No installation needed.
"@ | Set-Content -Path (Join-Path $out "PLAY-ME.txt") -Encoding UTF8

$zip = Join-Path $root "dist\voxelcraft-windows-x64.zip"
if (Test-Path $zip) { Remove-Item -Force $zip }
Write-Host "Zipping to $zip" -ForegroundColor Cyan
Compress-Archive -Path "$out\*" -DestinationPath $zip

Write-Host "Done." -ForegroundColor Green
Write-Host "  Folder: $out"
Write-Host "  Zip:    $zip"
