#!/usr/bin/env bash
set -e

if [ $# -lt 1 ]; then
  echo "Usage: $0 <deb|freebsd> [version]"
  exit 1
fi

mode="$1"
shift

case "$mode" in
  deb)
    if [ -z "$1" ]; then
      echo "Error: missing version for deb package."
      echo "Usage: $0 deb <version>"
      exit 1
    fi
    VERSION="$1"

    cargo build --release -p wallguard -p wallguard-cli

    PKGDIR="packages/debian"
    DEBIANDIR="$PKGDIR/DEBIAN"

    mkdir -p "$DEBIANDIR"
    mkdir -p "$PKGDIR/usr/local/bin"

    sed "s/__VERSION__/${VERSION}/g" "$PKGDIR/control.tpl" > "$DEBIANDIR/control"

    cp target/release/wallguard "$PKGDIR/usr/local/bin/"
    cp target/release/wallguard-cli "$PKGDIR/usr/local/bin/"

    dpkg-deb --build "$PKGDIR" .

    rm "$PKGDIR/usr/local/bin/wallguard" "$PKGDIR/usr/local/bin/wallguard-cli"
    rm -rf "$DEBIANDIR"
    ;;

  freebsd)
    if [ -z "$1" ]; then
      echo "Error: missing version (or release tag) for freebsd package."
      echo "Usage: $0 freebsd <version>"
      exit 1
    fi
    VERSION="$1"

    cargo build --release -p wallguard -p wallguard-cli

    PKGDIR="packages/freebsd"

    mkdir -p "$PKGDIR/usr/local/bin"

    cp target/release/wallguard "$PKGDIR/usr/local/bin/"
    cp target/release/wallguard-cli "$PKGDIR/usr/local/bin/"

    sed "s/__VERSION__/${VERSION}/g" $PKGDIR/+MANIFEST.tpl > "$PKGDIR/+MANIFEST"

    pkg create -M $PKGDIR/+MANIFEST -r $PKGDIR

    rm $PKGDIR/+MANIFEST $PKGDIR/usr/local/bin/wallguard $PKGDIR/usr/local/bin/wallguard-cli
    ;;

  *)
    echo "Unsupported mode: $mode"
    echo "Usage: $0 <deb|freebsd> [version]"
    exit 1
    ;;
esac
