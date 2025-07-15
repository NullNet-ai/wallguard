#!/bin/bash
set -e

case "$1" in
    deb)
        cargo build --release

        mkdir -p packages/debian/usr/local/bin
        cp target/release/wallguard packages/debian/usr/local/bin/
        cp target/release/wallguard-cli packages/debian/usr/local/bin/

        dpkg-deb --build packages/debian ./

        rm packages/debian/usr/local/bin/*

        ;;
    *)
        echo "Usage: $0 deb"
        echo "Unsupported or missing parameter."
        exit 1
        ;;
esac
