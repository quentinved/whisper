#!/bin/sh
# Whisper Secrets CLI installer
# Usage: curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/quentinved/Whisper/main/install.sh | sh

set -eu

REPO="quentinved/Whisper"
BINARY="whisper-secrets"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Detect OS
case "$(uname -s)" in
    Linux*)  OS="unknown-linux-gnu";;
    Darwin*) OS="apple-darwin";;
    MINGW*|MSYS*|CYGWIN*) OS="pc-windows-msvc";;
    *) echo "Error: Unsupported operating system: $(uname -s)"; exit 1;;
esac

# Detect architecture
case "$(uname -m)" in
    x86_64|amd64) ARCH="x86_64";;
    aarch64|arm64) ARCH="aarch64";;
    *) echo "Error: Unsupported architecture: $(uname -m)"; exit 1;;
esac

TARGET="${ARCH}-${OS}"

# Check supported combinations
case "${TARGET}" in
    x86_64-unknown-linux-gnu|aarch64-unknown-linux-gnu|aarch64-apple-darwin|x86_64-pc-windows-msvc) ;;
    *) echo "Error: Unsupported platform: ${TARGET}"; exit 1;;
esac

# Get latest version from GitHub API
echo "Fetching latest release..."
VERSION=$(curl -sSf "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"v([^"]+)".*/\1/')

if [ -z "$VERSION" ]; then
    echo "Error: Could not determine latest version"
    exit 1
fi

echo "Installing ${BINARY} v${VERSION} for ${TARGET}..."

# Download
ARCHIVE="${BINARY}-${VERSION}-${TARGET}"
if [ "$OS" = "pc-windows-msvc" ]; then
    URL="https://github.com/${REPO}/releases/download/v${VERSION}/${ARCHIVE}.zip"
    TMPFILE=$(mktemp /tmp/whisper-secrets-XXXXXX.zip)
    curl -sSfL "$URL" -o "$TMPFILE"
    unzip -o "$TMPFILE" -d /tmp
    cp "/tmp/${ARCHIVE}/${BINARY}.exe" "${INSTALL_DIR}/"
    rm -rf "$TMPFILE" "/tmp/${ARCHIVE}"
else
    URL="https://github.com/${REPO}/releases/download/v${VERSION}/${ARCHIVE}.tar.gz"
    TMPFILE=$(mktemp /tmp/whisper-secrets-XXXXXX.tar.gz)
    curl -sSfL "$URL" -o "$TMPFILE"
    tar xzf "$TMPFILE" -C /tmp
    cp "/tmp/${ARCHIVE}/${BINARY}" "${INSTALL_DIR}/"
    chmod +x "${INSTALL_DIR}/${BINARY}"
    rm -rf "$TMPFILE" "/tmp/${ARCHIVE}"
fi

echo ""
echo "  ${BINARY} v${VERSION} installed to ${INSTALL_DIR}/${BINARY}"
echo ""
echo "  Get started:"
echo "    whisper-secrets init"
echo "    whisper-secrets --help"
echo ""
