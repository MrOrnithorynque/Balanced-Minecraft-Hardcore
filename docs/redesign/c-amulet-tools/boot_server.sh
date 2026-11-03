#!/bin/bash
# usage: boot_server.sh <abs worktree> <JAVA_HOME> <loader: forge|fabric|neoforge> <mc version> <abs log dir> [during-script]
# Boots one dedicated server of <worktree> on port 25613 (RCON 25713, password bmh), under the two machine-wide locks:
#   - a Gradle slot (/private/tmp/bmh-gradle.N): the runServer Gradle process lives as long as the server,
#   - the server lock (/private/tmp/bmh-runserver.lock, with an `owner` file).
# Takes the slot, then tries the server lock for 90 s; if it does not get it, it gives the slot back (other sessions' builds
# need it) and retries after a random pause. Once the log says `Done`, runs [during-script] (args: port, mc version, loader),
# then stops the server with RCON `stop` (kills what listens on the port after 60 s) and frees both locks. 5 minute boot timeout.
W="$1"; JH="$2"; LOADER="$3"; MCV="$4"; LOGDIR="$5"; DURING="$6"
PORT=25613; RPORT=25713
S="$(cd "$(dirname "$0")" && pwd)"
[ "$(git -C "$W" rev-parse --show-toplevel)" = "$W" ] || { echo "tree guard failed for $W"; exit 70; }
mkdir -p "$LOGDIR"
LOG="$LOGDIR/server-$LOADER-$MCV.log"
RUN="$W/$LOADER/run"
slot=""; lock=""
cleanup() {
  [ -n "$GPID" ] && kill "$GPID" 2>/dev/null
  P=$(lsof -ti tcp:$PORT 2>/dev/null); [ -n "$P" ] && kill $P 2>/dev/null
  [ -n "$lock" ] && rm -rf "$lock"
  [ -n "$slot" ] && rm -rf "$slot"
}
trap cleanup EXIT

# 1. a Gradle slot and the server lock
for attempt in $(seq 1 100); do
  slot=""
  for i in $(seq 1 120); do
    for n in 1 2; do
      if mkdir /private/tmp/bmh-gradle.$n 2>/dev/null; then slot=/private/tmp/bmh-gradle.$n; break 2; fi
    done
    sleep 1
  done
  [ -z "$slot" ] && continue
  echo "c-amulet $$ $(date +%s)" > $slot/owner
  for i in $(seq 1 90); do
    if mkdir /private/tmp/bmh-runserver.lock 2>/dev/null; then lock=/private/tmp/bmh-runserver.lock; break; fi
    if [ -f /private/tmp/bmh-runserver.lock/owner ]; then
      OW=$(cat /private/tmp/bmh-runserver.lock/owner); OPORT=$(echo $OW | awk '{print $4}'); OTS=$(echo $OW | awk '{print $3}')
      if [ -n "$OTS" ] && [ $(( $(date +%s) - OTS )) -gt 1200 ] && ! lsof -ti tcp:$OPORT >/dev/null 2>&1; then rm -rf /private/tmp/bmh-runserver.lock; fi
    fi
    sleep 1
  done
  [ -n "$lock" ] && break
  rm -rf "$slot"; slot=""
  sleep $(( RANDOM % 20 + 5 ))
done
[ -z "$lock" ] && { echo "no slot and lock after all attempts" > "$LOG"; exit 76; }
echo "c-amulet $$ $(date +%s) $PORT" > $lock/owner

# 2. a fresh world and the server settings
rm -rf "$RUN/world"; mkdir -p "$RUN"
echo "eula=true" > "$RUN/eula.txt"
cat > "$RUN/server.properties" <<EOF
server-port=$PORT
online-mode=false
enforce-secure-profile=false
enable-rcon=true
rcon.port=$RPORT
rcon.password=bmh
level-type=flat
gamemode=survival
difficulty=peaceful
spawn-monsters=false
spawn-protection=0
view-distance=4
simulation-distance=4
max-tick-time=-1
motd=bmh part C test
EOF
echo "boot $LOADER $MCV in $W, started $(date +%s)" > "$LOG"

# 3. boot
( JAVA_HOME="$JH" sh "$W/gradlew" -p "$W" ":$LOADER:runServer" --max-workers=2 >> "$LOG" 2>&1 ) &
GPID=$!
START=$(date +%s); OK=0
while [ $(( $(date +%s) - START )) -lt 300 ]; do
  if grep -q 'Done (' "$LOG" 2>/dev/null; then OK=1; break; fi
  if ! kill -0 $GPID 2>/dev/null; then break; fi
  sleep 2
done
if [ $OK = 1 ]; then
  echo "BOOT OK after $(( $(date +%s) - START )) s" >> "$LOG"
  if [ -n "$DURING" ]; then "$DURING" $PORT "$MCV" "$LOADER" > "$LOGDIR/scenario-$LOADER-$MCV.log" 2>&1; echo "scenario exit $?" >> "$LOG"; fi
  python3 $S/rcon.py $RPORT stop >> "$LOG" 2>&1
  for i in $(seq 1 30); do kill -0 $GPID 2>/dev/null || break; sleep 2; done
else
  echo "BOOT FAILED or timed out (see above)" >> "$LOG"
fi
echo "FINISHED $(date +%s)" >> "$LOG"
