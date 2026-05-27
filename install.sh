#!/usr/bin/env bash
# WallGuard Universal Installer
#
# Usage:
#   curl -fsSL https://github.com/NullNet-ai/wallguard/releases/latest/download/install.sh | sudo bash
#
# Override version:
#   WALLGUARD_VERSION=0.1.19 curl -fsSL ... | sudo bash

set -euo pipefail

GITHUB_REPO="NullNet-ai/wallguard"
INSTALL_DIR="/usr/local/bin"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; BOLD='\033[1m'; NC='\033[0m'
die()  { echo -e "${RED}Error: $*${NC}" >&2; exit 1; }
info() { echo -e "${GREEN}==>${NC} ${BOLD}$*${NC}"; }
warn() { echo -e "${YELLOW}Warning: $*${NC}"; }

# ── Root check ────────────────────────────────────────────────────────────────
if [ "$(id -u)" -ne 0 ]; then
    die "Please run as root:  sudo bash install.sh"
fi

# ── curl / wget check ─────────────────────────────────────────────────────────
if command -v curl &>/dev/null; then
    _download() { curl -fsSL "$1" -o "$2"; }
elif command -v wget &>/dev/null; then
    _download() { wget -qO "$2" "$1"; }
else
    die "Neither curl nor wget found. Install one and retry."
fi

# ── Detect version ────────────────────────────────────────────────────────────
VERSION="${WALLGUARD_VERSION:-}"
if [ -z "$VERSION" ]; then
    info "Fetching latest WallGuard release..."
    VERSION=$(curl -sfL \
        "https://api.github.com/repos/${GITHUB_REPO}/releases/latest" \
        | grep '"tag_name"' | sed 's/.*"v\([^"]*\)".*/\1/')
    [ -z "$VERSION" ] && die "Could not determine the latest version. Set WALLGUARD_VERSION and retry."
fi
info "Installing WallGuard v${VERSION}"

# ── Detect architecture ───────────────────────────────────────────────────────
ARCH=$(uname -m)
case "$ARCH" in
    x86_64)        DEB_ARCH="amd64";  RPM_ARCH="x86_64";  TAR_ARCH="x86_64"  ;;
    aarch64|arm64) DEB_ARCH="arm64";  RPM_ARCH="aarch64"; TAR_ARCH="aarch64" ;;
    *) die "Unsupported architecture: $ARCH" ;;
esac

BASE_URL="https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}"

# ── Fallback: raw binary tarball ─────────────────────────────────────────────
install_tarball() {
    local tarball="wallguard-${VERSION}-linux-${TAR_ARCH}.tar.gz"
    info "Downloading binary tarball: ${tarball}"
    local tmp; tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' RETURN
    _download "${BASE_URL}/${tarball}" "${tmp}/wallguard.tar.gz" || \
        die "Download failed: ${BASE_URL}/${tarball}"
    tar -xzf "${tmp}/wallguard.tar.gz" -C "${tmp}"
    install -m755 "${tmp}/wallguard"     "${INSTALL_DIR}/wallguard"
    install -m755 "${tmp}/wallguard-cli" "${INSTALL_DIR}/wallguard-cli"
}

# ── OS detection and package install ─────────────────────────────────────────
if [ -f /etc/os-release ]; then
    # shellcheck source=/dev/null
    . /etc/os-release
    COMBINED="${ID_LIKE:-} ${ID:-}"

    case "$COMBINED" in
        *debian*|*ubuntu*)
            info "Debian/Ubuntu detected — installing .deb package"
            tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
            DEB="wallguard_${VERSION}_${DEB_ARCH}.deb"
            _download "${BASE_URL}/${DEB}" "${tmp}/wallguard.deb" || \
                die "Failed to download ${DEB}"
            dpkg -i "${tmp}/wallguard.deb"
            ;;

        *fedora*|*rhel*|*centos*|*suse*|*ol*)
            info "RPM-based Linux detected — installing .rpm package"
            tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
            RPM="wallguard-${VERSION}-1.${RPM_ARCH}.rpm"
            _download "${BASE_URL}/${RPM}" "${tmp}/wallguard.rpm" || \
                die "Failed to download ${RPM}"
            if command -v dnf &>/dev/null; then
                dnf install -y "${tmp}/wallguard.rpm"
            elif command -v yum &>/dev/null; then
                yum install -y "${tmp}/wallguard.rpm"
            else
                rpm -i "${tmp}/wallguard.rpm"
            fi
            ;;

        *)
            warn "Unknown Linux distribution — falling back to binary tarball"
            install_tarball
            ;;
    esac
else
    OS=$(uname -s)
    case "$OS" in
        Darwin)
            echo ""
            echo "macOS detected. Please download the .dmg from:"
            echo "  https://github.com/${GITHUB_REPO}/releases/tag/v${VERSION}"
            echo ""
            echo "Then open it and copy the binaries to /usr/local/bin."
            exit 0
            ;;
        FreeBSD)
            info "FreeBSD detected — installing .pkg"
            tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
            PKG="wallguard-${VERSION}.pkg"
            _download "${BASE_URL}/${PKG}" "${tmp}/wallguard.pkg" || \
                die "Failed to download ${PKG}"
            pkg add "${tmp}/wallguard.pkg"
            ;;
        *)
            warn "Unrecognised OS: ${OS} — trying binary tarball"
            install_tarball
            ;;
    esac
fi

# ── Done ──────────────────────────────────────────────────────────────────────
echo ""
info "WallGuard v${VERSION} installed successfully!"
echo ""
echo "  Agent:  $(command -v wallguard     2>/dev/null || echo "${INSTALL_DIR}/wallguard")"
echo "  CLI:    $(command -v wallguard-cli 2>/dev/null || echo "${INSTALL_DIR}/wallguard-cli")"
echo ""
echo "Get started:"
echo "  wallguard-cli --help"
