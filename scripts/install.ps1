#!/usr/bin/env pwsh
# Copyright 2019 the Deno authors. All rights reserved. MIT license.

$ErrorActionPreference = 'Stop'

$BinDir = "${Home}\.rgl\bin"

$RglZip = "$BinDir\rgl.zip"
$RglExe = "$BinDir\rgl.exe"

$Target = 'x86_64-pc-windows-msvc'
$DownloadUrl = "https://github.com/ink0rr/rgl/releases/latest/download/rgl-${Target}.zip"

if (!(Test-Path $BinDir)) {
  New-Item $BinDir -ItemType Directory | Out-Null
}

curl.exe -Lo $RglZip $DownloadUrl
tar.exe xf $RglZip -C $BinDir
Remove-Item $RglZip

$User = [System.EnvironmentVariableTarget]::User
$Path = [System.Environment]::GetEnvironmentVariable('Path', $User)
if (!(";${Path};".ToLower() -like "*;${BinDir};*".ToLower())) {
  [System.Environment]::SetEnvironmentVariable('Path', "${Path};${BinDir}", $User)
  $Env:Path += ";${BinDir}"
}

Write-Output ""
Write-Output "rgl was installed successfully to ${RglExe}"
Write-Output "Run 'rgl --help' to get started"
Write-Output ""
