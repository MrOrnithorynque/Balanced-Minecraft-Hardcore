# Part E self-test (throwaway, never shipped)

Headless test of the three echo abilities on a dedicated server, written because nobody can drive `runClient` in
this run. A fake `ServerPlayer` (built with a netty `EmbeddedChannel` as its connection) is added to the overworld
and driven by the server's own ticks. It checks, with exact numbers:

- Echo Strike: a full-strength Eternal Sword hit deals 11, queues one echo, 5 ticks later the nearest enemy loses
  half (5.5); a villager, a tamed wolf, an armor stand, a cow and an iron golem next to the target are spared; the
  echo does not echo; a weak swing queues nothing; an enemy 3.9 blocks away is echoed, one 6.4 blocks away is not.
- Axe smash: damage and fall distance for falls of 3, 10, 30, 1, 1.5 blocks and on the ground (base 18, vanilla crit
  x1.5, smash multiplier min(3, 1 + sqrt(0.16 * fall))).
- Echo Sense: 3 of 5 irons lit (stone and deepslate variants, 8 blocks in, 9 out, gold ignored), cyan glow, inset,
  tags, no ghost display after mining a lit ore, cooldown, gone after 5 s, the 64 cap on a coal wall, stale displays
  discarded.

It starts only if a file named `bmh-selftest` exists in the server's working directory (`<loader>/run/`), logs
`[E-SELFTEST] PASS|FAIL ...` lines, and halts the server when done (`FINISHED checks=44 failures=0`).

Use (main side, JDK 17; add `121` and JDK 21 for the 1.21.1 side):

```
sh docs/redesign/e-selftest/apply.sh <abs worktree>
BMH_PORT=25615 sh docs/redesign/e-selftest/runservers.sh <abs worktree> /opt/homebrew/opt/openjdk@17 <abs log prefix> selftest forge fabric neoforge
sh docs/redesign/e-selftest/remove.sh <abs worktree>
```

`runservers.sh` takes the machine-wide run lock once (`/private/tmp/bmh-runserver.lock`) and a Gradle slot per boot,
as in rule 3 of the brief. `rcon.py <port> <password> <command>` is the small RCON client it uses to stop a plain boot.
`E_SelfTestMixin` must never be in a commit: `bmh.mixins.json` is `required`, so every loader would crash on it.
