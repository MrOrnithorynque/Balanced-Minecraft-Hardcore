#!/bin/bash
# usage: apply.sh <abs worktree> [121]
# Copies the throwaway self-test into a worktree and registers its mixin. NEVER commit the result.
# The 1.21 argument drops the 1.20-only lines and uses the 1.21 cookie signature.
WT="$1"; SIDE="$2"
SD=$(dirname "$0")
if [ "$(git -C "$WT" rev-parse --show-toplevel)" != "$WT" ]; then echo "tree guard failed"; exit 1; fi
J="$WT/common/src/main/java/github/mrornithorynque/bmh"
mkdir -p "$J/selftest"
cp "$SD/EchoSelfTest.java.txt" "$J/selftest/EchoSelfTest.java"
cp "$SD/E_SelfTestMixin.java.txt" "$J/mixin/E_SelfTestMixin.java"
if [ "$SIDE" = "121" ]; then
  perl -pi -e 's/CommonListenerCookie\.createInitial\(profile\)/CommonListenerCookie.createInitial(profile, false)/; $_ = "" if /1\.20-ONLY/' "$J/selftest/EchoSelfTest.java"
fi
perl -pi -e 's/"E_ServerPlayerMixin"/"E_SelfTestMixin",\n        "E_ServerPlayerMixin"/ unless /E_SelfTestMixin/' "$WT/common/src/main/resources/bmh.mixins.json"
echo "applied to $WT"
