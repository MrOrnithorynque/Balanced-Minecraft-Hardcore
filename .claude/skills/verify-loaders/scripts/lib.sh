#!/bin/bash
# Shared by branch.sh, matrix.sh, server_only.sh, upgrade.sh and death_client.sh: source it (". lib.sh"), never run it.
# Paths, the JDK per Minecraft version, the two locks every server boot goes through (the server lock first, then a Gradle
# slot, each a directory with an owner file, polled every second) and the boot and stop of a dedicated server.
#
# Environment (all optional but the first):
#   BMH_BOT_MODULES  the node_modules folder that holds mineflayer (see ../SKILL.md, Setup)
#   BMH_VERIFY_OUT   logs, screenshots and summary.txt (default /private/tmp/bmh-verify)
#   BMH_REPO         the main checkout the verification worktrees hang off (default: found from this script's repository)
#   BMH_PORT         the dedicated server's port (default 25618; upgrade.sh sets it from BMH_UPGRADE_PORT, default 25619).
#                    A plain PORT variable is ignored. RCON is always port + 100, password bmh: the bots compute it the same way.
#   BMH_JDK17, BMH_JDK21   JDK homes (default Homebrew's openjdk@17 and openjdk@21)
#   BMH_LOCK_DIR, BMH_SLOT_PREFIX   the lock paths (default /private/tmp/bmh-runserver.lock and /private/tmp/bmh-gradle,
#                    slots <prefix>.1 and <prefix>.2). Other sessions of the same machine must use the same ones.
#   BMH_SESSION      the name written in the owner files (default verify)

SCRIPTS=${SCRIPTS:-$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)}
REPO=${BMH_REPO:-$(dirname "$(git -C "$SCRIPTS" rev-parse --path-format=absolute --git-common-dir)")}
OUTROOT=${BMH_VERIFY_OUT:-/private/tmp/bmh-verify}
BOT_MODULES=${BMH_BOT_MODULES:?set BMH_BOT_MODULES to the node_modules folder that holds mineflayer (see SKILL.md, Setup)}
export BOT_MODULES
PORT=${BMH_PORT:-25618}
RPORT=$((PORT + 100))
LOCKDIR=${BMH_LOCK_DIR:-/private/tmp/bmh-runserver.lock}
SLOTPREFIX=${BMH_SLOT_PREFIX:-/private/tmp/bmh-gradle}
OWNER=${BMH_SESSION:-verify}
HELD_LOCK=""; HELD_SLOT=""; GPID=""; UP=0; BOOT_START=0

command -v timeout >/dev/null || command -v gtimeout >/dev/null || { echo "no timeout command: brew install coreutils" >&2; exit 2; }
with_timeout() { local t=$1; shift; if command -v timeout >/dev/null; then timeout "$t" "$@"; else gtimeout "$t" "$@"; fi; }

jdk_for() { # <minecraft version> -> a JDK home
  case "$1" in
    1.21*) echo "${BMH_JDK21:-/opt/homebrew/opt/openjdk@21}" ;;
    *) echo "${BMH_JDK17:-/opt/homebrew/opt/openjdk@17}" ;;
  esac
}

# The server lock, then a Gradle slot, always in that order (two slots and one lock: a waiting boot can only hold the lock
# while builds cycle through the slots, so nobody deadlocks). A lock whose owner is older than 20 minutes and whose port is
# silent is dropped.
take_lock() {
  local o=$LOCKDIR/owner ots oport
  while ! mkdir "$LOCKDIR" 2>/dev/null; do
    if [ -f "$o" ]; then
      ots=$(awk '{print $3}' "$o"); oport=$(awk '{print $4}' "$o")
      if [ -n "$ots" ] && [ $(( $(date +%s) - ots )) -gt 1200 ] && ! lsof -ti tcp:"$oport" >/dev/null 2>&1; then rm -rf "$LOCKDIR"; fi
    fi
    sleep 1
  done
  HELD_LOCK=$LOCKDIR; echo "$OWNER $$ $(date +%s) $PORT" > "$HELD_LOCK/owner"
}
take_slot() {
  local n
  while :; do
    for n in 1 2; do
      if mkdir "$SLOTPREFIX.$n" 2>/dev/null; then
        HELD_SLOT=$SLOTPREFIX.$n; echo "$OWNER $$ $(date +%s)" > "$HELD_SLOT/owner"; return
      fi
    done
    sleep 1
  done
}
# Stop what this script started (Gradle, whatever listens on the port) and give both locks back.
release() {
  local p
  [ -n "$GPID" ] && kill "$GPID" 2>/dev/null
  p=$(lsof -ti tcp:"$PORT" 2>/dev/null); [ -n "$p" ] && kill $p 2>/dev/null
  [ -n "$HELD_LOCK" ] && rm -rf "$HELD_LOCK"; HELD_LOCK=""
  [ -n "$HELD_SLOT" ] && rm -rf "$HELD_SLOT"; HELD_SLOT=""
  GPID=""
}

# <origin branch or tag> -> a full ref, or a non-zero status
ref_of() {
  if git -C "$REPO" rev-parse -q --verify "refs/remotes/origin/$1" >/dev/null; then echo "refs/remotes/origin/$1"
  elif git -C "$REPO" rev-parse -q --verify "refs/tags/$1" >/dev/null; then echo "refs/tags/$1"
  else return 1; fi
}
# <worktree path> <full ref>: a worktree of its own at that ref, clean (ignored files such as build/ stay, for speed)
fresh_tree() {
  [ -d "$1" ] || git -C "$REPO" worktree add -q --detach "$1" "$2" || exit 3
  [ "$(git -C "$1" rev-parse --show-toplevel)" = "$1" ] || { echo "tree guard failed for $1"; exit 4; }
  git -C "$1" checkout -q -f --detach "$2" && git -C "$1" clean -q -fd
}

# <run dir> [extra property lines]: a fresh flat world's settings, offline mode, RCON on port + 100. The server listens on
# 127.0.0.1 only (server-ip, which RCON follows): the RCON password is published in this skill, and the bots connect locally.
write_server_properties() {
  local run=$1; shift
  mkdir -p "$run"; echo "eula=true" > "$run/eula.txt"
  printf '%s\n' "server-port=$PORT" server-ip=127.0.0.1 online-mode=false enforce-secure-profile=false enable-rcon=true "rcon.port=$RPORT" \
    rcon.password=bmh level-type=flat gamemode=survival difficulty=peaceful spawn-protection=0 view-distance=6 \
    simulation-distance=4 max-tick-time=-1 "$@" > "$run/server.properties"
}
# <worktree> <loader> <java home> <log>: runServer in the background (GPID), UP=1 once the log says "Done (".
# The first boot of a loader in a fresh worktree runs the plugin's setup first: wait up to 15 minutes.
boot_server() {
  ( cd "$1" && JAVA_HOME=$3 sh ./gradlew ":$2:runServer" --max-workers=2 ) > "$4" 2>&1 &
  GPID=$!; UP=0; BOOT_START=$(date +%s)
  while [ $(( $(date +%s) - BOOT_START )) -lt 900 ]; do
    grep -q 'Done (' "$4" 2>/dev/null && { UP=1; break; }
    kill -0 $GPID 2>/dev/null || break
    sleep 2
  done
}
# RCON stop, then up to 90 s for Gradle to end by itself
stop_server() {
  local i
  python3 -I "$SCRIPTS/rcon.py" "$RPORT" stop > /dev/null 2>&1
  for i in $(seq 1 45); do kill -0 $GPID 2>/dev/null || break; sleep 2; done
}
