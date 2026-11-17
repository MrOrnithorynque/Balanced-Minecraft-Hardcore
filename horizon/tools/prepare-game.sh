#!/bin/bash
# Prepares the game files that Horizon needs, once, from the owner's own copy of
# Minecraft 26.3. It never downloads Minecraft. It checks four things first. For
# each missing thing it prints one line that says what is missing and what to do,
# then it exits 78 and changes nothing. When all four are there, it builds and
# runs horizon-prepare from the Horizon source tree in the cache, then its sounds
# step and its --check and prints the folder of the prepared files.
#
# Usage: sh horizon/tools/prepare-game.sh [--minecraft-dir DIR] [--server-jar PATH] [--dry-run]
#   --minecraft-dir DIR  the folder of the official launcher
#                        (default: $HOME/Library/Application Support/minecraft)
#   --server-jar PATH    the 26.3 server jar (or the variable HORIZON_SERVER_JAR)
#   --dry-run            print the commands, run none of them
# Environment:
#   BMH_HORIZON_CACHE    the cache folder (default: ~/Library/Caches/bmh-horizon).
#                        The prepared files go to $BMH_HORIZON_CACHE/resources/26.3.
#
# Exit codes: 0 done, 78 something is missing, 64 a bad option, 1 horizon-prepare failed.
# Written for /bin/bash 3.2 on macOS: no associative arrays, no empty arrays.
set -eu

here="$(cd "$(dirname "$0")" && pwd)"
bundle="$(cd "$here/../native" && pwd)"
cache="${BMH_HORIZON_CACHE:-$HOME/Library/Caches/bmh-horizon}"

mc_dir="$HOME/Library/Application Support/minecraft"
server_jar="${HORIZON_SERVER_JAR:-}"
dry=0

while [ "$#" -gt 0 ]; do
    case "$1" in
        --minecraft-dir) [ "$#" -ge 2 ] || { echo "prepare-game.sh: --minecraft-dir needs a folder." >&2; exit 64; }
                         mc_dir="$2"; shift 2 ;;
        --server-jar)    [ "$#" -ge 2 ] || { echo "prepare-game.sh: --server-jar needs a path." >&2; exit 64; }
                         server_jar="$2"; shift 2 ;;
        --dry-run)       dry=1; shift ;;
        -h|--help)       sed -n '2,18p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "prepare-game.sh: unknown option '$1'." >&2; exit 64 ;;
    esac
done

if ! command -v python3 >/dev/null 2>&1; then
    echo "prepare-game.sh: python3 is needed to read the JSON files. Install it, then rerun." >&2
    exit 78
fi

# The commit and the game version come from the files of this repo, never from this script.
commit="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["horizon_revision"])' "$bundle/compatibility.json")"
game="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["game"])' "$bundle/compatibility.json")"
tree="$cache/horizon-mods/native/target/horizon/$commit"
out="$cache/resources/$game"
lock="$tree/research-lock.json"

missing=0
miss() {
    echo "MISSING: $1"
    missing=$((missing + 1))
}

# 1. Minecraft, played once from the official launcher.
# horizon-prepare reads versions/<v>/<v>.json, versions/<v>/<v>.jar, libraries/ and assets/indexes
# (crates/horizon-prepare/src/artifacts.rs, the official launcher's layout).
mc_missing=""
[ -f "$mc_dir/versions/$game/$game.json" ] || mc_missing="$mc_missing versions/$game/$game.json"
[ -f "$mc_dir/versions/$game/$game.jar" ] || mc_missing="$mc_missing versions/$game/$game.jar"
[ -d "$mc_dir/libraries" ] || mc_missing="$mc_missing libraries/"
[ -d "$mc_dir/assets/indexes" ] || mc_missing="$mc_missing assets/indexes/"
if [ -n "$mc_missing" ]; then
    miss "Minecraft $game is not in '$mc_dir' (missing:$mc_missing). Install the official Minecraft launcher, start the game version $game once, quit it, then rerun. If your launcher keeps its files elsewhere, pass --minecraft-dir DIR."
fi

# 2. The server jar, with the sha1 that research-lock.json pins.
# 4. (read here, because the sha1 and the Java version live in the tree's research-lock.json)
have_lock=0
[ -f "$lock" ] && have_lock=1
if [ -z "$server_jar" ]; then
    miss "The $game server jar is not given. Download it (the URL is downloads.server.url in versions/$game/$game.json of the Minecraft folder, or minecraft.net/download/server), then rerun with --server-jar PATH or set HORIZON_SERVER_JAR."
elif [ ! -f "$server_jar" ]; then
    miss "The server jar '$server_jar' does not exist. Download the $game server jar and pass its path with --server-jar."
elif [ "$have_lock" -eq 1 ]; then
    want_sha="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["minecraft"]["server_sha1"])' "$lock")"
    have_sha="$(shasum -a 1 "$server_jar" | cut -d ' ' -f 1)"
    if [ "$want_sha" != "$have_sha" ]; then
        miss "The server jar '$server_jar' has sha1 $have_sha, but research-lock.json pins $want_sha. Download the official $game server jar again."
    fi
else
    echo "NOTE: the sha1 of the server jar is not checked: research-lock.json is not in the cache yet (see the Horizon source line)."
fi

# 3. The JDK of the pinned major version.
jdk=""
if [ "$have_lock" -eq 1 ]; then
    major="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["minecraft"]["java_major"])' "$lock")"
    if [ -x "/opt/homebrew/opt/openjdk@$major/bin/javac" ]; then
        jdk="/opt/homebrew/opt/openjdk@$major"
    elif [ -n "${JAVA_HOME:-}" ] && [ -x "$JAVA_HOME/bin/javac" ] \
        && "$JAVA_HOME/bin/javac" -version 2>&1 | grep -Eq "^javac $major([.]|$)"; then
        jdk="$JAVA_HOME"
    else
        miss "No JDK $major found. Run: brew install openjdk@$major (it lands in /opt/homebrew/opt/openjdk@$major), or set JAVA_HOME to a JDK $major."
    fi
else
    echo "NOTE: the JDK version is not checked: research-lock.json is not in the cache yet (see the Horizon source line)."
fi

# 4. The Horizon source tree in the cache.
if [ ! -d "$tree" ] || [ "$have_lock" -eq 0 ]; then
    miss "The Horizon source tree is not in the cache ($tree). Run: sh horizon/tools/check.sh (it clones and compiles it once), then rerun."
fi

# cargo builds horizon-prepare.
cargo_bin="$(command -v cargo 2>/dev/null || true)"
if [ -z "$cargo_bin" ] && [ -x "$HOME/.cargo/bin/cargo" ]; then
    cargo_bin="$HOME/.cargo/bin/cargo"
fi
if [ -z "$cargo_bin" ]; then
    miss "cargo is not on the PATH. Install Rust (rustup, version 1.90 or newer), then rerun."
fi

if [ "$dry" -eq 1 ] || [ "$missing" -eq 0 ]; then
    echo "prepare-game.sh: commands (tree $tree):"
    echo "  (cd '$tree' && JAVA_HOME='${jdk:-<JDK>}' '${cargo_bin:-cargo}' run --release -p horizon-prepare -- --prepare --server-jar '${server_jar:-<server.jar>}' --minecraft-dir '$mc_dir' --output '$out')"
    echo "  (cd '$tree' && JAVA_HOME='${jdk:-<JDK>}' '${cargo_bin:-cargo}' run --release -p horizon-prepare -- sounds --minecraft-dir '$mc_dir' --output '$out')"
    echo "  (cd '$tree' && JAVA_HOME='${jdk:-<JDK>}' '${cargo_bin:-cargo}' run --release -p horizon-prepare -- --check --output '$out')"
fi

if [ "$missing" -gt 0 ]; then
    echo "prepare-game.sh: $missing thing(s) missing. Nothing was done." >&2
    exit 78
fi

if [ "$dry" -eq 1 ]; then
    echo "prepare-game.sh: dry run, nothing was run."
    exit 0
fi

mkdir -p "$out"
(cd "$tree" && JAVA_HOME="$jdk" "$cargo_bin" run --release -p horizon-prepare -- \
    --prepare --server-jar "$server_jar" --minecraft-dir "$mc_dir" --output "$out")
# --prepare does not copy the sounds (about 380 MB), and --check fails without them.
(cd "$tree" && JAVA_HOME="$jdk" "$cargo_bin" run --release -p horizon-prepare -- \
    sounds --minecraft-dir "$mc_dir" --output "$out")
(cd "$tree" && JAVA_HOME="$jdk" "$cargo_bin" run --release -p horizon-prepare -- --check --output "$out")
echo "prepare-game.sh: the prepared files are in:"
echo "$out"
