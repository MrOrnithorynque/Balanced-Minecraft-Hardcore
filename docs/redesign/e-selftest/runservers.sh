#!/bin/bash
# usage: runservers.sh <abs worktree> <JAVA_HOME> <abs log prefix> <selftest|plain> <loader>...
# Takes the machine-wide run lock once, then boots each loader in turn (one Gradle slot per boot), 7 minute limit each.
NAME=${BMH_NAME:-e-abilities}; PORT=${BMH_PORT:-25615}; RCON=$((PORT + 100))
WT="$1"; JH="$2"; PREFIX="$3"; MODE="$4"; shift 4
S=$(dirname "$0")
LOCK=/private/tmp/bmh-runserver.lock
SLOT=""
cleanup() { [ -n "$SLOT" ] && rm -rf "$SLOT"; rm -rf "$LOCK"; }
if [ "$(git -C "$WT" rev-parse --show-toplevel)" != "$WT" ]; then echo "tree guard failed" > "$PREFIX-guard.log"; exit 1; fi
mkdir -p "$(dirname "$PREFIX")"
QLOG="$PREFIX-queue.log"
echo "queued $(date)" > "$QLOG"
while ! mkdir "$LOCK" 2>/dev/null; do
  if [ -f "$LOCK/owner" ]; then
    read -r oname opid oepoch oport < "$LOCK/owner"
    now=$(date +%s)
    if [ -n "$oepoch" ] && [ $((now - oepoch)) -gt 1200 ] && ! nc -z 127.0.0.1 "${oport:-1}" 2>/dev/null; then
      echo "clearing stale run lock of $oname" >> "$QLOG"; rm -rf "$LOCK"; continue
    fi
  fi
  sleep 0.3
done
echo "$NAME $$ $(date +%s) $PORT" > "$LOCK/owner"
trap cleanup EXIT
echo "lock taken $(date)" >> "$QLOG"
for LOADER in "$@"; do
  LOG="$PREFIX-$LOADER.log"
  RUN="$WT/$LOADER/run"
  mkdir -p "$RUN"
  echo "eula=true" > "$RUN/eula.txt"
  cat > "$RUN/server.properties" <<PROPS
server-port=$PORT
online-mode=false
level-type=minecraft:flat
generate-structures=false
spawn-protection=0
view-distance=4
simulation-distance=4
max-tick-time=-1
enable-rcon=true
rcon.port=$RCON
rcon.password=bmh
motd=bmh part E
PROPS
  rm -rf "$RUN/world" "$RUN/logs/latest.log" "$RUN/logs/debug.log"
  if [ "$MODE" = "selftest" ]; then touch "$RUN/bmh-selftest"; else rm -f "$RUN/bmh-selftest"; fi
  SLOT=""
  while [ -z "$SLOT" ]; do
    for n in 1 2; do
      if mkdir /private/tmp/bmh-gradle.$n 2>/dev/null; then SLOT=/private/tmp/bmh-gradle.$n; echo "$NAME $$ $(date +%s)" > "$SLOT/owner"; break; fi
    done
    [ -z "$SLOT" ] && sleep 2
  done
  echo "$NAME $$ $(date +%s) $PORT" > "$LOCK/owner"
  echo "lock and slot taken $(date), mode $MODE, loader $LOADER" > "$LOG"
  ( JAVA_HOME="$JH" exec sh "$WT/gradlew" -p "$WT" ":$LOADER:runServer" --max-workers=2 >> "$LOG" 2>&1 < /dev/null ) &
  GPID=$!
  start=$(date +%s)
  stopped=0
  while kill -0 $GPID 2>/dev/null; do
    now=$(date +%s)
    if [ $((now - start)) -gt 420 ]; then echo "TIMEOUT: killing" >> "$LOG"; kill $(lsof -ti tcp:$PORT) 2>/dev/null; sleep 5; kill $GPID 2>/dev/null; break; fi
    if [ "$MODE" = "plain" ] && [ $stopped -eq 0 ] && grep -q 'Done (' "$LOG" 2>/dev/null; then
      sleep 3
      python3 "$S/rcon.py" $RCON bmh "stop" >> "$LOG" 2>&1
      stopped=1
    fi
    sleep 3
  done
  wait $GPID 2>/dev/null
  rm -rf "$SLOT"; SLOT=""
  echo "runserver finished $(date)" >> "$LOG"
done
