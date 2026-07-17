$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $projectRoot

npm.cmd ci
npm.cmd test
npm.cmd run build
npm.cmd run tauri build

Write-Host "Installers are available under src-tauri\target\release\bundle"
