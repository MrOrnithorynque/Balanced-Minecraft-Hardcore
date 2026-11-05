#!/bin/bash
# Fast check that needs no Gradle slot: javac -proc:none on the whole common/src/main/java tree against each Loom merged
# jar (vanilla Minecraft with Mojang names) plus the libraries it needs from the Gradle cache. It proves that every vanilla
# signature the shared code uses exists on that version. It does not prove that a mixin still hits its target, nor the
# loaders' own code: the real build does.
# usage: compile_check.sh [--selftest] [<minecraft version>...]
#   --selftest also compiles the self-test hook (hook/selftest, with the MockPlayers copy for each version), as selftest.sh
#   copies it into common/src.
#   With no version it reads minecraft_version in $TREE/gradle.properties: a value starting with 1.21 gives 1.21.1, anything
#   else gives 1.20 1.20.1 1.20.2 1.20.3 1.20.4.
#   BMH_TREE=<worktree>   the checkout to compile (default: the one that holds this script)
#   BMH_JDK17, BMH_JDK21  JDK homes (default Homebrew's openjdk@17 and openjdk@21; 1.21.x uses 21)
# A Loom jar is in ~/.gradle only after some branch of that version was built once with Gradle. Run it with bash: a file
# list kept in a zsh variable does not split, so the sources go through a file here.
# Exit status: 0 all compile, 1 a version fails, 2 a jar, a JDK or a library is missing.
set -u
SCRIPTS=$(cd "$(dirname "$0")" && pwd)
SELFTEST=0
[ "${1:-}" = --selftest ] && { SELFTEST=1; shift; }
TREE=${BMH_TREE:-$(git -C "$SCRIPTS" rev-parse --show-toplevel)}
GH=${GRADLE_USER_HOME:-$HOME/.gradle}
LIBS=$GH/caches/modules-2/files-2.1
LOOM=$GH/caches/fabric-loom/minecraftMaven/net/minecraft/minecraft-merged
SRC=$TREE/common/src/main/java
[ -d "$SRC" ] || { echo "no $SRC" >&2; exit 2; }
if [ $# -eq 0 ]; then
  MCV=$(sed -n 's/^minecraft_version=//p' "$TREE/gradle.properties" 2>/dev/null | head -1 | tr -d '[:space:]')
  case "$MCV" in
    1.21*) set -- 1.21.1 ;;
    *) set -- 1.20 1.20.1 1.20.2 1.20.3 1.20.4 ;;
  esac
fi
WORK=$(mktemp -d "${TMPDIR:-/tmp}/bmh-compile-check.XXXXXX")
trap 'rm -rf "$WORK"' EXIT
find "$SRC" -name '*.java' | sort | sed 's/.*/"&"/' > "$WORK/sources.txt"
FAILED=0; MISSING=0

# The library versions of a Minecraft version, one "group:artifact version" line each. Loom writes them in
# minecraft-info.json (mojang_minecraft_info.json in newer Loom). The Gradle cache is shared with every other project, and a
# newer Minecraft puts newer jars there: authlib 10 of Minecraft 26.3 broke every version on 2026-11-05.
pins() { # <minecraft version> <output file>
  local dir=$GH/caches/fabric-loom/$1 info
  for info in "$dir/minecraft-info.json" "$dir/mojang_minecraft_info.json"; do
    [ -f "$info" ] && break
  done
  python3 -I -c 'import json, sys
for l in json.load(open(sys.argv[1])).get("libraries", []):
    p = l["name"].split(":")
    if len(p) == 3:
        print(p[0] + ":" + p[1], p[2])' "$info" > "$2" 2>/dev/null \
    || { echo "$1: no Loom library list in $dir, the newest jars are used" >&2; : > "$2"; }
}
# The jar of a library: the version that the pins file lists. A library that Minecraft does not list (Mixin, jsr305,
# annotations) takes the newest jar in the cache. The path holds a hash folder, so sort on the file name.
lib() { # <group> <artifact> <pins file> <minecraft version>
  local want jar
  want=$(awk -v k="$1:$2" '$1 == k { print $2; exit }' "$3")
  if [ -n "$want" ]; then
    jar=$(find "$LIBS/$1/$2/$want" -name "$2-$want.jar" 2>/dev/null | head -1)
    [ -n "$jar" ] && { echo "$jar"; return; }
    echo "$4: $1:$2:$want is not in the Gradle cache, the newest jar is used" >&2
  fi
  find "$LIBS/$1/$2" -name "$2-[0-9]*.jar" ! -name '*-sources.jar' ! -name '*-javadoc.jar' ! -name '*-processor.jar' ! -name '*natives*' 2>/dev/null \
    | awk -F/ '{ print $NF "\t" $0 }' | sort -V | tail -1 | cut -f2
}
# The MockPlayers copy for a Minecraft version (the signatures of placeNewPlayer and ServerPlayer change at 1.20.2 and 1.20.5)
mock_variant() {
  case "$1" in
    1.20|1.20.1) echo 1.20.1 ;;
    1.20.*) echo 1.20.4 ;;
    *) echo 1.21.1 ;;
  esac
}
echo "compile_check: $(wc -l < "$WORK/sources.txt" | tr -d ' ') sources of $TREE"
for v in "$@"; do
  case "$v" in
    1.21*) JDK=${BMH_JDK21:-/opt/homebrew/opt/openjdk@21}; REL=21 ;;
    *) JDK=${BMH_JDK17:-/opt/homebrew/opt/openjdk@17}; REL=17 ;;
  esac
  CP=""
  pins "$v" "$WORK/pins-$v.txt"
  for coords in "org.spongepowered mixin" "org.joml joml" "org.slf4j slf4j-api" "com.mojang datafixerupper" "com.mojang brigadier" \
    "com.google.guava guava" "it.unimi.dsi fastutil" "com.google.code.findbugs jsr305" "com.mojang logging" "com.mojang authlib" \
    "com.google.code.gson gson" "org.apache.logging.log4j log4j-api" "org.apache.commons commons-lang3" "io.netty netty-buffer" \
    "io.netty netty-common" "io.netty netty-transport" "io.netty netty-codec" "io.netty netty-handler" "org.lwjgl lwjgl" "org.jetbrains annotations"; do
    read -r group artifact <<< "$coords"
    jar=$(lib "$group" "$artifact" "$WORK/pins-$v.txt" "$v")
    if [ -n "$jar" ]; then CP="$CP:$jar"; else echo "$v: missing library $group:$artifact (build any branch once, Gradle downloads it)" >&2; MISSING=1; fi
  done
  JAR=$(find "$LOOM" -mindepth 2 -maxdepth 2 -path "$LOOM/$v-loom.mappings.*" -name 'minecraft-merged-*.jar' ! -name '*sources*' 2>/dev/null | sort | head -1)
  if [ -z "$JAR" ]; then echo "$v: no Loom merged jar (build a $v branch once with Gradle)"; MISSING=1; continue; fi
  if [ ! -x "$JDK/bin/javac" ]; then echo "$v: no javac in $JDK"; MISSING=1; continue; fi
  mkdir -p "$WORK/out-$v"
  cp "$WORK/sources.txt" "$WORK/sources-$v.txt"
  if [ $SELFTEST = 1 ]; then
    mkdir -p "$WORK/mock-$v"; cp "$SCRIPTS/hook/selftest/mock/MockPlayers_$(mock_variant "$v").java" "$WORK/mock-$v/MockPlayers.java"
    find "$SCRIPTS/hook/selftest" -maxdepth 1 -name '*.java' | sed 's/.*/"&"/' >> "$WORK/sources-$v.txt"
    echo "\"$WORK/mock-$v/MockPlayers.java\"" >> "$WORK/sources-$v.txt"
  fi
  if "$JDK/bin/javac" --release "$REL" -proc:none -Xlint:none -nowarn -d "$WORK/out-$v" -cp "$JAR$CP" @"$WORK/sources-$v.txt" > "$WORK/javac-$v.log" 2>&1; then
    echo "$v: OK ($(find "$WORK/out-$v" -name '*.class' | wc -l | tr -d ' ') classes)"
  else
    echo "$v: FAILED"; grep -A2 'error:' "$WORK/javac-$v.log" | head -24; FAILED=1
  fi
done
[ $FAILED -eq 0 ] || exit 1
[ $MISSING -eq 0 ] || exit 2
