#!/bin/bash
# The in-game test of the core slice (settings, respawn, journey) of the BMH mod on Horizon.
# It drives a real game through horizon-ctl, offscreen. It needs the prepared game files
# (horizon/tools/prepare-game.sh). It never leaves a game running: a trap quits the game and
# removes the scratch world on every exit, also on a failure and on Ctrl+C.
#
# Usage: sh horizon/tools/test-slice.sh [--only a,b,...] [--keep] [--window] [--no-build]
#                                       [--resources DIR] [--bin-dir DIR] [--out DIR]
#   --only a,b   run only these scenarios (a b c d e f g h i k l n p q m j), default all
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
        -h|--help)   sed -n '2,27p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
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
    rm -rf "$sockdir"
    if [ "$keep" -eq 1 ]; then
        echo "test-slice.sh: scratch folder kept: $scratch"
    else
        rm -rf "$scratch/game" "$scratch/home"
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

# ---- summary -----------------------------------------------------------------------------------
echo
echo "test-slice.sh: $passed passed, $failed failed, $skipped skipped"
[ -z "$skip_lines" ] || printf '%s\n' "$skip_lines"
echo "test-slice.sh: game log: $(logfile) (kept only with --keep)"
if [ "$failed" -gt 0 ]; then
    exit 1
fi
exit 0
