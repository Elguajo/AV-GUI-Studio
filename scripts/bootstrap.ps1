<#
.SYNOPSIS
Installs everything a Windows build of GUI Studio needs into .tools/ inside this checkout.

.DESCRIPTION
Nothing is read from or written to other folders: no global Rust, no MinGW on PATH, no sibling
checkouts. Every download is pinned and checked against its SHA-256 (the Rust toolchain itself is
checked by rustup; craft-fonts by its Git commit). Running it again only repairs what is missing
or out of date, so it is also the way to move to new pins.

  .tools/rustup, .tools/cargo   rustup, the toolchain from rust-toolchain.toml, the Cargo cache
  .tools/mingw64                GCC, binutils and the MinGW-w64 headers and libraries (WinLibs)
  .tools/craft-fonts            the embedded fonts (CRAFT_FONTS_DIR in .cargo/config.toml)

Afterwards, load the environment into a shell with `. .\scripts\env.ps1` (PowerShell) or
`source scripts/env.sh` (Git Bash), then use cargo as usual.

.PARAMETER Only
Installs only the named parts: rust, mingw, fonts. CI uses `-Only fonts`.

.EXAMPLE
.\scripts\bootstrap.ps1
#>
[CmdletBinding()]
param(
    [ValidateSet('rust', 'mingw', 'fonts')]
    [string[]] $Only = @('rust', 'mingw', 'fonts')
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue' # Windows PowerShell 5.1 downloads crawl with a progress bar.
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

# --- Pins: change a version here, then run the script again. -----------------------------------
$Pins = @{
    # rustup-init for the GNU host; the toolchain version itself is in rust-toolchain.toml.
    RustupVersion = '1.29.3'
    RustupSha256  = 'f6289656ce42e1099f3cbc6f10ef6321dfde54773faf599dd36fd3b1f07d9103'
    # WinLibs GCC 16.2.0 (POSIX threads, SEH) + MinGW-w64 14.0.0 on MSVCRT, the C runtime Rust's
    # x86_64-pc-windows-gnu target links against.
    MingwUrl      = 'https://github.com/brechtsanders/winlibs_mingw/releases/download/16.2.0posix-23.1.3-14.0.0-msvcrt-r3/winlibs-x86_64-posix-seh-gcc-16.2.0-mingw-w64msvcrt-14.0.0-r3.zip'
    MingwSha256   = '62af32ea58acdea1318ea5dd8eb7017eaee694dd40b269d52a59c769dcfefb82'
    # storytold/craft-fonts: the commit FigCraft's CI used for the b828e6c7 snapshot this editor is
    # built on. Later commits add faces (Noto Sans for Indian scripts) that the snapshot's font
    # fallback list and its tests do not know.
    FontsRepo     = 'https://github.com/storytold/craft-fonts.git'
    FontsCommit   = '37827adc41fd165f9a9234ac40f94dbad353aa69'
}

$Root = Split-Path -Parent $PSScriptRoot
$Tools = Join-Path $Root '.tools'
$Downloads = Join-Path $Tools 'downloads'
New-Item -ItemType Directory -Force -Path $Downloads | Out-Null

# Runs a native program and fails on a non-zero exit code. Its stderr stays plain output: under
# Windows PowerShell 5.1 with ErrorActionPreference Stop, redirected stderr (rustup's and git's
# progress lines) would otherwise end the script.
function Invoke-Native([string] $Exe, [string[]] $Arguments) {
    $saved = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try {
        & $Exe @Arguments 2>&1 | ForEach-Object { "$_" }
        $code = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $saved
    }
    if ($code -ne 0) { throw "$(Split-Path -Leaf $Exe) $($Arguments -join ' ') failed ($code)" }
}

function Get-Verified([string] $Url, [string] $Sha256, [string] $Name) {
    $file = Join-Path $Downloads $Name
    if ((Test-Path $file) -and (Get-FileHash -Algorithm SHA256 $file).Hash -eq $Sha256) {
        return $file
    }
    Write-Host "Downloading $Name"
    $partial = "$file.partial"
    Invoke-WebRequest -UseBasicParsing -Uri $Url -OutFile $partial
    $actual = (Get-FileHash -Algorithm SHA256 $partial).Hash
    if ($actual -ne $Sha256) {
        Remove-Item -Force $partial
        throw "$Name`: SHA-256 is $actual, expected $Sha256. Nothing was installed."
    }
    Move-Item -Force $partial $file
    return $file
}

# Writes what a folder was installed from, so a changed pin replaces it.
function Test-Stamp([string] $Dir, [string] $Want) {
    $stamp = Join-Path $Dir '.installed-from'
    return (Test-Path $stamp) -and ((Get-Content -Raw $stamp).Trim() -eq $Want)
}
function Set-Stamp([string] $Dir, [string] $Value) {
    Set-Content -Encoding ascii -Path (Join-Path $Dir '.installed-from') -Value $Value
}

function Install-Rust {
    $env:RUSTUP_HOME = Join-Path $Tools 'rustup'
    $env:CARGO_HOME = Join-Path $Tools 'cargo'
    $rustup = Join-Path $env:CARGO_HOME 'bin\rustup.exe'
    if (-not (Test-Path $rustup)) {
        $v = $Pins.RustupVersion
        $init = Get-Verified "https://static.rust-lang.org/rustup/archive/$v/x86_64-pc-windows-gnu/rustup-init.exe" $Pins.RustupSha256 "rustup-init-$v-gnu.exe"
        Write-Host "Installing rustup $v into .tools"
        Invoke-Native $init @('-y', '--quiet', '--no-modify-path', '--profile', 'minimal', '--default-host', 'x86_64-pc-windows-gnu', '--default-toolchain', 'none')
        # Keep rustup at the pinned version: no updates of itself behind the script's back.
        Invoke-Native $rustup @('set', 'auto-self-update', 'disable')
    }
    Push-Location $Root
    try {
        # No argument: rustup installs what rust-toolchain.toml names, with its components and targets.
        Invoke-Native $rustup @('toolchain', 'install')
        Invoke-Native $rustup @('show', 'active-toolchain')
    } finally {
        Pop-Location
    }
}

function Install-Mingw {
    $dir = Join-Path $Tools 'mingw64'
    if (Test-Stamp $dir $Pins.MingwUrl) {
        Write-Host 'MinGW: up to date'
        return
    }
    $zip = Get-Verified $Pins.MingwUrl $Pins.MingwSha256 (Split-Path -Leaf $Pins.MingwUrl)
    if (Test-Path $dir) { Remove-Item -Recurse -Force $dir }
    Write-Host 'Unpacking MinGW into .tools/mingw64'
    # The archive holds one top-level mingw64/ folder. Windows' own bsdtar unpacks a zip far faster
    # than Expand-Archive.
    $tar = Join-Path $env:SystemRoot 'System32\tar.exe'
    if (Test-Path $tar) {
        Invoke-Native $tar @('-xf', $zip, '-C', $Tools)
    } else {
        Expand-Archive -Path $zip -DestinationPath $Tools
    }
    if (-not (Test-Path (Join-Path $dir 'bin\gcc.exe'))) { throw 'MinGW: bin\gcc.exe is missing after unpacking' }
    Set-Stamp $dir $Pins.MingwUrl
    Invoke-Native (Join-Path $dir 'bin\gcc.exe') @('-dumpfullversion')
}

function Install-Fonts {
    $dir = Join-Path $Tools 'craft-fonts'
    $commit = $Pins.FontsCommit
    if ((Test-Path (Join-Path $dir '.git')) -and ((git -C $dir rev-parse HEAD) -eq $commit)) {
        Write-Host 'craft-fonts: up to date'
        return
    }
    if (-not (Test-Path (Join-Path $dir '.git'))) {
        Invoke-Native git @('init', '--quiet', $dir)
    }
    Write-Host "Fetching craft-fonts $($commit.Substring(0, 7))"
    Invoke-Native git @('-C', $dir, 'fetch', '--quiet', '--depth', '1', $Pins.FontsRepo, $commit)
    Invoke-Native git @('-C', $dir, '-c', 'advice.detachedHead=false', 'checkout', '--quiet', '--force', 'FETCH_HEAD')
}

if ($Only -contains 'rust') { Install-Rust }
if ($Only -contains 'mingw') { Install-Mingw }
if ($Only -contains 'fonts') { Install-Fonts }
Write-Host 'Done. Load the environment with: . .\scripts\env.ps1   (Git Bash: source scripts/env.sh)'
