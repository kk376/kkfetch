#!/bin/sh
# Automatically remove any older conflicting cargo or local binaries in PATH
if [ -n "$SUDO_USER" ]; then
    USER_HOME=$(getent passwd "$SUDO_USER" | cut -d: -f6)
    if [ -n "$USER_HOME" ]; then
        for shadowed in "$USER_HOME/.cargo/bin/kkfetch" "$USER_HOME/.local/bin/kkfetch" "/usr/local/bin/kkfetch"; do
            if [ -f "$shadowed" ]; then
                rm -f "$shadowed" 2>/dev/null || true
            fi
        done
        rm -f "$USER_HOME/.cache/kkfetch/de_*.cache" "$USER_HOME/.cache/kkfetch/theme_*.cache" 2>/dev/null || true
    fi
fi
