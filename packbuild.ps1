<#
.SYNOPSIS
  Builds WallGuard release binaries and packages them as a Windows MSI.

.DESCRIPTION
  Compiles wallguard.exe and wallguard-cli.exe in release mode, downloads
  the Npcap runtime installer for bundling, stamps the version number into
  the WiX source template, and runs `wix build` to produce a self-contained
  MSI installer.

  The MSI embeds the Npcap installer and runs it silently if Npcap is not
  already present on the target machine.

  Prerequisites
  ─────────────
  • Rust toolchain (stable, x86_64-pc-windows-msvc)
  • WiX Toolset v4   (dotnet tool install --global wix)
  • Npcap SDK in LIB (for pcap crate compilation):
      $env:LIB = "C:\npcap-sdk\Lib\x64"
  • Internet access to download the Npcap runtime installer

  Npcap licensing note
  ─────────────────────
  Silent installation (/S) and redistribution of the Npcap installer
  require the Npcap OEM licence.  See https://npcap.com/oem/ for details.

.PARAMETER Version
  The version string to embed in the MSI (must match Cargo.toml, e.g. "0.1.19").

.PARAMETER NpcapVersion
  The Npcap installer version to download and bundle (default: "1.82").
  Update when a new Npcap release is available: https://npcap.com/dist/

.EXAMPLE
  .\packbuild.ps1 -Version 0.1.19
  .\packbuild.ps1 -Version 0.1.19 -NpcapVersion 1.82
#>
param(
    [Parameter(Mandatory = $true)]
    [string]$Version,

    [string]$NpcapVersion = "1.82"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoRoot     = $PSScriptRoot
$PkgDir       = Join-Path $RepoRoot "packages\windows"
$WxsTpl       = Join-Path $PkgDir   "wallguard.wxs.tpl"
$WxsOut       = Join-Path $PkgDir   "wallguard.wxs"
$NpcapDest    = Join-Path $PkgDir   "npcap-installer.exe"
$MsiOut       = Join-Path $RepoRoot "wallguard-$Version-x86_64.msi"

# ── 1. Download Npcap runtime installer (for bundling into the MSI) ───────────
if (Test-Path $NpcapDest) {
    Write-Host "Npcap installer already present at $NpcapDest — skipping download."
} else {
    $NpcapUrl = "https://npcap.com/dist/npcap-$NpcapVersion.exe"
    Write-Host "Downloading Npcap $NpcapVersion installer from $NpcapUrl ..."
    Invoke-WebRequest -Uri $NpcapUrl -OutFile $NpcapDest
    Write-Host "Saved to $NpcapDest"
}

# ── 2. Build release binaries ─────────────────────────────────────────────────
Write-Host "Building release binaries (this may take a while)..."
Push-Location $RepoRoot
try {
    cargo build --release -p wallguard -p wallguard-cli
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed (exit code $LASTEXITCODE)" }
} finally {
    Pop-Location
}

# ── 3. Stamp version into the .wxs template ───────────────────────────────────
Write-Host "Generating WiX source (version $Version)..."
(Get-Content $WxsTpl -Raw) -replace '__VERSION__', $Version |
    Set-Content $WxsOut -Encoding UTF8

# ── 4. Build the MSI (WiX v4) ────────────────────────────────────────────────
Write-Host "Building MSI: $MsiOut..."
Push-Location $RepoRoot
try {
    # Run wix from the repo root so that Source paths in the .wxs file
    # (e.g. "target\release\wallguard.exe") resolve correctly.
    wix build $WxsOut -out $MsiOut
    if ($LASTEXITCODE -ne 0) { throw "wix build failed (exit code $LASTEXITCODE)" }
} finally {
    Pop-Location
    # Always clean up the generated (non-template) .wxs file.
    Remove-Item $WxsOut -ErrorAction SilentlyContinue
}

Write-Host ""
Write-Host "Package created: $MsiOut"
Write-Host ""
Write-Host "  Install  : msiexec /i `"$MsiOut`" /qn"
Write-Host "  Uninstall: msiexec /x `"$MsiOut`" /qn"
Write-Host ""
Write-Host "After installing, start the agent with:"
Write-Host "  wallguard-cli start --control-channel-url <url>"
