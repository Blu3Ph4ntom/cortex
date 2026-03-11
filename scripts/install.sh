#!/usr/bin/env sh
set -eu

REPO="Blu3Ph4ntom/cortex"
INSTALL_DIR="${CORTEX_INSTALL_DIR:-$HOME/.local/bin}"

uname_s="$(uname -s)"
uname_m="$(uname -m)"

case "$uname_s" in
  Linux) os="linux" ;;
  Darwin) os="macos" ;;
  *)
    echo "unsupported OS: $uname_s" >&2
    exit 1
    ;;
esac

case "$uname_m" in
  x86_64|amd64) arch="x86_64" ;;
  arm64|aarch64) arch="aarch64" ;;
  *)
    echo "unsupported architecture: $uname_m" >&2
    exit 1
    ;;
esac

artifact="cortex-${os}-${arch}.tar.gz"
api_url="https://api.github.com/repos/${REPO}/releases/latest"

tmp_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT INT TERM

download_url="$(curl -fsSL "$api_url" | grep browser_download_url | grep "$artifact" | cut -d '"' -f 4)"

if [ -z "$download_url" ]; then
  echo "release artifact not found: $artifact" >&2
  exit 1
fi

mkdir -p "$INSTALL_DIR"
curl -fsSL "$download_url" -o "$tmp_dir/$artifact"
tar -xzf "$tmp_dir/$artifact" -C "$tmp_dir"
install "$tmp_dir/cortex" "$INSTALL_DIR/cortex"
install "$tmp_dir/cortexd" "$INSTALL_DIR/cortexd"

echo "installed cortex and cortexd to $INSTALL_DIR"
