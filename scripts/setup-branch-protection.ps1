# Repository ruleset for `main` on a personal (user) GitHub repo.
# Only the bypass user (WtekSupport) may push directly; others need PR + review.
# Requires: git credential with repo admin scope, or gh auth login.

$ErrorActionPreference = "Stop"
$Owner = "WtekSupport"
$Repo = "TGScanner"
$PayloadFile = Join-Path $PSScriptRoot "ruleset-main.json"

if (-not (Test-Path $PayloadFile)) {
    throw "Missing $PayloadFile"
}

$credText = @"
protocol=https
host=github.com

"@ | git credential fill 2>$null
if (-not $credText) { throw "No git credentials for github.com — run git push once or gh auth login" }
$token = ($credText -split "`n" | Where-Object { $_ -like 'password=*' }) -replace '^password=',''
if (-not $token) { throw "Could not read GitHub token from credential helper" }

$existing = curl.exe -s -H "Authorization: Bearer $token" -H "Accept: application/vnd.github+json" `
    "https://api.github.com/repos/$Owner/$Repo/rulesets"
if ($existing -match '"name": "main — owner push only"') {
    Write-Host "Ruleset already exists. Update via GitHub UI or delete old ruleset first."
    exit 0
}

curl.exe -s -w "`nHTTP:%{http_code}`n" -X POST `
    -H "Authorization: Bearer $token" `
    -H "Accept: application/vnd.github+json" `
    -H "Content-Type: application/json" `
    --data-binary "@$PayloadFile" `
    "https://api.github.com/repos/$Owner/$Repo/rulesets"
Write-Host "Done. Check: https://github.com/$Owner/$Repo/rules"
