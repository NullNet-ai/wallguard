#!/bin/bash
set -e

case "$1" in
    deb)
        cargo build --release -p wallguard -p wallguard-cli

        mkdir -p packages/debian/usr/local/bin
        cp target/release/wallguard packages/debian/usr/local/bin/
        cp target/release/wallguard-cli packages/debian/usr/local/bin/

        dpkg-deb --build packages/debian ./

        rm packages/debian/usr/local/bin/*

        ;;
    *)
        echo "Unsupported or missing parameter."
        exit 1
        ;;
esac
