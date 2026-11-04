# Protects branch `main`: only listed GitHub users may push (direct commits).
# Requires: gh CLI authenticated as repo admin (gh auth login).
# Usage: .\scripts\setup-branch-protection.ps1

$ErrorActionPreference = "Stop"
$Owner = "WtekSupport"
$Repo = "TGScanner"
$Branch = "main"
# GitHub login allowed to push to main (repo owner account)
$PushUsers = @("WtekSupport")

$gh = Get-Command gh -ErrorAction SilentlyContinue
if (-not $gh) {
    $ghPath = "C:\Temp\gh-cli\bin\gh.exe"
    if (Test-Path $ghPath) { $gh = $ghPath } else { throw "Install GitHub CLI: https://cli.github.com/" }
}

& $gh auth status 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) {
    throw "Run: gh auth login   (as $Owner or org admin)"
}

$payload = @{
    required_status_checks = $null
    enforce_admins = $true
    required_pull_request_reviews = $null
    restrictions = @{
        users = $PushUsers
        teams = @()
        apps = @()
    }
    allow_force_pushes = $false
    allow_deletions = $false
    block_creations = $false
} | ConvertTo-Json -Depth 5

$payload | & $gh api "repos/$Owner/$Repo/branches/$Branch/protection" -X PUT --input -
Write-Host "Branch protection applied: only $($PushUsers -join ', ') may push to $Branch."
