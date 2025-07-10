#!/bin/bash

set -euo pipefail

trap 'echo "Exit due to a previous failure. See output for detail." >&2' ERR

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Description:"
    echo "  Installation script of llman CLI utility."
    echo "  This bash script will install llman for the current user."
    echo "  This script should only be invoked from the root of project folder."
    echo
    echo "OPTIONS:"
    echo "  --system, -s: Install llman to /usr/local/bin/ (i.e. system-wide). Must run with sudo."
    echo "  Note: By default, llman will only be installed by cargo install for the current user only."
    echo
    echo "  --debug, -d: Install with debug info. Otherwise, build and install with release profile"
    echo
    echo "  --uninstall, -u: Uninstall llman for the current user. If --system or -s is supplied, remove globally."
    echo "  (i.e. remove from /usr/local/bin/)"
    echo
    echo "  --help, -h: Display this usage info and exit with 0. Should not combine with other options."
}

# parse cmdline options
SYS_FLAG="0"
DBG_FLAG="0"
UNINSTALL_FLAG="0"

while [[ "$#" -gt 0 ]]
do
    case "$1" in
        --system|-s)
            SYS_FLAG="1"
            shift
            ;;
        --debug|-d)
            DBG_FLAG="1"
            shift
            ;;
        --uninstall|-u)
            UNINSTALL_FLAG="1"
            shift
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            echo "Unrecognized option." >&2
            usage >&2
            exit 1
            ;;
    esac
done

OPTS=()
if [[ "$DBG_FLAG" -eq 1 ]]
then
    OPTS+=( "--debug" )
fi

# check if Rust is installed or not
if ! command -v cargo &> /dev/null
then
    echo "Rust is not installed for the user $USER. Go to https://www.rust-lang.org/tools/install and install Rust first." >&2
    exit 1
fi

# install / uninstall llman
if [[ "$UNINSTALL_FLAG" -eq 0 ]]
then
    cargo install --path . "${OPTS[@]}"
    if [[ "$SYS_FLAG" -eq 1 ]]
    then
        echo "Copying to /usr/local/bin..."
        sudo cp "$HOME"/.cargo/bin/llman /usr/local/bin/llman
        cargo uninstall llman
        echo "Done. You will need to manually remove /usr/local/bin/llman by yourself for uninstallation."
        echo "Or run $0 --uninstall --system for uninstallation"
    else
        echo "Installation for user $USER completed. Run llman --help to see usage info."
        echo "Run cargo uninstall llman if you want to remove it later."
    fi
else
    if [[ "$SYS_FLAG" -eq 1 ]]
    then
        sudo rm /usr/local/bin/llman
    else
        cargo uninstall llman
    fi
    if [[ -d ~/.llman ]]
    then
        rm -rf ~/.llman
    fi
    echo "Uninstallation completed."
fi
