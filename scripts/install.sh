#!/usr/bin/env sh
set -e

REPO="larscom/rusty-relay"
BINARY="rusty-relay-client"
TMPDIR=$(mktemp -d)
: ${USE_SUDO:="true"}
: ${INSTALL_DIR:="/usr/local/bin"}

trap 'rm -rf "$TMPDIR"' EXIT

echo "🔍 Detecting platform..."

OS=$(uname -s)
ARCH=$(uname -m)

runAsRoot() {
  if [ $EUID -ne 0 -a "$USE_SUDO" = "true" ]; then
    sudo "${@}"
  else
    "${@}"
  fi
}

case "$OS" in
    Linux)
        OS="linux"
        ;;
    Darwin)
        OS="macos"
        ;;
    *)
        echo "❌ Unsupported OS: $OS"
        echo "Supported: Linux, macOS"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64)
        ARCH="x86_64"
        ;;
    arm64|aarch64)
        ARCH="arm64"
        ;;
    *)
        echo "❌ Unsupported architecture: $ARCH"
        echo "Supported: x86_64, arm64"
        exit 1
        ;;
esac

echo "➡ OS: $OS"
echo "➡ ARCH: $ARCH"

API_URL="https://api.github.com/repos/$REPO/releases/latest"

echo "📦 Fetching latest release metadata..."
JSON=$(curl -sSf "$API_URL")

TAG=$(echo "$JSON" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
if [ -z "$TAG" ]; then
    echo "❌ Could not determine latest version"
    exit 1
fi

echo "🆕 Latest version: $TAG"

ASSET_NAME="$BINARY-$TAG-$OS-$ARCH"

URL=$(echo "$JSON" | grep -Eo "https://[^\"]*$ASSET_NAME[^\"\ ]*")

if [ -z "$URL" ]; then
    echo "❌ No matching binary found for: $OS-$ARCH"
    exit 1
fi

ARCHIVE_NAME="$ASSET_NAME.tar.gz"
ARCHIVE_FILE="$TMPDIR/$ARCHIVE_NAME"

echo "⬇ Downloading: $URL"
curl -s -L -o "$ARCHIVE_FILE" "$URL"

if echo "$ARCHIVE_NAME" | grep -q ".tar.gz"; then
    echo "📂 Extracting..."
    tar -xzf "$ARCHIVE_FILE" -C "$TMPDIR"
else
    echo "❌ Unexpected archive format (expected .tar.gz)"
    exit 1
fi

echo "📦 Installing to $INSTALL_DIR"
runAsRoot install -m 755 "$TMPDIR/$BINARY" /usr/local/bin/$BINARY

rm -rf "$TMPDIR"

echo "✔ Installed"
