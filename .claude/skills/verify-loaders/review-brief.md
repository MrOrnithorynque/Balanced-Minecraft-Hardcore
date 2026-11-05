# Reviewer brief (the verify-loaders skill hands this to one Sonnet agent after a run)

Time box: about 40 tool calls. Work on the red cells first, then the screenshots, then the odd things.

You read the results of a verification run of Balanced Minecraft Hardcore (a Minecraft mod) and write a report. You
do not fix anything. You run no build, no server, no client, no git command that changes state, and you call no
network service. You write exactly one file: the report.

The run folder is the `OUT` path given with this brief. It holds:

- `summary.txt`: per cell (a Minecraft version) one line from `branch.sh` and one from `selftest.sh`. Green for
  `branch.sh` is `build=ok`, `jars` all passed, and per loader `up=1`, `amulet` and `smoke` all passed, `srvErr=0`,
  `tag=1`, `client=1`, `shots=2`, `cliErr=0` (`amulet=-` means the branch has no redesigned amulet). Green for
  `selftest.sh` is, per loader, `up=1`, `end=1`, `fail=0` and the two numbers of `selftest=a/b` equal.
- `<label>/`: per cell `steps.txt`, `build.log`, `jars.txt`, `server-<loader>.log`, `bot-<loader>.log`,
  `amulet-<loader>.log`, `client-<loader>.log`, `selftest-<loader>.log`, `selftest.txt`, and the screenshots
  `<loader>-bmh-smoke-*.png`.
- Possibly `death-<label>/` (frames and `journey.log` of `death_client.sh`).

Do this, in order:

1. Read `summary.txt`. List every cell and loader that is not green, and say which field is wrong.
2. For each failure, open the matching log and find the cause: a `FAIL` line of a bot (`bot-*.log`, `amulet-*.log`),
   a `[BMH-SELFTEST] FAIL` line (`selftest-*.log`: the check name and its detail after `|`), a `FAIL` line of
   `jars.txt`, the first `ERROR` or exception with a `github.mrornithorynque.bmh` frame in a server or client log.
   Copy the line exactly, with the file name and the line number (`grep -n`).
3. Look at every screenshot. Say what each one shows: is the sky drawn, is a line of stars visible at night, is there
   a HUD, an error screen, a black or corrupt frame. Compare the same shot across loaders and cells: name any that
   differs from the others.
4. List anything odd in a green cell too: a wall time far above its siblings in `steps.txt`, `ERROR` lines with a bmh
   frame, a selftest check that passed with a suspicious detail (a count of 0 where something should happen).

Known harmless lines (do not report them): `Couldn't parse element ... bmh:blocks/test_block`, `No key layers in
MapLike[{}]`, Forge's netty `Reflective setAccessible(true) disabled`, the add-mixin skill's refmap and `JAVA_17` lines.

Rules of evidence: every claim in your report quotes a line that exists in a file of the run folder, with
`file:line`. The session that started you greps each quote; a quote it cannot find throws your finding away. If you
cannot find the cause of a failure, write "cause not found in the logs" and list the files you read. Never guess.

Write the report to `OUT/review.md` as you go, not at the end: create it with the list of step 1, then append one
section per red cell and loader as soon as you have its cause, then the screenshot notes, then the odd things. A run
that is cut off still leaves its findings. Last, put a one-line verdict at the top (ALL GREEN, or the number of red
cells). Your last message repeats the verdict line and the path of the report.
