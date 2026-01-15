#!/usr/bin/env bash
# Install Git hooks for TALON development
# Run this script once after cloning the repository

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔══════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║      TALON Git Hooks Installation                ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════╝${NC}"
echo ""

# Detect repository root
REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null)
if [ -z "$REPO_ROOT" ]; then
    echo -e "${RED}ERROR: Not in a git repository${NC}"
    exit 1
fi

HOOKS_DIR="$REPO_ROOT/.git/hooks"
SCRIPTS_DIR="$REPO_ROOT/scripts"

# Check if scripts directory exists
if [ ! -d "$SCRIPTS_DIR" ]; then
    echo -e "${RED}ERROR: scripts/ directory not found${NC}"
    exit 1
fi

# Check if pre-commit script exists
if [ ! -f "$SCRIPTS_DIR/pre-commit.sh" ]; then
    echo -e "${RED}ERROR: scripts/pre-commit.sh not found${NC}"
    exit 1
fi

echo -e "${YELLOW}[1/4] Checking prerequisites...${NC}"

# Check for cargo
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}✗ cargo not found${NC}"
    echo -e "${YELLOW}  Install Rust toolchain from: https://rustup.rs/${NC}"
    exit 1
else
    echo -e "${GREEN}✓ cargo found${NC}"
fi

# Check for git
if ! command -v git &> /dev/null; then
    echo -e "${RED}✗ git not found${NC}"
    exit 1
else
    echo -e "${GREEN}✓ git found${NC}"
fi

echo ""
echo -e "${YELLOW}[2/4] Installing Git hooks...${NC}"

# Create hooks directory if it doesn't exist
mkdir -p "$HOOKS_DIR"

# Install pre-commit hook
HOOK_FILE="$HOOKS_DIR/pre-commit"
if [ -f "$HOOK_FILE" ]; then
    # Backup existing hook
    BACKUP_FILE="${HOOK_FILE}.backup.$(date +%Y%m%d_%H%M%S)"
    echo -e "${YELLOW}→ Backing up existing hook to: ${BACKUP_FILE}${NC}"
    mv "$HOOK_FILE" "$BACKUP_FILE"
fi

# Create symbolic link or copy script
if ln -sf "../../scripts/pre-commit.sh" "$HOOK_FILE" 2>/dev/null; then
    echo -e "${GREEN}✓ Created symlink: .git/hooks/pre-commit -> scripts/pre-commit.sh${NC}"
else
    # Fallback to copying if symlink fails
    cp "$SCRIPTS_DIR/pre-commit.sh" "$HOOK_FILE"
    echo -e "${GREEN}✓ Copied: scripts/pre-commit.sh -> .git/hooks/pre-commit${NC}"
fi

# Make hook executable
chmod +x "$HOOK_FILE"
chmod +x "$SCRIPTS_DIR/pre-commit.sh"

echo ""
echo -e "${YELLOW}[3/4] Optional: Installing pre-commit framework...${NC}"

# Check if Python is available for pre-commit framework
if command -v python3 &> /dev/null || command -v python &> /dev/null; then
    PYTHON_CMD=$(command -v python3 || command -v python)
    echo -e "${GREEN}✓ Python found: $PYTHON_CMD${NC}"
    
    # Check if pre-commit is installed
    if command -v pre-commit &> /dev/null; then
        echo -e "${GREEN}✓ pre-commit framework already installed${NC}"
        
        # Install pre-commit hooks
        cd "$REPO_ROOT"
        if pre-commit install; then
            echo -e "${GREEN}✓ pre-commit framework configured${NC}"
        else
            echo -e "${YELLOW}⚠ Failed to configure pre-commit framework${NC}"
        fi
    else
        echo -e "${YELLOW}→ pre-commit framework not installed${NC}"
        echo -e "${YELLOW}  To install: pip install pre-commit${NC}"
        echo -e "${YELLOW}  Then run: pre-commit install${NC}"
    fi
else
    echo -e "${YELLOW}⚠ Python not found - skipping pre-commit framework${NC}"
    echo -e "${YELLOW}  The bash hook will still work${NC}"
fi

echo ""
echo -e "${YELLOW}[4/4] Installing recommended tools...${NC}"

# Check for cargo-deny
if command -v cargo-deny &> /dev/null; then
    echo -e "${GREEN}✓ cargo-deny installed${NC}"
else
    echo -e "${YELLOW}→ cargo-deny not installed (optional)${NC}"
    echo -e "${YELLOW}  To install: cargo install cargo-deny${NC}"
fi

# Check for cargo-audit
if command -v cargo-audit &> /dev/null; then
    echo -e "${GREEN}✓ cargo-audit installed${NC}"
else
    echo -e "${YELLOW}→ cargo-audit not installed (optional)${NC}"
    echo -e "${YELLOW}  To install: cargo install cargo-audit${NC}"
fi

# Final summary
echo ""
echo -e "${BLUE}╔══════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  ✓ Git hooks installation complete!             ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo -e "  1. Test the hook: ${BLUE}scripts/pre-commit.sh${NC}"
echo -e "  2. Make a commit to see it in action"
echo -e "  3. To skip hooks: ${BLUE}git commit --no-verify${NC} (not recommended)"
echo ""
echo -e "${YELLOW}Recommended optional tools:${NC}"
echo -e "  • pre-commit framework: ${BLUE}pip install pre-commit${NC}"
echo -e "  • cargo-deny: ${BLUE}cargo install cargo-deny${NC}"
echo -e "  • cargo-audit: ${BLUE}cargo install cargo-audit${NC}"
echo ""
