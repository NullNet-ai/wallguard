<?xml version="1.0" encoding="UTF-8"?>
<!--
  WiX v4 installer source for WallGuard.

  Prerequisites
  ─────────────
  • Rust toolchain targeting x86_64-pc-windows-msvc
  • WiX Toolset v4   (winget install WixToolset.WiX)

  Build
  ─────
  Run from the repo root on Windows:
      .\packbuild.ps1 -Version <version>

  This template is stamped by packbuild.ps1 (__VERSION__ → real version)
  before being fed to `wix build`.

  What the MSI does
  ─────────────────
  • Installs wallguard.exe and wallguard-cli.exe to
      C:\Program Files\WallGuard\
  • Appends that directory to the system PATH so both binaries are
    accessible from any terminal.
  • Does NOT register a Windows service — that is handled at runtime by
      wallguard-cli start
    which calls `sc create` (see autostart/windows.rs), mirroring the
    systemd / rc.d approach used on Linux / FreeBSD.
  • On uninstall, the PATH entry is removed automatically.

  UpgradeCode must remain constant across all versions of WallGuard.
-->
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">

  <Package Name="WallGuard"
           Version="__VERSION__"
           Manufacturer="NullNet"
           UpgradeCode="B8D4E6A2-1C3F-4B5D-8E9A-2C4D6F8A0B2E"
           Language="1033"
           Scope="perMachine">

    <!-- Replace any older installation automatically. -->
    <MajorUpgrade DowngradeErrorMessage="A newer version of WallGuard is already installed." />
    <MediaTemplate EmbedCab="yes" />

    <Feature Id="ProductFeature" Title="WallGuard" Level="1">
      <ComponentGroupRef Id="ProductComponents" />
    </Feature>

  </Package>

  <!-- ── Directory tree ──────────────────────────────────────────────────── -->
  <Fragment>
    <Directory Id="TARGETDIR" Name="SourceDir">
      <Directory Id="ProgramFiles64Folder">
        <!-- Installs to C:\Program Files\WallGuard\ -->
        <Directory Id="INSTALLFOLDER" Name="WallGuard" />
      </Directory>
    </Directory>
  </Fragment>

  <!-- ── Components ─────────────────────────────────────────────────────── -->
  <Fragment>
    <ComponentGroup Id="ProductComponents" Directory="INSTALLFOLDER">

      <!-- wallguard.exe — the background agent -->
      <Component Id="WallGuardExe">
        <File Source="target\release\wallguard.exe" KeyPath="yes" />
      </Component>

      <!-- wallguard-cli.exe — the management CLI -->
      <Component Id="WallGuardCliExe">
        <File Source="target\release\wallguard-cli.exe" KeyPath="yes" />
      </Component>

      <!-- Append INSTALLFOLDER to the system PATH.
           A registry key is required as the KeyPath for environment components. -->
      <Component Id="EnvPath">
        <RegistryValue Root="HKLM"
                       Key="SOFTWARE\WallGuard"
                       Name="InstallPath"
                       Type="string"
                       Value="[INSTALLFOLDER]"
                       KeyPath="yes" />
        <Environment Id="SystemPath"
                     Name="PATH"
                     Value="[INSTALLFOLDER]"
                     Permanent="no"
                     Part="last"
                     Action="set"
                     System="yes" />
      </Component>

    </ComponentGroup>
  </Fragment>

</Wix>
