# Builds tgscanner.exe into .\target\release (NOT Cursor sandbox cache).
$ErrorActionPreference = "Stop"
Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
Set-Location (Join-Path $PSScriptRoot "..")
cargo build --release --locked
$exe = Join-Path (Get-Location) "target\release\tgscanner.exe"
Get-Item -LiteralPath $exe | Format-List FullName, Length, LastWriteTime
Write-Host "Run: $exe"
