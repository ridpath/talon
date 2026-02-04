#!/usr/bin/env bash

set -e

INSTALL_DIR="${INSTALL_DIR:-$HOME/.talon}"
BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"
BINARY_NAME="talon"
REPO_URL="https://github.com/iamtalon/talon"
VERSION="${VERSION:-latest}"

detect_os() {
    case "$(uname -s)" in
        Linux*)     echo "linux";;
        Darwin*)    echo "macos";;
        CYGWIN*|MINGW*|MSYS*) echo "windows";;
        *)          echo "unknown";;
    esac
}

detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)   echo "x86_64";;
        aarch64|arm64)  echo "aarch64";;
        armv7l)         echo "armv7";;
        i686|i386)      echo "i686";;
        *)              echo "unknown";;
    esac
}

main() {
    echo "TALON Installer"
    echo "==============="
    echo ""

    OS=$(detect_os)
    ARCH=$(detect_arch)

    if [ "$OS" = "unknown" ] || [ "$ARCH" = "unknown" ]; then
        echo "Error: Unsupported OS ($OS) or architecture ($ARCH)"
        exit 1
    fi

    echo "Detected OS: $OS"
    echo "Detected Architecture: $ARCH"
    echo ""

    BINARY_URL="${REPO_URL}/releases/${VERSION}/download/talon-${OS}-${ARCH}"

    if [ ! -d "$INSTALL_DIR" ]; then
        echo "Creating installation directory: $INSTALL_DIR"
        mkdir -p "$INSTALL_DIR"
    fi

    if [ ! -d "$BIN_DIR" ]; then
        echo "Creating binary directory: $BIN_DIR"
        mkdir -p "$BIN_DIR"
    fi

    echo "Installing TALON to $INSTALL_DIR..."

    if [ -f "./target/release/$BINARY_NAME" ]; then
        echo "Using locally built binary"
        cp "./target/release/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
    elif [ -f "./$BINARY_NAME" ]; then
        echo "Using local binary"
        cp "./$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
    else
        echo "Error: No binary found. Please build TALON first with 'cargo build --release'"
        exit 1
    fi

    chmod +x "$INSTALL_DIR/$BINARY_NAME"

    if [ ! -L "$BIN_DIR/$BINARY_NAME" ]; then
        echo "Creating symlink in $BIN_DIR"
        ln -sf "$INSTALL_DIR/$BINARY_NAME" "$BIN_DIR/$BINARY_NAME"
    fi

    case ":${PATH}:" in
        *":${BIN_DIR}:"*)
            echo "PATH already contains $BIN_DIR"
            ;;
        *)
            echo ""
            echo "Adding $BIN_DIR to PATH"
            
            SHELL_RC=""
            if [ -n "$BASH_VERSION" ]; then
                SHELL_RC="$HOME/.bashrc"
            elif [ -n "$ZSH_VERSION" ]; then
                SHELL_RC="$HOME/.zshrc"
            else
                SHELL_RC="$HOME/.profile"
            fi

            if [ -f "$SHELL_RC" ]; then
                if ! grep -q "export PATH=\"\$BIN_DIR:\$PATH\"" "$SHELL_RC"; then
                    echo "" >> "$SHELL_RC"
                    echo "# TALON binary path" >> "$SHELL_RC"
                    echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$SHELL_RC"
                    echo "Added PATH export to $SHELL_RC"
                fi
            fi
            
            export PATH="$BIN_DIR:$PATH"
            ;;
    esac

    mkdir -p "$HOME/.talon"
    
    echo ""
    echo "Installation complete!"
    echo ""
    echo "TALON is installed at: $INSTALL_DIR/$BINARY_NAME"
    echo "Symlink created at: $BIN_DIR/$BINARY_NAME"
    echo ""
    echo "Run 'talon --help' to get started"
    echo "Run 'talon learn' for an interactive tutorial"
    echo "Run 'talon new' to see available exploit templates"
    echo ""
    
    if ! command -v talon &> /dev/null; then
        echo "Note: You may need to restart your shell or run:"
        echo "  source $SHELL_RC"
        echo "to use the 'talon' command"
    fi
}

main "$@"
