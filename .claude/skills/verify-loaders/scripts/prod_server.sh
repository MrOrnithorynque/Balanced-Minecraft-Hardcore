#!/bin/bash
# Production test of built jars: a real server from the loader's official installer (not a Gradle dev run), the tree's
# release jar alone in mods/ (with Fabric API on Fabric), a boot on a fresh flat world, then the smoke bot. It proves
# what a dev run cannot: the jar loads and its mixins apply outside the dev environment (no refmap fallback), on a
# loader version other than the one the jar was built against (26.1 and 26.1.1 for a jar built on 26.1.2, say).
# usage: prod_server.sh <tree with built jars> <minecraft version> <loader> <loader version> [fabric api version]
#   forge:    <loader version> is Forge's own number (62.0.9), the installer is <mc>-<version> on maven.minecraftforge.net
#   neoforge: the full NeoForge version (26.1.0.19-beta), from maven.neoforged.net
#   fabric:   the Fabric Loader version (0.20.1) and the Fabric API version (0.145.1+26.1), from meta.fabricmc.net
# Installers and Fabric API are kept in ~/.cache/bmh-prod; servers in $BMH_VERIFY_OUT/prod-<mc>-<loader>-<version>.
# On 26.x a Fabric server refuses the vanilla bot (the jar has no verify hook), so Fabric there is a boot only.
# One line: RESULT up=<0|1> mixinErrors=<n> <SMOKE n/m or -> bmh=<lines>. Same locks and port as branch.sh.
set -u
[ $# -ge 4 ] || { echo "usage: prod_server.sh <tree> <minecraft version> <loader> <loader version> [fabric api version]" >&2; exit 2; }
T=$1; MC=$2; L=$3; V=$4; FAPI=${5:-}
SCRIPTS=$(cd "$(dirname "$0")" && pwd)
. "$SCRIPTS/lib.sh"
need_bots
JH=$(jdk_for "$MC") || exit 2
JAVA=$JH/bin/java
D=$OUTROOT/prod-$MC-$L-$V; DL=$HOME/.cache/bmh-prod; mkdir -p "$D" "$DL"
JAR=$(ls "$T/$L"/build/libs/bmh-"$L"-*.jar 2>/dev/null | grep -v -E -- '-(sources|dev)\.jar$' | head -1)
[ -n "$JAR" ] || { echo "no built $L jar in $T/$L/build/libs (gradlew build first)"; exit 2; }
say() { echo "[$MC $L $V] $*"; }
if [ ! -f "$D/.installed" ]; then
  case $L in
    neoforge) I=$DL/neoforge-$V-installer.jar
              [ -f "$I" ] || curl -sSf -m 300 -o "$I" "https://maven.neoforged.net/releases/net/neoforged/neoforge/$V/neoforge-$V-installer.jar" || exit 3
              ( cd "$D" && "$JAVA" -jar "$I" --install-server "$D" > "$D/install.log" 2>&1 ) || { say "install failed, see $D/install.log"; exit 3; } ;;
    forge)    I=$DL/forge-$MC-$V-installer.jar
              [ -f "$I" ] || curl -sSf -m 300 -o "$I" "https://maven.minecraftforge.net/net/minecraftforge/forge/$MC-$V/forge-$MC-$V-installer.jar" || exit 3
              ( cd "$D" && "$JAVA" -jar "$I" --installServer "$D" > "$D/install.log" 2>&1 ) || { say "install failed, see $D/install.log"; exit 3; } ;;
    fabric)   [ -n "$FAPI" ] || { echo "fabric needs the Fabric API version" >&2; exit 2; }
              curl -sSf -m 300 -o "$D/fabric-server-launch.jar" "https://meta.fabricmc.net/v2/versions/loader/$MC/$V/1.1.2/server/jar" || exit 3
              mkdir -p "$D/mods"
              F=$DL/fabric-api-$FAPI.jar
              [ -f "$F" ] || curl -sSf -m 300 -o "$F" "https://maven.fabricmc.net/net/fabricmc/fabric-api/fabric-api/$FAPI/fabric-api-$FAPI.jar" || exit 3
              cp "$F" "$D/mods/" ;;
    *) echo "loader must be forge, fabric or neoforge" >&2; exit 2 ;;
  esac
  touch "$D/.installed"
fi
mkdir -p "$D/mods"; rm -f "$D"/mods/bmh-*.jar; cp "$JAR" "$D/mods/"
rm -rf "$D/world"; write_server_properties "$D"
case $L in
  neoforge) ARGS=$(ls "$D"/libraries/net/neoforged/neoforge/*/unix_args.txt | head -1); CMD=("$JAVA" -Xmx3G "@$ARGS" nogui) ;;
  forge)    ARGS=$(ls "$D"/libraries/net/minecraftforge/forge/*/unix_args.txt 2>/dev/null | head -1)
            if [ -n "$ARGS" ]; then CMD=("$JAVA" -Xmx3G "@$ARGS" nogui); else CMD=("$JAVA" -Xmx3G -jar "$(ls "$D"/forge-*.jar | grep -v installer | head -1)" nogui); fi ;;
  fabric)   CMD=("$JAVA" -Xmx3G -jar "$D/fabric-server-launch.jar" nogui) ;;
esac
trap release EXIT
take_lock
LOG=$D/server.log
( cd "$D" && with_timeout 1200 "${CMD[@]}" ) > "$LOG" 2>&1 < /dev/null &
GPID=$!; UP=0; T0=$(date +%s)
while [ $(( $(date +%s) - T0 )) -lt 900 ]; do grep -q 'Done (' "$LOG" && { UP=1; break; }; kill -0 $GPID 2>/dev/null || break; sleep 2; done
SMOKE=-
case "$L/$MC" in fabric/26.*) BOT=0 ;; *) BOT=1 ;; esac
if [ $UP = 1 ] && [ $BOT = 1 ]; then
  LIST=$(python3 -I "$SCRIPTS/jarcheck.py" "$T" "$MC" "$JH" 2>/dev/null)
  SMOKE_ITEMS=$(echo "$LIST" | sed -n 's/^ITEMS //p') SMOKE_RECIPES=$(echo "$LIST" | sed -n 's/^RECIPES //p') \
    with_timeout 600 node "$SCRIPTS/smoke_bot.js" $PORT "$(bot_version "$MC")" > "$D/smoke.log" 2>&1
  SMOKE=$(grep '^SMOKE' "$D/smoke.log" | tail -1)
fi
[ $UP = 1 ] && stop_server
kill $GPID 2>/dev/null
release
ERR=$(grep -c -E 'Mixin apply.*failed|InvalidInjectionException|MixinTransformerError|InvalidAccessorException|ClassCastException' "$LOG")
say "RESULT up=$UP mixinErrors=$ERR $SMOKE bmh=$(grep -c 'BalancedMcHardcoreMain\] \(Setup complete\|Loading\)' "$LOG")"
grep -E 'Mixin apply.*failed|InvalidInjectionException|ClassCastException' "$LOG" | head -3 | cut -c1-300
