#!/bin/bash
# One dedicated server of an existing verification worktree (verify-<label>, left by branch.sh at its branch), one bot
# script against it, then stop. Same locks and port as branch.sh.
# usage: server_only.sh <label> <minecraft version> <loader> <bot script: a file of this folder, or an absolute path> <result name>
#   e.g. server_only.sh 1.20.4 1.20.4 fabric exile_bot.js exile     (the Nether exile check)
# Results: $BMH_VERIFY_OUT/<label>/server-<loader>-<result name>.log and <result name>-<loader>.log, and a one-line answer.
# The bots' mineflayer installs itself (need_bots in lib.sh).
set -u
[ $# -eq 5 ] || { echo "usage: server_only.sh <label> <minecraft version> <loader> <bot script> <result name>" >&2; exit 2; }
LABEL=$1; MCV=$2; L=$3; BOT=$4; NAME=$5
SCRIPTS=$(cd "$(dirname "$0")" && pwd)
. "$SCRIPTS/lib.sh"
need_bots
case "$BOT" in /*) ;; *) BOT=$SCRIPTS/$BOT ;; esac
WT=$REPO/.claude/worktrees/verify-$LABEL
JH=$(jdk_for "$MCV") || exit 2
OUT=$OUTROOT/$LABEL; mkdir -p "$OUT"
[ "$(git -C "$WT" rev-parse --show-toplevel 2>/dev/null)" = "$WT" ] || { echo "tree guard failed for $WT (run branch.sh for label $LABEL first)"; exit 4; }
# The Fabric bot hook (26.x) goes in once this script holds the server lock, and only a Fabric run removes it.
trap 'release; [ "$L" = fabric ] && fabric_bot_unhook "$WT"' EXIT
take_lock; take_slot
[ "$L" = fabric ] && fabric_bot_hook "$WT" "$MCV"
RUN="$WT/$L/run"; rm -rf "$RUN/world"; write_server_properties "$RUN"
SLOG="$OUT/server-$L-$NAME.log"
boot_server "$WT" "$L" "$JH" "$SLOG"
if [ $UP = 1 ]; then
  # a bot kept elsewhere finds rcon.py next to itself
  [ "$(dirname "$BOT")" = "$SCRIPTS" ] || cp "$SCRIPTS/rcon.py" "$(dirname "$BOT")/rcon.py" 2>/dev/null
  with_timeout 600 node "$BOT" $PORT "$(bot_version "$MCV")" > "$OUT/$NAME-$L.log" 2>&1
  stop_server
fi
release
echo "$LABEL $L $NAME: up=$UP $(tail -1 "$OUT/$NAME-$L.log" 2>/dev/null) | log: $(grep -o 'Exile distance:[^\[]*' "$SLOG" | tr '\n' ';')"
