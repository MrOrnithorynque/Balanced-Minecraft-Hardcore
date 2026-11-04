#!/bin/bash
# The death flow, seen from a real client: a dedicated server makes a world, then a quick-play client with the hook in
# hook/death/ dies, presses the death screen's Respawn button, logs every position packet, respawn packet and BMH or
# amethyst sound with its time after the press, saves a frame every 3 ticks and quits. The hook is copied into the tree
# and removed at the end (interrupted or not); the tree's own changes stay.
# usage: death_client.sh <worktree> <label> <loader> [time of day] [level type] [immediate 0|1] [reload 0|1]
#   level type: minecraft:flat by default. A copied normal world can crash Forge 52.1.20 on its first tick: a landing
#   falling block breaks a neighbour whose loot drop asks for LootModifierManager before it is loaded. Not a BMH bug.
#   Every run makes a fresh world, so the level type always applies.
#   immediate=1 sets doImmediateRespawn (no death screen); reload=1 reloads the resource packs during the fade.
# Read <out>/journey.log: the first "position packet" after the "respawn packet" must be the far place, the
# "overlay gone" line must come, and "final respawned=true alive=true screen=- overlay=-" ends a good run.
# Never edit this file while a run executes it: bash reads it as it goes.
set -u
WT=$1; LABEL=$2; L=$3; DAYTIME=${4:-1000}; LEVELTYPE=${5:-minecraft:flat}; IMMEDIATE=${6:-0}; RELOAD=${7:-0}
SCRIPTS=$(cd "$(dirname "$0")" && pwd)
HOOK=$SCRIPTS/hook/death
OUT=${BMH_VERIFY_OUT:-/private/tmp/bmh-verify}/death-$LABEL
# No bot runs here, but lib.sh insists on the variable.
export BMH_BOT_MODULES=${BMH_BOT_MODULES:-/nonexistent} BMH_PORT=${BMH_PORT:-25620} BMH_SESSION=death-$LABEL
. "$SCRIPTS/lib.sh"
[ "$(git -C "$WT" rev-parse --show-toplevel)" = "$WT" ] || { echo "tree guard failed for $WT"; exit 4; }
MCV=$(sed -n 's/^minecraft_version=//p' "$WT/gradle.properties" | head -1 | tr -d '[:space:]')
JH=$(jdk_for "$MCV")
SRC=$WT/common/src/main/java/github/mrornithorynque/bmh
MIXINS=$WT/common/src/main/resources/bmh.mixins.json
mkdir -p "$OUT"
say() { echo "[$LABEL $(date '+%H:%M:%S')] $*" | tee -a "$OUT/run.log"; }

NEO_ORIG=""
unhook() {
  release
  rm -f "$SRC/client/JourneySmoke.java" "$SRC/mixin/Z_JourneySmokeClientMixin.java" "$SRC/mixin/Z_JourneySmokePacketMixin.java"
  [ -f "$OUT/bmh.mixins.json.orig" ] && cp "$OUT/bmh.mixins.json.orig" "$MIXINS"
  [ -n "$NEO_ORIG" ] && cp "$NEO_ORIG" "$WT/neoforge/build.gradle"
}
# The hook, removed on any exit: the trap is set before the first copy.
trap unhook EXIT
cp "$MIXINS" "$OUT/bmh.mixins.json.orig"
cp "$HOOK/JourneySmoke.java" "$SRC/client/JourneySmoke.java"
cp "$HOOK/Z_JourneySmokeClientMixin.java" "$HOOK/Z_JourneySmokePacketMixin.java" "$SRC/mixin/"
python3 -I - "$MIXINS" <<'PY'
import sys, json
p = sys.argv[1]; s = open(p).read()
t = s.replace('"client": [\n', '"client": [\n        "Z_JourneySmokeClientMixin",\n        "Z_JourneySmokePacketMixin",\n', 1)
assert t != s; json.loads(t); open(p, "w").write(t)
PY
[ $? = 0 ] || { echo "the mixins json patch failed"; exit 6; }

RUN=$WT/$L/run
say "mc=$MCV loader=$L jdk=$JH"
# 1. A dedicated server makes the world
rm -rf "$RUN/world"
if [ ! -f "$RUN/world/level.dat" ]; then
  write_server_properties "$RUN" spawn-monsters=false "motd=bmh journey" level-type=$LEVELTYPE level-seed=bmh-journey
  take_lock; take_slot
  SLOG=$OUT/server.log
  boot_server "$WT" "$L" "$JH" "$SLOG"
  say "server up=$UP after $(( $(date +%s) - BOOT_START )) s"
  [ $UP = 1 ] && stop_server
  release
  [ -f "$RUN/world/level.dat" ] || { say "no world made"; exit 5; }
fi

# 2. The quick-play client with the hook
mkdir -p "$RUN/saves"; rm -rf "$RUN/saves/journey"; cp -R "$RUN/world" "$RUN/saves/journey"; rm -f "$RUN/saves/journey/session.lock"
rm -f "$RUN"/screenshots/journey-*.png
cat > "$RUN/options.txt" <<OPT
onboardAccessibility:false
skipMultiplayerWarning:true
joinedFirstServer:true
tutorialStep:none
pauseOnLostFocus:false
soundCategory_master:0.0
fullscreen:false
renderDistance:6
guiScale:3
OPT
take_lock; take_slot
CLOG=$OUT/client.log
if [ "$L" = neoforge ]; then
  NEO_ORIG=$OUT/neoforge.build.gradle.orig; cp "$WT/neoforge/build.gradle" "$NEO_ORIG"
  python3 -I - "$WT/neoforge/build.gradle" <<'PY'
import sys
p = sys.argv[1]; s = open(p).read()
t = s.replace("        client {\n            client()\n", "        client {\n            client()\n            programArguments.addAll('--quickPlaySingleplayer', 'journey', '--width', '1280', '--height', '720')\n", 1)
assert t != s; open(p, "w").write(t)
PY
  ( cd "$WT" && BMH_JOURNEY_SMOKE=1 BMH_JOURNEY_TIME=$DAYTIME BMH_JOURNEY_IMMEDIATE=$IMMEDIATE BMH_JOURNEY_RELOAD=$RELOAD JAVA_HOME=$JH sh ./gradlew ":$L:runClient" --max-workers=2 ) > "$CLOG" 2>&1 &
else
  ( cd "$WT" && BMH_JOURNEY_SMOKE=1 BMH_JOURNEY_TIME=$DAYTIME BMH_JOURNEY_IMMEDIATE=$IMMEDIATE BMH_JOURNEY_RELOAD=$RELOAD JAVA_HOME=$JH sh ./gradlew ":$L:runClient" --max-workers=2 --args="--quickPlaySingleplayer journey --width 1280 --height 720" ) > "$CLOG" 2>&1 &
fi
GPID=$!
START=$(date +%s)
while [ $(( $(date +%s) - START )) -lt 900 ] && kill -0 $GPID 2>/dev/null; do sleep 3; done
if kill -0 $GPID 2>/dev/null; then
  say "TIMEOUT: killing the client"
  for p in $(pgrep -f "$WT/$L" 2>/dev/null); do kill "$p" 2>/dev/null; done
  sleep 5
fi
release
for f in "$RUN"/screenshots/journey-*.png; do [ -f "$f" ] && cp "$f" "$OUT/"; done
grep -E '\[JOURNEY-SMOKE\]|\[BalancedMcHardcoreMain\] (new respawn|No safe|Exile|Distance|No bed)' "$CLOG" > "$OUT/journey.log"
say "client done: $(grep -c 'JOURNEY-SMOKE' "$CLOG") hook lines, $(ls "$OUT"/journey-*.png 2>/dev/null | wc -l | tr -d ' ') frames, errors: $(grep -E '/ERROR\]|Exception|Mixin apply failed' "$CLOG" | grep -i -c -E 'bmh|mixin|mrornithorynque')"
