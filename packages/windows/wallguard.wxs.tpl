<?xml version="1.0" encoding="UTF-8"?>
<!--
  WiX v4 installer source for WallGuard.

  Prerequisites
  ─────────────
  • Rust toolchain targeting x86_64-pc-windows-msvc
  • WiX Toolset v4 (pin to v4.*; v5+ requires a paid EULA)
  • Npcap must be installed on the target machine before running this MSI.
    Download: https://npcap.com/#download
    (Silent/OEM bundling requires the Npcap OEM licence — https://npcap.com/oem/)

  Build
  ─────
  Run from the repo root on Windows:
      .\packbuild.ps1 -Version <version>

  This template is stamped by packbuild.ps1 (__VERSION__ → real version)
  before being fed to `wix build`.

  What the MSI does
  ─────────────────
  • Checks that Npcap is already installed; aborts with a clear message if not.
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

    <!--
      Detect whether Npcap is installed by probing the driver service
      registry key that every Npcap release creates.
    -->
    <Property Id="NPCAP_INSTALLED">
      <RegistrySearch Id="NpcapSearch"
                      Root="HKLM"
                      Key="SYSTEM\CurrentControlSet\Services\npcap"
                      Name="ImagePath"
                      Type="raw" />
    </Property>

    <!--
      Block the install if Npcap is absent.
      "Installed" is the MSI built-in that is true during upgrades/repairs,
      so existing installations are never blocked by this check.
    -->
    <Launch Condition="NPCAP_INSTALLED OR Installed"
            Message="WallGuard requires Npcap to be installed first.&#10;&#10;Please download and install Npcap from:&#10;    https://npcap.com/#download&#10;&#10;Then run this installer again." />

  </Package>

  <!-- ── Directory tree ──────────────────────────────────────────────────── -->
  <Fragment>
    <StandardDirectory Id="ProgramFiles64Folder">
      <!-- Installs to C:\Program Files\WallGuard\ -->
      <Directory Id="INSTALLFOLDER" Name="WallGuard" />
    </StandardDirectory>
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
