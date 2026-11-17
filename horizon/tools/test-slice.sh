#!/bin/bash
# The in-game test of the core slice (settings, respawn, journey) of the BMH mod on Horizon.
# It drives a real game through horizon-ctl, offscreen. It needs the prepared game files
# (horizon/tools/prepare-game.sh). It never leaves a game running: a trap quits the game and
# removes the scratch world on every exit, also on a failure and on Ctrl+C.
#
# Usage: sh horizon/tools/test-slice.sh [--only a,b,...] [--keep] [--window] [--no-build]
#                                       [--resources DIR] [--bin-dir DIR] [--out DIR]
#   --only a,b   run only these scenarios (a b c d e f g), default all
#   --keep       keep the scratch folder (world, game log, screenshots)
#   --window     run with a window instead of offscreen
#   --no-build   do not run "check.sh build" first (use --bin-dir or the cache's build)
#   --resources  the prepared game files (default $BMH_HORIZON_CACHE/resources/26.3)
#   --bin-dir    the folder with horizon-client and horizon-ctl
#   --out DIR    where the screenshots go (default: the scratch folder, printed at the end)
# Scenarios: a mods row, rules and rules file; b distant respawn; c immunity; d cinematic
#   screenshots; e rule off; f creative; g advancement.
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
        -h|--help)   sed -n '2,23p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
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
reply="$(HORIZON_HOME="$HOMEDIR" HORIZON_NO_AUDIO=1 HORIZON_TELEMETRY_URL=off HORIZON_NO_DISCORD=1 \
    HORIZON_LAUNCHER_NO_SELF_UPDATE=1 \
    "$CTL" --socket "$SOCK" --timeout 900 launch --game "$GAME" --world "$WORLD" --seed "$SEED" --wait \
    --resources "$resources" --game-dir "$GAMEDIR" --home "$HOMEDIR" --dev-mod "$content" $extra 2>&1)"
launch_status=$?
# The launch reply is the whole state of the game. Print only what a person needs.
echo "test-slice.sh: launch reply: $(jget "$reply" '{"ok": d.get("launch", {}).get("ok"), "pid": d.get("launch", {}).get("pid"), "screen": d.get("screen"), "dimension": d.get("dimension"), "player": d.get("player")}')"
# The pid is in the "launch" object of the reply (horizon-ctl 2026-11-17 first run), not at the top.
GAMEPID="$(jget "$reply" 'd.get("launch", {}).get("pid") or d.get("pid", "")')"
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
        warns="$(( $(count_log 'Sound command failed') + $(count_log 'not awarded to') ))"
        if [ "$warns" -eq 0 ]; then
            pass b4 "the arrival sounds and the advancement award ran with no warning"
        else
            fail b4 "$warns warning line(s) in the log: $(grep -m1 'Sound command failed' "$(logfile)" | sed 's/^[^]]*\]: //'); $(grep -m1 'not awarded to' "$(logfile)" | sed 's/^[^]]*\]: //')"
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

# ---- summary -----------------------------------------------------------------------------------
echo
echo "test-slice.sh: $passed passed, $failed failed, $skipped skipped"
[ -z "$skip_lines" ] || printf '%s\n' "$skip_lines"
echo "test-slice.sh: game log: $(logfile) (kept only with --keep)"
if [ "$failed" -gt 0 ]; then
    exit 1
fi
exit 0
