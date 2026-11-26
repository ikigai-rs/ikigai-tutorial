#!/usr/bin/env bash
# Exercise the prerequisite without depending on the installed Node version.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
cat > "$tmp/node" <<'STUB'
#!/usr/bin/env bash
printf '%s\n' "${STUB_NODE_VERSION}"
STUB
chmod +x "$tmp/node"
source scripts/require-node.sh
export PATH="$tmp:$PATH"
export STUB_NODE_VERSION=v12.22.9
expected='Found Node v12.22.9; Node 22 or newer is required (see README.md, The book is tested); install it with nvm install 22.'
if output="$(require_node 2>&1)"; then
    echo 'Node 12 was accepted' >&2
    exit 1
fi
if [[ "$output" != "$expected" ]]; then
    echo "Unexpected diagnostic: $output" >&2
    exit 1
fi
export STUB_NODE_VERSION=v22.0.0
require_node
export STUB_NODE_VERSION=v100.0.0
require_node
export STUB_NODE_VERSION=invalid
if require_node 2>/dev/null; then
    echo 'Invalid Node version was accepted' >&2
    exit 1
fi
