#!/usr/bin/env sh

set -eu

REPO="yumeminami/obsctl"
INSTALL_ROOT="${OBSCTL_INSTALL_DIR:-$HOME/.local/bin}"

command -v curl >/dev/null 2>&1 || {
  echo "error: curl is required to download obsctl" >&2
  exit 1
}

OS=$(uname -s 2>/dev/null || echo unknown)
ARCH=$(uname -m 2>/dev/null || echo unknown)

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64|amd64) SUFFIX="linux-x86_64" ;;
      aarch64|arm64) SUFFIX="linux-aarch64" ;;
      *) echo "error: unsupported Linux architecture: $ARCH" >&2; exit 1 ;;
    esac
    ;;
  Darwin)
    case "$ARCH" in
      x86_64) SUFFIX="macos-x86_64" ;;
      arm64) SUFFIX="macos-arm64" ;;
      *) echo "error: unsupported macOS architecture: $ARCH" >&2; exit 1 ;;
    esac
    ;;
  *)
    echo "error: unsupported operating system: $OS" >&2
    exit 1
    ;;
    esac

API_URL="https://api.github.com/repos/$REPO/releases/latest"
RELEASE_JSON=$(curl -fsSL -H "Accept: application/vnd.github+json" -H "User-Agent: obsctl-installer" "$API_URL") || {
  echo "error: failed to query latest release metadata" >&2
  exit 1
}

if [ -z "$RELEASE_JSON" ]; then
  echo "error: GitHub release metadata response was empty" >&2
  exit 1
fi

TAG=$(printf '%s\n' "$RELEASE_JSON" | grep -m1 '"tag_name"' | sed 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/')
if [ -z "$TAG" ]; then
  echo "error: unable to determine latest tag" >&2
  exit 1
fi

ASSET_NAME="obsctl-$TAG-$SUFFIX.tar.gz"
ASSET_URL=$(printf '%s' "$RELEASE_JSON" |
  grep -o '"browser_download_url":"[^"]*"' |
  grep "$ASSET_NAME" |
  sed 's/"browser_download_url":"//' |
  sed 's/"$//')

if [ -z "$ASSET_URL" ]; then
  echo "error: no release asset found for suffix $SUFFIX" >&2
  exit 1
fi

TMPDIR=$(mktemp -d)
TARBALL="$TMPDIR/package.tar.gz"

trap 'rm -rf "$TMPDIR"' EXIT INT TERM

echo "Downloading obsctl $TAG ($SUFFIX)..."
curl -fsSL "$ASSET_URL" -o "$TARBALL"

echo "Extracting binaries..."
tar -xzf "$TARBALL" -C "$TMPDIR"

mkdir -p "$INSTALL_ROOT"
install "$TMPDIR/obsctl" "$INSTALL_ROOT/obsctl"
install "$TMPDIR/obsctl_mcp" "$INSTALL_ROOT/obsctl_mcp"

echo "Installed to $INSTALL_ROOT"
echo "Add the following to your shell profile if necessary:"
echo "  export PATH=\"$INSTALL_ROOT:\$PATH\""
