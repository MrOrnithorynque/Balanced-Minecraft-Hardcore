#!/bin/bash
# Upgrade test across Minecraft versions on one loader: a world made by the old ref on its Minecraft version, then the
# same world and player under the new ref on a newer Minecraft version (vanilla data fixers plus BMH's own migrations,
# such as the game rule names of 26.1). upgrade.sh does the same within one Minecraft version.
# usage: cross_upgrade.sh <old origin branch or tag> <old mc version> <new origin branch or tag> <new mc version> <loader> <label>
#   e.g. cross_upgrade.sh mc-1.21.1 1.21.1 mc-26.1 26.1.2 neoforge 1211-to-26
# The old version must be 1.20.5 or later (cross_upgrade_bot.js gives items with components). With BMH_LOCAL=1 a local
# branch of that name comes first (lib.sh ref_of). Two worktrees of its own (xupgrade-old-<label>, xupgrade-new-<label>),
# port 25619 (BMH_UPGRADE_PORT overrides). Results: $BMH_VERIFY_OUT/xupgrade-<label>/ (bot-old.log, bot-new.log, the
# two server logs, and the error lines of the new server log).
set -u
[ $# -eq 6 ] || { echo "usage: cross_upgrade.sh <old ref> <old mc> <new ref> <new mc> <loader> <label>" >&2; exit 2; }
OLD=$1; OMC=$2; NEW=$3; NMC=$4; L=$5; LABEL=$6
SCRIPTS=$(cd "$(dirname "$0")" && pwd)
BMH_PORT=${BMH_UPGRADE_PORT:-25619}   # lib.sh reads BMH_PORT
. "$SCRIPTS/lib.sh"
need_bots
OJH=$(jdk_for "$OMC") || exit 2
NJH=$(jdk_for "$NMC") || exit 2
OUT=$OUTROOT/xupgrade-$LABEL; mkdir -p "$OUT"
WO=$REPO/.claude/worktrees/xupgrade-old-$LABEL; WN=$REPO/.claude/worktrees/xupgrade-new-$LABEL
trap 'release; [ "$L" = fabric ] && { fabric_bot_unhook "$WO"; fabric_bot_unhook "$WN"; }' EXIT

serve() { # <worktree> <java home> <minecraft version> <log> <mode>: boot, run the bot in <mode>, stop
  take_lock; take_slot
  boot_server "$1" "$L" "$2" "$4"
  if [ $UP = 1 ]; then
    with_timeout 300 node "$SCRIPTS/cross_upgrade_bot.js" $PORT "$(bot_version "$3")" "$5" > "$OUT/bot-$5.log" 2>&1
    stop_server
  fi
  release
  echo "$5 server ($3) up=$UP: $(tail -1 "$OUT/bot-$5.log" 2>/dev/null)"
}

git -C "$REPO" fetch -q origin
OREF=$(ref_of "$OLD") || { echo "no origin/$OLD and no tag $OLD"; exit 2; }
NREF=$(ref_of "$NEW") || { echo "no origin/$NEW and no tag $NEW"; exit 2; }
fresh_tree "$WO" "$OREF"; fresh_tree "$WN" "$NREF"
echo "== $LABEL: $OLD $(git -C "$WO" rev-parse --short HEAD) ($OMC) -> $NEW $(git -C "$WN" rev-parse --short HEAD) ($NMC), $L"
# On 26.x a Fabric server refuses the vanilla bot without the hook (lib.sh fabric_bot_hook).
[ "$L" = fabric ] && { fabric_bot_hook "$WO" "$OMC"; fabric_bot_hook "$WN" "$NMC"; }
rm -rf "$WO/$L/run/world"; write_server_properties "$WO/$L/run"
serve "$WO" "$OJH" "$OMC" "$OUT/server-old.log" old
rm -rf "$WN/$L/run/world"; write_server_properties "$WN/$L/run"
cp -R "$WO/$L/run/world" "$WN/$L/run/world" && rm -f "$WN/$L/run/world/session.lock"
serve "$WN" "$NJH" "$NMC" "$OUT/server-new.log" new
grep -E '/ERROR\]|Exception|Caused by|Failed to (load|read|parse)' "$OUT/server-new.log" \
  | grep -v -E 'No key layers|Unsafe unavailable|Native access|Realms|test_block' > "$OUT/new-errors.txt"
echo "errors in the new server log: $(wc -l < "$OUT/new-errors.txt" | tr -d ' ') (see new-errors.txt)"
