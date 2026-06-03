Name:           wallguard
Version:        __VERSION__
Release:        1%{?dist}
Summary:        WallGuard agent and CLI interface
License:        AGPL-3.0-only
URL:            https://github.com/NullNet-ai/wallguard

# Pre-built binaries — no source compilation needed.
%global debug_package %{nil}

Requires: libpcap, pipewire-libs, dbus-libs, libcap

%description
WallGuard is an agent-connector to the NullNet system that provides
network monitoring and security capabilities.

%install
install -Dm755 %{_builddir}/wallguard     %{buildroot}/usr/local/bin/wallguard
install -Dm755 %{_builddir}/wallguard-cli %{buildroot}/usr/local/bin/wallguard-cli

%files
%attr(0755,root,root) /usr/local/bin/wallguard
%attr(0755,root,root) /usr/local/bin/wallguard-cli

%changelog
* __DATE__ Anton Liashkevich <anton.liashkevich.eng@gmail.com> - __VERSION__-1
- Package build
