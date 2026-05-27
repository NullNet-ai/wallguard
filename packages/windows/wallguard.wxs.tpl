<?xml version="1.0" encoding="UTF-8"?>
<!--
  WiX v4 installer source for WallGuard.

  Prerequisites
  ─────────────
  • Rust toolchain targeting x86_64-pc-windows-msvc
  • WiX Toolset v4 (pin to v4.*; v5+ requires a paid EULA)
  • packages\windows\npcap-installer.exe  (downloaded by packbuild.ps1 / CI)

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
  • Installs Npcap (bundled) silently if it is not already present.
    Npcap is the packet-capture driver required by WallGuard's network
    monitoring.  The bundled installer is downloaded at build time from
    https://npcap.com/dist/ and embedded into the MSI.
    NOTE: Silent installation (/S) requires the Npcap OEM licence for
    production redistribution.  See https://npcap.com/oem/ for details.
  • Does NOT register a Windows service — that is handled at runtime by
      wallguard-cli start
    which calls `sc create` (see autostart/windows.rs), mirroring the
    systemd / rc.d approach used on Linux / FreeBSD.
  • On uninstall, the PATH entry is removed automatically.
    Npcap is intentionally NOT uninstalled — other software may depend on it.

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
      Detect whether Npcap is already installed by probing the driver
      service registry key that Npcap always creates.
      If NPCAP_INSTALLED is set (non-empty), we skip the bundled installer.
    -->
    <Property Id="NPCAP_INSTALLED">
      <RegistrySearch Id="NpcapSearch"
                      Root="HKLM"
                      Key="SYSTEM\CurrentControlSet\Services\npcap"
                      Name="ImagePath"
                      Type="raw" />
    </Property>

    <!--
      Embed the Npcap installer as a binary resource.
      The file is downloaded into packages\windows\ by packbuild.ps1 / CI
      before `wix build` runs.
    -->
    <Binary Id="NpcapBinary"
            SourceFile="packages\windows\npcap-installer.exe" />

    <!--
      Run the Npcap installer silently.
      /S           — silent (no UI)
      /winpcap_mode=no — do not install WinPcap compatibility shim
      Execute="deferred" + Impersonate="no" — runs as SYSTEM (required for
        driver installation).
    -->
    <CustomAction Id="InstallNpcap"
                  BinaryRef="NpcapBinary"
                  ExeCommand="/S /winpcap_mode=no"
                  Execute="deferred"
                  Impersonate="no"
                  Return="check" />

    <InstallExecuteSequence>
      <!--
        Only install Npcap when:
          • Npcap is not already present (NOT NPCAP_INSTALLED)
          • We are not in the middle of an uninstall (NOT REMOVE)
      -->
      <Custom Action="InstallNpcap" Before="InstallFinalize">
        NOT NPCAP_INSTALLED AND NOT REMOVE
      </Custom>
    </InstallExecuteSequence>

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
