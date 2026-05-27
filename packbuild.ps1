<#
.SYNOPSIS
  Builds WallGuard release binaries and packages them as a Windows MSI.

.DESCRIPTION
  Compiles wallguard.exe and wallguard-cli.exe in release mode, stamps the
  version number into the WiX source template, and runs `wix build` to
  produce an MSI installer.

  The MSI checks for Npcap at install time and aborts with a clear download
  link if it is absent.  Silent/bundled Npcap install is not included because
  it requires the Npcap OEM licence (https://npcap.com/oem/).

  Prerequisites
  ─────────────
  • Rust toolchain (stable, x86_64-pc-windows-msvc)
  • WiX Toolset v4  (winget install WixToolset.WiX  — pin to v4.*)
  • Npcap SDK in LIB (for the pcap crate to compile):
      $env:LIB = "C:\npcap-sdk\Lib\x64"
    SDK download: https://npcap.com/dist/npcap-sdk-1.13.zip

.PARAMETER Version
  The version string to embed in the MSI (must match Cargo.toml, e.g. "0.1.19").

.EXAMPLE
  .\packbuild.ps1 -Version 0.1.19
#>
param(
    [Parameter(Mandatory = $true)]
    [string]$Version
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoRoot = $PSScriptRoot
$PkgDir   = Join-Path $RepoRoot "packages\windows"
$WxsTpl   = Join-Path $PkgDir   "wallguard.wxs.tpl"
$WxsOut   = Join-Path $PkgDir   "wallguard.wxs"
$MsiOut   = Join-Path $RepoRoot "wallguard-$Version-x86_64.msi"

# ── 1. Build release binaries ─────────────────────────────────────────────────
Write-Host "Building release binaries (this may take a while)..."
Push-Location $RepoRoot
try {
    cargo build --release -p wallguard -p wallguard-cli
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed (exit code $LASTEXITCODE)" }
} finally {
    Pop-Location
}

# ── 2. Stamp version into the .wxs template ───────────────────────────────────
Write-Host "Generating WiX source (version $Version)..."
(Get-Content $WxsTpl -Raw) -replace '__VERSION__', $Version |
    Set-Content $WxsOut -Encoding UTF8

# ── 3. Build the MSI (WiX v4) ────────────────────────────────────────────────
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
Write-Host "Prerequisites on the target machine:"
Write-Host "  Npcap  https://npcap.com/#download  (install before running the MSI)"
Write-Host ""
Write-Host "  Install  : msiexec /i `"$MsiOut`" /qn"
Write-Host "  Uninstall: msiexec /x `"$MsiOut`" /qn"
Write-Host ""
Write-Host "After installing, start the agent with:"
Write-Host "  wallguard-cli start --control-channel-url <url>"
