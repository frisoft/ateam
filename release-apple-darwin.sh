#!/usr/bin/env bash

# example: ./release-apple-darwin.sh 0.7.1

set -euo pipefail

ver=$1
url="https://github.com/frisoft/ateam/releases/download/v$ver/ateam-v$ver-x86_64-apple-darwin.tar.xz"
output="/tmp/ateam-v$ver-x86_64-apple-darwin.tar.xz"
curl -fLo "$output" "$url"
echo $(sha256sum $output)
echo "Use the hash in the Homebrew formula."
