#!/bin/bash
# Runs the unit tests of the logic crate (horizon/native, package bmh_logic).
# Warnings are errors, as in the game build.
# Usage: sh horizon/tools/test-logic.sh [extra cargo test arguments]
set -eu

here="$(cd "$(dirname "$0")" && pwd)"
native="$(cd "$here/../native" && pwd)"

cd "$native"
RUSTFLAGS="-D warnings" exec cargo test "$@"
