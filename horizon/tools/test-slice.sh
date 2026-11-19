#!/bin/bash
# The in-game test of the core slice (settings, respawn, journey) of the BMH mod on Horizon.
# It drives a real game through horizon-ctl, offscreen. It needs the prepared game files
# (horizon/tools/prepare-game.sh). It never leaves a game running: a trap quits the game and
# removes the scratch world on every exit, also on a failure and on Ctrl+C.
#
# Usage: sh horizon/tools/test-slice.sh [--only a,b,...] [--keep] [--window] [--no-build]
#                                       [--resources DIR] [--bin-dir DIR] [--out DIR]
#   --only a,b   run only these scenarios (a b c d e f g h i k l n p q r s t u v w x y m j z), default all
#   --keep       keep the scratch folder (world, game log, screenshots)
#   --window     run with a window instead of offscreen
#   --no-build   do not run "check.sh build" first (use --bin-dir or the cache's build)
#   --resources  the prepared game files (default $BMH_HORIZON_CACHE/resources/26.3)
#   --bin-dir    the folder with horizon-client and horizon-ctl
#   --out DIR    where the screenshots go (default: the scratch folder, printed at the end)
# Scenarios: a mods row, rules and rules file; b distant respawn; c immunity; d cinematic
#   screenshots; e rule off; f creative; g advancement; h the nine items (one screenshot);
#   i a respawn point that does not work; k the Create New World button and two worlds; l the settings page
#   and the config file; n the journey (message rule, held input, immediate respawn); m a rule and a config
#   across a restart (it quits and relaunches); p the eternal area (reduced_debug_info, the six items after a death, a
#   restart while dead, a change of world while dead, the copy rule, the item on the ground, the bundle, the wake-up
#   reward; it quits and relaunches once); q the amulet (the star reading, the refusals, the charges, the naming
#   screens, the points across a death and a restart, the server checks; it quits and relaunches once);
#   r the Eternal tools (worn pickaxe dig time, Efficiency V, the last point, the death wear); s the moon ritual
#   (refused by day, looked away, a full ritual); v the sky (a star reading, its log lines, screenshots, the spyglass);
#   t the totem (Undying order, the extra totem, the anvil, E6, the save on a lethal hit, /kill); u the homecoming
#   (the record, the title, the awards, the rule); w the journey and the early respawn (immediate respawn, the arrival
#   frame, a stale arm, a screen over the death screen, a press before the departure); x a LAN guest below operator
#   level (/bmh rule and /gamerule bmh:... refused, /bmh point save works); y a Java world's rules imported (it quits and
#   relaunches); z /gamerule bmh:<rule> typed in the chat and the completion of /bmh rule (it makes a world with commands
#   and opens the scratch world again); o the Balanced Hardcore button greyed in Hardcore and in Re-Create;
#   j dead when the game closes (it quits and relaunches).
# Environment: BMH_HORIZON_CACHE (default ~/Library/Caches/bmh-horizon).
# Exit codes: 0 all checks passed (SKIPs allowed), 1 a check failed, 77 no prepared files,
#   64 a bad option, 3 the game did not start.
#
# Written for /bin/bash 3.2 on macOS: no associative arrays, no empty arrays, no mapfile.
set -u

here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
content="$repo/horizon/content"
cache="${BMH_HORIZON_CACHE:-$HOME/Library/Caches/bmh-horizon}"
game_version="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["game"])' "$repo/horizon/native/compatibility.json" 2>/dev/null || echo 26.3)"

only=""
keep=0
window=0
build=1
resources="$cache/resources/$game_version"
bindir=""
outdir=""

while [ "$#" -gt 0 ]; do
    case "$1" in
        --only)      [ "$#" -ge 2 ] || { echo "test-slice.sh: --only needs a list." >&2; exit 64; }
                     only=",$2,"; shift 2 ;;
        --keep)      keep=1; shift ;;
        --window)    window=1; shift ;;
        --no-build)  build=0; shift ;;
        --resources) [ "$#" -ge 2 ] || { echo "test-slice.sh: --resources needs a folder." >&2; exit 64; }
                     resources="$2"; shift 2 ;;
        --bin-dir)   [ "$#" -ge 2 ] || { echo "test-slice.sh: --bin-dir needs a folder." >&2; exit 64; }
                     bindir="$2"; shift 2 ;;
        --out)       [ "$#" -ge 2 ] || { echo "test-slice.sh: --out needs a folder." >&2; exit 64; }
                     outdir="$2"; shift 2 ;;
        -h|--help)   sed -n '2,28p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "test-slice.sh: unknown option '$1'." >&2; exit 64 ;;
    esac
done

wanted() { [ -z "$only" ] || case "$only" in *",$1,"*) return 0 ;; *) return 1 ;; esac; }

# ---- 1. The prepared game files --------------------------------------------------------------
if [ ! -d "$resources" ] || [ -z "$(ls -A "$resources" 2>/dev/null)" ]; then
    echo "SKIP: no prepared game files. Run horizon/tools/prepare-game.sh first."
    exit 77
fi

# ---- 2. The build ------------------------------------------------------------------------------
if [ -z "$bindir" ]; then
    if [ "$build" -eq 1 ]; then
        buildlog="$(mktemp "${TMPDIR:-/tmp}/bmh-build.XXXXXX")"
        sh "$here/check.sh" build > "$buildlog" 2>&1
        status=$?
        cat "$buildlog"
        bindir="$(tail -n 1 "$buildlog")"
        rm -f "$buildlog"
        if [ "$status" -ne 0 ]; then
            echo "test-slice.sh: the build failed (status $status)." >&2
            exit 3
        fi
    else
        bindir="$cache/horizon-mods/native/target/cargo/fast"
    fi
fi
CTL="$bindir/horizon-ctl"
GAME="$bindir/horizon-client"
if [ ! -x "$CTL" ] || [ ! -x "$GAME" ]; then
    echo "test-slice.sh: no horizon-ctl or horizon-client in '$bindir'. Build with: sh horizon/tools/check.sh build" >&2
    exit 3
fi

# ---- 3. The scratch folder ---------------------------------------------------------------------
mkdir -p "$cache"
scratch="$(mktemp -d "$cache/slice.XXXXXX")"
sockdir="$(mktemp -d /tmp/bmh-ts.XXXXXX)"   # short: a Unix socket path has about 100 characters
SOCK="$sockdir/c.sock"
GSOCK="$sockdir/g.sock"
HOMEDIR="$scratch/home"
GAMEDIR="$scratch/game"
WORLD="bmhslice"
SEED=20261111
SHOTS="${outdir:-$scratch/shots}"
mkdir -p "$HOMEDIR" "$GAMEDIR" "$SHOTS"
GAMEPID=""

passed=0
failed=0
skipped=0
skip_lines=""

pass() { passed=$((passed + 1)); echo "PASS $1: $2"; }
fail() { failed=$((failed + 1)); echo "FAIL $1: $2"; }
skip() { skipped=$((skipped + 1)); skip_lines="$skip_lines
SKIP $1: $2"; echo "SKIP $1: $2"; }

cleanup() {
    trap - EXIT INT TERM
    if [ -S "$SOCK" ] || [ -n "$GAMEPID" ]; then
        "$CTL" --socket "$SOCK" --timeout 20 quit >/dev/null 2>&1 || true
    fi
    # Wait for the game to end, then stop it by force. The pid is the launch reply's, else found
    # through the game folder that only this run's game has on its command line.
    if [ -z "$GAMEPID" ]; then
        GAMEPID="$(pgrep -f -- "--game-dir $GAMEDIR" 2>/dev/null | head -n 1 || true)"
    fi
    if [ -n "$GAMEPID" ]; then
        n=0
        while kill -0 "$GAMEPID" 2>/dev/null && [ "$n" -lt 20 ]; do sleep 1; n=$((n + 1)); done
        if kill -0 "$GAMEPID" 2>/dev/null; then
            kill -TERM "$GAMEPID" 2>/dev/null || true
            sleep 3
            kill -0 "$GAMEPID" 2>/dev/null && kill -KILL "$GAMEPID" 2>/dev/null || true
        fi
    fi
    # The LAN guest game of scenario x, if one was started.
    if [ -S "$GSOCK" ]; then
        "$CTL" --socket "$GSOCK" --timeout 20 quit >/dev/null 2>&1 || true
    fi
    gpid="$(pgrep -f -- "--game-dir $scratch/ggame" 2>/dev/null | head -n 1 || true)"
    if [ -n "$gpid" ]; then
        n=0
        while kill -0 "$gpid" 2>/dev/null && [ "$n" -lt 15 ]; do sleep 1; n=$((n + 1)); done
        kill -KILL "$gpid" 2>/dev/null || true
    fi
    rm -rf "$sockdir"
    if [ "$keep" -eq 1 ]; then
        echo "test-slice.sh: scratch folder kept: $scratch"
    else
        rm -rf "$scratch/game" "$scratch/home" "$scratch/ggame" "$scratch/ghome"
        # The screenshots are the result: keep them when they are in the scratch folder.
        if [ -n "$(ls -A "$SHOTS" 2>/dev/null)" ]; then
            echo "test-slice.sh: screenshots kept in: $SHOTS"
            [ "$SHOTS" = "$scratch/shots" ] || rm -rf "$scratch"
        else
            rm -rf "$scratch"
        fi
    fi
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

# ---- helpers -----------------------------------------------------------------------------------
# ctl ARGS: one horizon-ctl call on this run's socket. Prints the JSON reply, returns its status.
ctl() { "$CTL" --socket "$SOCK" --timeout "${CTL_TIMEOUT:-60}" "$@" 2>&1; }

# jget JSON EXPR: a Python expression over the reply d, printed; empty when it fails.
jget() {
    printf '%s' "$1" | python3 -c '
import sys, json
try:
    d = json.loads(sys.stdin.read().strip().splitlines()[-1])
    print(eval(sys.argv[1]))
except Exception:
    print("")
' "$2"
}

state() { ctl state; }
health_of() { jget "$(state)" 'd["player"]["health"]'; }
position_of() { jget "$(state)" '" ".join(str(v) for v in d["player"]["position"])'; }
logfile() { echo "$HOMEDIR/logs/latest.log"; }
count_log() { c="$(grep -c -- "$1" "$(logfile)" 2>/dev/null || true)"; echo "${c:-0}"; }

# wait_log PATTERN BEFORE SECONDS: wait until PATTERN has more lines than BEFORE.
wait_log() {
    n=0
    while [ "$n" -lt "$3" ]; do
        [ "$(count_log "$1")" -gt "$2" ] && return 0
        sleep 1
        n=$((n + 1))
    done
    return 1
}

# dist_check X Z SX SZ EXPECTED TOL: prints "<distance> ok|bad".
dist_check() {
    python3 - "$1" "$2" "$3" "$4" "$5" "$6" <<'EOF'
import sys, math
x, z, sx, sz, e, t = [float(v) for v in sys.argv[1:7]]
d = math.hypot(x - sx, z - sz)
print("%.1f %s" % (d, "ok" if abs(d - e) <= t else "bad"))
EOF
}

# near_check X Z SX SZ MAX: prints "<distance> ok|bad" (ok when within MAX of the spawn).
near_check() {
    python3 - "$1" "$2" "$3" "$4" "$5" <<'EOF'
import sys, math
x, z, sx, sz, m = [float(v) for v in sys.argv[1:6]]
d = math.hypot(x - sx, z - sz)
print("%.1f %s" % (d, "ok" if d <= m else "bad"))
EOF
}

# The expected distance of the Java formula (RespawnPositionHandler.java:215-244, and
# bmh_logic::respawn::distance): integer arithmetic. Test difficulty Normal (id 2), no Eternal Item.
expected_distance() {
    cfg="$GAMEDIR/config/bmh-common.toml"
    min=5000
    max=60000
    if [ -f "$cfg" ]; then
        v="$(sed -n 's/^minDistanceFromBed *= *\([0-9]*\).*/\1/p' "$cfg" | head -n 1)"; [ -n "$v" ] && min="$v"
        v="$(sed -n 's/^maxDistanceFromBed *= *\([0-9]*\).*/\1/p' "$cfg" | head -n 1)"; [ -n "$v" ] && max="$v"
    fi
    python3 - "$min" "$max" "$1" "$2" <<'EOF'
import sys
mn, mx, eternal, difficulty = [int(v) for v in sys.argv[1:5]]
mx = max(mx, mn)
easy = min(1 + eternal + difficulty, 9)
print(mn + (mx - mn) * easy // 9)
EOF
}

# death_and_respawn MODE WAIT_LANDING: kill the player, wait for the death event, wait for the
# mod's "Landing spot ready" line when WAIT_LANDING is 1, then respawn. MODE is "ctl" (the
# command "horizon-ctl respawn": it skips the mod's hold of the click, so the test waits for the
# landing line first, or the search may not be done) or "click" (click-button Respawn: the click
# goes through the death screen, so the mod's hold and the fade run). Sets DR_OK (1/0) and DR_NOTE.
DR_OK=0
DR_NOTE=""
death_and_respawn() {
    mode="$1"
    wait_landing="$2"
    DR_OK=0
    DR_NOTE=""
    seq="$(jget "$(ctl events)" 'd["last"]')"
    [ -n "$seq" ] || seq=0
    landed_before="$(count_log 'Landing spot ready')"
    ctl command "kill @s" >/dev/null
    if ! ctl wait-for event death --since "$seq" --timeout 30 >/dev/null; then
        DR_NOTE="no death event within 30 s"
        return 1
    fi
    if [ "$wait_landing" -eq 1 ]; then
        if ! wait_log 'Landing spot ready' "$landed_before" 150; then
            DR_NOTE="no 'Landing spot ready' line within 150 s"
            return 1
        fi
    else
        sleep 2
    fi
    if [ -n "${DR_HOOK:-}" ]; then "$DR_HOOK"; fi
    seq="$(jget "$(ctl events)" 'd["last"]')"
    [ -n "$seq" ] || seq=0
    if [ "$mode" = click ]; then
        if ! ctl click-button respawn >/dev/null; then
            DR_NOTE="click-button respawn failed, used 'horizon-ctl respawn'"
            ctl respawn >/dev/null
        fi
    else
        ctl respawn >/dev/null
    fi
    if ! ctl wait-for event respawn --since "$seq" --timeout 90 >/dev/null; then
        DR_NOTE="$DR_NOTE no respawn event within 90 s"
        return 1
    fi
    ctl wait 40 >/dev/null
    DR_OK=1
    return 0
}

# ---- 4. Launch ---------------------------------------------------------------------------------
echo "test-slice.sh: resources $resources"
echo "test-slice.sh: binaries  $bindir"
echo "test-slice.sh: scratch   $scratch"
mkdir -p "$GAMEDIR"
extra=""
[ "$window" -eq 1 ] || extra="--offscreen"
# launch_game [--wait]: starts the game on the scratch world (made at the first launch, opened again
# after). Sets reply, launch_status and GAMEPID.
launch_game() {
    reply="$(HORIZON_HOME="$HOMEDIR" HORIZON_NO_AUDIO=1 HORIZON_TELEMETRY_URL=off HORIZON_NO_DISCORD=1 \
        HORIZON_LAUNCHER_NO_SELF_UPDATE=1 \
        "$CTL" --socket "$SOCK" --timeout 900 launch --game "$GAME" --world "$WORLD" --seed "$SEED" "$@" \
        --resources "$resources" --game-dir "$GAMEDIR" --home "$HOMEDIR" --dev-mod "$content" $extra 2>&1)"
    launch_status=$?
    # The launch reply is the whole state of the game. Print only what a person needs.
    echo "test-slice.sh: launch reply: $(jget "$reply" '{"ok": d.get("launch", {}).get("ok"), "pid": d.get("launch", {}).get("pid"), "screen": d.get("screen"), "dimension": d.get("dimension"), "player": d.get("player")}')"
    # The pid is in the "launch" object of the reply (horizon-ctl 2026-11-17 first run), not at the top.
    GAMEPID="$(jget "$reply" 'd.get("launch", {}).get("pid") or d.get("pid", "")')"
    # Without --wait the reply has no pid (scenario j, 2026-11-17): find the game by its folder.
    if [ -z "$GAMEPID" ] || [ "$GAMEPID" = None ]; then
        GAMEPID="$(pgrep -f -- "--game-dir $GAMEDIR" 2>/dev/null | head -n 1 || true)"
    fi
}
launch_game --wait
if [ "$launch_status" -ne 0 ]; then
    echo "test-slice.sh: the game did not start or the world did not load (status $launch_status)." >&2
    echo "test-slice.sh: log tail:" >&2
    tail -n 40 "$(logfile)" >&2 2>/dev/null || true
    exit 3
fi

ctl command "difficulty normal" >/dev/null
ctl command "gamemode survival" >/dev/null
start_pos="$(position_of)"
spawn_x="$(echo "$start_pos" | cut -d ' ' -f 1)"
spawn_z="$(echo "$start_pos" | cut -d ' ' -f 3)"
if [ -z "$spawn_x" ] || [ -z "$spawn_z" ]; then
    echo "test-slice.sh: cannot read the player position: $(state)" >&2
    exit 3
fi
expected="$(expected_distance 0 2)"
# The world spawn, for the scenarios that measure from it after a /spawnpoint moved spawn_x.
world_x="$spawn_x"
world_z="$spawn_z"
echo "test-slice.sh: world spawn (the start position of the player): x=$spawn_x z=$spawn_z"
echo "test-slice.sh: expected distant respawn distance (Normal, no Eternal Item): $expected blocks"

DISTANT_DONE=0

# A respawn point. The mod can only start a distant respawn when the game hands it the world
# spawn as the block of the point it searched for. On the first run (2026-11-17) the game handed
# a random spot of the spawn area instead, so a player with no bed and no /spawnpoint got the
# vanilla respawn (scenario b, check b0). Every other scenario sets a /spawnpoint on the
# player's block first, so that it tests the rest of the chain. Sets POINT_SET and moves
# spawn_x and spawn_z to the point: the circle is round the point.
POINT_SET=0
ensure_point() {
    [ "$POINT_SET" -eq 1 ] && return 0
    # After a distant respawn the player may stand far from loaded terrain near the start: a
    # /spawnpoint needs a loaded position, and any loaded spot works (the circle is round it).
    st="$(state)"
    px="$(jget "$st" 'd["player"]["block_pos"][0]')"
    py="$(jget "$st" 'd["player"]["block_pos"][1]')"
    pz="$(jget "$st" 'd["player"]["block_pos"][2]')"
    reply="$(ctl command "spawnpoint @s $px $py $pz")"
    echo "test-slice.sh: spawnpoint at $px $py $pz: $(printf '%s' "$reply" | head -c 200)"
    spawn_x="$px"
    spawn_z="$pz"
    POINT_SET=1
}

# ---- b0. Distant respawn in a fresh world: no bed, no save yet -----------------------------------
# This runs before scenario a on purpose: a1 opens a menu and the game saves the world then. On the
# first run (2026-11-17) the mod gave the vanilla respawn here ("No distant respawn"), every time.
if wanted b; then
    echo "--- b0. fresh world, no bed, no save yet: the distant respawn from the world spawn"
    ctl command "gamemode survival" >/dev/null
    before_new="$(count_log 'new respawn pos and bed')"
    if death_and_respawn ctl 1; then
        pos="$(position_of)"
        res="$(dist_check "$(echo "$pos" | cut -d ' ' -f 1)" "$(echo "$pos" | cut -d ' ' -f 3)" "$spawn_x" "$spawn_z" "$expected" 64)"
        if [ "${res##* }" = ok ] && [ "$(count_log 'new respawn pos and bed')" -gt "$before_new" ]; then
            pass b0 "fresh world, no bed: distant respawn ${res%% *} blocks from the world spawn, expected $expected"
        else
            fail b0 "fresh world, no bed: ${res%% *} blocks from the world spawn, expected $expected; 'new respawn pos and bed' lines +$(( $(count_log 'new respawn pos and bed') - before_new )), 'No distant respawn' lines total $(count_log 'No distant respawn')"
        fi
    else
        fail b0 "no respawn happened: $DR_NOTE"
    fi
fi

# ---- a. Mods row and rules ---------------------------------------------------------------------
if wanted a; then
    echo "--- a. mods row and rules"
    if ctl options mods >/dev/null; then
        ui="$(ctl ui)"
        if printf '%s' "$ui" | grep -qi 'Balanced Minecraft Hardcore\|bmh'; then
            pass a1 "the mod is listed on Options > Mods"
        else
            fail a1 "Options > Mods opened but no widget names the mod: $(printf '%s' "$ui" | head -c 300)"
        fi
        ctl key esc tap >/dev/null
        ctl key esc tap >/dev/null
        if ! ctl wait-for screen playing --timeout 10 >/dev/null; then
            ctl resume >/dev/null
        fi
    else
        skip a1 "horizon-ctl 'options mods' did not open the Mods page"
    fi
    if [ "$(count_log 'content not loaded')" -gt 0 ]; then
        fail a4 "the game did not load the content mod: $(grep -m1 'content not loaded' "$(logfile)" | sed 's/^[^]]*\]: //')"
    else
        pass a4 "the content mod (items, language, advancements) loaded with no warning"
    fi
    rule_reply="$(ctl command "bmh rule")"
    msgs="$(ctl messages 10)"
    if printf '%s\n%s' "$rule_reply" "$msgs" | grep -q 'random_death_spawn_point'; then
        pass a2 "/bmh rule answers and names the rules"
    else
        fail a2 "/bmh rule gave no rule names. Reply: $(printf '%s' "$rule_reply" | head -c 300)"
    fi
    rules_file="$(find "$GAMEDIR" -path '*/bmh/rules.txt' 2>/dev/null | head -n 1)"
    if [ -z "$rules_file" ]; then
        fail a3 "no <world>/bmh/rules.txt under $GAMEDIR"
    else
        lines="$(grep -Ec '^[a-z_]+ = (true|false)$' "$rules_file")"
        if [ "$lines" -eq 6 ]; then
            pass a3 "rules.txt has $lines rule lines ($rules_file)"
        else
            fail a3 "rules.txt has $lines rule lines, expected 6 ($rules_file)"
        fi
    fi
fi

# ---- b. Distant respawn ------------------------------------------------------------------------
if wanted b; then
    echo "--- b. distant respawn (survival, rule on, path: wait for the landing line, then horizon-ctl respawn)"
    ctl command "gamemode survival" >/dev/null
    if [ "$POINT_SET" -eq 0 ]; then
        # b0s: the same as b0, after a save. On the first run b0 failed and b0s passed. The cause
        # is probably that level.dat has a stale spawn (y 64 instead of 126) until the first save,
        # and the mod reads the world spawn from that file.
        ctl save >/dev/null
        before_new="$(count_log 'new respawn pos and bed')"
        if death_and_respawn ctl 1; then
            pos="$(position_of)"
            res="$(dist_check "$(echo "$pos" | cut -d ' ' -f 1)" "$(echo "$pos" | cut -d ' ' -f 3)" "$spawn_x" "$spawn_z" "$expected" 64)"
            if [ "${res##* }" = ok ] && [ "$(count_log 'new respawn pos and bed')" -gt "$before_new" ]; then
                pass b0s "no bed: distant respawn ${res%% *} blocks from the world spawn, expected $expected"
            else
                fail b0s "no bed: ${res%% *} blocks from the world spawn, expected $expected; 'new respawn pos and bed' lines +$(( $(count_log 'new respawn pos and bed') - before_new )), 'No distant respawn' lines total $(count_log 'No distant respawn')"
            fi
        else
            fail b0s "no respawn happened: $DR_NOTE"
        fi
    fi
    ensure_point
    ctl command "gamemode survival" >/dev/null
    before_new="$(count_log 'new respawn pos and bed')"
    before_none="$(count_log 'No distant respawn')"
    if death_and_respawn ctl 1; then
        DISTANT_DONE=$((DISTANT_DONE + 1))
        pos="$(position_of)"
        px="$(echo "$pos" | cut -d ' ' -f 1)"
        py="$(echo "$pos" | cut -d ' ' -f 2)"
        pz="$(echo "$pos" | cut -d ' ' -f 3)"
        res="$(dist_check "$px" "$pz" "$spawn_x" "$spawn_z" "$expected" 64)"
        if [ "${res##* }" = ok ]; then
            pass b1 "distance from the world spawn ${res%% *} blocks, expected $expected (tolerance 64)"
        else
            fail b1 "distance from the world spawn ${res%% *} blocks, expected $expected (tolerance 64)"
        fi
        if [ "$(count_log 'new respawn pos and bed')" -gt "$before_new" ] && [ "$(count_log 'No distant respawn')" -le "$before_none" ]; then
            pass b2 "log: landing found ('new respawn pos and bed'), no 'No distant respawn'"
        else
            fail b2 "log: new-pos lines +$(( $(count_log 'new respawn pos and bed') - before_new )), 'No distant respawn' lines +$(( $(count_log 'No distant respawn') - before_none ))"
        fi
        # Alone in the world, after normal distant respawns (b0, b0s, b1): the sounds have nobody to
        # reach and the second award is already held. Neither is a failure, so the mod logs no
        # warning at all (first run, 2026-11-17: "Sound command failed", "not awarded to").
        warns="$(count_log 'WARN.*BalancedMcHardcoreMain')"
        if [ "$warns" -eq 0 ]; then
            pass b4 "no warning line of the mod after $(count_log 'new respawn pos and bed') distant respawn(s) alone in the world"
        else
            fail b4 "$warns warning line(s) of the mod in the log: $(grep -m1 'WARN.*BalancedMcHardcoreMain' "$(logfile)" | sed 's/^[^]]*\]: //')"
        fi
        st="$(state)"
        health="$(jget "$st" 'd["player"]["health"]')"
        ground="$(jget "$st" 'd["player"]["on_ground"]')"
        dim="$(jget "$st" 'd["dimension"]')"
        bx="$(jget "$st" 'd["player"]["block_pos"][0]')"
        by="$(jget "$st" 'd["player"]["block_pos"][1]')"
        bz="$(jget "$st" 'd["player"]["block_pos"][2]')"
        feet="$(jget "$(ctl block "$bx" "$by" "$bz")" 'd["block"]')"
        below="$(jget "$(ctl block "$bx" "$((by - 1))" "$bz")" 'd["block"]')"
        alive="$(python3 -c 'import sys; print(float(sys.argv[1]) > 0)' "${health:-0}" 2>/dev/null)"
        solid=bad
        case "$below" in ""|*air|*water|*lava|*leaves) solid=bad ;; *) solid=ok ;; esac
        case "$feet" in *water*|*lava*) solid=bad ;; esac
        if [ "$alive" = True ] && [ "$ground" = True ] && [ "$solid" = ok ]; then
            pass b3 "alive (health $health), on ground, feet in '$feet', on '$below', dimension $dim, y=$py"
        else
            fail b3 "health '$health', on_ground '$ground', feet '$feet', below '$below', dimension '$dim'"
        fi
    else
        fail b1 "no distant respawn happened: $DR_NOTE"
    fi
fi

# ---- c. Immunity -------------------------------------------------------------------------------
if wanted c; then
    echo "--- c. 200 ticks of damage immunity after the arrival"
    ensure_point
    ctl command "gamemode survival" >/dev/null
    if death_and_respawn ctl 1; then
        DISTANT_DONE=$((DISTANT_DONE + 1))
        h0="$(health_of)"
        # The game gives 60 ticks of immunity at a respawn. The mod gives 200. Hit at about
        # tick 140 after the respawn (the 40 ticks of death_and_respawn plus 100 here), so that a
        # vanilla-length immunity would fail this check.
        ctl wait 100 >/dev/null
        dmg="$(ctl command "damage @s 6")"
        if printf '%s' "$dmg" | grep -qi '"error"'; then
            skip c "the command 'damage' is not available: $(printf '%s' "$dmg" | head -c 200)"
        else
            ctl wait 10 >/dev/null
            h1="$(health_of)"
            ok1="$(python3 -c 'import sys; print(float(sys.argv[2]) >= float(sys.argv[1]))' "${h0:-0}" "${h1:-0}" 2>/dev/null)"
            if [ "$ok1" = True ]; then
                pass c1 "health before $h0, after the damage within the first 200 ticks $h1 (no drop)"
            else
                fail c1 "health dropped inside the immunity: before $h0, after $h1"
            fi
            ctl wait 150 >/dev/null
            h2="$(health_of)"
            ctl command "damage @s 6" >/dev/null
            ctl wait 5 >/dev/null
            h3="$(health_of)"
            ok2="$(python3 -c 'import sys; print(float(sys.argv[2]) < float(sys.argv[1]))' "${h2:-0}" "${h3:-0}" 2>/dev/null)"
            if [ "$ok2" = True ]; then
                pass c2 "after the immunity: health $h2 then $h3 (damage works again)"
            else
                fail c2 "damage after 200 ticks changed nothing: health $h2 then $h3"
            fi
            # c3: a death inside the immunity ends it (Java keeps the count on the player that dies).
            # Distant respawn, /kill at about tick 40 (it bypasses the immunity), rule off, vanilla
            # respawn (60 ticks of vanilla immunity), damage at tick 80: it must hurt. With the shield
            # kept, 160 - 80 ticks would be left and the damage would do nothing.
            if death_and_respawn ctl 1; then
                DISTANT_DONE=$((DISTANT_DONE + 1))
                ctl command "bmh rule random_death_spawn_point false" >/dev/null
                if death_and_respawn ctl 0; then
                    ctl wait 40 >/dev/null
                    h4="$(health_of)"
                    ctl command "damage @s 6" >/dev/null
                    ctl wait 5 >/dev/null
                    h5="$(health_of)"
                    ok3="$(python3 -c 'import sys; print(float(sys.argv[2]) < float(sys.argv[1]))' "${h4:-0}" "${h5:-0}" 2>/dev/null)"
                    if [ "$ok3" = True ]; then
                        pass c3 "a death inside the immunity ends it: after a vanilla respawn, tick 80, health $h4 then $h5"
                    else
                        fail c3 "the immunity outlived the death: after a vanilla respawn, tick 80, health $h4 then $h5"
                    fi
                else
                    fail c3 "no vanilla respawn after the death inside the immunity: $DR_NOTE"
                fi
                ctl command "bmh rule random_death_spawn_point true" >/dev/null
            else
                fail c3 "no distant respawn for the death inside the immunity: $DR_NOTE"
            fi
        fi
    else
        fail c1 "no distant respawn happened: $DR_NOTE"
    fi
fi

# ---- d. Cinematic screenshots ------------------------------------------------------------------
shot_n=0
shot() {
    shot_n=$((shot_n + 1))
    f="$SHOTS/$(printf '%02d' "$shot_n")-$1.png"
    ctl screenshot "$f" >/dev/null
    shot_files="$shot_files $f"
}
shot_files=""
shot_death() { shot death-screen; }
if wanted d; then
    echo "--- d. cinematic screenshots (path: click-button Respawn, so the mod's hold and fade run)"
    ensure_point
    ctl command "gamemode survival" >/dev/null
    DR_HOOK=shot_death
    # The death screen shot is taken inside death_and_respawn, after the landing line, before the
    # click. The shots after the click are taken here, in the background of the wait for the event:
    # death_and_respawn blocks, so a second pass takes the arrival shots.
    if death_and_respawn click 1; then
        DISTANT_DONE=$((DISTANT_DONE + 1))
        shot arrival-after-40-ticks
        sleep 3
        shot arrival-plus-3s
        sleep 5
        shot arrival-plus-8s
    else
        fail d1 "no respawn for the screenshots: $DR_NOTE"
    fi
    DR_HOOK=""
    # The black, the sky and the dissolve happen between the click and the arrival. A second death
    # takes quick shots right after the click (half a second apart), no wait for the event between.
    seq="$(jget "$(ctl events)" 'd["last"]')"; [ -n "$seq" ] || seq=0
    landed_before="$(count_log 'Landing spot ready')"
    ctl command "kill @s" >/dev/null
    if ctl wait-for event death --since "$seq" --timeout 30 >/dev/null && wait_log 'Landing spot ready' "$landed_before" 150; then
        seq="$(jget "$(ctl events)" 'd["last"]')"; [ -n "$seq" ] || seq=0
        ctl click-button respawn >/dev/null || ctl respawn >/dev/null
        i=0
        while [ "$i" -lt 10 ]; do
            shot "journey-$(printf '%02d' "$i")"
            sleep 1
            i=$((i + 1))
        done
        ctl wait-for event respawn --since "$seq" --timeout 90 >/dev/null
        ctl wait 40 >/dev/null
        shot journey-arrived
    else
        fail d2 "the second death for the journey frames did not reach the landing line"
    fi
    missing_shot=0
    count_shot=0
    for f in $shot_files; do
        count_shot=$((count_shot + 1))
        if [ ! -s "$f" ]; then missing_shot=$((missing_shot + 1)); echo "  empty or missing: $f"; fi
    done
    if [ "$count_shot" -gt 0 ] && [ "$missing_shot" -eq 0 ]; then
        pass d3 "$count_shot screenshots written, none empty: $SHOTS"
    else
        fail d3 "$missing_shot of $count_shot screenshots are missing or empty: $SHOTS"
    fi
    echo "  A person or a reviewer must look at them: the script cannot judge them."
fi

# ---- e. Rule off -------------------------------------------------------------------------------
if wanted e; then
    echo "--- e. rule random_death_spawn_point false: vanilla respawn"
    ensure_point
    ctl command "gamemode survival" >/dev/null
    ctl command "bmh rule random_death_spawn_point false" >/dev/null
    rules_file="$(find "$GAMEDIR" -path '*/bmh/rules.txt' 2>/dev/null | head -n 1)"
    if [ -n "$rules_file" ] && grep -q '^random_death_spawn_point = false' "$rules_file"; then
        pass e1 "the rule is false in rules.txt"
    else
        fail e1 "rules.txt does not show random_death_spawn_point = false (file: ${rules_file:-none})"
    fi
    before_new="$(count_log 'new respawn pos and bed')"
    if death_and_respawn ctl 0; then
        pos="$(position_of)"
        res="$(near_check "$(echo "$pos" | cut -d ' ' -f 1)" "$(echo "$pos" | cut -d ' ' -f 3)" "$spawn_x" "$spawn_z" 200)"
        if [ "${res##* }" = ok ] && [ "$(count_log 'new respawn pos and bed')" -eq "$before_new" ]; then
            pass e2 "vanilla respawn: ${res%% *} blocks from the world spawn, no new landing line"
        else
            fail e2 "${res%% *} blocks from the world spawn (max 200); landing lines +$(( $(count_log 'new respawn pos and bed') - before_new ))"
        fi
    else
        fail e2 "no respawn: $DR_NOTE"
    fi
    ctl command "bmh rule random_death_spawn_point true" >/dev/null
    if grep -q '^random_death_spawn_point = true' "$rules_file" 2>/dev/null; then
        pass e3 "the rule is true again"
    else
        fail e3 "the rule was not set back to true"
    fi
fi

# ---- f. Creative -------------------------------------------------------------------------------
if wanted f; then
    echo "--- f. creative: a death gives the vanilla respawn"
    ensure_point
    ctl command "gamemode creative" >/dev/null
    before_new="$(count_log 'new respawn pos and bed')"
    if death_and_respawn ctl 0; then
        pos="$(position_of)"
        res="$(near_check "$(echo "$pos" | cut -d ' ' -f 1)" "$(echo "$pos" | cut -d ' ' -f 3)" "$spawn_x" "$spawn_z" 200)"
        if [ "${res##* }" = ok ] && [ "$(count_log 'new respawn pos and bed')" -eq "$before_new" ]; then
            pass f1 "creative respawn: ${res%% *} blocks from the world spawn, no landing line"
        else
            fail f1 "${res%% *} blocks from the world spawn (max 200); landing lines +$(( $(count_log 'new respawn pos and bed') - before_new ))"
        fi
    else
        fail f1 "no respawn in creative: $DR_NOTE"
    fi
    ctl command "gamemode survival" >/dev/null
fi

# ---- g. Advancement ----------------------------------------------------------------------------
if wanted g; then
    echo "--- g. advancement bmh:journey/exile after the first distant respawn"
    ensure_point
    ctl command "gamemode survival" >/dev/null
    if [ "$DISTANT_DONE" -eq 0 ]; then
        death_and_respawn ctl 1 && DISTANT_DONE=$((DISTANT_DONE + 1))
    fi
    # The award comes from server_tick, a tick or two after the respawn.
    ctl wait 40 >/dev/null
    ctl save >/dev/null
    adv_file=""
    for f in $(find "$GAMEDIR" -path '*advancements*' -name '*.json' 2>/dev/null); do
        if grep -q 'bmh:journey/exile' "$f" 2>/dev/null; then adv_file="$f"; fi
    done
    if [ -n "$adv_file" ]; then
        done_flag="$(python3 -c '
import json, sys
d = json.load(open(sys.argv[1]))
print(d.get("bmh:journey/exile", {}).get("done"))' "$adv_file")"
        if [ "$done_flag" = True ]; then
            pass g1 "bmh:journey/exile is done in $adv_file"
        else
            fail g1 "bmh:journey/exile is in $adv_file but not done ($done_flag)"
        fi
    else
        # No advancement file: ask the game. Revoking an advancement that the player has says
        # "Revoked"; one that the player lacks says it could not revoke.
        rv="$(ctl command "advancement revoke @s only bmh:journey/exile")"
        if printf '%s' "$rv" | grep -qi 'revoked'; then
            pass g1 "advancement revoke found bmh:journey/exile granted (it is now revoked)"
        elif printf '%s' "$rv" | grep -qi 'unknown advancement'; then
            fail g1 "the game does not know bmh:journey/exile (the advancement data is not loaded): $(printf '%s' "$rv" | head -c 200)"
        elif printf '%s' "$rv" | grep -qi "couldn't\|could not\|no advancement"; then
            fail g1 "the player does not have bmh:journey/exile: $(printf '%s' "$rv" | head -c 300)"
        else
            skip g1 "no advancement file in the world and the revoke reply was not readable: $(printf '%s' "$rv" | head -c 200)"
        fi
    fi
fi

# ---- h. The nine items -------------------------------------------------------------------------
# Not about respawn: an earlier run's screenshots of the items were lost, so this keeps one.
if wanted h; then
    echo "--- h. the nine items of the mod in the inventory, one screenshot"
    ctl command "gamemode survival" >/dev/null
    for id in bmh:eternal_sword bmh:eternal_axe bmh:eternal_pickaxe bmh:eternal_bundle bmh:navigator_amulet \
        bmh:totem_of_eternity bmh:sleeping_totem bmh:dormant_totem bmh:echo_heart; do
        ctl give "$id" 1 >/dev/null
    done
    ctl wait 10 >/dev/null
    ctl open-inventory >/dev/null
    ctl wait 10 >/dev/null
    f="$SHOTS/h-items-inventory.png"
    ctl screenshot "$f" >/dev/null
    ctl close >/dev/null
    if [ -s "$f" ]; then
        pass h1 "inventory screenshot written, $(wc -c < "$f" | tr -d ' ') bytes: $f"
    else
        fail h1 "no inventory screenshot, or an empty one: $f"
    fi
    # Six of them are Eternal Items: they would change the distance of the next deaths.
    ctl command "clear @s" >/dev/null
fi

# ---- i. A respawn point that does not work -----------------------------------------------------
# Java takes vanilla's resolution first: a forced point with no room for a player is a missing
# respawn block, so the exile is round the world spawn, and the player is told
# (block.minecraft.spawn.not_valid). The control tool can block a forced point; a real bed slept in
# and a Nether anchor are hand checks (TESTING.md).
if wanted i; then
    echo "--- i. a /spawnpoint blocked by stone at death: the exile centre is the world spawn"
    ctl command "gamemode survival" >/dev/null
    st="$(state)"
    px="$(jget "$st" 'd["player"]["block_pos"][0]')"
    py="$(jget "$st" 'd["player"]["block_pos"][1]')"
    pz="$(jget "$st" 'd["player"]["block_pos"][2]')"
    qx=$((px + 3))
    reply="$(ctl command "spawnpoint @s $qx $py $pz")"
    echo "test-slice.sh: spawnpoint at $qx $py $pz: $(printf '%s' "$reply" | head -c 200)"
    ctl command "setblock $qx $py $pz minecraft:stone" >/dev/null
    ctl command "setblock $qx $((py + 1)) $pz minecraft:stone" >/dev/null
    POINT_SET=0
    before_missing="$(count_log 'reads minecraft:stone: Missing')"
    if death_and_respawn ctl 1; then
        DISTANT_DONE=$((DISTANT_DONE + 1))
        pos="$(position_of)"
        res="$(dist_check "$(echo "$pos" | cut -d ' ' -f 1)" "$(echo "$pos" | cut -d ' ' -f 3)" "$world_x" "$world_z" "$expected" 64)"
        missing="$(( $(count_log 'reads minecraft:stone: Missing') - before_missing ))"
        if [ "${res##* }" = ok ] && [ "$missing" -gt 0 ]; then
            pass i1 "blocked point: distant respawn ${res%% *} blocks from the world spawn ($world_x, $world_z), expected $expected; log: the point reads stone, Missing"
        else
            fail i1 "blocked point: ${res%% *} blocks from the world spawn ($world_x, $world_z), expected $expected; 'Missing' lines +$missing; $(grep 'Respawn point' "$(logfile)" | tail -n 1 | sed 's/^[^]]*\]: //')"
        fi
        msgs="$(ctl messages 20)"
        if printf '%s' "$msgs" | grep -qi 'no home bed\|spawn.not_valid'; then
            pass i2 "the player is told: $(printf '%s' "$msgs" | grep -oi 'you have no home bed[^"]*' | head -n 1)"
        else
            fail i2 "no 'no home bed' line in the chat: $(printf '%s' "$msgs" | head -c 300)"
        fi
    else
        fail i1 "no respawn happened: $DR_NOTE"
    fi
fi

# ---- k. The Create New World button ------------------------------------------------------------------
# game-checks-1 (2026-11-17). The button is drawn by the mod, not a widget: "ui" does not list it. Its
# state is the mod setting file (CHOICE_FILE), and a new world copies it into <world>/bmh/rules.txt.
# Settings fix round 1 (2026-11-17): the button sits in the Game tab, in the row under Allow Commands, as
# wide as the tab's buttons. Its place is read from the bounds of the Allow Commands widget of "ui".
CHOICE_FILE="$GAMEDIR/horizon-mods/.horizon-mod-data/bmh/balanced_hardcore"
choice_now() { cat "$CHOICE_FILE" 2>/dev/null | tr -d ' \n'; }
# button_xy: the middle of the button, "X Y" in GUI pixels: the middle of Allow Commands, then the
# button's own row (its height 20 and the row gap 8, then half of the 20 high button). Empty when the
# open tab is not the Game tab.
button_xy() {
    jget "$(ctl ui)" '" ".join(str(int(v)) for v in (lambda b: (b[0] + b[2] / 2, b[1] + b[3] + 8 + 10))([w["bounds"] for w in d["widgets"] if w["id"]=="allow-commands"][0]))'
}
# flip_button: one click on the button of the Game tab.
flip_button() {
    xy="$(button_xy)"
    ctl click-at "${xy%% *}" "${xy##* }" >/dev/null
    sleep 1
}

# leave_world: pause, Save and Quit to Title, wait for the title screen.
leave_world() {
    # A journey that is still running holds the screen (its hold screen takes Escape): wait for the end.
    ctl wait-for screen playing --timeout 30 >/dev/null
    ctl pause >/dev/null
    ctl click save-and-quit-to-title >/dev/null
    ctl wait-for screen title --timeout 60 >/dev/null
    sleep 3
}

# open_create_screen: the title screen to the Create New World screen. It also opens the screen
# directly when no world is saved yet.
open_create_screen() {
    ctl click singleplayer >/dev/null
    sleep 2
    ctl click create-new-world >/dev/null
    sleep 2
}

# make_world NAME: on the Create New World screen, name the world, create it, wait for it to open.
make_world() {
    ctl type --field name --replace "$1" >/dev/null
    ctl click create-new-world >/dev/null
    ctl wait-for loaded --timeout 200 >/dev/null
    sleep 4
}

# gui_width: the GUI width of the open screen (ui reply).
gui_width() { jget "$(ctl ui)" 'int(d["gui_size"][0])'; }

if wanted k; then
    echo "--- k. the Create New World button: where it is, what a click does, what the worlds hold"
    ctl command "gamemode survival" >/dev/null
    leave_world
    open_create_screen
    gw="$(gui_width)"
    if [ "$(jget "$(ctl ui)" '[w["id"] for w in d["widgets"] if w["kind"]=="tab"]')" = "['game', 'world', 'more']" ]; then
        pass k0 "the Create New World screen is open (GUI width $gw)"
    else
        fail k0 "the Create New World screen did not open: $(ctl ui | head -c 300)"
    fi
    flip_button
    c1="$(choice_now)"
    flip_button
    c2="$(choice_now)"
    if [ "$c1" = false ] && [ "$c2" = true ]; then
        pass k1 "a click on the button (row under Allow Commands, '$(button_xy)') flips the choice: first click '$c1', second click '$c2' (file $CHOICE_FILE)"
    else
        fail k1 "the clicks did not flip the choice: first '$c1', second '$c2' (button at '$(button_xy)')"
    fi
    # The button is 210 wide in the Game tab, under Allow Commands. It never lies over a tab: a click on
    # the middle of the More tab must open that tab and leave the choice alone.
    mb="$(jget "$(ctl ui)" '[w["bounds"] for w in d["widgets"] if w["id"]=="more"][0]')"
    mx="$(jget "$(ctl ui)" 'int([w["bounds"] for w in d["widgets"] if w["id"]=="more"][0][0] + [w["bounds"] for w in d["widgets"] if w["id"]=="more"][0][2] / 2)')"
    bxy="$(button_xy)"
    ctl click-at "$mx" 12 >/dev/null
    sleep 1
    c3="$(choice_now)"
    if [ -z "$mx" ] || [ -z "$c2" ]; then
        fail k2 "no More tab or no choice file to read (tab bounds '$mb', choice '$c2')"
    elif [ "$c3" = "$c2" ]; then
        pass k2 "a click on the middle of the More tab ($mx, 12; tab bounds $mb) leaves the button alone"
    else
        fail k2 "a click on the middle of the More tab ($mx, 12; tab bounds $mb) flipped the button ('$c2' to '$c3'): the button lies over the tab"
    fi
    # The button shows in the Game tab only. On the More tab (open now) and on the World tab, a click at
    # the place the button has in the Game tab must not flip it.
    more_open="$(jget "$(ctl ui)" '[w["selected"] for w in d["widgets"] if w["id"]=="more"][0]')"
    ctl click-at "${bxy%% *}" "${bxy##* }" >/dev/null
    sleep 1
    c4="$(choice_now)"
    ctl click world >/dev/null
    sleep 1
    ctl click-at "${bxy%% *}" "${bxy##* }" >/dev/null
    sleep 1
    c4w="$(choice_now)"
    if [ "$more_open" = True ] && [ -n "$bxy" ] && [ "$c4" = "$c2" ] && [ "$c4w" = "$c2" ]; then
        pass k2b "no button on the More and World tabs: a click at the Game tab's place ($bxy) leaves the choice at '$c2'"
    else
        fail k2b "a click at the button's place ($bxy) changed the choice on another tab: More open '$more_open', '$c2' to '$c4' (More), '$c4w' (World)"
    fi
    ctl click game >/dev/null
    sleep 1
    # Back to ON (the state of a Java world), then make the ON world.
    [ "$(choice_now)" = true ] || flip_button
    sleep 1
    make_world bmhon
    f="$GAMEDIR/saves/bmhon/bmh/rules.txt"
    if [ -f "$f" ] && [ "$(grep -Ec '^[a-z_]+ = true$' "$f")" -eq 6 ]; then
        pass k3 "a world made with the button ON: six rules true ($f)"
    else
        fail k3 "a world made with the button ON: rules.txt: $(grep -c . "$f" 2>/dev/null) lines, $(grep -Ec '= true$' "$f" 2>/dev/null) true"
    fi
    leave_world
    open_create_screen
    flip_button
    make_world bmhoff
    f="$GAMEDIR/saves/bmhoff/bmh/rules.txt"
    if [ -f "$f" ] && [ "$(grep -Ec '^[a-z_]+ = false$' "$f")" -eq 6 ]; then
        pass k4 "a world made with the button OFF: six rules false ($f)"
    else
        fail k4 "a world made with the button OFF: rules.txt: $(grep -c . "$f" 2>/dev/null) lines, $(grep -Ec '= false$' "$f" 2>/dev/null) false"
    fi
    leave_world
    # The Java button starts ON on every new Create New World screen: its state is the rules of the
    # screen. The choice of the last world (OFF) must not come back. The first click then turns it OFF,
    # which also shows that the button's own state, not only the file, started ON.
    open_create_screen
    c5="$(choice_now)"
    flip_button
    c5a="$(choice_now)"
    flip_button
    c5b="$(choice_now)"
    if [ "$c5" = true ] && [ "$c5a" = false ] && [ "$c5b" = true ]; then
        pass k5 "a new Create New World screen starts ON (as the Java button does): file '$c5', first click '$c5a', second '$c5b'"
    else
        fail k5 "a new Create New World screen: file '$c5' (the Java button starts ON each time), first click '$c5a', second '$c5b'"
    fi
    # Cancel and a new screen: OFF is forgotten again.
    flip_button
    c5o="$(choice_now)"
    ctl click cancel >/dev/null
    sleep 1
    ctl click create-new-world >/dev/null
    sleep 2
    c5c="$(choice_now)"
    if [ "$c5o" = false ] && [ "$c5c" = true ]; then
        pass k5c "Cancel with the button OFF, then a new Create New World screen: ON again"
    else
        fail k5c "Cancel with the button '$c5o', then a new Create New World screen: '$c5c' (expected ON)"
    fi
    # The editor of a Superflat world is part of the same screen: the choice stays while the player
    # opens it and comes back.
    flip_button
    c5f="$(choice_now)"
    ctl click world >/dev/null
    sleep 0.5
    ctl click world-type >/dev/null
    sleep 0.5
    ctl click customize >/dev/null
    sleep 1
    ctl click done >/dev/null
    sleep 1
    ctl click game >/dev/null
    sleep 1
    c5d="$(choice_now)"
    if [ "$c5f" = false ] && [ "$c5d" = false ]; then
        pass k5d "the Superflat editor and back to the Game tab keep the choice ($c5d)"
    else
        fail k5d "the Superflat editor and back: before '$c5f', after '$c5d' (expected OFF, OFF)"
    fi
    # Data Packs opens a screen of the game's own over the Create New World screen, and that screen
    # fires the screen hooks with its own id. It belongs to the same Create New World screen: the
    # choice (OFF here) stays. "ui" lists no button of that screen, so the test clicks its Done button
    # by place: the right one of two 150 wide buttons, 8 apart, 26 above the bottom edge (read from a
    # screenshot at GUI 427 x 240). The count of tab widgets tells that the screen opened and closed.
    tabs_now() { jget "$(ctl ui)" 'len([w for w in d["widgets"] if w["kind"]=="tab"])'; }
    [ "$(choice_now)" = false ] || flip_button
    c5g="$(choice_now)"
    ctl click more >/dev/null
    sleep 1
    ctl click data-packs >/dev/null
    sleep 2
    dp_tabs="$(tabs_now)"
    dxy="$(jget "$(ctl ui)" '" ".join(str(int(v)) for v in (d["gui_size"][0] / 2 + 79, d["gui_size"][1] - 16))')"
    ctl screenshot "$SHOTS/k5e-data-packs-open.png" >/dev/null
    ctl click-at "${dxy%% *}" "${dxy##* }" >/dev/null
    sleep 2
    dp_back="$(tabs_now)"
    ctl click game >/dev/null
    sleep 1
    c5h="$(choice_now)"
    ctl screenshot "$SHOTS/k5e-game-tab-after-data-packs.png" >/dev/null
    if [ "$dp_tabs" != 0 ] || [ "$dp_back" != 3 ]; then
        fail k5e "the Data Packs screen did not open and close as the test expects (tab widgets: $dp_tabs with it open, $dp_back after a click on Done at '$dxy'): the check proves nothing"
    elif [ "$c5g" = false ] && [ "$c5h" = false ]; then
        pass k5e "Data Packs and back: the choice stays OFF (Done clicked at '$dxy')"
    else
        fail k5e "the choice changed behind the Data Packs screen: '$c5g' before, '$c5h' after (expected false, false)"
    fi
    # Game Rules opens a list inside the title's own screen. The choice stays there too.
    [ "$(choice_now)" = false ] || flip_button
    c5j="$(choice_now)"
    ctl click more >/dev/null
    sleep 1
    ctl click game-rules >/dev/null
    sleep 2
    gr_ids="$(jget "$(ctl ui)" '[w["id"] for w in d["widgets"]][:8]')"
    ctl screenshot "$SHOTS/k5f-game-rules-open.png" >/dev/null
    ctl click done >/dev/null
    sleep 2
    gr_back="$(tabs_now)"
    if [ "$gr_back" != 3 ]; then
        ctl key esc tap >/dev/null
        sleep 2
        gr_back="$(tabs_now)"
    fi
    ctl click game >/dev/null
    sleep 1
    c5k="$(choice_now)"
    if [ "$gr_back" != 3 ]; then
        fail k5f "the Game Rules list did not close as the test expects (tab widgets after Done and Escape: $gr_back; its widgets: $gr_ids): the check proves nothing"
    elif [ "$c5j" = false ] && [ "$c5k" = false ]; then
        pass k5f "Game Rules and back: the choice stays OFF"
    else
        fail k5f "the choice changed behind the Game Rules list: '$c5j' before, '$c5k' after (expected false, false)"
    fi
    ctl click cancel >/dev/null
    sleep 1
    ctl click back >/dev/null
    sleep 1
    ctl open "$WORLD" >/dev/null
    ctl wait-for loaded --timeout 200 >/dev/null
    sleep 3
    ctl command "gamemode survival" >/dev/null
    if [ "$(jget "$(state)" 'd["screen"]')" = playing ]; then
        pass k6 "the test world is open again"
    else
        fail k6 "the test world did not open again: $(state | head -c 200)"
    fi
fi

# ---- l. The settings page and the config file ----------------------------------------------------------
cfgval() { sed -n "s/^$1 *= *\([0-9]*\).*/\1/p" "$GAMEDIR/config/bmh-common.toml" 2>/dev/null | head -n 1; }
# cfg_click ID COUNT: click a widget of the settings page COUNT times, then wait for the reload.
cfg_click() {
    i=0
    while [ "$i" -lt "$2" ]; do ctl click "$1" >/dev/null; sleep 0.3; i=$((i + 1)); done
    sleep 1.5
}
if wanted l; then
    echo "--- l. Options > Mods > BMH settings: the six buttons, the limits, the file, the next death"
    ctl command "gamemode survival" >/dev/null
    ctl options mods >/dev/null
    sleep 1
    if jget "$(ctl ui)" '[w["label"] for w in d["widgets"]]' | grep -q 'Balanced Minecraft Hardcore'; then
        pass l0 "Options > Mods has the BMH row: $(jget "$(ctl ui)" '[w["label"] for w in d["widgets"] if "Balanced" in (w["label"] or "")][0]')"
    else
        fail l0 "no BMH row on Options > Mods: $(ctl ui | head -c 300)"
    fi
    ctl click settings >/dev/null
    sleep 1
    ids="$(jget "$(ctl ui)" '" ".join(sorted(w["id"] for w in d["widgets"] if w["id"] != "done"))')"
    if [ "$ids" = "amulet-points amulet-points-2 maximum-respawn-distance maximum-respawn-distance-2 minimum-respawn-distance minimum-respawn-distance-2" ]; then
        pass l1 "the settings page has the six stepper buttons: $(jget "$(ctl ui)" '" | ".join(w["label"] for w in d["widgets"] if w["id"] != "done")')"
    else
        fail l1 "the settings page buttons are: $ids"
    fi
    cfg_click minimum-respawn-distance-2 1
    cfg_click minimum-respawn-distance 2
    if [ "$(cfgval minDistanceFromBed)" = 15000 ]; then
        pass l2 "min distance: one click down at 5000 changes nothing, two clicks up give 15000 in the file"
    else
        fail l2 "min distance in the file: $(cfgval minDistanceFromBed), expected 15000"
    fi
    cfg_click maximum-respawn-distance-2 12
    if [ "$(cfgval maxDistanceFromBed)" = 15000 ] && [ "$(cfgval minDistanceFromBed)" = 15000 ]; then
        pass l3 "the max stops at the min: max $(cfgval maxDistanceFromBed), min $(cfgval minDistanceFromBed)"
    else
        fail l3 "the max went below the min or the min moved: max $(cfgval maxDistanceFromBed), min $(cfgval minDistanceFromBed)"
    fi
    cfg_click minimum-respawn-distance 1
    if [ "$(cfgval minDistanceFromBed)" = 15000 ]; then
        pass l4 "the min cannot pass the max: still $(cfgval minDistanceFromBed)"
    else
        fail l4 "the min passed the max: $(cfgval minDistanceFromBed) with max $(cfgval maxDistanceFromBed)"
    fi
    cfg_click maximum-respawn-distance 45
    cfg_click amulet-points-2 6
    lo="$(cfgval maxAmuletPoints)"
    cfg_click amulet-points 20
    hi="$(cfgval maxAmuletPoints)"
    if [ "$(cfgval maxDistanceFromBed)" = 200000 ] && [ "$lo" = 1 ] && [ "$hi" = 16 ]; then
        pass l5 "the ranges hold: max distance 200000, amulet points $lo at the bottom and $hi at the top"
    else
        fail l5 "the ranges: max distance $(cfgval maxDistanceFromBed) (expected 200000), amulet points $lo and $hi (expected 1 and 16)"
    fi
    # The same distance for the next death with no restart: min = max = 15000.
    cfg_click maximum-respawn-distance-2 37
    cfg_click amulet-points-2 11
    ctl click done >/dev/null
    sleep 1
    ctl click done >/dev/null
    sleep 1
    ctl key esc tap >/dev/null
    ctl resume >/dev/null
    sleep 1
    if [ "$(cfgval minDistanceFromBed)" = 15000 ] && [ "$(cfgval maxDistanceFromBed)" = 15000 ]; then
        ensure_point
        ctl command "gamemode survival" >/dev/null
        if death_and_respawn ctl 1; then
            pos="$(position_of)"
            res="$(dist_check "$(echo "$pos" | cut -d ' ' -f 1)" "$(echo "$pos" | cut -d ' ' -f 3)" "$spawn_x" "$spawn_z" 15000 64)"
            if [ "${res##* }" = ok ]; then
                pass l6 "the next death uses the new distance with no restart: ${res%% *} blocks, expected 15000"
            else
                fail l6 "the next death after the clicks: ${res%% *} blocks, expected 15000"
            fi
        else
            fail l6 "no respawn: $DR_NOTE"
        fi
    else
        fail l6 "the file does not hold min = max = 15000: $(cfgval minDistanceFromBed) $(cfgval maxDistanceFromBed)"
    fi
    # Back to the defaults through the page (5000, 60000, 5).
    ctl options mods >/dev/null
    sleep 1
    ctl click settings >/dev/null
    sleep 1
    cfg_click maximum-respawn-distance 9
    cfg_click minimum-respawn-distance-2 2
    ctl click done >/dev/null
    sleep 1
    ctl click done >/dev/null
    sleep 1
    ctl key esc tap >/dev/null
    ctl resume >/dev/null
    if [ "$(cfgval minDistanceFromBed)" = 5000 ] && [ "$(cfgval maxDistanceFromBed)" = 60000 ] && [ "$(cfgval maxAmuletPoints)" = 5 ]; then
        pass l7 "back to the defaults through the page: 5000, 60000, 5"
    else
        fail l7 "the defaults did not come back: $(cfgval minDistanceFromBed) $(cfgval maxDistanceFromBed) $(cfgval maxAmuletPoints)"
    fi
fi

# ---- n. The journey on the real death screen path --------------------------------------------------------
# journey_probe DIR HOLD: die, wait for the landing, click Respawn, then for 7 s take a screenshot and read
# the screen name and the position after each one. HOLD=1 also holds W and taps Escape, E, Space once the
# mod's hold screen is up. Prints: cream_max mod_frames positions_during_mod black_frames pil_ok
journey_probe() {
    mkdir -p "$1"
    CTL_BIN="$CTL" CTL_SOCK="$SOCK" PROBE_DIR="$1" PROBE_HOLD="$2" PROBE_LOG="$(logfile)" python3 -W ignore - <<'EOF'
import os, subprocess, json, time
ctl, sock, out, hold, log = os.environ["CTL_BIN"], os.environ["CTL_SOCK"], os.environ["PROBE_DIR"], os.environ["PROBE_HOLD"] == "1", os.environ["PROBE_LOG"]
def c(*a):
    return subprocess.run([ctl, "--socket", sock, "--timeout", "60"] + list(a), capture_output=True, text=True).stdout
def landings():
    try:
        return open(log, errors="ignore").read().count("Landing spot ready")
    except OSError:
        return 0
try:
    from PIL import Image
    pil = True
except ImportError:
    pil = False
n0 = landings()
c("command", "kill @s")
c("wait-for", "screen", "dead", "--timeout", "30")
t = time.time()
while landings() <= n0 and time.time() - t < 150:
    time.sleep(0.5)
time.sleep(1.5)
t0 = time.time()
c("click-button", "respawn")
cream, mod_frames, black, positions, i, sent = 0, 0, 0, set(), 0, False
while time.time() - t0 < 7:
    f = "%s/f%03d.png" % (out, i)
    c("screenshot", f)
    st = json.loads(c("state") or "{}")
    screen = st.get("screen")
    if screen == "mod":
        mod_frames += 1
        if st.get("player"):
            positions.add(tuple(round(v, 1) for v in st["player"]["position"]))
        if hold and not sent:
            c("key", "w", "down")
            for k in ("esc", "e", "space"):
                c("key", k, "tap")
            sent = True
    if pil:
        im = Image.open(f).convert("RGB").resize((640, 360))
        # Only frames of the hold screen: after the dissolve starts the frame is the world, and a pale
        # sand or snow block has the colour of the message (full run 2026-11-17: 278 pixels with the rule off, probably sand).
        if screen == "mod":
            k = sum(1 for r, g, b in im.getdata() if r > 230 and 215 < g < 250 and 180 < b < 225 and r - b > 25)
            cream = max(cream, k)
        if sum(im.convert("L").getdata()) / (640 * 360) < 3:
            black += 1
        os.remove(f)
    i += 1
if hold:
    c("key", "w", "up")
    c("release")
print(cream, mod_frames, len(positions), black, 1 if pil else 0)
EOF
}

if wanted n; then
    echo "--- n. the journey on the death screen path: message rule, held input, immediate respawn"
    ensure_point
    ctl command "gamemode survival" >/dev/null
    ctl command "bmh rule random_death_spawn_point true" >/dev/null
    ctl command "bmh rule display_text_on_respawn true" >/dev/null
    out="$(journey_probe "$SHOTS/n-on" 0)"
    set -- $out
    if [ "$5" != 1 ]; then
        skip n1 "python3 has no PIL: the pixels of the message cannot be counted"
    else
        if [ "$2" -gt 5 ] && [ "$1" -gt 100 ]; then
            pass n1 "rule on: the journey showed $2 frames of the hold screen, $1 cream pixels of message at most, $4 black frames"
        else
            fail n1 "rule on: hold frames $2, cream pixels $1 (expected more than 100), black frames $4"
        fi
        ctl wait 60 >/dev/null
        ctl command "bmh rule display_text_on_respawn false" >/dev/null
        out="$(journey_probe "$SHOTS/n-off" 0)"
        set -- $out
        if [ "$2" -gt 5 ] && [ "$1" -le 30 ]; then
            pass n2 "rule off: the journey ran ($2 hold frames, $4 black frames) and showed no message ($1 cream pixels at most)"
        else
            fail n2 "rule off: hold frames $2, cream pixels $1 (expected at most 30)"
        fi
        ctl command "bmh rule display_text_on_respawn true" >/dev/null
    fi
    ctl wait 60 >/dev/null
    out="$(journey_probe "$SHOTS/n-hold" 1)"
    set -- $out
    if [ "$2" -gt 5 ] && [ "$3" -eq 1 ]; then
        pass n3 "W held and Escape, E, Space tapped on the hold screen: $2 frames, $3 position, screen stayed 'mod'"
    else
        fail n3 "during the journey: hold frames $2, distinct positions $3 (expected 1 over more than 5 frames)"
    fi
    pos_a="$(position_of)"
    ctl move forward 1 >/dev/null
    pos_b="$(position_of)"
    if [ "$pos_a" != "$pos_b" ]; then
        pass n4 "the player moves again after the journey: $pos_a to $pos_b"
    else
        fail n4 "the player cannot move after the journey: $pos_a"
    fi
    rm -rf "$SHOTS/n-on" "$SHOTS/n-off" "$SHOTS/n-hold"
    # Immediate respawn: the game rule skips the death screen, so the mod's hold never runs. The exile
    # must still happen (the area log expects the cinematic to start in black at the arrival).
    ctl wait 60 >/dev/null
    ctl command "gamerule immediate_respawn true" >/dev/null
    warn_before="$(count_log 'WARN.*BalancedMcHardcoreMain')"
    ctl command "kill @s" >/dev/null
    sleep 6
    pos="$(position_of)"
    res="$(dist_check "$(echo "$pos" | cut -d ' ' -f 1)" "$(echo "$pos" | cut -d ' ' -f 3)" "$spawn_x" "$spawn_z" "$expected" 64)"
    warns="$(( $(count_log 'WARN.*BalancedMcHardcoreMain') - warn_before ))"
    ctl command "gamerule immediate_respawn false" >/dev/null
    if [ "${res##* }" = ok ] && [ "$warns" -eq 0 ]; then
        pass n5 "immediate_respawn true: the exile still happens, ${res%% *} blocks from the point, expected $expected"
    else
        fail n5 "immediate_respawn true: ${res%% *} blocks from the point, expected $expected; new WARN lines $warns: $(grep 'WARN.*BalancedMcHardcoreMain' "$(logfile)" | tail -n 1 | sed 's/^[^]]*\]: //')"
    fi
    ctl wait 40 >/dev/null
fi

# ---- p. The eternal area: C14, C11, C13, E10, E11, E12 ---------------------------------------------------
# Game checks 2 (2026-11-18). The hand run of the same flows is in docs/horizon/log/game-checks-2.md. The F3
# screen is a hand check (kept screenshots in docs/horizon/log/eternal-shots/): here the check reads the rule
# that the F3 screen reads (reduced_debug_info). Difficulty is peaceful inside p and q (a zombie killed the
# player once) and goes back to normal at the end.

# inv_count ID: how many of an item the player carries (0 when none).
inv_count() {
    jget "$(ctl inventory)" 'd["counts"].get("'"$1"'", 0)'
}
# inv_slot ID: the first slot (hotbar, then main) that holds an item. Empty when none.
inv_slot() {
    jget "$(ctl inventory)" '([s["slot"] for k in ("hotbar", "main") for s in d.get(k, []) if s.get("item") == "'"$1"'"] or [""])[0]'
}
# inv_comp ID: the components text of the first stack of an item.
inv_comp() {
    jget "$(ctl inventory)" '([s.get("components", "") for k in ("hotbar", "main") for s in d.get(k, []) if s.get("item") == "'"$1"'"] or [""])[0]'
}
# daytime: the time of day in ticks, from "time_of_day" in the "time" object of state ("time query daytime" does not exist on 26.3: it
# wants a timeline).
daytime() {
    python3 -c 'import sys; v = sys.argv[1]; print(int(float(v)) % 24000 if v else "")' "$(jget "$(state)" 'd["time"]["time_of_day"]')"
}
# sleep_through: interact with the bed at X Y Z, then wait until the night is skipped (up to 300 ticks).
sleep_through() {
    ctl interact "$1" "$2" "$3" >/dev/null
    k=0
    while [ "$k" -lt 15 ]; do
        ctl wait 20 >/dev/null
        dtv="$(daytime)"
        [ -n "$dtv" ] && [ "$dtv" -lt 5000 ] && break
        k=$((k + 1))
    done
    ctl wait 20 >/dev/null
}
# six_once: "yes" when each of the six Eternal Items is carried exactly once.
six_once() {
    jget "$(ctl inventory)" '"yes" if all(d["counts"].get(i, 0) == 1 for i in ("bmh:eternal_sword", "bmh:eternal_axe", "bmh:eternal_pickaxe", "bmh:eternal_bundle", "bmh:totem_of_eternity", "bmh:sleeping_totem")) else str(d["counts"])'
}
# kept_key_on_disk: "True" when a player file of the scratch world holds the key of the kept stacks.
kept_key_on_disk() {
    python3 - "$GAMEDIR/saves/$WORLD" <<'EOF'
import sys, os, gzip
found = False
for root, _, files in os.walk(sys.argv[1]):
    for f in files:
        if f.endswith(".dat") and "players" in root:
            try:
                if b"bmh_eternal_kept" in gzip.open(os.path.join(root, f)).read():
                    found = True
            except Exception:
                pass
print(found)
EOF
}
# center N: the middle of block N on its axis (a negative block N is not N + 0.5 written as text).
center() { python3 -c 'import sys; print(int(sys.argv[1]) + 0.5)' "$1"; }
# open_sky: clear a column of air above the player, so the amulet sees the sky. Sets PX PY PZ (the block).
open_sky() {
    st="$(state)"
    PX="$(jget "$st" 'd["player"]["block_pos"][0]')"
    PY="$(jget "$st" 'd["player"]["block_pos"][1]')"
    PZ="$(jget "$st" 'd["player"]["block_pos"][2]')"
    ctl command "fill $((PX - 1)) $((PY + 1)) $((PZ - 1)) $((PX + 1)) $((PY + 40)) $((PZ + 1)) minecraft:air" >/dev/null
    ctl wait 40 >/dev/null
}
# platform: a stone floor and an air box round the player, a bed two blocks east (foot at PX+2, head at PX+3),
# the player on its block, looking at the bed. Sets PX PY PZ.
platform() {
    st="$(state)"
    PX="$(jget "$st" 'd["player"]["block_pos"][0]')"
    PY="$(jget "$st" 'd["player"]["block_pos"][1]')"
    PZ="$(jget "$st" 'd["player"]["block_pos"][2]')"
    ctl command "fill $((PX - 2)) $((PY - 1)) $((PZ - 2)) $((PX + 5)) $((PY - 1)) $((PZ + 2)) minecraft:stone" >/dev/null
    ctl command "fill $((PX - 2)) $PY $((PZ - 2)) $((PX + 5)) $((PY + 4)) $((PZ + 2)) minecraft:air" >/dev/null
    ctl command "setblock $((PX + 2)) $PY $PZ minecraft:white_bed[part=foot,facing=east]" >/dev/null
    ctl command "setblock $((PX + 3)) $PY $PZ minecraft:white_bed[part=head,facing=east]" >/dev/null
    ctl command "tp @s $(center "$PX") $PY $(center "$PZ")" >/dev/null
    ctl wait 20 >/dev/null
    ctl look -90 35 >/dev/null
    ctl wait 5 >/dev/null
}

P_ENT=""
P_DAT=""
p_hook() {
    P_ENT="$(ctl entities 12 item)"
    ctl save >/dev/null
    P_DAT="$(kept_key_on_disk)"
}

if wanted p; then
    echo "--- p. eternal: reduced_debug_info, the six items after death, the copy rule, the item on the ground, the bundle, the wake-up reward"
    ensure_point
    ctl command "difficulty peaceful" >/dev/null
    ctl command "gamemode survival" >/dev/null
    ctl command "time set 6000" >/dev/null
    ctl command "clear @s" >/dev/null
    # The world saves reduced_debug_info when the mod forces it on, and the mod forgets its value before when a world
    # opens again (Java does the same, DisableDebugInfo.java:21-24). A world that earlier scenarios saved and opened
    # again starts with the rule true, so p2 and p3 would read a value that is not the mod's. Set it by hand, in
    # creative, as the player must.
    ctl command "gamemode creative" >/dev/null
    ctl wait 30 >/dev/null
    ctl command "gamerule reduced_debug_info false" >/dev/null
    ctl wait 10 >/dev/null
    ctl command "gamemode survival" >/dev/null
    ctl command "bmh rule remove_coordinates_in_survival true" >/dev/null
    warn_before_p="$(count_log 'did not run')"
    ctl wait 30 >/dev/null
    # C14: the rule that F3 reads.
    rdi() { jget "$(ctl command 'gamerule reduced_debug_info')" 'd["chat"][0].split()[-1]'; }
    r1="$(rdi)"
    ctl command "gamemode creative" >/dev/null
    ctl wait 20 >/dev/null
    r2="$(rdi)"
    ctl command "gamemode survival" >/dev/null
    ctl wait 20 >/dev/null
    r3="$(rdi)"
    ctl command "bmh rule remove_coordinates_in_survival false" >/dev/null
    ctl wait 20 >/dev/null
    r4="$(rdi)"
    ctl command "bmh rule remove_coordinates_in_survival true" >/dev/null
    ctl wait 20 >/dev/null
    r5="$(rdi)"
    if [ "$r1" = true ] && [ "$r3" = true ] && [ "$r5" = true ]; then
        pass p1 "survival, rule on: reduced_debug_info is true (F3 hides the coordinates: kept screenshot eternal-shots/c14-f3-survival.png)"
    else
        fail p1 "survival with the BMH rule on: reduced_debug_info is '$r1', '$r3' after creative, '$r5' after the rule came back (expected true)"
    fi
    if [ "$r2" = false ]; then
        pass p2 "creative: reduced_debug_info is false (F3 shows the coordinates: eternal-shots/c14-f3-creative.png)"
    else
        fail p2 "creative: reduced_debug_info is '$r2' (expected false)"
    fi
    if [ "$r4" = false ]; then
        pass p3 "bmh rule remove_coordinates_in_survival false: reduced_debug_info is false (eternal-shots/c14-f3-ruleoff.png)"
    else
        fail p3 "the rule is off and reduced_debug_info is '$r4' (expected false)"
    fi
    if [ "$(count_log 'did not run')" -eq "$warn_before_p" ]; then
        pass p4 "no 'did not run' warning of the C14 command"
    else
        fail p4 "the C14 command did not run: $(grep -m1 'did not run' "$(logfile)" | sed 's/^[^]]*\]: //')"
    fi

    # C11: six Eternal Items, a Dormant Totem and an amulet, then a death.
    ctl command 'give @s bmh:eternal_bundle[bundle_contents=[{id:"minecraft:stone",count:1}]]' >/dev/null
    for id in bmh:eternal_sword bmh:eternal_axe bmh:eternal_pickaxe bmh:totem_of_eternity bmh:sleeping_totem bmh:dormant_totem bmh:navigator_amulet; do
        ctl give "$id" 1 >/dev/null
    done
    ctl wait 40 >/dev/null
    p_six0="$(six_once)"
    DR_HOOK=p_hook
    if death_and_respawn ctl 1; then
        DR_HOOK=""
        if printf '%s' "$P_ENT" | grep -q 'bmh:navigator_amulet' && printf '%s' "$P_ENT" | grep -q 'bmh:dormant_totem' && ! printf '%s' "$P_ENT" | grep -Eq 'bmh:(eternal_|totem_of_eternity|sleeping_totem)'; then
            pass p5 "at the death only the amulet and the Dormant Totem fall; no Eternal Item lies on the ground"
        else
            fail p5 "items on the ground at the death: $(printf '%s' "$P_ENT" | head -c 400)"
        fi
        if [ "$P_DAT" = True ]; then
            pass p6 "while dead the player file holds bmh_eternal_kept"
        else
            fail p6 "while dead no player file holds bmh_eternal_kept ($P_DAT)"
        fi
        ctl save >/dev/null
        if [ "$p_six0" = yes ] && [ "$(six_once)" = yes ] && [ "$(inv_count bmh:dormant_totem)" = 0 ] && [ "$(inv_count bmh:navigator_amulet)" = 0 ]; then
            pass p7 "after the respawn the six Eternal Items are back, once each; the Dormant Totem and the amulet are not"
        else
            fail p7 "after the respawn: $(six_once) (before the death: $p_six0)"
        fi
        if printf '%s' "$(inv_comp bmh:eternal_bundle)" | grep -q 'minecraft:stone'; then
            pass p8 "the bundle still holds its stone"
        else
            fail p8 "the bundle after the respawn: $(inv_comp bmh:eternal_bundle)"
        fi
        if [ "$(kept_key_on_disk)" = False ]; then
            pass p9 "after the respawn no player file holds bmh_eternal_kept"
        else
            fail p9 "the key bmh_eternal_kept is still in a player file after the respawn"
        fi
        if death_and_respawn ctl 1 && [ "$(six_once)" = yes ]; then
            pass p10 "a second death gives no second copy of any Eternal Item"
        else
            fail p10 "after a second death: $(six_once) $DR_NOTE"
        fi
    else
        DR_HOOK=""
        fail p5 "no respawn happened: $DR_NOTE"
    fi

    # C11, a restart while dead.
    seq="$(jget "$(ctl events)" 'd["last"]')"; [ -n "$seq" ] || seq=0
    ctl command "kill @s" >/dev/null
    if ctl wait-for event death --since "$seq" --timeout 30 >/dev/null; then
        sleep 2
        b_land="$(count_log 'Landing spot ready')"
        ctl quit >/dev/null
        n=0
        while [ -n "$GAMEPID" ] && kill -0 "$GAMEPID" 2>/dev/null && [ "$n" -lt 60 ]; do sleep 1; n=$((n + 1)); done
        launch_game
        if [ "$launch_status" -ne 0 ]; then
            fail p11 "the second launch failed (status $launch_status)"
        elif ! ctl wait-for screen dead --timeout 300 >/dev/null; then
            fail p11 "the reopened world shows no death screen"
        else
            if [ "$(count_log 'Landing spot ready')" -lt "$b_land" ]; then b_land=0; fi
            wait_log 'Landing spot ready' "$b_land" 150
            seq="$(jget "$(ctl events)" 'd["last"]')"; [ -n "$seq" ] || seq=0
            ctl respawn >/dev/null
            ctl wait-for event respawn --since "$seq" --timeout 90 >/dev/null
            ctl wait 40 >/dev/null
            ctl save >/dev/null
            if [ "$(six_once)" = yes ] && [ "$(kept_key_on_disk)" = False ]; then
                pass p11 "dead when the game closed: after the restart the six come back once and the key is gone from the file"
            else
                fail p11 "after a restart while dead: $(six_once), key on disk: $(kept_key_on_disk)"
            fi
        fi
    else
        fail p11 "no death event within 30 s"
    fi

    # C11, a change of world while dead (check 15 of the area log).
    ctl command "difficulty peaceful" >/dev/null
    seq="$(jget "$(ctl events)" 'd["last"]')"; [ -n "$seq" ] || seq=0
    ctl command "kill @s" >/dev/null
    if ctl wait-for event death --since "$seq" --timeout 30 >/dev/null; then
        sleep 3
        # The first click on the Title Screen button did nothing in a hand run: click again until the title shows.
        n=0
        while [ "$(jget "$(state)" 'd.get("screen")')" != title ] && [ "$n" -lt 12 ]; do
            ctl click title-screen >/dev/null 2>&1
            sleep 5
            n=$((n + 1))
        done
        ctl open bmhpb 5 >/dev/null
        sleep 8
        ctl wait-for screen playing --timeout 200 >/dev/null
        ctl wait-for loaded --timeout 200 >/dev/null
        sleep 3
        ctl command "gamemode survival" >/dev/null
        b_inv="$(jget "$(ctl inventory)" 'd["counts"]')"
        b_land="$(count_log 'Landing spot ready')"
        # World B was made by "open", so its rules follow the Create New World choice file (k leaves it OFF): no
        # distant respawn is promised there, so do not wait for a landing line.
        if death_and_respawn ctl 0; then
            b_after="$(jget "$(ctl inventory)" 'd["counts"]')"
            ctl wait-for screen playing --timeout 60 >/dev/null
            ctl pause >/dev/null
            ctl click save-and-quit-to-title >/dev/null
            n=0
            while [ "$(jget "$(state)" 'd.get("screen")')" != title ] && [ "$n" -lt 60 ]; do sleep 2; n=$((n + 1)); done
            sleep 2
            b_land="$(count_log 'Landing spot ready')"
            ctl open "$WORLD" "$SEED" >/dev/null
            if ctl wait-for screen dead --timeout 200 >/dev/null; then
                wait_log 'Landing spot ready' "$b_land" 150
                seq="$(jget "$(ctl events)" 'd["last"]')"; [ -n "$seq" ] || seq=0
                ctl respawn >/dev/null
                ctl wait-for event respawn --since "$seq" --timeout 90 >/dev/null
                ctl wait 40 >/dev/null
                ctl save >/dev/null
                if [ "$b_inv" = "{}" ] && [ "$b_after" = "{}" ] && [ "$(six_once)" = yes ] && [ "$(kept_key_on_disk)" = False ]; then
                    pass p12 "change of world while dead: world B gave nothing ($b_after); world A gave the six once and cleared the key"
                else
                    fail p12 "change of world while dead: world B before '$b_inv', after '$b_after'; world A: $(six_once), key on disk $(kept_key_on_disk)"
                fi
            else
                fail p12 "world A did not show the death screen after it was opened again"
            fi
        else
            fail p12 "no respawn in world B: $DR_NOTE"
        fi
    else
        fail p12 "no death event in world A"
    fi

    # E11, E10: a second Eternal Sword. A failed step above may leave the player dead: respawn first.
    if [ "$(jget "$(state)" 'd.get("screen")')" = dead ]; then ctl respawn >/dev/null; ctl wait 60 >/dev/null; fi
    ctl command "difficulty peaceful" >/dev/null
    ctl command "gamemode survival" >/dev/null
    ctl command 'kill @e[type=item]' >/dev/null
    ctl command "tag @e[type=item] remove x" >/dev/null
    ctl give bmh:eternal_sword 1 >/dev/null
    ctl wait 120 >/dev/null
    ent="$(ctl entities 12 item)"
    if [ "$(inv_count bmh:eternal_sword)" = 1 ] && printf '%s' "$ent" | grep -q 'bmh:eternal_sword'; then
        pass p13 "E11: a second Eternal Sword lies on the ground and is not picked up (120 ticks)"
    else
        fail p13 "E11: sword count $(inv_count bmh:eternal_sword); items near: $(printf '%s' "$ent" | head -c 300)"
    fi
    data="$(ctl command 'data get entity @e[type=item,sort=nearest,limit=1]')"
    if printf '%s' "$data" | grep -q 'Health: 32767s' && printf '%s' "$data" | grep -q 'bmh:eternal_sword'; then
        pass p14 "E10: the dropped sword has Health 32767s"
    else
        fail p14 "E10: $(printf '%s' "$data" | head -c 400)"
    fi
    ctl command "tag @e[type=item,sort=nearest,limit=1] add x" >/dev/null
    ctl command "tick sprint 6000" >/dev/null
    n=0
    age=""
    while [ "$n" -lt 60 ]; do
        sleep 2
        n=$((n + 1))
        data="$(ctl command 'data get entity @e[type=item,tag=x,limit=1]')"
        age="$(printf '%s' "$data" | sed -n 's/.*Age: \(-\{0,1\}[0-9]*\)s.*/\1/p')"
        [ -n "$age" ] && [ "$age" -lt 0 ] && break
    done
    if printf '%s' "$data" | grep -q 'bmh:eternal_sword' && [ -n "$age" ] && [ "$age" -lt 0 ]; then
        pass p15 "E10: after tick sprint 6000 the sword is still there and its Age went back to $age"
    else
        fail p15 "E10 after 6000 ticks: $(printf '%s' "$data" | head -c 300)"
    fi
    ctl command 'kill @e[type=item]' >/dev/null

    # E12: the bundle.
    ctl command "time set 6000" >/dev/null
    ctl give dirt 4 >/dev/null
    ctl give ender_chest 1 >/dev/null
    ctl wait 10 >/dev/null
    bslot="$(inv_slot bmh:eternal_bundle)"
    eslot="$(inv_slot minecraft:ender_chest)"
    ctl open-inventory >/dev/null
    ctl wait 5 >/dev/null
    ctl click-slot "$bslot" >/dev/null
    ctl click-slot "$eslot" >/dev/null
    ctl click-slot "$bslot" >/dev/null
    ctl wait 40 >/dev/null
    ctl close >/dev/null
    if [ "$(inv_count minecraft:ender_chest)" = 1 ] && ! printf '%s' "$(inv_comp bmh:eternal_bundle)" | grep -q 'ender_chest'; then
        pass p16 "E12: an Ender Chest put into the bundle comes out again within 40 ticks"
    else
        fail p16 "E12: ender chests carried $(inv_count minecraft:ender_chest); the bundle holds $(inv_comp bmh:eternal_bundle)"
    fi
    ctl command 'kill @e[type=item]' >/dev/null
    ctl select bmh:eternal_bundle >/dev/null
    ctl use-item >/dev/null
    ctl wait 15 >/dev/null
    ent="$(ctl entities 8 item)"
    if printf '%s' "$ent" | grep -q 'minecraft:stone' && ! printf '%s' "$(inv_comp bmh:eternal_bundle)" | grep -q 'stone'; then
        pass p17 "E12: a right click with a loaded bundle drops its stacks on the ground and empties it"
    else
        fail p17 "E12: items near after the click: $(printf '%s' "$ent" | head -c 300); bundle: $(inv_comp bmh:eternal_bundle)"
    fi
    ctl command 'kill @e[type=item]' >/dev/null
    ctl use-item >/dev/null
    ctl wait 15 >/dev/null
    if [ "$(jget "$(ctl entities 8 item)" 'len(d["entities"])')" = 0 ]; then
        pass p18 "E12: a right click with the empty bundle drops nothing"
    else
        fail p18 "E12: the empty bundle dropped something: $(ctl entities 8 item | head -c 300)"
    fi

    # C13: the wake-up reward.
    ctl command "clear @s minecraft:ender_chest" >/dev/null
    ctl command "clear @s minecraft:dirt" >/dev/null
    platform
    ctl command "effect clear @s" >/dev/null
    ctl command "time set 13000" >/dev/null
    ctl command "damage @s 4" >/dev/null
    ctl wait 5 >/dev/null
    h0="$(health_of)"
    sleep_through $((PX + 2)) "$PY" "$PZ"
    eff="$(ctl effects)"
    h1="$(health_of)"
    if printf '%s' "$eff" | grep -q '"id":"haste"' && printf '%s' "$eff" | grep -q '"id":"resistance"' \
        && [ "$(jget "$eff" '[e["amplifier"] for e in d["effects"]["active"] if e["id"] in ("haste","resistance")]')" = "[1, 1]" ] \
        && [ "$(python3 -c 'import sys; print(float(sys.argv[2]) > float(sys.argv[1]))' "${h0:-0}" "${h1:-0}")" = True ]; then
        pass p19 "C13: after the sleep health went $h0 to $h1 and Haste II and Resistance II are on"
    else
        fail p19 "C13: health $h0 then $h1; effects: $(printf '%s' "$eff" | head -c 300)"
    fi
    ctl command "bmh rule food_and_health_regen_on_wake_up false" >/dev/null
    ctl command "effect clear @s" >/dev/null
    ctl command "time set 13000" >/dev/null
    ctl command "tp @s $(center "$PX") $PY $(center "$PZ")" >/dev/null
    ctl wait 20 >/dev/null
    ctl look -90 35 >/dev/null
    ctl wait 5 >/dev/null
    sleep_through $((PX + 2)) "$PY" "$PZ"
    eff="$(ctl effects)"
    dt="$(daytime)"
    slept="$(python3 -c 'import sys; print(int(sys.argv[1]) % 24000 < 5000)' "${dt:-13000}")"
    if [ "$(jget "$eff" 'len(d["effects"]["active"])')" = 0 ] && [ "$slept" = True ]; then
        pass p20 "C13: with food_and_health_regen_on_wake_up false the night was slept (day time $dt) and no effect is on"
    else
        fail p20 "C13 rule off: day time $dt, effects after the sleep: $(printf '%s' "$eff" | head -c 300)"
    fi
    ctl command "bmh rule food_and_health_regen_on_wake_up true" >/dev/null
    ctl command "time set 6000" >/dev/null
    ctl command "effect clear @s" >/dev/null
    ctl command "clear @s" >/dev/null
    ctl command "difficulty normal" >/dev/null
    # Sleeping in the bed moved the respawn point there. The later scenarios (m, j) measure the exile from the point
    # that ensure_point set, so put a point at the player again.
    POINT_SET=0
    ensure_point
    ctl wait 20 >/dev/null
fi

# ---- q. The Navigator's Amulet: the reading, the refusals, the points -----------------------------------------
# amulet_try LABEL: wait out the cooldown, use the amulet, print the action bar.
amulet_try() {
    ctl wait 50 >/dev/null
    ctl use-item >/dev/null
    ctl wait 4 >/dev/null
    jget "$(ctl messages 1)" 'd["actionbar"]'
}
if wanted q; then
    echo "--- q. amulet: star reading, refusals, charges, naming screen, points across death and restart, server checks"
    ensure_point
    ctl command "difficulty peaceful" >/dev/null
    ctl command "gamemode survival" >/dev/null
    ctl command "clear @s" >/dev/null
    ctl give bmh:navigator_amulet 1 >/dev/null
    ctl wait 10 >/dev/null
    ctl select bmh:navigator_amulet >/dev/null
    ctl look 0 -20 >/dev/null
    open_sky
    ctl command "weather clear" >/dev/null
    ctl command "time set 18000" >/dev/null
    ctl wait 40 >/dev/null
    reads_before="$(count_log 'Star reading for')"
    bar="$(amulet_try)"
    comp="$(inv_comp bmh:navigator_amulet)"
    if printf '%s' "$bar" | grep -Eq '^X -?[0-9]+ .+ Y -?[0-9]+ .+ Z -?[0-9]+$' && [ "$(count_log 'Star reading for')" -gt "$reads_before" ] \
        && printf '%s' "$comp" | grep -q 'bmh_starlight:2' && printf '%s' "$comp" | grep -q 'bmh_reading'; then
        pass q1 "night, clear, open sky: action bar '$bar', log line, charges 2 left, the reading on the stack"
    else
        fail q1 "the star reading: action bar '$bar', log lines +$(( $(count_log 'Star reading for') - reads_before )), stack $comp"
    fi
    rv="$(ctl command 'advancement revoke @s only bmh:navigation/stargazer')"
    if printf '%s' "$rv" | grep -qi 'revoked'; then
        pass q2 "the advancement navigation/stargazer was granted by the reading"
    else
        fail q2 "no advancement navigation/stargazer: $(printf '%s' "$rv" | head -c 200)"
    fi
    # The sky refusals, in the order of the Java code. The new amulet has charges, so each message is the refusal.
    ctl command "weather rain" >/dev/null
    ctl wait 60 >/dev/null
    bar="$(amulet_try)"
    case "$bar" in *clouds*) pass q3 "rain: '$bar'" ;; *) fail q3 "rain: action bar '$bar'" ;; esac
    ctl command "weather clear" >/dev/null
    ctl wait 160 >/dev/null
    ctl command "time set 6000" >/dev/null
    bar="$(amulet_try)"
    case "$bar" in *"wait for night"*) pass q4 "day: '$bar'" ;; *) fail q4 "day: action bar '$bar'" ;; esac
    ctl command "time set 18000" >/dev/null
    ctl command "effect give @s blindness 60 0 true" >/dev/null
    bar="$(amulet_try)"
    ctl command "effect clear @s" >/dev/null
    case "$bar" in *"open sky"*) pass q5 "blindness: '$bar'" ;; *) fail q5 "blindness: action bar '$bar'" ;; esac
    ctl command "fill $((PX - 3)) $((PY + 4)) $((PZ - 3)) $((PX + 3)) $((PY + 4)) $((PZ + 3)) minecraft:stone" >/dev/null
    ctl wait 60 >/dev/null
    bar="$(amulet_try)"
    ctl command "fill $((PX - 3)) $((PY + 4)) $((PZ - 3)) $((PX + 3)) $((PY + 4)) $((PZ + 3)) minecraft:air" >/dev/null
    case "$bar" in *"open sky"*) pass q6 "under a roof: '$bar'" ;; *) fail q6 "under a roof: action bar '$bar'" ;; esac
    ctl wait 60 >/dev/null
    # Charges: one is left. Read it, then the next use says there is no starlight; the next night refills.
    # The Java refill: a night index that differs from the stack's refills it. q4 and q5 changed the time of day
    # (day, then night again), so the amulet is full here: three readings, then the message.
    reads_q7=0
    bar2=""
    for try in 1 2 3 4 5; do
        bar2="$(amulet_try)"
        case "$bar2" in *"No starlight"*) break ;; esac
        reads_q7=$((reads_q7 + 1))
    done
    case "$bar2" in
        *"No starlight"*)
            if [ "$reads_q7" -eq 3 ]; then
                pass q7 "three readings work, then the amulet says: '$bar2'"
            else
                fail q7 "$reads_q7 readings before '$bar2' (expected 3, one for each charge)"
            fi ;;
        *) fail q7 "after $reads_q7 readings the amulet never said that the starlight is gone: '$bar2'" ;;
    esac
    ctl command "time set 42000" >/dev/null
    ctl wait 40 >/dev/null
    if printf '%s' "$(inv_comp bmh:navigator_amulet)" | grep -q 'bmh_starlight:3'; then
        pass q8 "the charges are full again on the next night"
    else
        fail q8 "the charges after the next night: $(inv_comp bmh:navigator_amulet)"
    fi
    # The naming screens.
    platform
    ctl select bmh:navigator_amulet >/dev/null
    ctl key shift down >/dev/null
    ctl wait 5 >/dev/null
    ctl use-item >/dev/null
    ctl wait 15 >/dev/null
    if [ "$(jget "$(state)" 'd.get("screen")')" = mod ]; then
        pass q9 "sneak and use on a bed opens the mod screen (naming screen: kept screenshot amulet-shots/naming-screen-bed.png)"
    else
        fail q9 "sneak and use on a bed: screen '$(jget "$(state)" 'd.get("screen")')'"
    fi
    ctl type "Camp" >/dev/null
    ctl wait 5 >/dev/null
    ctl key enter tap >/dev/null
    ctl wait 20 >/dev/null
    bar="$(jget "$(ctl messages 1)" 'd["actionbar"]')"
    case "$bar" in *"Point saved: Camp"*) pass q10 "Save: '$bar'" ;; *) fail q10 "Save: action bar '$bar', screen '$(jget "$(state)" 'd.get("screen")')'" ;; esac
    ctl key shift up >/dev/null
    ctl command "setblock $((PX + 2)) $PY $((PZ + 1)) minecraft:lodestone" >/dev/null
    ctl wait 5 >/dev/null
    for pitch in 8 4 12 16 20; do
        ctl look -63 "$pitch" >/dev/null
        ctl wait 3 >/dev/null
        [ "$(jget "$(ctl looking-at)" 'd["block"]["block"] if d.get("block") else None')" = minecraft:lodestone ] && break
    done
    ctl key shift down >/dev/null
    ctl wait 10 >/dev/null
    ctl use-item >/dev/null
    ctl wait 15 >/dev/null
    lode_screen="$(jget "$(state)" 'd.get("screen")')"
    ctl key esc tap >/dev/null
    ctl key shift up >/dev/null
    ctl wait 10 >/dev/null
    if [ "$lode_screen" = mod ]; then
        pass q11 "sneak and use on a lodestone opens the mod screen (amulet-shots/naming-screen-lodestone.png)"
    else
        fail q11 "sneak and use on a lodestone (looking at '$(jget "$(ctl looking-at)" 'd["block"]["block"] if d.get("block") else None')'): screen '$lode_screen'"
    fi
    # Server checks by typed command (the screens send the same request).
    bar_a="$(jget "$(ctl messages 1)" 'd["actionbar"]')"
    ctl command "bmh point save $((PX + 2)) $PY $PZ One" >/dev/null
    ctl command "bmh point save $((PX + 2)) $PY $PZ Two" >/dev/null
    ctl wait 3 >/dev/null
    bar_b="$(jget "$(ctl messages 1)" 'd["actionbar"]')"
    case "$bar_b" in
        *"renamed: One"*) pass q12 "two changes inside one second: only the first took effect ('$bar_b')" ;;
        *) fail q12 "two changes inside one second: action bar '$bar_b' (expected 'Point renamed: One')" ;;
    esac
    ctl wait 30 >/dev/null
    ctl command "tp @s $(center $((PX + 20))) $((PY + 5)) $(center "$PZ")" >/dev/null
    ctl wait 40 >/dev/null
    ctl command "bmh point save $((PX + 2)) $PY $PZ Far" >/dev/null
    ctl wait 3 >/dev/null
    bar_c="$(jget "$(ctl messages 1)" 'd["actionbar"]')"
    case "$bar_c" in *"Stand next to"*) pass q13 "20 blocks away: '$bar_c'" ;; *) fail q13 "20 blocks away: action bar '$bar_c'" ;; esac
    ctl command "tp @s $(center "$PX") $PY $(center "$PZ")" >/dev/null
    ctl wait 40 >/dev/null
    ctl select 9 >/dev/null
    ctl wait 30 >/dev/null
    ctl command "bmh point save $((PX + 2)) $PY $PZ Hand" >/dev/null
    ctl wait 3 >/dev/null
    bar_d="$(jget "$(ctl messages 1)" 'd["actionbar"]')"
    case "$bar_d" in *"Hold the Navigator"*) pass q14 "no amulet in hand: '$bar_d'" ;; *) fail q14 "no amulet in hand: action bar '$bar_d'" ;; esac
    ctl select bmh:navigator_amulet >/dev/null
    # The point survives a death (the amulet itself falls) and a restart: the next reading names it.
    point_read() {
        ctl command "clear @s bmh:navigator_amulet" >/dev/null
        ctl give bmh:navigator_amulet 1 >/dev/null
        ctl wait 10 >/dev/null
        ctl select bmh:navigator_amulet >/dev/null
        ctl look 0 -20 >/dev/null
        open_sky
        ctl command "time set 18000" >/dev/null
        ctl wait 40 >/dev/null
        amulet_try >/dev/null
        inv_comp bmh:navigator_amulet
    }
    ctl command "gamemode survival" >/dev/null
    if death_and_respawn ctl 1; then
        ctl wait-for screen playing --timeout 60 >/dev/null
        comp="$(point_read)"
        if printf '%s' "$comp" | grep -q 'names:\["One"'; then
            pass q15 "after a death the reading names the point (Camp, renamed One by q12): $comp"
        else
            fail q15 "after a death the reading does not name the point One: $comp"
        fi
    else
        fail q15 "no respawn: $DR_NOTE"
    fi
    ctl save >/dev/null
    ctl quit >/dev/null
    n=0
    while [ -n "$GAMEPID" ] && kill -0 "$GAMEPID" 2>/dev/null && [ "$n" -lt 60 ]; do sleep 1; n=$((n + 1)); done
    launch_game --wait
    if [ "$launch_status" -eq 0 ]; then
        ctl command "difficulty peaceful" >/dev/null
        ctl command "gamemode survival" >/dev/null
        ctl wait-for screen playing --timeout 60 >/dev/null
        comp="$(point_read)"
        if printf '%s' "$comp" | grep -q 'names:\["One"'; then
            pass q16 "after a restart the reading names the point (Camp, renamed One by q12): $comp"
        else
            fail q16 "after a restart the reading does not name the point One: $comp"
        fi
    else
        fail q16 "the second launch failed (status $launch_status)"
    fi
    ctl command "clear @s" >/dev/null
    ctl command "time set 6000" >/dev/null
    ctl command "difficulty normal" >/dev/null
fi

# ---- t. The totem area: the save, the dead flag, the Totem of Undying order, the extra totem, the anvil, E6, /kill ----
# Game checks 3 (2026-11-18). t1 to t5 prove or refute the fault of difference 12 of docs/horizon/log/totem.md (a
# canceled living_death leaves the dead flag of the body set). Difficulty is peaceful in t, so that no mob interferes.
# T_TYPE overrides the damage type of the lethal hit (default minecraft:generic).
t_counts() { jget "$(ctl inventory)" 'd["counts"].get("'"$1"'", 0)'; }
if wanted t; then
    echo "--- t. totem: Undying order, extra totem, anvil, E6, the save on a lethal hit, the dead flag, /kill"
    ensure_point
    ctl command "difficulty peaceful" >/dev/null
    ctl command "gamemode survival" >/dev/null
    ctl command "clear @s" >/dev/null
    ctl command "effect clear @s" >/dev/null
    ctl command "time set 6000" >/dev/null
    # t6: a Totem of Undying in the hand saves first; the Totem of Eternity stays.
    ctl command "give @s minecraft:totem_of_undying" >/dev/null
    ctl command "give @s bmh:totem_of_eternity" >/dev/null
    ctl command "select 1" >/dev/null
    ctl wait 10 >/dev/null
    ctl command "damage @s 100 minecraft:generic" >/dev/null
    ctl wait 20 >/dev/null
    T_EFF="$(ctl effects)"
    echo "t-evidence t6: health $(health_of), counts $(jget "$(ctl inventory)" 'd["counts"]'), effects $(jget "$T_EFF" '[e["id"] for e in d["effects"]["active"]]')"
    if [ "$(t_counts minecraft:totem_of_undying)" = 0 ] && [ "$(t_counts bmh:totem_of_eternity)" = 1 ] \
        && printf '%s' "$T_EFF" | grep -q regeneration && ! printf '%s' "$T_EFF" | grep -q fire_resistance; then
        pass t6 "the Totem of Undying saved first, the Totem of Eternity stayed, no Fire Resistance"
    else
        fail t6 "Undying order: counts $(jget "$(ctl inventory)" 'd["counts"]'), effects $(jget "$T_EFF" '[e["id"] for e in d["effects"]["active"]]'), health $(health_of)"
    fi
    # t7: a second Totem of Eternity drops at once.
    ctl command "effect clear @s" >/dev/null
    ctl command "clear @s" >/dev/null
    ctl command "give @s bmh:totem_of_eternity 2" >/dev/null
    ctl wait 30 >/dev/null
    t_ent="$(ctl entities 8 item)"
    if [ "$(t_counts bmh:totem_of_eternity)" = 1 ] && printf '%s' "$t_ent" | grep -q totem_of_eternity; then
        pass t7 "of two Totems of Eternity one stayed and one dropped on the ground"
    else
        fail t7 "the extra totem: counts $(jget "$(ctl inventory)" 'd["counts"]'), items near: $(printf '%s' "$t_ent" | head -c 300)"
    fi
    # t8: the anvil awakening (Hollow Totem + Echo Heart), then E6 (gold blocks on an Eternal tool).
    ctl command "kill @e[type=item]" >/dev/null
    ctl command "clear @s" >/dev/null
    platform
    ctl command "setblock $((PX + 2)) $PY $PZ minecraft:anvil" >/dev/null
    ctl wait 20 >/dev/null
    ctl command "kill @e[type=item]" >/dev/null
    ctl command "clear @s" >/dev/null
    ctl command "xp 45" >/dev/null
    ctl command "give @s bmh:dormant_totem" >/dev/null
    ctl command "give @s bmh:echo_heart" >/dev/null
    ctl wait 10 >/dev/null
    ctl interact $((PX + 2)) "$PY" "$PZ" >/dev/null
    ctl wait 10 >/dev/null
    ctl click-slot "$(inv_slot bmh:dormant_totem)" shift >/dev/null
    ctl click-slot "$(inv_slot bmh:echo_heart)" shift >/dev/null
    ctl wait 10 >/dev/null
    T_MENU="$(ctl menu)"
    echo "t-evidence t8: menu $(printf '%s' "$T_MENU" | head -c 900)"
    if printf '%s' "$T_MENU" | grep -q 'bmh:totem_of_eternity'; then
        pass t8 "the anvil shows a Totem of Eternity as result"
    else
        fail t8 "no Totem of Eternity result in the anvil menu"
    fi
    ctl close >/dev/null
    # t9 (E6): damage 20000, repair_cost 40, two gold blocks: expect damage 3500, cost 15 levels, penalty 40.
    ctl command "clear @s" >/dev/null
    ctl command "give @s bmh:eternal_sword[repair_cost=40,damage=20000]" >/dev/null
    ctl command "give @s minecraft:gold_block 2" >/dev/null
    ctl wait 10 >/dev/null
    ctl interact $((PX + 2)) "$PY" "$PZ" >/dev/null
    ctl wait 10 >/dev/null
    ctl click-slot "$(inv_slot bmh:eternal_sword)" shift >/dev/null
    ctl click-slot "$(inv_slot minecraft:gold_block)" shift >/dev/null
    ctl wait 10 >/dev/null
    T_MENU="$(ctl menu)"
    echo "t-evidence t9: menu $(printf '%s' "$T_MENU" | head -c 1400)"
    if printf '%s' "$T_MENU" | grep -q '3500' && printf '%s' "$T_MENU" | grep -q '15'; then
        pass t9 "E6: the anvil result has damage 3500 and cost 15"
    else
        fail t9 "E6: the anvil result does not show damage 3500 and cost 15 (see the t-evidence line)"
    fi
    ctl close >/dev/null
    ctl command "clear @s" >/dev/null
    ctl command "setblock $((PX + 2)) $PY $PZ minecraft:air" >/dev/null
    # t1 to t5: the lethal hit with only a Totem of Eternity.
    ctl command "effect clear @s" >/dev/null
    ctl command "give @s bmh:totem_of_eternity" >/dev/null
    ctl wait 10 >/dev/null
    t_before_effects="$(count_log 'The game refused the save effects')"
    t_before_health="$(count_log 'The game refused the save health')"
    t_before_saved="$(count_log 'saved .* and is now dormant')"
    ctl command "damage @s 100 ${T_TYPE:-minecraft:generic}" >/dev/null
    ctl wait 20 >/dev/null
    T_STATE="$(state)"
    T_EFF="$(ctl effects)"
    T_INV="$(ctl inventory)"
    T_H1="$(health_of)"
    echo "t-evidence: health after the hit: $T_H1; screen: $(jget "$T_STATE" 'd.get("screen")')"
    echo "t-evidence: effects: $(printf '%s' "$T_EFF" | head -c 600)"
    echo "t-evidence: counts: $(jget "$T_INV" 'd["counts"]')"
    echo "t-evidence: log lines: refused effects $(( $(count_log 'The game refused the save effects') - t_before_effects )), refused health $(( $(count_log 'The game refused the save health') - t_before_health )), saved $(( $(count_log 'saved .* and is now dormant') - t_before_saved ))"
    if [ "$(jget "$T_STATE" 'd.get("screen")')" != dead ] && python3 -c 'import sys; sys.exit(0 if float(sys.argv[1]) > 0 else 1)' "${T_H1:-0}"; then
        pass t1 "the player lives after a lethal hit with a Totem of Eternity: health $T_H1"
    else
        fail t1 "the player does not live: health $T_H1, screen $(jget "$T_STATE" 'd.get("screen")')"
    fi
    if [ "$(jget "$T_INV" 'd["counts"].get("bmh:sleeping_totem", 0)')" = 1 ] && [ "$(jget "$T_INV" 'd["counts"].get("bmh:totem_of_eternity", 0)')" = 0 ]; then
        pass t2 "the totem became a Dormant Totem"
    else
        fail t2 "the totem did not become a Dormant Totem: $(jget "$T_INV" 'd["counts"]')"
    fi
    if [ "$(jget "$T_EFF" 'sorted(e["id"] for e in d["effects"]["active"] if e["id"] in ("regeneration","absorption","fire_resistance"))')" = "['absorption', 'fire_resistance', 'regeneration']" ]; then
        pass t3 "Regeneration, Absorption and Fire Resistance are on the player"
    else
        fail t3 "the three save effects are not all on the player: $(jget "$T_EFF" '[(e["id"], e["amplifier"]) for e in d["effects"]["active"]]')"
    fi
    if [ "$(count_log 'saved .* and is now dormant')" -gt "$t_before_saved" ]; then
        pass t4 "the log has the 'saved ... and is now dormant' line"
    else
        fail t4 "the log has no 'saved ... and is now dormant' line (refused effects: $(( $(count_log 'The game refused the save effects') - t_before_effects )), refused health: $(( $(count_log 'The game refused the save health') - t_before_health )))"
    fi
    ctl command "effect clear @s" >/dev/null
    ctl wait 5 >/dev/null
    t_h2="$(health_of)"
    ctl command "damage @s 1 minecraft:generic" >/dev/null
    ctl wait 10 >/dev/null
    t_h3="$(health_of)"
    echo "t-evidence: health before /damage @s 1: $t_h2, after: $t_h3"
    if python3 -c 'import sys; a, b = float(sys.argv[1]), float(sys.argv[2]); sys.exit(0 if a > 0 and b < a else 1)' "${t_h2:-0}" "${t_h3:-0}"; then
        pass t5 "a later /damage @s 1 lowers the health: $t_h2 then $t_h3"
    else
        fail t5 "no live player takes a later /damage @s 1: health $t_h2 then $t_h3"
    fi
    # t10: /kill kills a player with a Totem of Eternity; the totem is not spent.
    if [ "$(jget "$(state)" 'd.get("screen")')" = dead ]; then ctl respawn >/dev/null; ctl wait 40 >/dev/null; fi
    ctl command "clear @s" >/dev/null
    ctl command "give @s bmh:totem_of_eternity" >/dev/null
    ctl wait 10 >/dev/null
    ctl command "kill @s" >/dev/null
    ctl wait 20 >/dev/null
    if [ "$(jget "$(state)" 'd.get("screen")')" = dead ]; then
        pass t10 "/kill kills a player who carries a Totem of Eternity"
    else
        fail t10 "/kill did not kill: screen $(jget "$(state)" 'd.get("screen")'), health $(health_of)"
    fi
    ctl respawn >/dev/null
    ctl wait 40 >/dev/null
    ctl command "difficulty normal" >/dev/null
fi

# ---- u. The homecoming: the record, the title, the awards, the rule -------------------------------------
# Game checks 3 (2026-11-18). After a distant respawn the player walks back (here: teleports) to the exile
# center, which is the respawn point he left. Reads the log, the title of `messages` and the log's goal lines.
if wanted u; then
    echo "--- u. homecoming: the record, the title and subtitle, the awards, the rule display_text_on_respawn false"
    ensure_point
    ctl command "difficulty peaceful" >/dev/null
    ctl command "gamemode survival" >/dev/null
    ctl command "bmh rule display_text_on_respawn true" >/dev/null
    ctl command "advancement revoke @s everything" >/dev/null
    ctl command "spawnpoint @s" >/dev/null
    u_pos="$(position_of)"
    u_x="$(echo "$u_pos" | cut -d ' ' -f 1)"; u_y="$(echo "$u_pos" | cut -d ' ' -f 2)"; u_z="$(echo "$u_pos" | cut -d ' ' -f 3)"
    # The point moved to the player: the later scenarios measure from it (m2 read the old point and failed).
    spawn_x="$u_x"
    spawn_z="$u_z"
    u_rec="$(count_log 'Homecoming record')"
    if death_and_respawn ctl 1; then
        if [ "$(count_log 'Homecoming record')" -gt "$u_rec" ]; then
            pass u1 "a record was kept at the exile: $(grep 'Homecoming record' "$(logfile)" | tail -n 1 | sed 's/^[^]]*\]: //')"
        else
            fail u1 "no 'Homecoming record' line after the distant respawn"
        fi
        u_home="$(count_log 'is home after')"
        ctl command "tp @s $u_x $u_y $u_z" >/dev/null
        wait_log 'is home after' "$u_home" 10
        ctl wait 10 >/dev/null
        u_msg="$(ctl messages)"
        u_adv="$(ctl command "advancement revoke @s only bmh:journey/home_at_last")"
        echo "t-evidence u2: title $(jget "$u_msg" 'd.get("title")') subtitle $(jget "$u_msg" 'd.get("subtitle")'); advancement: $(jget "$u_adv" 'd["chat"]'); log: $(grep 'is home after' "$(logfile)" | tail -n 1 | sed 's/^[^]]*\]: //')"
        if printf '%s' "$(jget "$u_msg" 'd.get("title")')" | grep -q 'Home' && printf '%s' "$(jget "$u_msg" 'd.get("subtitle")')" | grep -qi 'blocks'; then
            pass u2 "title Home and a subtitle of blocks shown: $(jget "$u_msg" 'd.get("subtitle")')"
        else
            fail u2 "no Home title with a subtitle of blocks"
        fi
        if printf '%s' "$u_adv" | grep -qi 'revoked'; then
            pass u3 "advancement Home at last awarded"
        else
            fail u3 "advancement Home at last not awarded: $(jget "$u_adv" 'd["chat"]')"
        fi
        # The rule off: a new exile, the walk back gives no title, the awards still come.
        ctl command "bmh rule display_text_on_respawn false" >/dev/null
        ctl command "advancement revoke @s everything" >/dev/null
        ctl command "tp @s $u_x $u_y $u_z" >/dev/null
        if death_and_respawn ctl 1; then
            u_home="$(count_log 'is home after')"
            ctl command "tp @s $u_x $u_y $u_z" >/dev/null
            wait_log 'is home after' "$u_home" 10
            ctl wait 10 >/dev/null
            u_msg="$(ctl messages)"
            u_adv="$(ctl command "advancement revoke @s only bmh:journey/home_at_last")"
            echo "t-evidence u4: title $(jget "$u_msg" 'd.get("title")') subtitle $(jget "$u_msg" 'd.get("subtitle")'); advancement: $(jget "$u_adv" 'd["chat"]')"
            if [ "$(count_log 'is home after')" -gt "$u_home" ] && [ -z "$(jget "$u_msg" 'd.get("title")')" -o "$(jget "$u_msg" 'd.get("title")')" = None ] && printf '%s' "$u_adv" | grep -qi 'revoked'; then
                pass u4 "rule false: homecoming happened, no Home title, the advancement came"
            else
                fail u4 "rule false: see the t-evidence line"
            fi
        else
            fail u4 "no second respawn: $DR_NOTE"
        fi
        ctl command "bmh rule display_text_on_respawn true" >/dev/null
        ctl command "difficulty normal" >/dev/null
    else
        fail u1 "no respawn: $DR_NOTE"
    fi
fi

# ---- r. The Eternal tools: dig speed of a worn pickaxe (E4), the last point (E2), the death wear (C12) ----
# Single player owner only. Peaceful, so nothing attacks the platform. A dig is timed by the "ticks" of
# "horizon-ctl break". It does not prove the sweep, the echo or the axe (they need hostile mobs).
r_dig() {
    # r_dig ITEM_SPEC [ENCHANT_ID LEVEL]: stone in front of the player, break it, print "ticks block damage".
    ctl command "clear @s" >/dev/null
    ctl command "give @s $1" >/dev/null
    ctl wait 5 >/dev/null
    ctl select 1 >/dev/null
    if [ -n "${2:-}" ]; then
        r_en="$(ctl enchant "$2" "$3")"
        echo "t-evidence r_enchant: $(printf '%s' "$r_en" | head -c 200); held: $(inv_comp "${1%%[*}")"
    fi
    ctl command "setblock $((PX + 1)) $PY $PZ minecraft:stone" >/dev/null
    ctl wait 5 >/dev/null
    r_rep="$(ctl break $((PX + 1)) "$PY" "$PZ")"
    r_ticks="$(jget "$r_rep" 'd.get("ticks")')"
    r_block="$(jget "$r_rep" 'd.get("block")')"
    # Vanilla waits 5 ticks after a block breaks before the next dig starts: take them off every dig but the first.
    if [ "${R_DIGS:-0}" -gt 0 ] && [ -n "$r_ticks" ]; then r_ticks=$((r_ticks - 5)); fi
    R_DIGS=$(( ${R_DIGS:-0} + 1 ))
    ctl wait 20 >/dev/null
    r_late="$(ctl block $((PX + 1)) "$PY" "$PZ" | head -c 160)"
    echo "t-evidence r_dig late: block 20 ticks later: $r_late"
    r_block="$(jget "$r_late" 'd.get("block")')"
    echo "t-evidence r_dig $1 ${2:-}: $(printf '%s' "$r_rep" | head -c 300)"
}
r_damage() { inv_comp "$1" | sed -n 's/.*damage[^0-9]*\([0-9][0-9]*\).*/\1/p' | head -n 1; }
if wanted r; then
    echo "--- r. Eternal tools: worn pickaxe dig time, Efficiency V, the last point, the death wear"
    ensure_point
    ctl command "difficulty peaceful" >/dev/null
    ctl command "gamemode survival" >/dev/null
    ctl command "effect clear @s" >/dev/null
    ctl command "time set 6000" >/dev/null
    platform
    # Calibration of the dig clock: vanilla tools, Java 8 ticks for iron (speed 6) and 23 for wood (speed 2) on stone.
    r_dig minecraft:iron_pickaxe
    echo "t-evidence r0: iron pickaxe on stone $r_ticks ticks (Java 8)"
    r_dig minecraft:wooden_pickaxe
    echo "t-evidence r0: wooden pickaxe on stone $r_ticks ticks (Java 23)"
    if [ "$r_ticks" = 23 ]; then
        pass r0 "the dig clock matches Java: a wooden pickaxe takes 23 ticks on stone (5 ticks of vanilla delay taken off)"
    else
        fail r0 "the dig clock: a wooden pickaxe takes '$r_ticks' ticks on stone, Java gives 23"
    fi
    r_dig bmh:eternal_pickaxe
    r_fresh="$r_ticks"; r_fresh_block="$r_block"
    r_dig "bmh:eternal_pickaxe[damage=32999]"
    r_worn="$r_ticks"; r_worn_block="$r_block"
    r_worn_dmg="$(r_damage bmh:eternal_pickaxe)"
    echo "t-evidence r1: fresh $r_fresh ticks, worn $r_worn ticks, worn damage after the dig $r_worn_dmg"
    if [ -n "$r_fresh" ] && [ -n "$r_worn" ] && [ "$r_worn" -gt "$r_fresh" ] && [ "$r_worn_block" = "minecraft:air" ]; then
        pass r1 "E4: a worn pickaxe digs stone slower than a fresh one ($r_worn ticks against $r_fresh), the block is gone"
    else
        fail r1 "E4: worn $r_worn ticks (block $r_worn_block), fresh $r_fresh ticks (block $r_fresh_block)"
    fi
    if [ -n "$r_worn" ] && [ "$r_worn" -ge 10 ] && [ "$r_worn" -le 16 ]; then
        pass r2 "E4: the worn dig takes $r_worn ticks, Java gives 12 (speed 4, stone, ground)"
    else
        fail r2 "E4: the worn dig takes '$r_worn' ticks, Java gives 12"
    fi
    r_dig "bmh:eternal_pickaxe[damage=32999]" efficiency 5
    r_eff="$r_ticks"
    if [ -n "$r_eff" ] && [ "$r_eff" -ge 1 ] && [ "$r_eff" -le 5 ] && [ "$r_block" = "minecraft:air" ]; then
        pass r3 "E4: worn with Efficiency V the dig takes $r_eff ticks (Java 2) and the block is still gone 20 ticks later"
    else
        fail r3 "E4: worn with Efficiency V the dig takes '$r_eff' ticks (Java 2), block 20 ticks later '$r_block'"
    fi
    # r4 (E2): from 32998, five blocks; the tool stays and its damage stays at or under 32999.
    ctl command "clear @s" >/dev/null
    ctl command "give @s bmh:eternal_pickaxe[damage=32998]" >/dev/null
    ctl wait 5 >/dev/null
    ctl select 1 >/dev/null
    r_n=0
    while [ "$r_n" -lt 5 ]; do
        ctl command "setblock $((PX + 1)) $PY $PZ minecraft:stone" >/dev/null
        ctl wait 3 >/dev/null
        ctl break $((PX + 1)) "$PY" "$PZ" >/dev/null
        r_n=$((r_n + 1))
    done
    r_after="$(r_damage bmh:eternal_pickaxe)"
    echo "t-evidence r4: damage after five blocks '$r_after', count $(t_counts bmh:eternal_pickaxe)"
    if [ "$(t_counts bmh:eternal_pickaxe)" = 1 ] && [ -n "$r_after" ] && [ "$r_after" -le 32999 ] && [ "$r_after" -ge 32998 ]; then
        pass r4 "E2: five blocks with the pickaxe at 32998: it is still there, damage $r_after"
    else
        fail r4 "E2: after five blocks the pickaxe count is $(t_counts bmh:eternal_pickaxe), damage '$r_after'"
    fi
    # r5 to r7 (C12): the 7 percent wear at a death.
    ctl command "clear @s" >/dev/null
    ctl command "give @s bmh:eternal_sword" >/dev/null
    ctl wait 5 >/dev/null
    r_d0="$(r_damage bmh:eternal_sword)"; r_d0="${r_d0:-0}"
    if death_and_respawn ctl 1; then
        ctl wait 10 >/dev/null
        r_d1="$(r_damage bmh:eternal_sword)"
        echo "t-evidence r5: sword damage before $r_d0, after the death '$r_d1'"
        if [ -n "$r_d1" ] && [ $((r_d1 - r_d0)) -eq 2310 ]; then
            pass r5 "C12: a death in survival wears the sword by 2310 (7 percent of 33000)"
        else
            fail r5 "C12: sword damage $r_d0 then '$r_d1', expected +2310"
        fi
        ctl command "bmh rule damage_eternal_item_when_dead false" >/dev/null
        if death_and_respawn ctl 1; then
            ctl wait 10 >/dev/null
            r_d2="$(r_damage bmh:eternal_sword)"
            if [ "$r_d2" = "$r_d1" ]; then
                pass r6 "C12: with the rule off a death does not wear the sword ($r_d2)"
            else
                fail r6 "C12: rule off, sword damage $r_d1 then '$r_d2'"
            fi
        else
            fail r6 "no respawn: $DR_NOTE"
        fi
        ctl command "bmh rule damage_eternal_item_when_dead true" >/dev/null
        ctl command "gamemode creative" >/dev/null
        ctl command "kill @s" >/dev/null
        ctl wait 40 >/dev/null
        ctl respawn >/dev/null 2>&1 || true
        ctl wait 40 >/dev/null
        r_d3="$(r_damage bmh:eternal_sword)"
        echo "t-evidence r7: after a /kill in creative: sword damage '$r_d3', screen $(jget "$(state)" 'd.get("screen")')"
        if [ "$r_d3" = "$r_d1" ]; then
            pass r7 "C12: a death in creative does not wear the sword ($r_d3)"
        else
            fail r7 "C12: creative death, sword damage $r_d1 then '$r_d3'"
        fi
        ctl command "gamemode survival" >/dev/null
    else
        fail r5 "no respawn: $DR_NOTE"
    fi
    ctl command "clear @s" >/dev/null
    ctl command "difficulty normal" >/dev/null
fi

# ---- s. The moon ritual: refused by day, a fail when the aim leaves the moon, a full ritual at night -------
# At time 17850 the moon is nearly straight up (angle 360 at 18000), so pitch -90 aims at it for the 300
# ticks of the ritual. The ritual log lines come from bmh_ritual.rs. The sound, the hand pose and the
# particles are never tested here (muted, no screenshot).
if wanted s; then
    echo "--- s. moon ritual: refused by day, looked away, a full ritual at night"
    ensure_point
    ctl command "difficulty peaceful" >/dev/null
    ctl command "gamemode survival" >/dev/null
    ctl command "weather clear" >/dev/null
    ctl command "effect clear @s" >/dev/null
    platform
    ctl command "clear @s" >/dev/null
    ctl command "give @s bmh:sleeping_totem" >/dev/null
    ctl wait 5 >/dev/null
    ctl select 1 >/dev/null
    # s1: by day the start is refused, no record.
    ctl command "time set 6000" >/dev/null
    ctl look 0 -90 >/dev/null
    ctl wait 10 >/dev/null
    s_started="$(count_log 'Moon ritual started')"
    ctl mouse right down >/dev/null
    ctl wait 30 >/dev/null
    s_msg="$(ctl messages)"
    ctl mouse right up >/dev/null
    echo "t-evidence s1: messages $(printf '%s' "$s_msg" | head -c 400)"
    if [ "$(count_log 'Moon ritual started')" = "$s_started" ] && [ "$(t_counts bmh:sleeping_totem)" = 1 ]; then
        pass s1 "by day the ritual does not start and the Dormant Totem stays: $(jget "$s_msg" 'd.get("actionbar")')"
    else
        fail s1 "by day: started lines $s_started then $(count_log 'Moon ritual started'), totems $(t_counts bmh:sleeping_totem)"
    fi
    # s2: at night, aimed at the moon, then looked away: it fails.
    ctl wait 30 >/dev/null
    ctl command "time set 17850" >/dev/null
    ctl command "xp set @s 0 levels" >/dev/null
    ctl command "xp add @s 40 levels" >/dev/null
    ctl look 0 -90 >/dev/null
    ctl wait 10 >/dev/null
    s_failed="$(count_log 'Moon ritual failed.*looked_away')"
    s_started="$(count_log 'Moon ritual started')"
    ctl mouse right down >/dev/null
    ctl wait 60 >/dev/null
    s_mid_started="$(count_log 'Moon ritual started')"
    s_mid_level="$(jget "$(state)" '(d["player"]["xp"]["level"] if isinstance(d["player"].get("xp"), dict) else d["player"].get("xp"))')"
    ctl look 0 0 >/dev/null
    ctl wait 30 >/dev/null
    s_msg="$(ctl messages)"
    ctl mouse right up >/dev/null
    echo "t-evidence s2: level at 60 ticks '$s_mid_level', messages $(printf '%s' "$s_msg" | head -c 300); log: $(grep 'Moon ritual' "$(logfile)" | tail -n 3 | cut -c1-200 | tr '\n' '|')"
    if [ "$s_mid_started" -gt "$s_started" ] && [ "$(count_log 'Moon ritual failed.*looked_away')" -gt "$s_failed" ] && [ "$(t_counts bmh:sleeping_totem)" = 1 ]; then
        pass s2 "night, aimed at the moon: the ritual starts (acting player answers), looking away fails it ('looked_away'), the levels drained meanwhile ($s_mid_level of 40), the Dormant Totem stays"
    else
        fail s2 "night: started lines $s_started then $s_mid_started, failed lines $s_failed then $(count_log 'Moon ritual failed'), totems $(t_counts bmh:sleeping_totem)"
    fi
    if [ "$(count_log 'started with no acting player')" = 0 ]; then
        pass s3 "native_acting_player answered in the use start: no 'no acting player' line"
    else
        fail s3 "native_acting_player gave None: $(count_log 'started with no acting player') refusal lines"
    fi
    # s4: a full ritual: 30 levels taken, a Totem of Eternity in the hand, the advancement.
    ctl wait 40 >/dev/null
    ctl command "advancement revoke @s only bmh:eternity/moon_ritual" >/dev/null
    ctl command "time set 17850" >/dev/null
    ctl command "xp set @s 0 levels" >/dev/null
    ctl command "xp add @s 40 levels" >/dev/null
    ctl look 0 -90 >/dev/null
    ctl wait 10 >/dev/null
    s_done="$(count_log 'Moon ritual done')"
    ctl mouse right down >/dev/null
    ctl wait 340 >/dev/null
    ctl mouse right up >/dev/null
    ctl wait 10 >/dev/null
    s_level="$(jget "$(state)" '(d["player"]["xp"]["level"] if isinstance(d["player"].get("xp"), dict) else d["player"].get("xp"))')"
    s_adv="$(ctl command "advancement revoke @s only bmh:eternity/moon_ritual")"
    echo "t-evidence s4 xp raw: $(jget "$(state)" 'd["player"].get("xp")')"
    echo "t-evidence s4: level after '$s_level', counts $(jget "$(ctl inventory)" 'd["counts"]'), advancement: $(jget "$s_adv" 'd["chat"]'), log: $(grep 'Moon ritual' "$(logfile)" | tail -n 2 | cut -c1-200 | tr '\n' '|')"
    if [ "$(count_log 'Moon ritual done')" -gt "$s_done" ] && [ "$(t_counts bmh:totem_of_eternity)" = 1 ] && [ "$(t_counts bmh:sleeping_totem)" = 0 ]; then
        pass s4 "the full ritual: the Dormant Totem became a Totem of Eternity, log line 'Moon ritual done'"
    else
        fail s4 "the full ritual: done lines $s_done then $(count_log 'Moon ritual done'), counts $(jget "$(ctl inventory)" 'd["counts"]')"
    fi
    if [ "$s_level" = 10 ]; then
        pass s5 "the ritual took 30 levels: 40 became 10"
    else
        fail s5 "the levels after the ritual: '$s_level', expected 10 (from 40)"
    fi
    if printf '%s' "$s_adv" | grep -qi 'revoked'; then
        pass s6 "the advancement Moon-touched was held after the ritual"
    else
        fail s6 "the advancement eternity/moon_ritual was not held: $(printf '%s' "$s_adv" | head -c 200)"
    fi
    ctl command "clear @s" >/dev/null
    ctl command "difficulty normal" >/dev/null
fi

# ---- v. The sky: a star reading draws (screenshots a person must look at), no "does not draw" line ---------
# The log lines are the facts; the screenshots go to docs/horizon/log/sky-shots/ (scaled to 1280 px) and a
# person must look at them: only a person can say that the chain is over the stars and that terrain hides it.
if wanted v; then
    echo "--- v. sky: a star reading, the log lines, screenshots, the spyglass name"
    ensure_point
    skyshots="$repo/docs/horizon/log/sky-shots"
    mkdir -p "$skyshots"
    ctl command "difficulty peaceful" >/dev/null
    ctl command "gamemode survival" >/dev/null
    ctl command "weather clear" >/dev/null
    ctl command "clear @s" >/dev/null
    platform
    open_sky
    ctl command "time set 18000" >/dev/null
    ctl give bmh:navigator_amulet 1 >/dev/null
    ctl give minecraft:spyglass 1 >/dev/null
    ctl wait 10 >/dev/null
    ctl select bmh:navigator_amulet >/dev/null
    ctl look 0 -20 >/dev/null
    ctl wait 40 >/dev/null
    v_point="$(ctl command "bmh point save $((PX + 2)) $PY $PZ Camp")"
    echo "t-evidence v: point save: $(printf '%s' "$v_point" | head -c 200)"
    # A stone slab 60 blocks up: from there trees and hills do not hide the low stars of the chain.
    ctl command "fill $((PX - 1)) $((PY + 59)) $((PZ - 1)) $((PX + 1)) $((PY + 59)) $((PZ + 1)) minecraft:stone" >/dev/null
    ctl command "tp @s $(center "$PX") $((PY + 60)) $(center "$PZ")" >/dev/null
    ctl wait 70 >/dev/null
    v_before="$(count_log 'Star reading toward')"
    ctl use-item >/dev/null
    ctl wait 20 >/dev/null
    v_lines="$(grep 'Star reading toward' "$(logfile)" | tail -n "$(( $(count_log 'Star reading toward') - v_before ))" 2>/dev/null)"
    echo "t-evidence v: reading lines: $(printf '%s' "$v_lines" | cut -c1-200 | tr '\n' '|')"
    if [ "$(count_log 'Star reading toward')" -gt "$v_before" ]; then
        pass v1 "the sky area started the reading: $(printf '%s' "$v_lines" | wc -l | tr -d ' ') 'Star reading toward' lines"
    else
        fail v1 "no 'Star reading toward' line after the amulet reading"
    fi
    if [ "$(count_log 'The star reading does not draw')" = 0 ]; then
        pass v2 "no 'The star reading does not draw in this session' line (the pipeline built)"
    else
        fail v2 "the star reading does not draw: $(grep 'The star reading does not draw' "$(logfile)" | head -n 1 | cut -c1-300)"
    fi
    if [ "$(count_log 'Sky azimuth self-check')" = 1 ] && grep 'Sky azimuth self-check' "$(logfile)" | grep -q 'azimuth 0 aims at (0, -100), azimuth 90 at (100, 0); expected (0, -100) and (100, 0)'; then
        pass v3 "the azimuth self-check line is in the log once, with the expected values"
    else
        fail v3 "the azimuth self-check line: $(grep 'Sky azimuth self-check' "$(logfile)" | head -n 2 | cut -c1-250)"
    fi
    # The camp line is the first non-spawn line; take the first line of the reading for the view.
    v_first="$(printf '%s' "$v_lines" | head -n 1)"
    v_az="$(printf '%s' "$v_first" | sed -n 's/.*Star reading toward \(-\{0,1\}[0-9.]*\):.*/\1/p')"
    v_lastaz="$(printf '%s' "$v_first" | sed -n 's/.*last at azimuth \(-\{0,1\}[0-9.]*\),.*/\1/p')"
    v_alt="$(printf '%s' "$v_first" | sed -n 's/.*altitude \(-\{0,1\}[0-9.]*\).*/\1/p')"
    echo "t-evidence v: first line azimuth '$v_az', last star azimuth '$v_lastaz', altitude '$v_alt'"
    if [ -n "$v_az" ] && [ -n "$v_alt" ]; then
        v_yaw="$(python3 -c 'import sys; print((float(sys.argv[1]) + 180) % 360)' "$v_az")"
        v_pitch="$(python3 -c 'import sys; print(-float(sys.argv[1]) / 2 - 10)' "$v_alt")"
        ctl look "$v_yaw" "$v_pitch" >/dev/null
        ctl wait 40 >/dev/null
        ctl screenshot "$skyshots/v4-reading-toward-point.png" >/dev/null
        # V2: through a spyglass, aim at the last star of the first line: its name shows under the centre. The 1.8 degrees
        # of extra pitch were measured on a screenshot: the spyglass view put that star 190 px above the centre at the log's altitude.
        ctl select minecraft:spyglass >/dev/null
        ctl look "$(python3 -c 'import sys; print((float(sys.argv[1]) + 180) % 360)' "$v_lastaz")" "$(python3 -c 'import sys; print(-float(sys.argv[1]) - 1.8)' "$v_alt")" >/dev/null
        ctl mouse right down >/dev/null
        ctl wait 25 >/dev/null
        ctl screenshot "$skyshots/v5-spyglass-on-last-star-early.png" >/dev/null
        ctl wait 55 >/dev/null
        ctl screenshot "$skyshots/v5-spyglass-on-last-star.png" >/dev/null
        ctl mouse right up >/dev/null
        ctl select bmh:navigator_amulet >/dev/null
        [ -s "$skyshots/v5-spyglass-on-last-star.png" ] && sips -Z 1280 "$skyshots/v5-spyglass-on-last-star.png" >/dev/null 2>&1
        ctl select bmh:navigator_amulet >/dev/null
        ctl look "$v_yaw" "$v_pitch" >/dev/null
        ctl wait 60 >/dev/null
        ctl screenshot "$skyshots/v4-reading-toward-point-later.png" >/dev/null
        ctl look 0 0 >/dev/null
        ctl wait 20 >/dev/null
        ctl screenshot "$skyshots/v4-looking-at-horizon.png" >/dev/null
        for f in "$skyshots/v4-reading-toward-point.png" "$skyshots/v4-reading-toward-point-later.png" "$skyshots/v4-looking-at-horizon.png"; do
            [ -s "$f" ] && sips -Z 1280 "$f" >/dev/null 2>&1
        done
        if [ -s "$skyshots/v4-reading-toward-point.png" ]; then
            pass v4 "screenshots written to docs/horizon/log/sky-shots/ (a person must look at them)"
        else
            fail v4 "no screenshot"
        fi
        if [ -s "$skyshots/v5-spyglass-on-last-star.png" ]; then
            pass v5 "spyglass screenshot written (a person must look for the name 'Camp' under the centre)"
        else
            fail v5 "no spyglass screenshot"
        fi
    else
        fail v4 "no azimuth or altitude to aim the view at"
    fi
    ctl command "clear @s" >/dev/null
    ctl command "difficulty normal" >/dev/null
fi

# ---- w. The journey after fix round 1: immediate respawn, the arrival frame, a stale arm, a screen over the death screen, a press before the departure ----
# series_probe MODE DIR [BUTTON]: take a screenshot and the state about every 0.1 s and tell what the screens did. MODE:
#   immediate  the capture starts, then "kill @s" (the game rule immediate_respawn is on, there is no death screen)
#   arrival    "kill @s", wait for the death screen and the mod's "Landing spot ready" line, start the capture, then "respawn"
#   click      the player is dead: start the capture, then click the button BUTTON of the death screen at once
#              (or run the horizon-ctl command in PROBE_ACTION, for example "key esc tap")
# Prints one JSON line: frames, cadence, black frames, the screens in order, the frames that show the arrival world before the first
# black frame, the frames of a plain respawn at the point, the reply of the click. It also writes DIR/sheet.png, a contact sheet.
series_probe() {
    mkdir -p "$2"
    CTL_BIN="$CTL" CTL_SOCK="$SOCK" PROBE_DIR="$2" PROBE_MODE="$1" PROBE_BUTTON="${3:-respawn}" PROBE_LOG="$(logfile)" \
        PROBE_SECS="${PROBE_SECS:-8}" PROBE_PX="$spawn_x" PROBE_PZ="$spawn_z" python3 -W ignore - <<'PYEOF'
import os, re, json, math, time, subprocess, threading
ctl, sock, out, mode, button, log = [os.environ[k] for k in ("CTL_BIN", "CTL_SOCK", "PROBE_DIR", "PROBE_MODE", "PROBE_BUTTON", "PROBE_LOG")]
secs, px, pz = float(os.environ["PROBE_SECS"]), float(os.environ["PROBE_PX"]), float(os.environ["PROBE_PZ"])
from PIL import Image, ImageDraw
def c(*a):
    return subprocess.run([ctl, "--socket", sock, "--timeout", "60"] + list(a), capture_output=True, text=True).stdout
def logtext():
    try:
        return open(log, errors="ignore").read()
    except OSError:
        return ""
def landings():
    return logtext().count("Landing spot ready")
frames, stop, t0 = [], threading.Event(), [time.time()]
def capture():
    i = 0
    while not stop.is_set():
        s = time.time()
        f = "%s/f%03d.png" % (out, i)
        c("screenshot", f)
        try:
            st = json.loads(c("state") or "{}")
        except ValueError:
            st = {}
        pl = st.get("player") or {}
        luma = None
        try:
            luma = sum(Image.open(f).convert("L").resize((64, 36)).getdata()) / (64 * 36)
        except Exception:
            pass
        frames.append({"i": i, "t": round(s - t0[0], 2), "screen": st.get("screen"), "pos": pl.get("position"), "health": pl.get("health"), "luma": luma, "file": f})
        i += 1
        time.sleep(max(0.0, 0.1 - (time.time() - s)))
cap = threading.Thread(target=capture)
mark, reply = 0.0, ""
if mode == "immediate":
    t0[0] = time.time(); cap.start(); time.sleep(0.5)
    mark = time.time() - t0[0]; c("command", "kill @s")
elif mode == "arrival":
    n0 = landings()
    c("command", "kill @s")
    c("wait-for", "screen", "dead", "--timeout", "30")
    t = time.time()
    while landings() <= n0 and time.time() - t < 150:
        time.sleep(0.5)
    time.sleep(1.0)
    t0[0] = time.time(); cap.start(); time.sleep(0.3)
    mark = time.time() - t0[0]; reply = c("respawn")
else:
    t0[0] = time.time(); cap.start()
    mark = time.time() - t0[0]
    act = os.environ.get("PROBE_ACTION")
    reply = c(*act.split()) if act else c("click-button", button)
time.sleep(secs)
stop.set(); cap.join()
land = re.findall(r"Landing spot ready after \d+ ms: \[(-?\d+), (-?\d+), (-?\d+)\]", logtext())
lx, lz = (float(land[-1][0]), float(land[-1][2])) if land else (None, None)
def dist(p, x, z):
    return math.hypot(p[0] - x, p[2] - z) if p and x is not None else None
black = [f for f in frames if f["luma"] is not None and f["luma"] < 3]
fb = black[0]["i"] if black else None
arrival_before = [f["i"] for f in frames if f["luma"] is not None and f["luma"] >= 3 and f["screen"] in ("playing", None)
                  and (fb is None or f["i"] < fb) and f["t"] >= mark and dist(f["pos"], lx, lz) is not None and dist(f["pos"], lx, lz) < 200]
plain = [f["i"] for f in frames if f["t"] >= mark and (f["health"] or 0) > 0 and f["screen"] == "playing" and f["luma"] is not None and f["luma"] >= 3
         and dist(f["pos"], px, pz) is not None and dist(f["pos"], px, pz) < 64]
seq = []
for f in frames:
    if not seq or seq[-1] != f["screen"]:
        seq.append(f["screen"])
cadence = sorted(frames[k + 1]["t"] - frames[k]["t"] for k in range(len(frames) - 1))
# The contact sheet: one thumbnail per frame, a label on each.
cols, tw, th = 8, 160, 90
rows = (len(frames) + cols - 1) // cols
sheet = Image.new("RGB", (cols * tw, max(rows, 1) * th), (40, 40, 40))
d = ImageDraw.Draw(sheet)
for f in frames:
    try:
        im = Image.open(f["file"]).convert("RGB").resize((tw, th))
    except Exception:
        continue
    x, y = (f["i"] % cols) * tw, (f["i"] // cols) * th
    sheet.paste(im, (x, y))
    d.rectangle([x, y, x + 100, y + 10], fill=(0, 0, 0))
    d.text((x + 2, y), "%d %.1fs %s" % (f["i"], f["t"], f["screen"]), fill=(255, 255, 0))
sheet = sheet.resize((1280, int(1280 * sheet.height / sheet.width)))
sheet.save(out + "/sheet.png")
for f in frames:
    try:
        os.remove(f["file"])
    except OSError:
        pass
print(json.dumps({"frames": len(frames), "cadence": round(cadence[len(cadence) // 2], 2) if cadence else None, "black": len(black), "first_black": fb,
                  "mark": round(mark, 2), "first_black_t": black[0]["t"] if black else None, "screens": seq, "arrival_before_black": arrival_before,
                  "plain_respawn": plain, "reply": reply.strip()[:200], "mod_frames": sum(1 for f in frames if f["screen"] == "mod")}))
PYEOF
}

if wanted w; then
    echo "--- w. the journey after the fix: immediate respawn, the arrival frame, a stale arm, a screen over the death screen, a press before the departure"
    ensure_point
    ctl command "gamemode survival" >/dev/null
    ctl command "bmh rule random_death_spawn_point true" >/dev/null
    ctl command "bmh rule display_text_on_respawn true" >/dev/null
    ctl command "gamerule immediate_respawn false" >/dev/null
    # A first, plain exile: the player now carries an exile record (its home is the point). The early respawn of w1 puts the
    # player at that point for a moment, so the homecoming must not fire there.
    death_and_respawn ctl 1 >/dev/null
    ctl wait 100 >/dev/null
    sleep 4
    # w1: immediate_respawn true. The early respawn, then the move to the landing, once each.
    ctl command "gamerule immediate_respawn true" >/dev/null
    home_before="$(count_log 'is home after')"
    log_mark="$(wc -l < "$(logfile)")"
    warn_before="$(count_log 'WARN.*BalancedMcHardcoreMain')"
    rec_before="$(count_log 'Homecoming record:')"
    out="$(PROBE_SECS=12 series_probe immediate "$SHOTS/w1")"
    echo "test-slice.sh: w1 series: $out"
    ctl command "gamerule immediate_respawn false" >/dev/null
    new_log="$(tail -n +$((log_mark + 1)) "$(logfile)")"
    early_n="$(printf '%s\n' "$new_log" | grep -c 'did not end before the respawn')"
    moved_n="$(printf '%s\n' "$new_log" | grep -c 'moved to the landing after an early respawn')"
    rec_n="$(( $(count_log 'Homecoming record:') - rec_before ))"
    warns="$(( $(count_log 'WARN.*BalancedMcHardcoreMain') - warn_before ))"
    early_line="$(printf '%s\n' "$new_log" | grep -n 'did not end before the respawn' | head -n 1 | cut -d: -f1)"
    moved_line="$(printf '%s\n' "$new_log" | grep -n 'moved to the landing after an early respawn' | head -n 1 | cut -d: -f1)"
    pos="$(position_of)"
    res="$(dist_check "$(echo "$pos" | cut -d ' ' -f 1)" "$(echo "$pos" | cut -d ' ' -f 3)" "$spawn_x" "$spawn_z" "$expected" 64)"
    if [ "${res##* }" = ok ] && [ "$warns" -eq 0 ] && [ "$early_n" -eq 1 ] && [ "$moved_n" -eq 1 ] && [ "${early_line:-9999}" -lt "${moved_line:-0}" ] && [ "$rec_n" -eq 1 ]; then
        pass w1 "immediate_respawn true: ${res%% *} blocks from the point, no new WARN, the early respawn line then the move to the landing (once each), one homecoming record"
    else
        fail w1 "immediate_respawn true: ${res%% *} blocks (expected $expected); WARN +$warns; early line x$early_n (line ${early_line:-none}), moved line x$moved_n (line ${moved_line:-none}); homecoming records +$rec_n"
    fi
    home_n="$(( $(count_log 'is home after') - home_before ))"
    if [ "$home_n" -eq 0 ]; then
        pass w1d "the early respawn at the old home is no homecoming: no 'is home after' line"
    else
        fail w1d "the early respawn put the player at the vanilla point, in the old exile's home, and the homecoming fired there: $home_n 'is home after' line(s) (title, bell and awards for a death)"
    fi
    # The advancement file: the journey/exile award is done. "Once" cannot be told apart in the file: the log has one record line.
    ctl save >/dev/null
    adv_file=""
    for f in $(find "$GAMEDIR" -path '*advancements*' -name '*.json' 2>/dev/null); do
        if grep -q 'bmh:journey/exile' "$f" 2>/dev/null; then adv_file="$f"; fi
    done
    adv_done="$([ -n "$adv_file" ] && python3 -c '
import json, sys
d = json.load(open(sys.argv[1]))
print(d.get("bmh:journey/exile", {}).get("done"))' "$adv_file")"
    if [ "$adv_done" = True ]; then
        pass w1b "after the early respawn bmh:journey/exile is done in the advancement file"
    else
        fail w1b "bmh:journey/exile after the early respawn: file '${adv_file:-none}', done '$adv_done'"
    fi
    # The series: black from the departure, then the sky and the message at the landing. A person must look at the sheet.
    sbl="$(jget "$out" 'd["black"]')"
    smod="$(jget "$out" 'd["mod_frames"]')"
    sab="$(jget "$out" 'len(d["arrival_before_black"])')"
    if [ "${sbl:-0}" -ge 3 ] && [ "${smod:-0}" -ge 5 ] && [ "${sab:-1}" -eq 0 ]; then
        pass w1c "immediate_respawn: $sbl black frames, $smod frames of the mod screen, no frame of the landing world before the black (first black at $(jget "$out" 'd["first_black_t"]') s, the kill at $(jget "$out" 'd["mark"]') s; sheet $SHOTS/w1/sheet.png; screens: $(jget "$out" 'd["screens"]'))"
    else
        fail w1c "immediate_respawn series: black frames ${sbl:-?} (expected 3 or more), mod frames ${smod:-?} (5 or more), landing-world frames before the black ${sab:-?} (0); screens $(jget "$out" 'd["screens"]')"
    fi
    ctl wait 60 >/dev/null

    # w2: the arrival frame. Kill, wait for the landing line, "respawn", frames every 0.1 s.
    out="$(PROBE_SECS=7 series_probe arrival "$SHOTS/w2")"
    echo "test-slice.sh: w2 series: $out"
    sbl="$(jget "$out" 'd["black"]')"
    sab="$(jget "$out" 'len(d["arrival_before_black"])')"
    if [ "${sbl:-0}" -ge 3 ] && [ "${sab:-1}" -eq 0 ]; then
        pass w2 "arrival frame: $sbl black frames, no frame of the arrival world before the first black (screens: $(jget "$out" 'd["screens"]'), cadence $(jget "$out" 'd["cadence"]') s; sheet $SHOTS/w2/sheet.png)"
    else
        fail w2 "arrival frame: black frames ${sbl:-?} (3 or more), frames of the arrival world before the black: $(jget "$out" 'd["arrival_before_black"]'); screens $(jget "$out" 'd["screens"]')"
    fi
    ctl wait 60 >/dev/null

    # w3: a stale arm. immediate_respawn true: die once. Then it is false and the rule is off: the click respawns at once.
    ctl command "gamerule immediate_respawn true" >/dev/null
    ctl command "kill @s" >/dev/null
    ctl wait 100 >/dev/null
    sleep 6
    ctl command "gamerule immediate_respawn false" >/dev/null
    ctl command "bmh rule random_death_spawn_point false" >/dev/null
    ctl command "kill @s" >/dev/null
    ctl wait-for screen dead --timeout 30 >/dev/null
    sleep 2
    out="$(PROBE_SECS=4 series_probe click "$SHOTS/w3" respawn)"
    echo "test-slice.sh: w3 series: $out"
    pos="$(position_of)"
    res="$(near_check "$(echo "$pos" | cut -d ' ' -f 1)" "$(echo "$pos" | cut -d ' ' -f 3)" "$spawn_x" "$spawn_z" 64)"
    sbl="$(jget "$out" 'd["black"]')"
    smod="$(jget "$out" 'd["mod_frames"]')"
    if [ "${sbl:-1}" -eq 0 ] && [ "${smod:-1}" -eq 0 ] && [ "${res##* }" = ok ]; then
        pass w3 "stale arm: rule off after an immediate respawn, the click respawned at once at the point (${res%% *} blocks from it), no black frame, no mod screen (sheet $SHOTS/w3/sheet.png)"
    else
        fail w3 "stale arm: black frames ${sbl:-?}, mod frames ${smod:-?} (both 0), ${res%% *} blocks from the point (64 at most); screens $(jget "$out" 'd["screens"]')"
    fi
    ctl command "bmh rule random_death_spawn_point true" >/dev/null
    ctl wait 60 >/dev/null

    # w4: a screen over the death screen. The only one the control tool opens is the quit confirmation of "Title Screen".
    # Closing it (Escape or its Respawn button) is itself a request to respawn, so "close it, then click Respawn" cannot be
    # split: the check is that the closing request is held (the fade starts) and no plain respawn happens. Two variants.
    for variant in escape button; do
        ctl command "kill @s" >/dev/null
        ctl wait-for screen dead --timeout 30 >/dev/null
        sleep 3
        ctl click-button "Title Screen" >/dev/null
        sleep 1
        echo "test-slice.sh: w4 $variant: widgets over the death screen: $(jget "$(ctl ui)" '[w["label"] for w in d["widgets"]]')"
        if [ "$variant" = escape ]; then act="key esc tap"; else act="click-button Respawn"; fi
        out="$(PROBE_SECS=6 PROBE_ACTION="$act" series_probe click "$SHOTS/w4-$variant")"
        echo "test-slice.sh: w4 $variant series: $out"
        smod="$(jget "$out" 'd["mod_frames"]')"
        sbl="$(jget "$out" 'd["black"]')"
        sply="$(jget "$out" 'len(d["plain_respawn"])')"
        if [ "${smod:-0}" -ge 1 ] && [ "${sply:-1}" -eq 0 ] && [ "${sbl:-0}" -ge 1 ]; then
            pass w4-$variant "a screen over the death screen, closed with '$act': the request was held ($smod frames of the mod screen, $sbl black), no plain respawn at the point (sheet $SHOTS/w4-$variant/sheet.png)"
        else
            fail w4-$variant "a screen over the death screen, closed with '$act': mod frames ${smod:-?} (1 or more), black frames ${sbl:-?} (1 or more), plain respawn frames ${sply:-?} (0); screens $(jget "$out" 'd["screens"]')"
        fi
        ctl wait 100 >/dev/null
        sleep 4
    done

    # w5: a press before the departure. Die and click Respawn at once.
    land_before="$(count_log 'Landing spot ready')"
    ctl command "kill @s" >/dev/null
    ctl wait-for screen dead --timeout 30 >/dev/null
    out="$(PROBE_SECS=3 series_probe click "$SHOTS/w5" respawn)"
    echo "test-slice.sh: w5 series: $out"
    sply="$(jget "$out" 'len(d["plain_respawn"])')"
    smod="$(jget "$out" 'd["mod_frames"]')"
    if [ "${sply:-1}" -eq 0 ]; then
        pass w5 "a press right after the death: no plain respawn at the point; click reply '$(jget "$out" 'd["reply"]')', screens $(jget "$out" 'd["screens"]'), $smod mod frames (sheet $SHOTS/w5/sheet.png)"
    else
        fail w5 "a press right after the death: $sply frames of a plain respawn at the point; screens $(jget "$out" 'd["screens"]')"
    fi
    # Finish the death: if the click was ignored, press again once the landing is found.
    if [ "$(jget "$(state)" 'd.get("screen")')" = dead ]; then
        wait_log 'Landing spot ready' "$land_before" 150 >/dev/null
        ctl click-button respawn >/dev/null
    fi
    ctl wait 100 >/dev/null
    sleep 6
fi

# ---- x. A LAN guest below operator level 2: /bmh rule is unknown, /bmh point save works -------------------
# The guest is a second game in its own folders that joins the first one over the loopback address. A LAN guest has
# permission level 0 here, also when the world is open with "commands" (checked 2026-11-18: even "time set" is refused).
# So a level of 2 or more is only tested through the owner (scenario a).
if wanted x; then
    echo "--- x. a LAN guest below operator level 2"
    ensure_point
    ctl command "gamemode survival" >/dev/null
    ctl command "bmh rule random_death_spawn_point true" >/dev/null
    rules_file="$GAMEDIR/saves/$WORLD/bmh/rules.txt"
    ctl tp 3 127 3 >/dev/null 2>&1
    ctl wait 40 >/dev/null
    lan="$(ctl lan open)"
    port="$(jget "$lan" 'd["port"]')"
    mkdir -p "$scratch/ghome" "$scratch/ggame"
    greply="$(HORIZON_HOME="$scratch/ghome" HORIZON_NO_AUDIO=1 HORIZON_TELEMETRY_URL=off HORIZON_NO_DISCORD=1 HORIZON_LAUNCHER_NO_SELF_UPDATE=1 \
        "$CTL" --socket "$GSOCK" --timeout 300 launch --game "$GAME" --resources "$resources" --game-dir "$scratch/ggame" --home "$scratch/ghome" \
        --dev-mod "$content" $extra 2>&1)"
    gctl() { "$CTL" --socket "$GSOCK" --timeout 60 "$@" 2>&1; }
    if [ -z "$port" ] || [ "$port" = None ] || [ ! -S "$GSOCK" ]; then
        skip x1 "no LAN port or the guest game did not start: $(printf '%s' "$lan" | head -c 150) / $(printf '%s' "$greply" | head -c 150)"
    else
        gctl connect "localhost:$port" Guest >/dev/null
        n=0
        while [ "$n" -lt 40 ]; do
            if ctl lan status | grep -q '"name":"Guest"'; then break; fi
            sleep 1
            n=$((n + 1))
        done
        if ! ctl lan status | grep -q '"name":"Guest"'; then
            skip x1 "the guest did not join within 40 s (lan status: $(ctl lan status | head -c 200))"
        else
            sleep 4
            rules_before="$(cat "$rules_file")"
            bad_before="$(jget "$(gctl chat-log 50)" 'sum(1 for m in d["chat"] if "Unknown or incomplete command" in m)')"
            gctl command "bmh rule" >/dev/null
            gctl command "bmh rule random_death_spawn_point false" >/dev/null
            sleep 2
            bad_after="$(jget "$(gctl chat-log 50)" 'sum(1 for m in d["chat"] if "Unknown or incomplete command" in m)')"
            styled="$(gctl messages 4)"
            red="$(jget "$styled" 'sum(1 for m in d.get("styled", [])[:4] if m.get("translate") == "command.unknown.command" and m.get("color") == "red")')"
            if [ "$(cat "$rules_file")" = "$rules_before" ] && [ "$(( ${bad_after:-0} - ${bad_before:-0} ))" -ge 2 ] && [ "${red:-0}" -ge 2 ]; then
                pass x1 "a LAN guest (level 0) runs /bmh rule and /bmh rule <name> false: red unknown-command text twice ($red styled red), bmh/rules.txt unchanged"
            else
                fail x1 "a LAN guest's /bmh rule: unknown-command lines +$(( ${bad_after:-0} - ${bad_before:-0} )) (2 expected), red $red; rules.txt changed: $([ "$(cat "$rules_file")" = "$rules_before" ] && echo no || echo yes)"
            fi
            # x2: /bmh point save works for the guest on a bed. The owner gives the amulet and puts the bed; the guest saves.
            gpos="$(gctl state)"
            gx="$(jget "$gpos" 'int(d["player"]["block_pos"][0])')"
            gy="$(jget "$gpos" 'int(d["player"]["block_pos"][1])')"
            gz="$(jget "$gpos" 'int(d["player"]["block_pos"][2])')"
            ctl command "give Guest bmh:navigator_amulet 1" >/dev/null
            ctl command "setblock $((gx + 1)) $gy $gz minecraft:red_bed[facing=east,part=foot]" >/dev/null
            ctl command "setblock $((gx + 2)) $gy $gz minecraft:red_bed[facing=east,part=head]" >/dev/null
            sleep 1
            gctl select bmh:navigator_amulet >/dev/null
            gctl command "bmh point save $((gx + 1)) $gy $gz GuestCamp" >/dev/null
            sleep 2
            bar="$(jget "$(gctl messages 1)" 'd["actionbar"]')"
            if [ "$bar" = "Point saved: GuestCamp" ]; then
                pass x2 "a LAN guest (level 0) saves a point on a bed with /bmh point save: action bar '$bar'"
            else
                fail x2 "a LAN guest's /bmh point save on a bed at $((gx + 1)) $gy $gz: action bar '$bar'; chat $(jget "$(gctl messages 3)" 'd["chat"][:3]')"
            fi
            # x3: the Java command for a BMH rule. A level 0 guest types /gamerule bmh:... : the mod takes the
            # line, runs it as the guest, and /bmh rule refuses it. The rules file stays as it was.
            rules_before="$(cat "$rules_file")"
            bad_before="$(jget "$(gctl chat-log 50)" 'sum(1 for m in d["chat"] if "Unknown or incomplete command" in m)')"
            taken_before="$(count_log 'taken from a player')"
            gctl chat "/gamerule bmh:random_death_spawn_point false" >/dev/null
            sleep 3
            bad_after="$(jget "$(gctl chat-log 50)" 'sum(1 for m in d["chat"] if "Unknown or incomplete command" in m)')"
            if [ "$(cat "$rules_file")" = "$rules_before" ] && [ "$(( ${bad_after:-0} - ${bad_before:-0} ))" -ge 1 ] \
                && [ "$(count_log 'taken from a player')" -gt "$taken_before" ]; then
                pass x3 "a LAN guest (level 0) typing /gamerule bmh:random_death_spawn_point false is taken by the mod (log line) and refused (unknown-command line), rules.txt unchanged"
            else
                fail x3 "a LAN guest's /gamerule bmh:...: unknown-command lines +$(( ${bad_after:-0} - ${bad_before:-0} )) (1 expected); rules.txt changed: $([ "$(cat "$rules_file")" = "$rules_before" ] && echo no || echo yes); 'taken' log lines +$(( $(count_log 'taken from a player') - taken_before )); chat $(jget "$(gctl messages 4)" 'd["chat"][:4]')"
            fi
            gctl disconnect >/dev/null
        fi
    fi
    gctl quit >/dev/null 2>&1
    ctl lan close >/dev/null
    sleep 3
fi

# ---- y. A Java world's rules are imported: bmh:<rule> false in game_rules.dat, no bmh/rules.txt ---------------
# The scratch world is closed, its game_rules.dat gets two BMH keys at false (a byte, a text: both forms the import reads),
# its rules file is deleted, and the game opens it again. A dedicated server (the console and the load order) has no
# runner in horizon/tools: not tested.
if wanted y; then
    echo "--- y. a Java world's rules, imported at the first open with the mod"
    ensure_point
    ctl command "gamemode survival" >/dev/null
    ctl save >/dev/null
    ctl quit >/dev/null
    n=0
    while [ -n "$GAMEPID" ] && kill -0 "$GAMEPID" 2>/dev/null && [ "$n" -lt 60 ]; do sleep 1; n=$((n + 1)); done
    rules_file="$GAMEDIR/saves/$WORLD/bmh/rules.txt"
    dat="$GAMEDIR/saves/$WORLD/data/minecraft/game_rules.dat"
    cp "$dat" "$scratch/game_rules.dat.before" 2>/dev/null || true
    rm -f "$rules_file"
    python3 - "$dat" <<'PYEOF'
import gzip, sys
p = sys.argv[1]
raw = gzip.open(p).read()
# The root compound holds "data", a compound. Entries: type byte, name (u16 length + bytes), value.
head = b"\x0a\x00\x04data"
i = raw.index(head) + len(head)
def entry_byte(name, v):
    n = name.encode()
    return b"\x01" + len(n).to_bytes(2, "big") + n + bytes([v])
def entry_text(name, v):
    n, t = name.encode(), v.encode()
    return b"\x08" + len(n).to_bytes(2, "big") + n + len(t).to_bytes(2, "big") + t
add = entry_byte("bmh:random_death_spawn_point", 0) + entry_text("bmh:display_text_on_respawn", "false")
gzip.open(p, "wb").write(raw[:i] + add + raw[i:])
PYEOF
    launch_game --wait
    if [ "$launch_status" -ne 0 ]; then
        fail y1 "the launch after editing game_rules.dat failed (status $launch_status)"
    else
        rf="$(cat "$rules_file" 2>/dev/null)"
        if printf '%s\n' "$rf" | grep -q '^random_death_spawn_point = false' && printf '%s\n' "$rf" | grep -q '^display_text_on_respawn = false' \
            && [ "$(printf '%s\n' "$rf" | grep -c ' = true')" -eq 4 ]; then
            pass y1 "a Java world with two BMH keys at false (a byte, a text) and no rules file: bmh/rules.txt has those two off and four on"
        else
            fail y1 "bmh/rules.txt after the import: $(printf '%s' "$rf" | grep ' = ' | tr '\n' ';')"
        fi
        reply="$(ctl command "bmh rule")"
        if printf '%s' "$reply" | grep -q 'random_death_spawn_point is currently set to false' && printf '%s' "$reply" | grep -q 'display_text_on_respawn is currently set to false' \
            && printf '%s' "$reply" | grep -q 'only_carry_one_eternal_item_type is currently set to true'; then
            pass y2 "/bmh rule lists the two imported rules off and the others on"
        else
            fail y2 "/bmh rule after the import: $(printf '%s' "$reply" | head -c 400)"
        fi
        if [ "$(count_log 'World rules created: java world = true')" -ge 1 ]; then
            pass y3 "the log tells it: World rules created: java world = true"
        else
            fail y3 "no log line 'World rules created: java world = true': $(grep 'World rules created' "$(logfile)" | tail -n 1)"
        fi
        ctl command "bmh rule random_death_spawn_point true" >/dev/null
        ctl command "bmh rule display_text_on_respawn true" >/dev/null
    fi
fi

# ---- m. Rules and config across a restart --------------------------------------------------------------
# A rule set to false must survive a quit. A hand edited config value outside its range gives the default
# of its key and the file is written back once (NeoForge's ModConfigSpec corrects it the same way). Runs
# the game twice, so it runs just before j.
if wanted m; then
    echo "--- m. a rule at false and a config out of range, then a quit and a second launch"
    ensure_point
    ctl command "gamemode survival" >/dev/null
    ctl command "bmh rule display_text_on_respawn false" >/dev/null
    ctl save >/dev/null
    ctl quit >/dev/null
    n=0
    while [ -n "$GAMEPID" ] && kill -0 "$GAMEPID" 2>/dev/null && [ "$n" -lt 60 ]; do sleep 1; n=$((n + 1)); done
    cp "$GAMEDIR/config/bmh-common.toml" "$scratch/bmh-common.toml.before" 2>/dev/null || true
    cat > "$GAMEDIR/config/bmh-common.toml" <<'TOML'
#Minimum distance from bed to respawn
#Range: 5000 ~ 200000
minDistanceFromBed = 3000
#Maximum distance from bed to respawn
#Range: 5000 ~ 200000
maxDistanceFromBed = 999999
#Most points a player can keep on the Navigator's Amulet
#Range: 1 ~ 16
maxAmuletPoints = 99
TOML
    launch_game --wait
    if [ "$launch_status" -ne 0 ]; then
        fail m1 "the second launch failed (status $launch_status)"
    else
        reply="$(ctl command "bmh rule display_text_on_respawn")"
        if printf '%s' "$reply" | grep -q 'set to false'; then
            pass m1 "the rule at false survived the quit: $(jget "$reply" 'd["chat"][0]')"
        else
            fail m1 "the rule did not survive the quit: $(printf '%s' "$reply" | head -c 200)"
        fi
        ctl command "bmh rule display_text_on_respawn true" >/dev/null
        ctl command "gamemode survival" >/dev/null
        if death_and_respawn ctl 1; then
            pos="$(position_of)"
            res="$(dist_check "$(echo "$pos" | cut -d ' ' -f 1)" "$(echo "$pos" | cut -d ' ' -f 3)" "$spawn_x" "$spawn_z" 23333 64)"
            if [ "${res##* }" = ok ]; then
                pass m2 "min 3000, max 999999, points 99 in the file: the defaults apply, ${res%% *} blocks, expected 23333"
            else
                fail m2 "out of range values in the file: ${res%% *} blocks, expected 23333 (the defaults, Normal)"
            fi
        else
            fail m2 "no respawn: $DR_NOTE"
        fi
        # NeoForge's ModConfigSpec writes the corrected file back. The mod does the same, once: each key
        # holds its default, the Java comments are there, and the log tells one rewrite.
        cfg_after="$(grep -E '^[a-zA-Z]+ *=' "$GAMEDIR/config/bmh-common.toml" | tr -d ' ' | tr '\n' ' ')"
        cfg_rewrites="$(count_log 'config file held a value outside its range')"
        cfg_ranges="$(grep -c '^#Range' "$GAMEDIR/config/bmh-common.toml" 2>/dev/null || true)"
        if [ "$(cfgval minDistanceFromBed)" = 5000 ] && [ "$(cfgval maxDistanceFromBed)" = 60000 ] \
            && [ "$(cfgval maxAmuletPoints)" = 5 ] && [ "${cfg_ranges:-0}" -eq 3 ] && [ "$cfg_rewrites" -eq 1 ]; then
            pass m3 "the out of range file is written back once, with the defaults: $cfg_after"
        else
            fail m3 "the out of range file after the start: $cfg_after, $cfg_ranges '#Range' lines (expected 3), $cfg_rewrites rewrite lines in the log (expected 1)"
        fi
        if [ -f "$scratch/bmh-common.toml.before" ]; then
            cp "$scratch/bmh-common.toml.before" "$GAMEDIR/config/bmh-common.toml"
            sleep 2
        fi
    fi
fi

# ---- j. Dead when the game closes ---------------------------------------------------------------
# Java keeps the death's distance and searches at the respawn. Here the dead player comes back with
# no search: the mod must start one. This quits the game and launches it again on the same world,
# so it runs last.
if wanted j; then
    echo "--- j. dead when the game closes, the world opens again: the exile still happens"
    POINT_SET=0
    ensure_point
    ctl command "gamemode survival" >/dev/null
    seq="$(jget "$(ctl events)" 'd["last"]')"; [ -n "$seq" ] || seq=0
    ctl command "kill @s" >/dev/null
    if ! ctl wait-for event death --since "$seq" --timeout 30 >/dev/null; then
        fail j1 "no death event within 30 s"
    else
        sleep 2
        # The second launch may start a new log or add to this one: count from here.
        b_land="$(count_log 'Landing spot ready')"
        b_new="$(count_log 'new respawn pos and bed')"
        ctl quit >/dev/null
        n=0
        while [ -n "$GAMEPID" ] && kill -0 "$GAMEPID" 2>/dev/null && [ "$n" -lt 60 ]; do sleep 1; n=$((n + 1)); done
        if [ -n "$GAMEPID" ] && kill -0 "$GAMEPID" 2>/dev/null; then
            fail j1 "the game did not quit within 60 s (pid $GAMEPID)"
        else
            echo "test-slice.sh: the game quit with the player dead; launching it again on the same world"
            launch_game
            if [ "$(count_log 'Landing spot ready')" -lt "$b_land" ]; then
                b_land=0; b_new=0
            fi
            if [ "$launch_status" -ne 0 ]; then
                fail j1 "the second launch failed (status $launch_status): $(printf '%s' "$reply" | tail -n 1 | head -c 300)"
            elif ! ctl wait-for screen dead --timeout 300 >/dev/null; then
                fail j1 "the reopened world shows no death screen: screen $(jget "$(state)" 'd.get("screen")')"
            elif ! wait_log 'Landing spot ready' "$b_land" 150; then
                fail j1 "the reopened world: the player is dead, but no new 'Landing spot ready' line within 150 s"
            else
                # Horizon kills a player who logs in dead once more ("joined the game", then "died"):
                # that death starts the search (first run of j, 2026-11-17).
                pass j1 "the reopened world: death screen, then a new search: $(grep 'Landing spot ready' "$(logfile)" | tail -n 1 | sed 's/^[^]]*\]: //')"
                seq="$(jget "$(ctl events)" 'd["last"]')"; [ -n "$seq" ] || seq=0
                ctl respawn >/dev/null
                if ctl wait-for event respawn --since "$seq" --timeout 90 >/dev/null; then
                    ctl wait 40 >/dev/null
                    pos="$(position_of)"
                    res="$(dist_check "$(echo "$pos" | cut -d ' ' -f 1)" "$(echo "$pos" | cut -d ' ' -f 3)" "$spawn_x" "$spawn_z" "$expected" 64)"
                    if [ "${res##* }" = ok ] && [ "$(count_log 'new respawn pos and bed')" -gt "$b_new" ]; then
                        pass j2 "exile after the restart: ${res%% *} blocks from the point, expected $expected"
                    else
                        fail j2 "after the restart: ${res%% *} blocks from the point, expected $expected; 'new respawn pos and bed' lines $(count_log 'new respawn pos and bed'), 'No distant respawn' lines $(count_log 'No distant respawn')"
                    fi
                else
                    fail j2 "no respawn event within 90 s after the restart"
                fi
            fi
        fi
    fi
fi

# ---- z. /gamerule bmh:... (S1) ---------------------------------------------------------------------------------
# The Java command for a BMH rule. Horizon's /gamerule knows vanilla rules only, so the mod hears every typed
# command (player_command), stops the line and runs `bmh rule ...` as the player on the next server tick. The
# owner types it in the chat box (horizon-ctl chat), as a person does.
if wanted z; then
    echo "--- z. /gamerule bmh:<rule> typed in the chat"
    ensure_point
    ctl command "gamemode survival" >/dev/null
    # z0: the scratch world has Allow Commands off, so the owner's chat commands run at level 0. The mod takes
    # the line, runs it as the owner, and /bmh rule refuses it, as vanilla's /gamerule refuses a player below
    # level 2. Nothing changes.
    rules_file="$GAMEDIR/saves/$WORLD/bmh/rules.txt"
    ctl command "bmh rule display_text_on_respawn true" >/dev/null
    rules_before="$(cat "$rules_file")"
    taken_before="$(count_log 'taken from a player')"
    ctl chat "/gamerule bmh:display_text_on_respawn false" >/dev/null
    sleep 2
    if [ "$(cat "$rules_file")" = "$rules_before" ] && [ "$(count_log 'taken from a player')" -gt "$taken_before" ] \
        && jget "$(ctl messages 3)" '" ".join(d["chat"][:3])' | grep -q 'Unknown or incomplete command'; then
        pass z0 "the owner of a world with Allow Commands off (level 0): /gamerule bmh:... is taken and refused, rules.txt unchanged"
    else
        fail z0 "level 0 owner: rules.txt changed: $([ "$(cat "$rules_file")" = "$rules_before" ] && echo no || echo yes); taken lines +$(( $(count_log 'taken from a player') - taken_before )); chat $(jget "$(ctl messages 3)" 'd["chat"][:3]')"
    fi
    # The other checks need a world with Allow Commands on: make one the way a player does.
    leave_world
    open_create_screen
    n=0
    while ! jget "$(ctl ui)" '[w["label"] for w in d["widgets"] if w["id"]=="allow-commands"][0]' | grep -q ON && [ "$n" -lt 3 ]; do
        ctl click allow-commands >/dev/null; sleep 1; n=$((n + 1))
    done
    make_world bmhcmd
    rules_file="$GAMEDIR/saves/bmhcmd/bmh/rules.txt"
    rule_now() { sed -n 's/^display_text_on_respawn = //p' "$rules_file" | head -n 1; }
    warn_before="$(count_log 'ran nothing for the line')"
    ctl chat "/gamerule bmh:display_text_on_respawn false" >/dev/null
    sleep 2
    set_text="$(jget "$(ctl messages 6)" '" | ".join(d["chat"][:6])')"
    if [ "$(rule_now)" = false ] && printf '%s' "$set_text" | grep -q 'Game rule bmh:display_text_on_respawn is now set to false'; then
        pass z1 "/gamerule bmh:display_text_on_respawn false in the chat: rules.txt says false, answer: $(printf '%s' "$set_text" | head -c 160)"
    else
        fail z1 "/gamerule bmh:display_text_on_respawn false: rules.txt '$(rule_now)', chat: $(printf '%s' "$set_text" | head -c 300)"
    fi
    ctl chat "/gamerule bmh:display_text_on_respawn" >/dev/null
    sleep 2
    q_text="$(jget "$(ctl messages 3)" '" | ".join(d["chat"][:3])')"
    if printf '%s' "$q_text" | grep -q 'Game rule bmh:display_text_on_respawn is currently set to false'; then
        pass z2 "/gamerule bmh:display_text_on_respawn queries: $(printf '%s' "$q_text" | head -c 160)"
    else
        fail z2 "the query of /gamerule bmh:display_text_on_respawn: $(printf '%s' "$q_text" | head -c 300)"
    fi
    ctl chat "/gamerule bmh:display_text_on_respawn true" >/dev/null
    sleep 2
    if [ "$(rule_now)" = true ]; then
        pass z3 "/gamerule bmh:display_text_on_respawn true puts it back (rules.txt true)"
    else
        fail z3 "rules.txt after /gamerule bmh:display_text_on_respawn true: '$(rule_now)'"
    fi
    ctl chat "/gamerule keep_inventory true" >/dev/null
    sleep 2
    v_text="$(jget "$(ctl messages 3)" '" | ".join(d["chat"][:3])')"
    ctl chat "/gamerule keep_inventory false" >/dev/null
    sleep 1
    if printf '%s' "$v_text" | grep -q 'keep_inventory is now set to true'; then
        pass z4 "a vanilla rule is left to vanilla: $(printf '%s' "$v_text" | head -c 160)"
    else
        fail z4 "/gamerule keep_inventory true: $(printf '%s' "$v_text" | head -c 300)"
    fi
    ctl chat "/gamerule bmh:nonsense true" >/dev/null
    sleep 2
    u_text="$(jget "$(ctl messages 3)" '" | ".join(d["chat"][:3])')"
    if printf '%s' "$u_text" | grep -q 'Unknown BMH rule: nonsense'; then
        pass z5 "an unknown bmh: rule is named: $(printf '%s' "$u_text" | head -c 160)"
    else
        fail z5 "/gamerule bmh:nonsense true: $(printf '%s' "$u_text" | head -c 300)"
    fi
    if [ "$(count_log 'ran nothing for the line')" -eq "$warn_before" ]; then
        pass z6 "no 'ran nothing' warning in the log: every queued line ran"
    else
        fail z6 "the log has 'ran nothing for the line' warnings: $(grep 'ran nothing for the line' "$(logfile)" | tail -n 2)"
    fi
    # z8: the rule is kept in the world: set false, save, quit to the title, open the world again.
    ctl chat "/gamerule bmh:only_carry_one_eternal_item_type false" >/dev/null
    sleep 2
    leave_world
    ctl open bmhcmd >/dev/null
    ctl wait-for loaded --timeout 200 >/dev/null
    sleep 3
    ctl chat "/gamerule bmh:only_carry_one_eternal_item_type" >/dev/null
    sleep 2
    k_text="$(jget "$(ctl messages 3)" '" | ".join(d["chat"][:3])')"
    if printf '%s' "$k_text" | grep -q 'only_carry_one_eternal_item_type is currently set to false'; then
        pass z8 "the rule stays false in the world across a quit to the title: $(printf '%s' "$k_text" | head -c 160)"
    else
        fail z8 "after the world opened again: $(printf '%s' "$k_text" | head -c 300)"
    fi
    # z9: the chat box completes the rule name of /bmh rule (the kind word? asks the server, which answers
    # through tab_complete_v2). A person types "/bmh rule rand" and presses Tab.
    ctl key t tap >/dev/null
    sleep 1
    ctl type "/bmh rule rand" >/dev/null
    sleep 2
    ctl screenshot "$SHOTS/z9-completion.png" >/dev/null
    suggestion_rows="$(jget "$(ctl ui)" 'len([w for w in d["widgets"] if w["kind"]=="row"])')"
    ctl key tab tap >/dev/null
    sleep 1
    ctl screenshot "$SHOTS/z9-completion-applied.png" >/dev/null
    if [ "${suggestion_rows:-0}" -ge 1 ]; then
        pass z9 "typing '/bmh rule rand' in the chat box shows $suggestion_rows suggestion row (screenshot z9-completion.png shows random_death_spawn_point)"
    else
        fail z9 "typing '/bmh rule rand' shows no suggestion row: $(ctl ui | head -c 400)"
    fi
    # Escape closes the suggestions first, then the chat box: press it until the game is playing again.
    n=0
    while [ "$(jget "$(state)" 'd["screen"]')" != playing ] && [ "$n" -lt 4 ]; do
        ctl key esc tap >/dev/null
        sleep 1
        n=$((n + 1))
    done
    leave_world
    ctl open "$WORLD" >/dev/null
    ctl wait-for loaded --timeout 200 >/dev/null
    sleep 3
    ctl command "gamemode survival" >/dev/null
fi

if wanted o; then
    echo "--- o. the Balanced Hardcore button: greyed in Hardcore, Re-Create"
    ctl command "gamemode survival" >/dev/null
    leave_world
    open_create_screen
    gm_label() { jget "$(ctl ui)" '[w["label"] for w in d["widgets"] if w["id"]=="game-mode"][0]'; }
    n=0
    while ! gm_label | grep -q Hardcore && [ "$n" -lt 4 ]; do ctl click game-mode >/dev/null; sleep 1; n=$((n + 1)); done
    flip_c0="$(choice_now)"
    xy="$(button_xy)"
    ctl screenshot "$SHOTS/o1-hardcore-greyed.png" >/dev/null
    ctl click-at "${xy%% *}" "${xy##* }" >/dev/null
    sleep 1
    flip_c1="$(choice_now)"
    if gm_label | grep -q Hardcore && [ -n "$flip_c0" ] && [ "$flip_c0" = "$flip_c1" ]; then
        pass o1 "Game Mode: Hardcore: a click on the button ($xy) leaves the choice at '$flip_c0' (greyed, screenshot o1-hardcore-greyed.png)"
    else
        fail o1 "Hardcore: label '$(gm_label)', choice '$flip_c0' before and '$flip_c1' after a click at '$xy'"
    fi
    n=0
    while gm_label | grep -q Hardcore && [ "$n" -lt 4 ]; do ctl click game-mode >/dev/null; sleep 1; n=$((n + 1)); done
    ctl screenshot "$SHOTS/o2-survival-active.png" >/dev/null
    flip_button
    flip_c2="$(choice_now)"
    if [ -n "$flip_c0" ] && [ "$flip_c2" != "$flip_c0" ]; then
        pass o2 "back in $(gm_label): the button takes clicks again ('$flip_c0' to '$flip_c2')"
    else
        fail o2 "back in $(gm_label): choice '$flip_c0' then '$flip_c2' after a click"
    fi
    # Re-Create: a world made with the button OFF, then Re-Create from the world list.
    flip_button
    [ "$(choice_now)" = false ] || flip_button
    make_world bmhrc
    leave_world
    ctl click singleplayer >/dev/null
    sleep 2
    recreate_choice_of() {
        ctl click "$1" >/dev/null; sleep 1
        ctl click re-create >/dev/null; sleep 2
    }
    recreate_choice_of bmhrc
    rc_off="$(choice_now)"
    ctl screenshot "$SHOTS/o3-recreate-of-off-world.png" >/dev/null
    ctl click cancel >/dev/null; sleep 1
    recreate_choice_of "$WORLD"
    rc_on="$(choice_now)"
    ctl screenshot "$SHOTS/o4-recreate-of-on-world.png" >/dev/null
    if [ "$rc_off" = false ] && [ "$rc_on" = true ]; then
        pass o3 "Re-Create of a world whose rules are all false shows OFF ('$rc_off'), of a world with rules on shows ON ('$rc_on')"
    else
        fail o3 "Re-Create: world with rules false gave '$rc_off' (false expected), world with rules on gave '$rc_on' (true expected)"
    fi
    ctl click cancel >/dev/null; sleep 1
    ctl click back >/dev/null; sleep 1
    ctl open "$WORLD" >/dev/null
    ctl wait-for loaded --timeout 200 >/dev/null
    sleep 3
    ctl command "gamemode survival" >/dev/null
fi

# ---- summary -----------------------------------------------------------------------------------
echo
echo "test-slice.sh: $passed passed, $failed failed, $skipped skipped"
[ -z "$skip_lines" ] || printf '%s\n' "$skip_lines"
echo "test-slice.sh: game log: $(logfile) (kept only with --keep)"
if [ "$failed" -gt 0 ]; then
    exit 1
fi
exit 0
