#!/bin/bash
# Starts the built game in a window, with the BMH mod inside and the content mod on,
# so that a person can play it. The game uses its own folders in the cache (worlds,
# options, logs): it touches no other Horizon or Minecraft install.
#
# Usage: sh horizon/tools/play.sh [--build] [game arguments...]
#   --build   build the game from this repo's bundle first (sh horizon/tools/check.sh build)
# Example: sh horizon/tools/play.sh --world Test --seed 5
# Environment:
#   BMH_HORIZON_CACHE  the cache folder (default: ~/Library/Caches/bmh-horizon)
# Exit codes: 78 the built game or the prepared game files are missing; else the game's own.
#
# The game it starts is the one the last "check.sh build" made. Run with --build after a
# change to the mod. Written for /bin/bash 3.2 on macOS.
set -eu

here="$(cd "$(dirname "$0")" && pwd)"
content="$(cd "$here/../content" && pwd)"
cache="${BMH_HORIZON_CACHE:-$HOME/Library/Caches/bmh-horizon}"
bin="$cache/horizon-mods/native/target/cargo/fast/horizon-client"
resources="$cache/resources/26.3"

if [ "${1:-}" = "--build" ]; then
    shift
    sh "$here/check.sh" build
fi

if [ ! -x "$bin" ]; then
    echo "play.sh: no built game at $bin. Run: sh horizon/tools/check.sh build" >&2
    exit 78
fi
if [ ! -f "$resources/horizon-resources.json" ]; then
    echo "play.sh: no prepared game files in $resources. Run: sh horizon/tools/prepare-game.sh" >&2
    exit 78
fi

home="$cache/play/home"
game="$cache/play/game"
mkdir -p "$home" "$game"
echo "play.sh: worlds in $game/saves, log in $home/logs/latest.log"

HORIZON_HOME="$home" HORIZON_TELEMETRY_URL=off HORIZON_NO_DISCORD=1 HORIZON_LAUNCHER_NO_SELF_UPDATE=1 \
    exec "$bin" --resources "$resources" --game-dir "$game" --home "$home" --dev-mod "$content" "$@"
