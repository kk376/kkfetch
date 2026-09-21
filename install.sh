#!/usr/bin/env bash
# KKFetch Universal POSIX Installer
set -euo pipefail

PREFIX="${PREFIX:-/usr/local}"
BIN_DIR="${PREFIX}/bin"
SHARE_DIR="${PREFIX}/share"
MANDIR="${SHARE_DIR}/man/man1"
DOCDIR="${SHARE_DIR}/doc/kkfetch"
BASH_COMP="${SHARE_DIR}/bash-completion/completions"
ZSH_COMP="${SHARE_DIR}/zsh/site-functions"
FISH_COMP="${SHARE_DIR}/fish/vendor_completions.d"

echo "========================================================"
echo "               KKFetch Universal Installer              "
echo "========================================================"

# Determine source binary
SOURCE_BIN=""
if [ -f "target/release/kkfetch" ]; then
    SOURCE_BIN="target/release/kkfetch"
elif [ -f "kkfetch" ]; then
    SOURCE_BIN="kkfetch"
else
    echo "Error: kkfetch binary not found. Build with 'cargo build --release' first." >&2
    exit 1
fi

# Automatically remove conflicting/shadowed binaries in user environment
SUDO_CALLER="${SUDO_USER:-$USER}"
if [ -n "$SUDO_CALLER" ]; then
    CALLER_HOME=$(getent passwd "$SUDO_CALLER" 2>/dev/null | cut -d: -f6 || echo "${HOME:-}")
    if [ -n "$CALLER_HOME" ]; then
        for conflicting in "$CALLER_HOME/.cargo/bin/kkfetch" "$CALLER_HOME/.local/bin/kkfetch"; do
            if [ -f "$conflicting" ]; then
                rm -f "$conflicting" 2>/dev/null || true
            fi
        done
        rm -f "$CALLER_HOME/.cache/kkfetch/de_*.cache" "$CALLER_HOME/.cache/kkfetch/theme_*.cache" 2>/dev/null || true
    fi
fi

# Create target directories
echo "Installing to ${PREFIX}..."
mkdir -p "${BIN_DIR}" "${MANDIR}" "${DOCDIR}" "${BASH_COMP}" "${ZSH_COMP}" "${FISH_COMP}"

# Install binary
install -m 0755 "${SOURCE_BIN}" "${BIN_DIR}/kkfetch"
echo "  Installed ${BIN_DIR}/kkfetch"

# Install shell completions
if [ -f "completions/kkfetch.bash" ]; then
    install -m 0644 "completions/kkfetch.bash" "${BASH_COMP}/kkfetch"
    echo "  Installed Bash completion"
fi
if [ -f "completions/_kkfetch" ]; then
    install -m 0644 "completions/_kkfetch" "${ZSH_COMP}/_kkfetch"
    echo "  Installed Zsh completion"
fi
if [ -f "completions/kkfetch.fish" ]; then
    install -m 0644 "completions/kkfetch.fish" "${FISH_COMP}/kkfetch.fish"
    echo "  Installed Fish completion"
fi

# Install manual page
if [ -f "docs/kkfetch.1" ]; then
    install -m 0644 "docs/kkfetch.1" "${MANDIR}/kkfetch.1"
    echo "  Installed man page to ${MANDIR}/kkfetch.1"
fi

# Install documentation and license
if [ -f "README.md" ]; then
    install -m 0644 "README.md" "${DOCDIR}/README.md"
fi
if [ -f "LICENSE" ]; then
    install -m 0644 "LICENSE" "${DOCDIR}/LICENSE"
fi

echo "========================================================"
echo "KKFetch successfully installed to ${BIN_DIR}/kkfetch"
echo "Run 'kkfetch --doctor' to verify installation health."
echo "========================================================"
