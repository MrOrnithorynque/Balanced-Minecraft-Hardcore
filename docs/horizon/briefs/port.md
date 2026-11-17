# Brief: port one area of BMH to Horizon

You port ONE area of the Minecraft mod "Balanced Minecraft Hardcore" (BMH, Java) to Horizon (Minecraft: Java
Edition 26.3 written again in Rust, with native mods that run at hook points). Nine other agents port the other
areas at the same time, each in its own git worktree. The bar is parity: the Horizon mod must behave as the Java
mod does, down to the numbers, the order of the conditions, the texts and the sounds.

## Where you are

You run in your own git worktree of the BMH repo. Call its root R:

    git rev-parse --show-toplevel            # this is R
    git branch --show-current                # must start with "worktree-": a throwaway branch of your own
    git merge-base --is-ancestor BASE HEAD   # exit 0: the base is right, skip the next line
    git reset --hard BASE                    # only when the line above failed

Run each one as its own plain command, with the literal sha in place of BASE: the app refuses `git -C`, `$(...)`
and compound lines in a worktree, as too complex to verify.

Run the reset only when the branch name starts with `worktree-` and R is not the coordinator's tree (below). It is
the one git command that changes history that you may run, once, before you have written anything.

Guard, after that: `ls "$R/horizon/native/mixins/bmh/horizon-mixin.toml"` must succeed, and R must NOT be
`/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/busy-visvesvaraya-4dfefc`
(that is the coordinator's tree: if you are in it, stop and say so). Use R in every path of the repo. Never write
outside R, except through the two build scripts below.

Other paths (absolute, read only):

- HN = /Users/louisobenicherousse/Library/Caches/bmh-horizon/horizon-mods/native/target/horizon/fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca
  Horizon's source at the Nightly the mod compiles against. Hook points are `native_<hook>` functions of `Game`,
  most in HN/crates/horizon-client/src/game/native_hooks.rs (large: search it, never read it whole), some in other
  HN/crates/horizon-client/src/game/native_*.rs files. The typed view of the same hooks and helpers (trait `Host`,
  events) is HN/crates/horizon-client/src/mod_api.rs. HN/docs/native-mods.md documents every hook and helper.
- MODS = /Users/louisobenicherousse/Library/Caches/bmh-horizon/horizon-mods
  Horizon's own native mods, real code to copy patterns from: MODS/native/smp (server logic, commands, storage),
  recipes and minimap (screens, HUD and world drawing), zoom, coords-hud, voicechat-server (mod messages).

## Read first, in this order

1. R/docs/horizon/ARCHITECTURE.md: the contract. Decisions, your files, how a hook reaches your area, the state
   rules, the module and helper rules. Follow it to the letter.
2. R/docs/horizon/HOOKS.md: every wired hook, its parameters, its side, which areas it calls and in which order.
3. R/docs/horizon/GAPS.md: the rows of your area (given in your prompt): tier, how each is built, what may differ.
4. The audit and verification sections for those rows: R/docs/horizon/audit/ (your prompt names the files). They
   hold a behaviour spec of the Java code and the Horizon mapping, with citations. They cite Horizon main's line
   numbers: find each function again in HN. Where an audit and its verification differ, the verification wins.
   R/docs/horizon/audit/6-single-player-owner.md explains which hooks fire for the single-player owner, and its
   traps: `server_players()` is empty inside a player hook; outside a player's own hook, `give_stack`,
   `give_levels` and `take_items` on that player can fail; `living_death` runs with no player swapped in.
5. The Java sources of your area (your prompt names them) under R/common/src/main/java/github/mrornithorynque/bmh/,
   and their NeoForge wiring in R/neoforge/src/main/java/github/mrornithorynque/bmh/neoforge/. The Java code is the
   truth for behaviour. An audit's summary can be wrong: when it matters, read the Java.

## What to write

- Your logic file R/horizon/native/mixins/bmh/logic/src/<file>.rs (crate `bmh_logic`): every formula, table,
  constant, state machine, text key and message layout of your area, as pure Rust with no game type. Each constant
  carries the Java `file:line` it comes from in a comment. Unit tests pin the Java values (compute a few cases by
  hand from the Java code and assert them), cover the edges (zero, caps, the order of the conditions) and, for your
  messages, round trips and truncated input (use the Reader and Writer of `wire.rs`).
- Your module file R/horizon/native/mixins/bmh/bmh_<area>.rs: the bodies of your `on_<hook>` functions. They read
  the game, call the logic, and carry out what it answers. Keep them thin.
- R/docs/horizon/log/<area>.md (new file, yours): a running log from your first step, updated after every
  milestone, good enough for a fresh agent to resume from it alone. Numbered steps, no clock times. It ends with:
  - "Differences from the Java mod": each one, with the reason and the GAPS.md row that allows it. A difference
    GAPS.md does not list needs a strong reason, written here.
  - "To check in the game": every assumption that only a running game can confirm (a hook fires at the moment you
    think, a helper works for the single-player owner from that hook, a command gives the toast), each with how to
    check it (the command to type, what to look at).
  - "Changes outside my files": every line you changed in a file you do not own (below), with the reason.

Files you do not own: the manifest `horizon-mixin.toml`, the `hook_*.rs` helpers, `bmh.rs`, `logic/src/lib.rs`,
`logic/src/wire.rs`, `logic/src/ids.rs`, `logic/Cargo.toml`, HOOKS.md, and every other area's files. You may change
them in your worktree only for one of these, as small as possible, and you list it: wire a hook your area needs
that is not wired yet (a new `hook_<hook>.rs`, its `[[injection]]`, its HOOKS.md row), add a binding your function
needs to an existing helper (then pass it to your function only), add an id or a `pub mod` line. Never change what
another area's function receives. You may add keys to R/horizon/content/assets/bmh/lang/en_us.json (and the same
key to the other lang files with the English text) when the Java mod had no key for a text you must show; list
them. Do not edit other content files: report what the content mod lacks.

## Rules of the port

- Parity first. Take every number, range, tick count, order of checks, text key, sound id, volume and pitch from
  the Java source. Do not round, simplify or "improve".
- Use tier N and tier F hooks and helpers only (stamped in the Nightly, or pending fragments compiled into it). For
  a row that GAPS.md rates X or G, build the substitute GAPS.md names (section 3) and nothing more: no injection
  into a function of the game that is not a hook point (decision D8).
- Before you rely on a hook or helper: read its doc comment AND its call site in HN. Check its side (window's game,
  server's game or both), whether a player is "swapped in" at that point, what it can change. The name alone lies.
  What you cannot settle by reading goes into "To check in the game".
- A mod message is the only way from server to client and back (decisions D6, D7). Your area's channel is
  `bmh:<area>`. Find how a mod sends and hears one in HN/docs/native-mods.md ("Mod messages") and in
  MODS/native/voicechat-server. The server checks everything a client sends.
- State: `SERVER_*` and `CLIENT_*` statics behind a `Mutex` or an atomic, never `thread_local!`, never a lock held
  across a call into the game. What must survive a restart goes on the player or the level (attachments).
- No `unwrap()`, `expect()`, indexing or arithmetic that can panic on data from the world, a file or a message.
- Warnings are errors. No dead code: a function exists when something calls it.
- Texts for players come from the lang files (translatable text), with the Java keys.
- Comments say what the code does and why, in Simplified Technical English: short sentences, active voice, simple
  present, no idioms. Never an em-dash or an en-dash, anywhere.

## Gates (both must pass before you report "done")

    sh "$R/horizon/tools/test-logic.sh"
    BMH_HORIZON_CACHE=/Users/louisobenicherousse/Library/Caches/bmh-horizon sh "$R/horizon/tools/check.sh"

The first runs the unit tests (seconds). The second compiles your worktree's bundle into Horizon; it waits for a
lock that the other agents share, so a run can take a few minutes: run it in the foreground, with a timeout of ten
minutes, and do not run it more often than you need. Compile early: wire one function through first, then grow.
No other cargo command, no gradle, no download, no git add, commit, stash or branch change. No sub-agents, no
background jobs. The shell is zsh: quote globs, never `echo ====`, `cd` only in a subshell.

The game runs on this machine since 2026-11-17 (`docs/horizon/TESTING.md`, `docs/horizon/log/first-run.md`).
Only an agent whose prompt gives it "game rights" may build the game and run it: two builds at once would swap
the binary under a running test. Without game rights, a compile and the unit tests are all the proof you can
give: say exactly that in your report, and never write that a feature "works". With game rights: run the
scenarios of `horizon/tools/test-slice.sh` that cover your area (`--only`), add a scenario for what you built or
fixed, always offscreen and with no game left running, and cite the log line, the reply or the screenshot for
each claim.

## Your structured result

- worktree: R.
- status: done (all rows built, both gates pass), partial, or blocked.
- gates: the last lines each gate printed.
- files: every file you changed, yours and others'.
- rows: for each GAPS.md row of your area: built / built as the substitute / not built, in one line.
- differences: the differences from the Java mod.
- game_checks: the number of entries in "To check in the game" and the three that matter most.
- open_questions.
