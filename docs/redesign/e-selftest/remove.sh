#!/bin/bash
# usage: remove.sh <abs worktree>   undoes apply.sh
WT="$1"
if [ "$(git -C "$WT" rev-parse --show-toplevel)" != "$WT" ]; then echo "tree guard failed"; exit 1; fi
J="$WT/common/src/main/java/github/mrornithorynque/bmh"
rm -rf "$J/selftest" "$J/mixin/E_SelfTestMixin.java"
perl -0pi -e 's/\s*"E_SelfTestMixin",//' "$WT/common/src/main/resources/bmh.mixins.json"
echo "removed from $WT"
