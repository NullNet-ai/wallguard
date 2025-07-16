#!/bin/bash
set -e

case "$1" in
    deb)
        cargo build --release -p wallguard -p wallguard-cli

        mkdir -p packages/debian/usr/local/bin
        cp target/release/wallguard packages/debian/usr/local/bin/
        cp target/release/wallguard-cli packages/debian/usr/local/bin/

        dpkg-deb --build packages/debian ./

        rm packages/debian/usr/local/bin/wallguard
        rm packages/debian/usr/local/bin/wallguard-cli
        ;;
    freebsd)
        cargo build --release -p wallguard -p wallguard-cli

        mkdir -p packages/freebsd/usr/local/bin
        cp target/release/wallguard packages/freebsd/usr/local/bin/
        cp target/release/wallguard-cli packages/freebsd/usr/local/bin/

        pkg create -M packages/freebsd/+MANIFEST -r packages/freebsd

        rm packages/freebsd/usr/local/bin/wallguard
        rm packages/freebsd/usr/local/bin/wallguard-cli
    *)
        echo "Unsupported or missing parameter."
        exit 1
        ;;
esac
