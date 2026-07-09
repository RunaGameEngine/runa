param(
    [Parameter(Mandatory)]
    [ValidateSet('major', 'minor', 'patch', 'alpha', 'beta')]
    [string]$Bump,

    [string]$PreRelease
)

# ── helpers ──────────────────────────────────────────────────────────────────
function Get-CurrentVersion {
    $content = Get-Content -Path "$PSScriptRoot/../Cargo.toml" -Raw
    if ($content -match 'version\s*=\s*"([^"]+)"') {
        return $matches[1]
    }
    throw "Cannot find version in Cargo.toml"
}

function Bump-Version {
    param([string]$Version, [string]$BumpType, [string]$PreRelease)

    if ($Version -match '^(\d+)\.(\d+)\.(\d+)(?:-(\w+)\.(\d+))?$') {
        $major = [int]$matches[1]
        $minor = [int]$matches[2]
        $patch = [int]$matches[3]
        $preName = $matches[4]
        $preNum = if ($matches[5]) { [int]$matches[5] } else { 0 }

        switch ($BumpType) {
            'major' {
                $major += 1; $minor = 0; $patch = 0
                $preName = $null; $preNum = 0
            }
            'minor' {
                $minor += 1; $patch = 0
                $preName = $null; $preNum = 0
            }
            'patch' {
                $patch += 1
                $preName = $null; $preNum = 0
            }
            'alpha' {
                if ($preName -eq 'alpha') { $preNum += 1 }
                else { $preName = 'alpha'; $preNum = 1 }
            }
            'beta' {
                if ($preName -eq 'beta') { $preNum += 1 }
                else { $preName = 'beta'; $preNum = 1 }
            }
        }

        if ($preName) {
            return "$major.$minor.$patch-$preName.$preNum"
        }
        return "$major.$minor.$patch"
    }
    throw "Unrecognized version format: $Version"
}

# ── main ─────────────────────────────────────────────────────────────────────
$ErrorActionPreference = "Stop"
Push-Location $PSScriptRoot/..

# 1. Check clean state
$status = git status --porcelain
if ($status) {
    Write-Host "Working directory is not clean. Commit or stash first." -ForegroundColor Red
    git status
    Pop-Location; exit 1
}

# 2. Get current version
$current = Get-CurrentVersion
Write-Host "Current version: $current" -ForegroundColor Cyan

# 3. Compute new version
$new = Bump-Version -Version $current -BumpType $Bump
if ($PreRelease) { $new = "$new-$PreRelease" }
Write-Host "New version: $new" -ForegroundColor Green

# 4. Update Cargo.toml
$content = Get-Content -Path "Cargo.toml" -Raw
$content = $content -replace '^version\s*=\s*"[^"]+"', "version = `"$new`""
Set-Content -Path "Cargo.toml" -Value $content -NoNewline

# 5. Generate changelog with git-cliff (unreleased → new tag)
if (Get-Command git-cliff -ErrorAction SilentlyContinue) {
    Write-Host "Generating changelog..." -ForegroundColor Cyan
    git-cliff --unreleased --tag "$new" --prepend CHANGELOG.md
} else {
    Write-Host "git-cliff not installed. Install: cargo install git-cliff" -ForegroundColor Yellow
    Write-Host "Skipping changelog generation." -ForegroundColor Yellow
}

# 6. Commit
git add Cargo.toml CHANGELOG.md
git commit -m "Release v$new"
git tag -a "v$new" -m "Release v$new"

# 7. Done
Write-Host @"

Release v$new ready.
Review: git show HEAD
Push:   git push origin main --tags
"@ -ForegroundColor Green

Pop-Location
