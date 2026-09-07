#!/bin/sh
set -e

INSTALL_DIR="${JULIX_HOME:-$HOME/.julix}"
BIN_DIR="$INSTALL_DIR/bin"
VERSION="${JULIX_VERSION:-latest}"
REPO="furimeo/julix"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux*) PLATFORM="linux";;
    Darwin*) PLATFORM="macos";;
    *) echo "Unsupported OS: $OS"; exit 1;;
esac

case "$ARCH" in
    x86_64|amd64) ARCH="x86_64";;
    arm64|aarch64) ARCH="aarch64";;
    *) echo "Unsupported arch: $ARCH"; exit 1;;
esac

echo "Installing Julix $VERSION for $PLATFORM-$ARCH..."

mkdir -p "$BIN_DIR"

if [ "$VERSION" = "latest" ]; then
    URL="https://github.com/$REPO/releases/latest/download/julix-$ARCH-$PLATFORM.tar.gz"
else
    URL="https://github.com/$REPO/releases/download/$VERSION/julix-$ARCH-$PLATFORM.tar.gz"
fi

TMP=$(mktemp -d)
curl -sSfL "$URL" -o "$TMP/julix.tar.gz"
tar -xzf "$TMP/julix.tar.gz" -C "$TMP"
cp "$TMP/julix" "$BIN_DIR/julix"
chmod +x "$BIN_DIR/julix"

ln -sf julix "$BIN_DIR/lixvm"
ln -sf julix "$BIN_DIR/jujit"

rm -rf "$TMP"

if ! echo "$PATH" | grep -q "$BIN_DIR"; then
    RC=""
    if [ -n "$SHELL" ]; then
        SHELL_NAME=$(basename "$SHELL")
        case "$SHELL_NAME" in
            bash) RC="$HOME/.bashrc";;
            zsh) RC="$HOME/.zshrc";;
            fish) RC="$HOME/.config/fish/config.fish";;
        esac
    fi
    if [ -z "$RC" ]; then
        if [ -f "$HOME/.bashrc" ]; then RC="$HOME/.bashrc"
        elif [ -f "$HOME/.zshrc" ]; then RC="$HOME/.zshrc"
        else RC="$HOME/.profile"
        fi
    fi
    echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$RC"
    echo "Added $BIN_DIR to PATH in $RC"
    echo "Run: source $RC"
fi

echo "Done. Julix installed to $BIN_DIR/julix"
julix --version
