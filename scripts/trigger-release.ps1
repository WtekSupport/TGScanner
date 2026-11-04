param(
    [string]$Tag = "v1.0.1",
    [string]$Ref = "main"
)

$ErrorActionPreference = "Stop"
Push-Location (Join-Path $PSScriptRoot "..")
try {
    $credText = @"
protocol=https
host=github.com

"@ | git credential fill 2>$null
    $token = ($credText -split "`n" | Where-Object { $_ -like "password=*" }) -replace "^password=", "" | Select-Object -First 1
    if (-not $token) { throw "No GitHub token (git credential)." }

    $body = @{ ref = $Ref; inputs = @{ tag = $Tag } } | ConvertTo-Json -Compress
    $headers = @{
        Authorization            = "Bearer $token"
        Accept                   = "application/vnd.github+json"
        "X-GitHub-Api-Version"   = "2022-11-28"
    }
    Invoke-RestMethod -Method Post `
        -Uri "https://api.github.com/repos/WtekSupport/TGScanner/actions/workflows/release.yml/dispatches" `
        -Headers $headers -Body $body -ContentType "application/json"
    Write-Host "Release workflow dispatched for tag $Tag (ref $Ref)."
}
finally {
    Pop-Location
}
