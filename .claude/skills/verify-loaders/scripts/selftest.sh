#!/bin/bash
# The server self-tests. It copies hook/selftest into common/src of a tree, with the MockPlayers copy for the Minecraft
# version. Per loader it boots a dedicated server on a fresh flat world with BMH_SELFTEST=1 and reads its [BMH-SELFTEST]
# lines. The server halts itself after the last test. The script never commits. It removes the hook at the end, also
# when it is interrupted. It runs no bot.
# With --local it changes the tree you name: it deletes <loader>/run/world and writes server.properties and eula.txt.
# usage: selftest.sh <origin branch or tag | --local=<tree>> <minecraft version> <label> [loaders...]
#   An origin branch or tag runs in the worktree verify-<label> at that ref (the one branch.sh uses). BMH_LOCAL=1 takes
#   the local branch of that name instead (no push needed).
#   --local=<tree> runs in that tree as it is on disk, uncommitted work included: no push needed.
#   loaders default to forge fabric, plus neoforge when the tree builds it (1.21.1, or neoforge_version in gradle.properties).
# Results in $BMH_VERIFY_OUT/<label>/: selftest-<loader>.log (the server log) and one line per loader in selftest.txt,
# plus one line for the cell in $BMH_VERIFY_OUT/summary.txt. selftest.txt lines:
#   <label> <sha> <loader> up=1 selftest=<passed>/<total> end=1 fail=0
# Green is up=1, end=1 and passed = total. Exit status: 0 all green, 1 a check failed or a server never ended, 2 usage.
set -u
[ $# -ge 3 ] || { echo "usage: selftest.sh <origin branch or tag | --local=<tree>> <minecraft version> <label> [loaders...]" >&2; exit 2; }
WHERE=$1; MCV=$2; LABEL=$3; shift 3
SCRIPTS=$(cd "$(dirname "$0")" && pwd)
. "$SCRIPTS/lib.sh"
JH=$(jdk_for "$MCV") || exit 2
# A port of its own (default 25638, RCON + 100): the bot scripts use 25618 to 25620.
PORT=${BMH_SELFTEST_PORT:-25638}; RPORT=$((PORT + 100))
OUT=$OUTROOT/$LABEL
BMH_SRC=common/src/main/java/github/mrornithorynque/bmh
MIXINS=common/src/main/resources/bmh.mixins.json
mkdir -p "$OUT"
say() { echo "$*" | tee -a "$OUT/steps.txt"; }

case "$WHERE" in
  --local=*)
    WT=${WHERE#--local=}
    [ "$(git -C "$WT" rev-parse --show-toplevel 2>/dev/null)" = "$WT" ] || { echo "not the root of a git tree: $WT" >&2; exit 2; }
    # +local: the tree has uncommitted changes (tracked or new files), the run folders aside
    SHA="$(git -C "$WT" rev-parse --short HEAD)$([ -n "$(git -C "$WT" status --porcelain -- . ':!*/run/*')" ] && echo '+local')"
    ;;
  *)
    WT=$REPO/.claude/worktrees/verify-$LABEL
    git -C "$REPO" fetch -q origin
    REF=$(ref_of "$WHERE") || { say "no origin/$WHERE, no tag $WHERE$([ "${BMH_LOCAL:-0}" = 1 ] && echo ", no local branch $WHERE")"; exit 2; }
    SHA=$(git -C "$REPO" rev-parse --short "$REF^{commit}")
    fresh_tree "$WT" "$REF"
    ;;
esac

# The MockPlayers copy for a Minecraft version (the signatures of placeNewPlayer and ServerPlayer change at 1.20.2 and
# 1.20.5, and 26.1 renames or moves most of what the tests call). Keep it the same as mock_variant in compile_check.sh.
case "$MCV" in
  1.20|1.20.1) MOCK=1.20.1 ;;
  1.20.*) MOCK=1.20.4 ;;
  26.*) MOCK=26.1 ;;
  *) MOCK=1.21.1 ;;
esac

# The hook goes in, and comes out whatever happens. A tree that already has a selftest folder is not ours to touch.
[ -e "$WT/$BMH_SRC/selftest" ] && { echo "$WT/$BMH_SRC/selftest exists: remove it first" >&2; exit 2; }
BACKUP=$(mktemp "${TMPDIR:-/tmp}/bmh-mixins.XXXXXX")
cp "$WT/$MIXINS" "$BACKUP"
unhook() {
  cp "$BACKUP" "$WT/$MIXINS" && rm -f "$BACKUP"
  rm -rf "$WT/$BMH_SRC/selftest"
  rm -f "$WT/$BMH_SRC/mixin/Z_SelfTestMixin.java"
}
trap 'release; unhook' EXIT
mkdir -p "$WT/$BMH_SRC/selftest"
find "$SCRIPTS/hook/selftest" -maxdepth 1 -name '*.java' ! -name 'Z_*' -exec cp {} "$WT/$BMH_SRC/selftest/" \;
cp "$SCRIPTS/hook/selftest/mock/MockPlayers_$MOCK.java" "$WT/$BMH_SRC/selftest/MockPlayers.java"
cp "$SCRIPTS/hook/selftest/Z_SelfTestMixin.java" "$WT/$BMH_SRC/mixin/Z_SelfTestMixin.java"
python3 -I - "$WT/$MIXINS" <<'PY'
import json, sys
p = sys.argv[1]; d = json.load(open(p)); d.setdefault("mixins", []).append("Z_SelfTestMixin")
open(p, "w").write(json.dumps(d, indent=4) + "\n")
PY

LOADERS="$*"
if [ -z "$LOADERS" ]; then
  LOADERS="forge fabric"
  case "$MCV" in 1.21*) LOADERS="$LOADERS neoforge" ;; *) grep -q '^neoforge_version=' "$WT/gradle.properties" && LOADERS="$LOADERS neoforge" ;; esac
fi
say "== selftest $LABEL: $WHERE at $SHA, Minecraft $MCV, loaders: $LOADERS"

GREEN=1
for L in $LOADERS; do
  take_lock; take_slot
  RUN="$WT/$L/run"; rm -rf "$RUN/world"
  write_server_properties "$RUN" spawn-monsters=false "motd=bmh self-test"
  LOG="$OUT/selftest-$L.log"
  ( cd "$WT" && BMH_SELFTEST=1 JAVA_HOME=$JH sh ./gradlew ":$L:runServer" --max-workers=2 ) > "$LOG" 2>&1 &
  GPID=$!; UP=0; END=0; BOOT_START=$(date +%s)
  # Up to 15 minutes for the first boot of a fresh worktree, then up to 10 minutes for the tests
  while [ $(( $(date +%s) - BOOT_START )) -lt 1500 ]; do
    [ $UP = 0 ] && grep -q 'Done (' "$LOG" 2>/dev/null && UP=1
    grep -q '\[BMH-SELFTEST\] END' "$LOG" 2>/dev/null && { END=1; break; }
    kill -0 $GPID 2>/dev/null || break
    [ $UP = 0 ] && [ $(( $(date +%s) - BOOT_START )) -gt 900 ] && break
    sleep 2
  done
  # The server halts itself after END: give Gradle up to 90 s to end, then release kills what is left
  for i in $(seq 1 45); do kill -0 $GPID 2>/dev/null || break; sleep 2; done
  release
  RES=$(grep -o '\[BMH-SELFTEST\] END [0-9]*/[0-9]*' "$LOG" | tail -1 | awk '{print $3}')
  FAILS=$(grep -c '\[BMH-SELFTEST\] FAIL' "$LOG")
  say "$LABEL $SHA $L up=$UP selftest=${RES:--} end=$END fail=$FAILS"
  echo "$LABEL $SHA $L up=$UP selftest=${RES:--} end=$END fail=$FAILS" >> "$OUT/selftest.txt"
  grep '\[BMH-SELFTEST\] FAIL' "$LOG" | sed 's/^.*\[BMH-SELFTEST\] /  /' | tee -a "$OUT/steps.txt"
  [ $UP = 1 ] && [ $END = 1 ] && [ "$FAILS" = 0 ] || GREEN=0
  CELLS="${CELLS:-} | $L up=$UP selftest=${RES:--} end=$END fail=$FAILS"
done
echo "$LABEL $SHA selftest${CELLS:-}" >> "$OUTROOT/summary.txt"
[ $GREEN = 1 ]
