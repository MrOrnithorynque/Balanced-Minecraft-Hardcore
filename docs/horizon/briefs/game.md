# Brief of the game agent of a wave (the one agent with game rights)

You check in the real game the areas that a wave merged. You work in the coordinator's tree, on the branch
`horizon-26.3`, after the merges. Your prompt gives: the areas of the wave and the sha of the tip.

## Guards

- T = /Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/busy-visvesvaraya-4dfefc.
  `git rev-parse --show-toplevel` must print T and `git branch --show-current` must print `horizon-26.3`. Stop and
  say so if not. Use absolute paths. The shell is zsh: quote globs, `cd` only in a subshell.
- You have the game rights: you alone run `sh horizon/tools/check.sh build`, `sh horizon/tools/test-slice.sh` and
  the game. Before each build, look for the owner's own game: `ps -axo comm | grep horizon-client`. If one runs
  that you did not start, do not build: wait (a background command with an until loop) and try again.
- Games run offscreen, with no audio, in the scratch folders that `test-slice.sh` makes. Never leave a game
  running. Never read or touch `~/Library/Caches/bmh-horizon/play` (the owner's own play folder) nor
  `~/Library/Application Support/minecraft`. No download. No push. No `--trust-native-code`.
- No git add, commit, stash or branch change: the coordinator commits. No sub-agents.
- A wait of 50 seconds or more is a background command with an until loop. Cap every loop.

## Read first

T/.claude/CLAUDE.md, the skill T/.claude/skills/horizon-build-test/SKILL.md, T/docs/horizon/TESTING.md, the log of
each area of the wave (T/docs/horizon/log/<area>.md: "To check in the game" and the list of checks its port agent
asked for), and the scenarios that exist in T/horizon/tools/test-slice.sh (k and n are good examples: how the
script launches the game, reads a file, clicks, fails).

## What to do, for each area of the wave

1. Write the area's scenario in `test-slice.sh` (the next free letters), with one PASS or FAIL line for each
   check. A check reads a fact: a file, a line of the game's log, a reply of `horizon-ctl`, never a guess. Add
   the scenario to the script's header and to the table of TESTING.md ("The checks of test-slice.sh": what it
   does, what a pass proves, what it does not prove).
2. Build once, then run your scenarios with `--only` and `--no-build`.
3. When a check fails, find out who is wrong. If the test is wrong, fix the test. If the mod is wrong and the fix
   is inside the area's own files (its module, its logic file), fix the mod, add or change the unit test that pins
   it, run the two gates (`sh horizon/tools/test-logic.sh`, `sh horizon/tools/check.sh`), build, run again. If
   the fix needs a design choice or another area's file, do not fix: leave the check red and report it. Never
   weaken a check to make it pass.
4. Keep evidence: for what a person must see (a screen, an item, a bar, the debug screen), a screenshot under
   T/docs/horizon/log/<area>-shots/, scaled down to 1280 pixels wide (`sips -Z 1280`). Open each screenshot
   yourself and say in the log what it shows. A file that you cite must be on disk when you finish.
5. Write T/docs/horizon/log/game-checks-<wave>.md from your first step: numbered steps, no clock time that you
   did not read from `date`, a table of the defects that are left (area, severity, what happens, what is
   expected, the evidence, one command that shows it), and what was never tested (the sound is never tested: all
   runs are muted).

At the end run the whole script once (`sh horizon/tools/test-slice.sh --no-build`) and give its last line.

## Result

For each area: the checks that pass, the checks that fail and why, what you fixed in the mod (file and line), the
screenshots (paths), what a person must still look at by hand. Then the last line of the full run. Say plainly
what you did not test.
