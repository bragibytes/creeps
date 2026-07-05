# Install the Creeps client on Windows.
# Usage: irm https://raw.githubusercontent.com/bragibytes/creeps/main/scripts/install.ps1 | iex

$ErrorActionPreference = "Stop"

$Repo = "bragibytes/creeps"
$InstallDir = if ($env:REALM_INSTALL_DIR) { $env:REALM_INSTALL_DIR } else { "$env:USERPROFILE\.local\bin" }
$Archive = "realm-x86_64-pc-windows-msvc.zip"

function Get-LatestTag {
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
    return $release.tag_name
}

function Install-FromRelease {
    $tag = Get-LatestTag
    $url = "https://github.com/$Repo/releases/download/$tag/$Archive"
    Write-Host "→ Downloading $url..."

    $tmp = New-Item -ItemType Directory -Force -Path (Join-Path $env:TEMP "realm-install")
    $zipPath = Join-Path $tmp $Archive
    Invoke-WebRequest -Uri $url -OutFile $zipPath
    Expand-Archive -Path $zipPath -DestinationPath $tmp -Force

    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    Copy-Item (Join-Path $tmp "realm.exe") (Join-Path $InstallDir "realm.exe") -Force
    Write-Host "✓ Installed to $InstallDir\realm.exe"
}

function Install-FromCargo {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        throw "No release found and Rust/cargo is not installed. Install Rust from https://rustup.rs"
    }
    Write-Host "→ Building from source (requires Rust)..."
    cargo install --locked --git "https://github.com/$Repo.git" --bin realm
}

try {
    Install-FromRelease
} catch {
    Install-FromCargo
}

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$InstallDir*") {
    Write-Host ""
    Write-Host "Add to your PATH (run in PowerShell):"
    Write-Host "  [Environment]::SetEnvironmentVariable('Path', `"$userPath;$InstallDir`", 'User')"
}

Write-Host ""
Write-Host "  realm          # full-screen UI"
Write-Host "  realm --plain  # simple scrollback mode"
Write-Host ""
Write-Host "Type 'register' or 'login' when prompted. No setup required."