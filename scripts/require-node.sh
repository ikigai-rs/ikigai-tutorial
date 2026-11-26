#!/usr/bin/env bash
# Source this before running Node or a tool that needs it.
require_node() {
    local version major
    version="$(node --version 2>/dev/null)" || version="not installed"
    if [[ "$version" =~ ^v([0-9]+)\.[0-9]+\.[0-9]+([.-].*)?$ ]]; then
        major="${BASH_REMATCH[1]}"
        if (( 10#$major >= 22 )); then
            return 0
        fi
    fi
    echo "Found Node $version; Node 22 or newer is required (see README.md, The book is tested); install it with nvm install 22." >&2
    return 1
}
