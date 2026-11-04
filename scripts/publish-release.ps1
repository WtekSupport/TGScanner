# Creates GitHub Release for existing tag and uploads Windows binary (uses git credential).
param(
    [string]$Tag = "v1.0.1",
    [string]$AssetPath = "$PSScriptRoot\..\dist\tgscanner-windows-x86_64.exe",
    [string]$AssetName = "tgscanner-windows-x86_64.exe"
)

$ErrorActionPreference = "Stop"

Push-Location (Join-Path $PSScriptRoot "..")
try {
    $credText = @"
protocol=https
host=github.com

"@ | git credential fill 2>$null
    $token = ($credText -split "`n" | Where-Object { $_ -like "password=*" }) -replace "^password=", "" | Select-Object -First 1
}
finally {
    Pop-Location
}
if (-not $token) {
    throw "No git credential for github.com — run git push once or set a PAT."
}

$headers = @{
    Authorization      = "Bearer $token"
    Accept             = "application/vnd.github+json"
    "X-GitHub-Api-Version" = "2022-11-28"
}

$bodyText = @"
## TGScanner $Tag

First **1.x** release — desktop lookup tool for Telegram.

See [CHANGELOG.md](https://github.com/WtekSupport/TGScanner/blob/$Tag/CHANGELOG.md) for full notes.

**Windows:** ``$AssetName`` below.
"@

$payload = @{
    tag_name = $Tag
    name     = "TGScanner $($Tag.TrimStart('v'))"
    body     = $bodyText
    draft    = $false
    prerelease = $false
} | ConvertTo-Json

try {
    $release = Invoke-RestMethod -Method Post `
        -Uri "https://api.github.com/repos/WtekSupport/TGScanner/releases" `
        -Headers $headers -Body $payload -ContentType "application/json; charset=utf-8"
}
catch {
    $resp = $_.ErrorDetails.Message
    if ($resp -and ($resp -match "already_exists")) {
        $release = Invoke-RestMethod `
            -Uri "https://api.github.com/repos/WtekSupport/TGScanner/releases/tags/$Tag" `
            -Headers $headers
    }
    else {
        throw
    }
}

if (-not (Test-Path -LiteralPath $AssetPath)) {
    throw "Asset not found: $AssetPath"
}

$uploadBase = ($release.upload_url -replace "\{.*$", "")
$uploadUri = "${uploadBase}?name=$AssetName"
$uploadHeaders = @{
    Authorization = "Bearer $token"
    Accept        = "application/vnd.github+json"
    "Content-Type" = "application/octet-stream"
}

Invoke-RestMethod -Method Post -Uri $uploadUri -Headers $uploadHeaders -InFile $AssetPath | Out-Null
Write-Host "Release published: $($release.html_url)"
