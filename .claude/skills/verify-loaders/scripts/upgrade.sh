#!/bin/bash
# Upgrade test on one loader: a world made by the old release, then the same world and player under the new code.
# usage: upgrade.sh <old origin branch or tag> <new origin branch or tag> <minecraft version> <loader> <label>
#   e.g. upgrade.sh mc-1.20.4-v1.4.0 dev-49.0.31 1.20.4 fabric 1.20.4
# Two worktrees of its own (upgrade-old-<label>, upgrade-new-<label>) under <repo>/.claude/worktrees, the same locks as
# branch.sh, port 25619 (RCON 25719; BMH_UPGRADE_PORT overrides). upgrade_bot.js gives the items of a long-time player on
# the old server and checks what carried over on the new one. Results: $BMH_VERIFY_OUT/upgrade-<label>/.
# The bots' mineflayer installs itself (need_bots in lib.sh).
set -u
[ $# -eq 5 ] || { echo "usage: upgrade.sh <old origin branch or tag> <new origin branch or tag> <minecraft version> <loader> <label>" >&2; exit 2; }
OLD=$1; NEW=$2; MCV=$3; L=$4; LABEL=$5
SCRIPTS=$(cd "$(dirname "$0")" && pwd)
BMH_PORT=${BMH_UPGRADE_PORT:-25619}   # lib.sh reads BMH_PORT
. "$SCRIPTS/lib.sh"
need_bots
JH=$(jdk_for "$MCV") || exit 2
OUT=$OUTROOT/upgrade-$LABEL; mkdir -p "$OUT"
trap release EXIT

serve() { # <worktree> <log> <mode>: boot, run the bot in <mode>, stop
  take_lock; take_slot
  boot_server "$1" "$L" "$JH" "$2"
  if [ $UP = 1 ]; then
    with_timeout 300 node "$SCRIPTS/upgrade_bot.js" $PORT "$MCV" "$3" > "$OUT/bot-$3.log" 2>&1
    stop_server
  fi
  release
  echo "$3 server up=$UP: $(tail -1 "$OUT/bot-$3.log" 2>/dev/null)"
}

git -C "$REPO" fetch -q origin
OREF=$(ref_of "$OLD") || { echo "no origin/$OLD and no tag $OLD"; exit 2; }
NREF=$(ref_of "$NEW") || { echo "no origin/$NEW and no tag $NEW"; exit 2; }
WO=$REPO/.claude/worktrees/upgrade-old-$LABEL; WN=$REPO/.claude/worktrees/upgrade-new-$LABEL
fresh_tree "$WO" "$OREF"; fresh_tree "$WN" "$NREF"
echo "== $LABEL: $OLD $(git -C "$WO" rev-parse --short HEAD) -> $NEW $(git -C "$WN" rev-parse --short HEAD), $L"
rm -rf "$WO/$L/run/world"; write_server_properties "$WO/$L/run"
serve "$WO" "$OUT/server-old.log" old
rm -rf "$WN/$L/run/world"; write_server_properties "$WN/$L/run"
cp -R "$WO/$L/run/world" "$WN/$L/run/world" && rm -f "$WN/$L/run/world/session.lock"
serve "$WN" "$OUT/server-new.log" new
echo "errors in the new server log: $(grep -E '/ERROR\]|Exception|Caused by|Failed to (load|read)' "$OUT/server-new.log" | grep -v -c 'test_block')"
