#!/bin/sh
# Check if the invoking user has an older conflicting cargo or local binary in PATH
if [ -n "$SUDO_USER" ]; then
    USER_HOME=$(getent passwd "$SUDO_USER" | cut -d: -f6)
    if [ -n "$USER_HOME" ]; then
        for shadowed in "$USER_HOME/.cargo/bin/kkfetch" "$USER_HOME/.local/bin/kkfetch" "/usr/local/bin/kkfetch"; do
            if [ -f "$shadowed" ]; then
                echo "--------------------------------------------------------------------------------"
                echo "KKFetch Packaging Notice:"
                echo "Detected an existing binary at: $shadowed"
                echo "Your shell PATH may prioritize it over the package binary at /usr/bin/kkfetch."
                echo "To avoid version conflicts, run: rm -f \"$shadowed\""
                echo "--------------------------------------------------------------------------------"
            fi
        done
    fi
fi
