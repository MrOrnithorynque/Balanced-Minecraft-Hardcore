# Brief of a review agent (one area, one lens)

You review one ported area of the BMH mod for Horizon. Your prompt gives: the area, the lens (`parity`,
`semantics` or `both`), the tree T to read (an absolute path: the worktree of the area's port agent) and the file
to write. You change no file of the mod.

## Guards

Run `git rev-parse --show-toplevel` from T: it must print T. Use absolute paths. Read only: no edit of any file
except the one file named below, no git command that changes anything, no build, no game, no network. Do not run
`check.sh`, `test-slice.sh`, `play.sh` or `prepare-game.sh`. You may run `sh <T>/horizon/tools/test-logic.sh`
once.

Other paths (absolute, read only):

- HN = /Users/louisobenicherousse/Library/Caches/bmh-horizon/horizon-mods/native/target/horizon/fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca
  (Horizon's source at the pinned Nightly; `crates/horizon-client/src/game/native_hooks.rs` is large: search it,
  never read it whole; `docs/native-mods.md` documents every hook and helper).
- The Java mod: T/common/src/main/java/github/mrornithorynque/bmh/ and
  T/neoforge/src/main/java/github/mrornithorynque/bmh/neoforge/.

## Read first

T/docs/horizon/ARCHITECTURE.md (decisions, state rules), the rows of the area in T/docs/horizon/GAPS.md, the
area's log T/docs/horizon/log/<area>.md (its "Differences from the Java mod" and "Changes outside my files"), the
area's module files T/horizon/native/mixins/bmh/bmh_<area>*.rs and its logic file under
T/horizon/native/mixins/bmh/logic/src/, and the `hook_*.rs` helpers that call the area.

## The two lenses

- `parity` (with the Java mod). For each formula, constant, range, tick count, order of conditions, text key,
  sound, volume, pitch and message in the Java source of the area, find the Rust counterpart and compare. Check
  that each difference is listed in the area log and allowed by its GAPS.md row. A Java behaviour with no Rust
  counterpart and no listed difference is a finding.
- `semantics` (use of the host game). For each hook and helper the area uses, open its doc comment and its call
  site in HN. Check the side (the window's game, the server's game or both), whether a player is swapped in at
  that point, what the hook may change, the order of the calls in the `hook_*.rs` helper, and what happens for the
  single-player owner (T/docs/horizon/audit/6-single-player-owner.md). Check the state rules: no lock held across
  a call into the game, nothing lost by a restart that the Java mod keeps, no panic on data from the world, a
  file or a message. Check that the server checks all that a client sends.
- `both`: the two lenses, in that order, in one file.

Your task is adversarial: look for what is wrong, not for what is right. A compile and unit tests passed already,
so the faults that are left are faults of meaning.

## The evidence rule

Each finding carries its evidence on both sides, as `path:line` and the quoted line: ours, and the Java source or
the host game. Open the file at the line and copy the quote. A finding whose quote you cannot find at that place
is not a finding: drop it. Do not report style.

## What to write

One file: T/docs/horizon/review/<area>-<lens>.json (for `both`: `<area>-both.json`). Write it after your first
finding and update it as you go, major findings first, so that a cut still leaves findings. The shape:

    {
      "area": "<area>", "lens": "<lens>", "reviewed_commit": "<the sha of HEAD of T, or 'worktree'>",
      "status": "open: not fixed yet",
      "verdict": "<two sentences>",
      "findings": [
        { "severity": "major or minor", "file": "<path:line>", "what": "<the fault, one or two sentences>",
          "evidence": { "ours": "<path:line and quote>", "host": "<path:line and quote, or empty>",
                        "java": "<path:line and quote, or empty>" },
          "fix": "<what to change>" }
      ]
    }

`major`: a player sees a wrong behaviour, loses an item or a state, or a client can make the server do what it
must not. `minor`: the rest. Simplified Technical English. Never an em-dash or an en-dash.

## Result

The count of findings by severity, the three worst in one line each, and what you could not check.
