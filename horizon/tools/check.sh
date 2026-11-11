#!/bin/bash
# Compiles the native bundle of this repo (horizon/native) into Horizon with
# Horizon's own tool: native.py check. Prints only what matters and exits with
# the status of the tool.
#
# Usage: sh horizon/tools/check.sh
# Environment:
#   BMH_HORIZON_CACHE  the cache folder (default: ~/Library/Caches/bmh-horizon).
#                      It holds horizon-mods (a clone of the registry repo, with
#                      native.py and its build cache), the lock and the logs.
#
# Written for /bin/bash 3.2 on macOS: no associative arrays, no empty arrays.
set -eu

here="$(cd "$(dirname "$0")" && pwd)"
bundle="$(cd "$here/../native" && pwd)"
cache="${BMH_HORIZON_CACHE:-$HOME/Library/Caches/bmh-horizon}"
mods="$cache/horizon-mods"
lock="$cache/check.lock"
logs="$cache/logs"

mkdir -p "$cache" "$logs"

if [ ! -f "$mods/native/tools/native.py" ]; then
    echo "check.sh: no horizon-mods in $cache. Cloning it (depth 1)."
    git clone --quiet --depth 1 https://github.com/horizon-rust/horizon-mods "$mods"
fi

# The lock: native.py has one stage folder per set of mods, so two runs must
# not overlap. A lock whose process is dead is taken over.
waited=0
while ! mkdir "$lock" 2>/dev/null; do
    owner="$(cat "$lock/pid" 2>/dev/null || true)"
    if [ -n "$owner" ] && ! kill -0 "$owner" 2>/dev/null; then
        echo "check.sh: the lock of process $owner is stale. Taking it over."
        rm -rf "$lock"
        continue
    fi
    if [ "$waited" -eq 0 ]; then
        echo "check.sh: another check (process ${owner:-unknown}) holds $lock. Waiting."
    fi
    if [ "$waited" -ge 1200 ]; then
        echo "check.sh: gave up after 20 minutes. Process ${owner:-unknown} still holds $lock." >&2
        exit 75
    fi
    sleep 1
    waited=$((waited + 1))
done
echo "$$" > "$lock/pid"
trap 'rm -rf "$lock"' EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

log="$logs/check-$(date +%Y%m%d-%H%M%S)-$$.log"
echo "check.sh: bundle $bundle"
echo "check.sh: full log $log"

status=0
(cd "$mods" && python3 native/tools/native.py check "$bundle") > "$log" 2>&1 || status=$?

# Cargo progress lines carry no news: drop them, print the rest.
grep -Ev '^[[:space:]]*(Checking|Compiling|Downloaded|Downloading|Locking|Adding|Updating)[[:space:]]' "$log" || true

if [ "$status" -ne 0 ]; then
    echo "check.sh: native.py check failed with status $status." >&2
fi
exit "$status"
