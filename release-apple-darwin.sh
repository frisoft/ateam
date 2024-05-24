#!/usr/bin/env bash

# example: ./release-apple-darwin.sh 1.0.7

set -euo pipefail

ver=$1
url="https://github.com/frisoft/ateam/releases/download/v$ver/ateam-v$ver-x86_64-apple-darwin.tar.xz"
output="/tmp/ateam-v$ver-x86_64-apple-darwin.tar.xz"
curl -fLo "$output" "$url"
x86_64_sha256=$(sha256sum $output | awk '{print $1}')

url="https://github.com/frisoft/ateam/releases/download/v$ver/ateam-v$ver-aarch64-apple-darwin.tar.xz"
output="/tmp/ateam-v$ver-aarch64-apple-darwin.tar.xz"
curl -fLo "$output" "$url"
aarch64_sha256=$(sha256sum $output | awk '{print $1}')

echo 'class Ateam < Formula'
echo '  desc "A tool that helps optimize the code review process"'
echo '  homepage "https://github.com/frisoft/ateam"'
echo "  version \"$ver\""
echo ''
echo '  if Hardware::CPU.intel?'
echo "    url \"https://github.com/frisoft/ateam/releases/download/v$ver/ateam-v$ver-x86_64-apple-darwin.tar.xz\""
echo "    sha256 \"$x86_64_sha256\""
echo '  elsif Hardware::CPU.arm?'
echo "    url \"https://github.com/frisoft/ateam/releases/download/v$ver/ateam-v$ver-aarch64-apple-darwin.tar.xz\""
echo "    sha256 \"$aarch64_sha256\""
echo '  end'
echo ''
echo '  def install'
echo '    bin.install "ateam"'
echo '  end'
echo 'end'
