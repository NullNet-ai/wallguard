#!/usr/bin/env bash
# packbuild.sh — build WallGuard release packages
#
# Usage:
#   ./packbuild.sh deb     <version>   — Debian/Ubuntu .deb
#   ./packbuild.sh rpm     <version>   — Fedora/CentOS .rpm
#   ./packbuild.sh macos   <version>   — macOS .dmg  (must run on macOS)
#   ./packbuild.sh tarball <version>   — Linux binary .tar.gz
#   ./packbuild.sh freebsd <version>   — FreeBSD .pkg
#   ./packbuild.sh windows             — prints Windows instructions
#
# GLIBC compatibility (deb / rpm / tarball):
#   Binaries are built with cargo-zigbuild targeting glibc 2.17, which
#   means they run on Ubuntu 18.04+, Debian 10+, CentOS/RHEL 7+, and any
#   other glibc-based Linux with glibc >= 2.17.
#   If cargo-zigbuild is not installed the script falls back to a plain
#   cargo build (which inherits the host glibc version).
#
set -euo pipefail

# ── helpers ───────────────────────────────────────────────────────────────────
die()  { echo "Error: $*" >&2; exit 1; }
need() { [ -n "${1:-}" ] || die "missing version argument — usage: $0 $mode <version>"; }

# ── argument parsing ──────────────────────────────────────────────────────────
[ $# -lt 1 ] && { echo "Usage: $0 <deb|rpm|macos|tarball|freebsd|windows> [version]"; exit 1; }
mode="$1"; shift

# ── shared: build Linux/FreeBSD binaries ──────────────────────────────────────
#
# If WALLGUARD_BIN_DIR is set, the binaries are assumed to be pre-built and
# sitting in that directory — the build step is skipped entirely.  This is
# how CI uses the script: it runs cargo-zigbuild explicitly in a prior step
# (so failures are visible) and then passes the output directory here.
#
# Without WALLGUARD_BIN_DIR the script tries cargo-zigbuild first (pins
# glibc to 2.17), falling back to plain cargo build for local dev use.
#
# Sets:  $BIN_DIR — directory containing the compiled binaries
#
build_linux_bins() {
    if [ -n "${WALLGUARD_BIN_DIR:-}" ]; then
        echo "==> Using pre-built binaries from WALLGUARD_BIN_DIR=$WALLGUARD_BIN_DIR"
        BIN_DIR="$WALLGUARD_BIN_DIR"
        return
    fi

    local target="x86_64-unknown-linux-gnu"

    if cargo-zigbuild --version &>/dev/null 2>&1; then
        echo "==> cargo-zigbuild found — targeting glibc 2.17 (max compat)"
        rustup target add "${target}" 2>/dev/null || true
        cargo zigbuild --target "${target}.2.17" \
            --release -p wallguard -p wallguard-cli
        BIN_DIR="target/${target}/release"
    else
        echo "==> cargo-zigbuild not found — using native cargo build"
        echo "    (install it for maximum glibc compatibility:"
        echo "     pip install ziglang && cargo install cargo-zigbuild)"
        cargo build --release -p wallguard -p wallguard-cli
        BIN_DIR="target/release"
    fi
}

# ── deb ───────────────────────────────────────────────────────────────────────
deb() {
    need "${1:-}"; local VERSION="$1"

    build_linux_bins

    local PKGDIR="packages/debian"
    local DEBIANDIR="$PKGDIR/DEBIAN"
    mkdir -p "$DEBIANDIR" "$PKGDIR/usr/local/bin"

    sed "s/__VERSION__/${VERSION}/g" "$PKGDIR/control.tpl" > "$DEBIANDIR/control"

    cp "$BIN_DIR/wallguard"     "$PKGDIR/usr/local/bin/"
    cp "$BIN_DIR/wallguard-cli" "$PKGDIR/usr/local/bin/"

    dpkg-deb -Zxz --build "$PKGDIR" .

    # clean up staged files (keep template)
    rm "$PKGDIR/usr/local/bin/wallguard" "$PKGDIR/usr/local/bin/wallguard-cli"
    rm -rf "$DEBIANDIR"

    echo "==> Created: wallguard_${VERSION}_amd64.deb"
}

# ── rpm ───────────────────────────────────────────────────────────────────────
rpm() {
    need "${1:-}"; local VERSION="$1"
    command -v rpmbuild &>/dev/null || die "rpmbuild not found. Install: sudo apt-get install rpm (Ubuntu) / dnf install rpm-build (Fedora)"

    build_linux_bins

    local RPMROOT="$HOME/rpmbuild"
    mkdir -p "$RPMROOT"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

    cp "$BIN_DIR/wallguard"     "$RPMROOT/BUILD/"
    cp "$BIN_DIR/wallguard-cli" "$RPMROOT/BUILD/"

    local DATE; DATE=$(date "+%a %b %d %Y")
    sed "s/__VERSION__/${VERSION}/g; s/__DATE__/${DATE}/g" \
        packages/rpm/wallguard.spec.tpl > "$RPMROOT/SPECS/wallguard.spec"

    rpmbuild -bb "$RPMROOT/SPECS/wallguard.spec"

    # copy the finished RPM to the current directory
    find "$RPMROOT/RPMS/" -name "wallguard-*.rpm" -exec cp {} . \;

    echo "==> Created: wallguard-${VERSION}-1.x86_64.rpm  (or similar name above)"
}

# ── tarball ───────────────────────────────────────────────────────────────────
tarball() {
    need "${1:-}"; local VERSION="$1"

    build_linux_bins

    local STAGING; STAGING=$(mktemp -d)
    cp "$BIN_DIR/wallguard"     "$STAGING/"
    cp "$BIN_DIR/wallguard-cli" "$STAGING/"

    tar -czf "wallguard-${VERSION}-linux-x86_64.tar.gz" -C "$STAGING" wallguard wallguard-cli
    rm -rf "$STAGING"

    echo "==> Created: wallguard-${VERSION}-linux-x86_64.tar.gz"
}

# ── macos ─────────────────────────────────────────────────────────────────────
macos() {
    need "${1:-}"; local VERSION="$1"
    [ "$(uname -s)" = "Darwin" ] || die "macOS DMG must be built on macOS."

    echo "==> Building universal binary (x86_64 + arm64)..."
    rustup target add x86_64-apple-darwin aarch64-apple-darwin 2>/dev/null || true
    cargo build --target x86_64-apple-darwin  --release -p wallguard -p wallguard-cli
    cargo build --target aarch64-apple-darwin --release -p wallguard -p wallguard-cli

    echo "==> Creating fat (universal) binaries..."
    lipo -create \
        target/x86_64-apple-darwin/release/wallguard \
        target/aarch64-apple-darwin/release/wallguard \
        -output target/wallguard-universal
    lipo -create \
        target/x86_64-apple-darwin/release/wallguard-cli \
        target/aarch64-apple-darwin/release/wallguard-cli \
        -output target/wallguard-cli-universal

    echo "==> Assembling DMG contents..."
    local STAGING; STAGING=$(mktemp -d)
    local APPDIR="$STAGING/WallGuard"
    mkdir -p "$APPDIR"
    cp target/wallguard-universal     "$APPDIR/wallguard"
    cp target/wallguard-cli-universal "$APPDIR/wallguard-cli"
    chmod +x "$APPDIR/wallguard" "$APPDIR/wallguard-cli"

    cat > "$APPDIR/README.txt" <<'EOF'
WallGuard CLI Tools
===================
To install, open Terminal and run:

  sudo cp wallguard wallguard-cli /usr/local/bin/

Then verify with:
  wallguard-cli --help
EOF

    echo "==> Building DMG..."
    hdiutil create \
        -volname "WallGuard ${VERSION}" \
        -srcfolder "$APPDIR" \
        -ov -format UDZO \
        "wallguard-${VERSION}-macos.dmg"

    rm -rf "$STAGING"
    echo "==> Created: wallguard-${VERSION}-macos.dmg"
}

# ── freebsd ───────────────────────────────────────────────────────────────────
freebsd() {
    need "${1:-}"; local VERSION="$1"

    cargo build --release -p wallguard -p wallguard-cli

    local PKGDIR="packages/freebsd"
    mkdir -p "$PKGDIR/usr/local/bin"

    cp target/release/wallguard     "$PKGDIR/usr/local/bin/"
    cp target/release/wallguard-cli "$PKGDIR/usr/local/bin/"

    sed "s/__VERSION__/${VERSION}/g" "$PKGDIR/+MANIFEST.tpl" > "$PKGDIR/+MANIFEST"

    pkg create -M "$PKGDIR/+MANIFEST" -r "$PKGDIR"

    rm "$PKGDIR/+MANIFEST" \
       "$PKGDIR/usr/local/bin/wallguard" \
       "$PKGDIR/usr/local/bin/wallguard-cli"
}

# ── windows ───────────────────────────────────────────────────────────────────
windows() {
    echo "Windows packaging requires running packbuild.ps1 on Windows."
    echo "Prerequisites: Rust toolchain + WiX Toolset v4 (winget install WixToolset.WiX)"
    echo ""
    echo "  .\\packbuild.ps1 -Version <version>"
    exit 1
}

# ── dispatch ──────────────────────────────────────────────────────────────────
case "$mode" in
    deb|rpm|tarball|macos|freebsd|windows) "$mode" "$@" ;;
    *) echo "Unsupported mode: $mode"
       echo "Usage: $0 <deb|rpm|macos|tarball|freebsd|windows> [version]"
       exit 1 ;;
esac
