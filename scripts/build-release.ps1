# Builds tgscanner.exe into .\target\release (NOT Cursor sandbox CARGO_TARGET_DIR cache).
$ErrorActionPreference = "Stop"
$ProjectRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $ProjectRoot
Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
$targetDir = Join-Path $ProjectRoot "target"
cargo build --release --locked --target-dir $targetDir
$exe = Join-Path $targetDir "release\tgscanner.exe"
Get-Item -LiteralPath $exe | Format-List FullName, Length, LastWriteTime
Write-Host "Run: $exe"
