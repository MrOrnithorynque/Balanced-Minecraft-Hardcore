#!/bin/bash
# Verifies one BMH branch on every loader with nobody at the keyboard: a clean-checkout build and its jar checks, then for each
# loader a dedicated server (the amulet scenario, then smoke_bot.js) and a singleplayer client with the throwaway smoke
# hook (hook/), run on a copy of that server's world. It works in its own worktree <repo>/.claude/worktrees/verify-<label>,
# never commits, and removes the hook at the end (also when it is interrupted).
# usage: branch.sh <origin branch or tag> <minecraft version> <label> [loaders...]
#   loaders default to forge fabric, plus neoforge when the build made a NeoForge jar.
# Results: $BMH_VERIFY_OUT/<label>/ (logs, screenshots, steps.txt) and one line in $BMH_VERIFY_OUT/summary.txt.
# The bots' mineflayer installs itself (need_bots in lib.sh). Unattended: charger in and lid open (caffeinate -i covers idle sleep only, see ../SKILL.md).
set -u
[ $# -ge 3 ] || { echo "usage: branch.sh <origin branch or tag> <minecraft version> <label> [loaders...]" >&2; exit 2; }
BR=$1; MCV=$2; LABEL=$3; shift 3
SCRIPTS=$(cd "$(dirname "$0")" && pwd)
. "$SCRIPTS/lib.sh"
need_bots
WT=$REPO/.claude/worktrees/verify-$LABEL
OUT=$OUTROOT/$LABEL
JH=$(jdk_for "$MCV") || exit 2
BMH_SRC=common/src/main/java/github/mrornithorynque/bmh
mkdir -p "$OUT"
say() { echo "$*" | tee -a "$OUT/steps.txt"; }
unhook() { # the throwaway hook never outlives the run
  [ -d "$WT" ] || return 0
  git -C "$WT" checkout -q -- common/src/main/resources/bmh.mixins.json neoforge/build.gradle 2>/dev/null
  rm -f "$WT/$BMH_SRC/client/SmokeClient.java" "$WT/$BMH_SRC/mixin/Z_SmokeClientMixin.java"
}
trap 'release; unhook' EXIT

# 1. My own worktree at the pushed branch, guarded
git -C "$REPO" fetch -q origin
REF=$(ref_of "$BR") || { say "no origin/$BR, no tag $BR$([ "${BMH_LOCAL:-0}" = 1 ] && echo ", no local branch $BR")"; exit 2; }
SHA=$(git -C "$REPO" rev-parse --short "$REF^{commit}")
fresh_tree "$WT" "$REF"
say "== $LABEL: $BR at $SHA, Minecraft $MCV, $(date '+%H:%M:%S')"
rm -rf "$WT"/forge/build/libs "$WT"/fabric/build/libs "$WT"/neoforge/build/libs

# 2. Clean-checkout build and the jar checks
take_slot
( cd "$WT" && JAVA_HOME=$JH sh ./gradlew build --max-workers=2 ) > "$OUT/build.log" 2>&1
release
BUILD=$(grep -q 'BUILD SUCCESSFUL' "$OUT/build.log" && echo ok || echo FAILED)
say "build $BUILD"
[ "$BUILD" = ok ] || { echo "$LABEL $SHA build FAILED" >> "$OUTROOT/summary.txt"; exit 5; }
python3 -I "$SCRIPTS/jarcheck.py" "$WT" "$MCV" "$JH" > "$OUT/jars.txt" 2>&1
JARS=$(grep '^JARS ' "$OUT/jars.txt" | cut -d' ' -f2)
ITEMS=$(grep '^ITEMS ' "$OUT/jars.txt" | cut -d' ' -f2-)
RECIPES=$(grep '^RECIPES ' "$OUT/jars.txt" | cut -d' ' -f2-)
say "jars $JARS"
LOADERS="$*"
if [ -z "$LOADERS" ]; then
  LOADERS="forge fabric"; ls "$WT"/neoforge/build/libs/*.jar >/dev/null 2>&1 && LOADERS="$LOADERS neoforge"
fi

# 3. The throwaway client hook: its copy in common/src is never committed, unhook removes it (the source stays in scripts/hook/)
cp "$SCRIPTS/hook/SmokeClient.java" "$WT/$BMH_SRC/client/SmokeClient.java"
cp "$SCRIPTS/hook/Z_SmokeClientMixin.java" "$WT/$BMH_SRC/mixin/Z_SmokeClientMixin.java"
python3 -I - "$WT/common/src/main/resources/bmh.mixins.json" <<'PY'
import json, sys
p = sys.argv[1]; d = json.load(open(p)); d.setdefault("client", []).append("Z_SmokeClientMixin")
open(p, "w").write(json.dumps(d, indent=4) + "\n")
PY

LINE="$LABEL $SHA build=$BUILD jars=$JARS"
for L in $LOADERS; do
  RUN="$WT/$L/run"
  # 4. Dedicated server, fresh flat world, the two bots
  rm -rf "$RUN/world"
  write_server_properties "$RUN" spawn-monsters=false "motd=bmh verification"
  take_lock; take_slot
  SLOG="$OUT/server-$L.log"
  boot_server "$WT" "$L" "$JH" "$SLOG"
  AM="-"; SM="-"
  if [ $UP = 1 ]; then
    say "$L server up after $(( $(date +%s) - BOOT_START )) s"
    # The amulet scenario of this skill checks the named points. A tree from 1.5.0 to 1.6.x (one Home) runs the Home
    # version kept on main in docs/redesign (mc-1.21.1 has no docs/redesign); a tree before 1.5.0 gets amulet=-.
    AMBOT=""
    if [ -f "$WT/$BMH_SRC/handlers/AmuletPoints.java" ]; then
      AMBOT=$SCRIPTS/amulet_scenario.js
    elif grep -q 'bmh_starlight' "$WT/$BMH_SRC/handlers/AmuletReading.java" 2>/dev/null \
        && git -C "$REPO" show main:docs/redesign/c-amulet-tools/amulet_scenario.js > "$OUT/amulet_scenario_home.js" 2>/dev/null; then
      AMBOT=$OUT/amulet_scenario_home.js
      cp "$SCRIPTS/rcon.py" "$OUT/rcon.py"
    fi
    if [ -n "$AMBOT" ]; then
      with_timeout 600 node "$AMBOT" $PORT "$MCV" > "$OUT/amulet-$L.log" 2>&1
      AM="$(grep -c '^PASS' "$OUT/amulet-$L.log")/$(grep -c -E '^(PASS|FAIL)' "$OUT/amulet-$L.log")"
    fi
    with_timeout 600 env SMOKE_ITEMS="$ITEMS" SMOKE_RECIPES="$RECIPES" node "$SCRIPTS/smoke_bot.js" $PORT "$MCV" > "$OUT/bot-$L.log" 2>&1
    SM=$(grep '^SMOKE ' "$OUT/bot-$L.log" | tail -1 | cut -d' ' -f2)
    stop_server
  else
    say "$L server did NOT reach Done"
  fi
  release
  SERR=$(grep -E '/ERROR\]|Exception|Caused by|Mixin apply failed' "$SLOG" | grep -i -E 'bmh|mixin|mrornithorynque' | grep -v -c 'test_block')
  TAG=$(grep -c 'echo_sense_ores lists 19 blocks' "$SLOG")
  say "$L server: up=$UP amulet=$AM smoke=$SM bmh-errors=$SERR echo-tag-lines=$TAG"

  # 5. Singleplayer client on a copy of that world, with the smoke hook
  CS="-"; SHOTS=0; CERR="-"
  if [ $UP = 1 ] && [ -f "$RUN/world/level.dat" ]; then
    mkdir -p "$RUN/saves"; rm -rf "$RUN/saves/smoke"; cp -R "$RUN/world" "$RUN/saves/smoke"; rm -f "$RUN/saves/smoke/session.lock"
    rm -f "$RUN"/screenshots/bmh-smoke-*.png
    cat > "$RUN/options.txt" <<OPT
onboardAccessibility:false
skipMultiplayerWarning:true
joinedFirstServer:true
tutorialStep:none
pauseOnLostFocus:false
soundCategory_master:0.0
fullscreen:false
renderDistance:6
OPT
    take_lock; take_slot
    CLOG="$OUT/client-$L.log"
    QP="--quickPlaySingleplayer smoke --width 1280 --height 720"
    if [ "$L" = neoforge ]; then
      # ModDevGradle takes --args as the whole command line (main class first): put the arguments in the run config instead
      python3 -I - "$WT/neoforge/build.gradle" <<'PY' || say "neoforge/build.gradle has no client run block to patch: the client will not find the world"
import sys
p = sys.argv[1]; s = open(p).read()
t = s.replace("        client {\n            client()\n", "        client {\n            client()\n            programArguments.addAll('--quickPlaySingleplayer', 'smoke', '--width', '1280', '--height', '720')\n", 1)
if t == s:
    sys.exit(1)
open(p, "w").write(t)
PY
      ( cd "$WT" && BMH_SMOKE=1 JAVA_HOME=$JH sh ./gradlew ":$L:runClient" --max-workers=2 ) > "$CLOG" 2>&1 &
    else
      ( cd "$WT" && BMH_SMOKE=1 JAVA_HOME=$JH sh ./gradlew ":$L:runClient" --max-workers=2 --args="$QP" ) > "$CLOG" 2>&1 &
    fi
    GPID=$!
    START=$(date +%s)
    while [ $(( $(date +%s) - START )) -lt 600 ] && kill -0 $GPID 2>/dev/null; do sleep 3; done
    if kill -0 $GPID 2>/dev/null; then
      echo "TIMEOUT: killing the client" >> "$CLOG"
      for p in $(pgrep -f "$WT/$L" 2>/dev/null); do kill "$p" 2>/dev/null; done
      sleep 5
    fi
    release
    CS=$(grep -c '\[BMH-SMOKE\] stopping' "$CLOG")
    for f in "$RUN"/screenshots/bmh-smoke-*.png; do [ -f "$f" ] && { cp "$f" "$OUT/$L-$(basename "$f")"; SHOTS=$((SHOTS + 1)); }; done
    CERR=$(grep -E '/ERROR\]|/WARN\].*(Missing textures|Unable to (load|resolve)|Exception)|Exception|Caused by|Mixin apply failed' "$CLOG" | grep -i -E 'bmh|mixin|mrornithorynque' | grep -v -c 'test_block')
  fi
  say "$L client: reached-stop=$CS screenshots=$SHOTS bmh-errors=$CERR"
  LINE="$LINE | $L up=$UP amulet=$AM smoke=$SM srvErr=$SERR tag=$TAG client=$CS shots=$SHOTS cliErr=$CERR"
done

# 6. Remove the hook, leave the worktree as the branch
unhook
say "finished $(date '+%H:%M:%S')"
echo "$LINE" >> "$OUTROOT/summary.txt"
