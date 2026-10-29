#!/bin/sh
# Rebuild the river and the four views from the fixture, then fail if any differs from the
# committed copy. Offline and deterministic: nothing here reaches the network.
set -u
H=$(cd "$(dirname "$0")" && pwd)
"$H/iki" --quiet < "$H/ws/save-searches.iki" > /dev/null || exit 1
"$H/iki" --quiet < "$H/ws/refresh.iki" > /dev/null || exit 1
git -C "$H" diff --exit-code -- ws/river.nt ws/out
