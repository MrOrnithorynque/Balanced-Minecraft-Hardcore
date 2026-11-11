#!/bin/bash
# Builds the content mod (horizon/content) and checks the .hmod file that the
# build writes, with Horizon's cargo-horizon-mod tool.
#
# Usage: sh horizon/tools/content-check.sh
# Environment:
#   BMH_HORIZON_MOD_BIN  the cargo-horizon-mod binary (default: cargo-horizon-mod on the PATH).
set -eu

here="$(cd "$(dirname "$0")" && pwd)"
content="$(cd "$here/../content" && pwd)"
bin="${BMH_HORIZON_MOD_BIN:-cargo-horizon-mod}"

if ! command -v "$bin" >/dev/null 2>&1; then
    echo "content-check.sh: no cargo-horizon-mod binary ($bin). Set BMH_HORIZON_MOD_BIN." >&2
    exit 127
fi

cd "$content"
echo "content-check.sh: build in $content"
"$bin" horizon-mod build
echo "content-check.sh: check target/horizon-mod/bmh.hmod"
"$bin" horizon-mod check target/horizon-mod/bmh.hmod
