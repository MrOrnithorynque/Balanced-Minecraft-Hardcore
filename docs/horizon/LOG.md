# BMH on Horizon: build log

A running log of the Horizon port. Each step has a number, what was built and how it was checked. A fresh agent
can resume from this file and `ARCHITECTURE.md` alone. Clock times are left out on purpose.

## Step 1. The skeleton (integrator)

Built on 2026-11-11, on the Nightly of 2026-11-08 (`fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca`, hook API 411).

What exists now:

- `horizon/native/Cargo.toml`: a workspace with the logic crate as its only member (for `cargo test`).
- `horizon/native/compatibility.json`: game 26.3, `"hooks": 410` (the highest stamped number among the wired hooks,
  `gui_render_post`), the Horizon commit and the horizon-mixins commit `e8cd0aa38cffef49cb2e088ef716baa107148c02`.
- `horizon/native/mixins/bmh/logic/`: package `bmh_logic`, edition 2024, no dependency. One file per area (empty
  but for a comment), `ids.rs` (mod id, version 1.9.0 from gradle.properties, the nine item ids, `channel(area)`)
  and `wire.rs` (a byte `Writer` and `Reader`: u8, u32, i32, i64, f32, f64, bool, str with a 4096-byte cap; the
  reader gives None on short or bad input and never panics).
- `horizon/native/mixins/bmh/bmh.rs`: module `bmh` with `ID`, `VERSION` and `row()`, the Options > Mods row. This
  row is the only behaviour of the skeleton.
- `horizon/native/mixins/bmh/bmh_<area>.rs`: the ten area modules in `game.rs`. Each has one empty `on_<hook>`
  function per hook it uses, with a comment that names the GAPS.md row.
- Four area modules outside the game module, for hooks with no `Game`: `bmh_tools_lib.rs` (client library),
  `bmh_sky_render.rs` (horizon-client-render), `bmh_totem_world.rs` and `bmh_amulet_world.rs` (horizon-world).
  All four are `pub` modules. HOOKS.md, "Hooks with no Game", gives the rule.
- `horizon/native/mixins/bmh/hook_<hook>.rs`: 63 helpers, one per hook point. Each checks `native_on` first where
  the hook has a `Game`, then calls the areas in the order of HOOKS.md.
- `horizon/native/mixins/bmh/horizon-mixin.toml`: 15 modules, one crate, 63 injections (all `HEAD`, all by
  `target`). Injection ids are `hook_<hook>`, because horizon-mixins refuses an injection id that equals a module
  id (`E_DUPLICATE_ID` on `settings`).
- `horizon/tools/check.sh`, `horizon/tools/test-logic.sh`, `horizon/tools/content-check.sh`.
- `docs/horizon/HOOKS.md`.

How it was built: the probe bundle's shape, then one hook (`player_respawn_position`), then a batch of eleven with
every hook that lives outside `Game` (`living_incoming_damage`, `hopper_pickup_item`, `living_fall`,
`modify_default_components`, `world_drawn`, `render_needs`), then the client and network hooks, then all 63. Each
batch compiled with `check.sh`.

Lessons for the next agents:

- Module ids and injection ids share one namespace in horizon-mixins. Keep `hook_` on injection ids.
- A helper is pasted as a nested function inside the target function, so `crate::` is the target's crate. A
  helper for a hook in `horizon-world`, `horizon-client-render` or the client library can only call a module in
  that same crate.
- The `[[crate]]` (`bmh_logic`) is a dependency of `horizon-client` only. The four modules outside the game
  module cannot call it.
- `check.sh` takes about 10 to 50 seconds on a warm cache.

### Final output of check.sh

`BMH_HORIZON_CACHE=<scratchpad>/src sh horizon/tools/check.sh` (exit 0):

```
check.sh: bundle /Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/busy-visvesvaraya-4dfefc/horizon/native
check.sh: full log /private/tmp/claude-501/-Users-louisobenicherousse-Projects-Balanced-Minecraft-Hardcore--claude-worktrees-busy-visvesvaraya-4dfefc/98064f78-7ec0-4895-a546-21e1425d1f47/scratchpad/src/logs/check-20261111-160913-84090.log
bmh 1.9.0 on Horizon fdc87ae
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 11.01s
{"mods": ["bmh"], "horizon": "fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca", "check": "ok"}
```

### Final output of test-logic.sh (cargo test)

`sh horizon/tools/test-logic.sh` (exit 0):

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.00s
     Running unittests src/lib.rs (target/debug/deps/bmh_logic-42580a3fcd4d8704)

running 11 tests
test ids::tests::a_channel_has_the_mod_id ... ok
test wire::tests::a_bad_bool_gives_none ... ok
test ids::tests::every_item_is_in_the_mod_namespace ... ok
test wire::tests::a_short_read_of_each_type_gives_none ... ok
test wire::tests::a_string_that_is_not_utf8_gives_none ... ok
test wire::tests::every_primitive_round_trips ... ok
test wire::tests::remaining_counts_the_unread_bytes ... ok
test wire::tests::a_string_over_the_cap_gives_none ... ok
test wire::tests::every_truncation_gives_none_and_never_panics ... ok
test wire::tests::special_floats_round_trip_bit_for_bit ... ok
test wire::tests::the_writer_cuts_a_long_string_on_a_char_boundary ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests bmh_logic

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

### Final output of content-check.sh

`BMH_HORIZON_MOD_BIN=<scratchpad>/target-tools/debug/cargo-horizon-mod sh horizon/tools/content-check.sh` (exit 0):

```
content-check.sh: build in /Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/busy-visvesvaraya-4dfefc/horizon/content
Built ./target/horizon-mod/bmh.hmod
content-check.sh: check target/horizon-mod/bmh.hmod
bmh 1.9.0 (mod, API 0.2, both)
  731255 bytes, SHA-256 2ff906c26c2df965537892eedb21be5cc8fd8576bc85165b1c633bf1f2592069, unsigned
  may add blocks, items, mob effects and entity types to your worlds
  no code: content only, nothing to run
```

## Step 2. Review fixes (integrator)

Two reviewers read and ran the skeleton and gave five findings. Each one was checked against the files, the Java
mod and the Nightly's source first. All five are real. The fixes stay in `horizon/native` and `HOOKS.md`:
`ARCHITECTURE.md` is outside this step's files, so the new rules are in `HOOKS.md` (sections "Rules", "Contracts"
and "Content files").

1. Fixed. Message ownership and the journey and respawn messages. `wire.rs` now says what ARCHITECTURE.md says:
   every mod message lives in `wire.rs`. It holds `JOURNEY_CHANNEL` (`bmh:journey`, server to client) with
   `JourneySignal` (`Departure` 1, `Ready` 2, `Arrival { name }` 3) and `RESPAWN_CHANNEL` (`bmh:respawn`, client to
   server) with `RespawnRequest` (`Held` 1), each with `encode`, `decode` and tests (round trip, fixed tag bytes,
   truncation, extra bytes, unknown tag, channel names). HOOKS.md "Contracts" names giver and reader.
2. Fixed. The exile reaches homecoming. New `bmh_logic::contract::Exile` (`dimension`, `center`, `distance`, Java
   `RespawnPositionHandler.Exile`). `bmh_respawn::on_player_respawned` gives back `Option<Exile>` (the stub gives
   None). `bmh_homecoming::on_player_respawned` became `on_exile(game, player, &Exile)`, and
   `hook_player_respawned.rs` calls it only for `Some`. The record also needs the amulet points
   (HomecomingHandler.java:104-108), so `bmh_amulet::points(game, player) -> Vec<contract::AmuletPoint>` is a stub
   contract that `on_exile` calls. HOOKS.md: the `player_respawned` row and "Contracts".
3. Fixed. `ids.rs` has `ETERNAL_ITEMS` and `is_eternal(&str)`: the six classes that implement `IEternalItem`
   (sword, axe, pickaxe, bundle, `totem_of_eternity`, `sleeping_totem`), not the amulet, the Hollow Totem or the
   Echo Heart. The one-per-type key is the item id (EternalItemHandler.java:90 compares `getItem()`), so no second
   function. Test `the_eternal_items_are_the_java_ieternalitem_set`.
4. Fixed. HOOKS.md "Rules" has a "Side" rule: `game.native_window()` (native_hooks.rs:10726) tells the window's
   `Game` from a server's; world changes only when it is false; answers the same on both sides; the twelve `both`
   hooks listed; `living_fall` and `modify_default_components` answer alike on both sides because they cannot tell.
5. Fixed. HOOKS.md "Content files" gives each content file one owner: lang files to the integrator (areas list new
   keys in their LOG entry), each `minecraft:impossible` advancement to the area that awards it (from the Java
   `award(...)` calls), `sounds.json` and `assets/bmh/sounds/` to journey, everything else to the integrator.

Not changed: the 63 injections and the manifest. `cargo fmt` reformatted two test lines of `wire.rs`.

### Final output of check.sh (step 2)

`BMH_HORIZON_CACHE=<scratchpad>/src sh horizon/tools/check.sh` (exit 0):

```
check.sh: bundle /Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/busy-visvesvaraya-4dfefc/horizon/native
check.sh: full log /private/tmp/claude-501/-Users-louisobenicherousse-Projects-Balanced-Minecraft-Hardcore--claude-worktrees-busy-visvesvaraya-4dfefc/98064f78-7ec0-4895-a546-21e1425d1f47/scratchpad/src/logs/check-20261111-162557-96203.log
bmh 1.9.0 on Horizon fdc87ae
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 9.94s
{"mods": ["bmh"], "horizon": "fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca", "check": "ok"}
```

### Final output of test-logic.sh (step 2)

`sh horizon/tools/test-logic.sh` (exit 0):

```
   Compiling bmh_logic v1.9.0 (/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/busy-visvesvaraya-4dfefc/horizon/native/mixins/bmh/logic)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
     Running unittests src/lib.rs (target/debug/deps/bmh_logic-42580a3fcd4d8704)

running 17 tests
test ids::tests::every_item_is_in_the_mod_namespace ... ok
test wire::tests::a_bad_bool_gives_none ... ok
test ids::tests::a_channel_has_the_mod_id ... ok
test ids::tests::the_eternal_items_are_the_java_ieternalitem_set ... ok
test wire::tests::a_bad_journey_signal_gives_none ... ok
test wire::tests::a_short_read_of_each_type_gives_none ... ok
test wire::tests::a_string_that_is_not_utf8_gives_none ... ok
test wire::tests::a_string_over_the_cap_gives_none ... ok
test wire::tests::every_journey_signal_round_trips ... ok
test wire::tests::every_primitive_round_trips ... ok
test wire::tests::remaining_counts_the_unread_bytes ... ok
test wire::tests::every_truncation_gives_none_and_never_panics ... ok
test wire::tests::special_floats_round_trip_bit_for_bit ... ok
test wire::tests::the_channels_follow_the_area_names ... ok
test wire::tests::the_journey_tags_are_fixed ... ok
test wire::tests::the_respawn_request_round_trips_and_refuses_bad_bytes ... ok
test wire::tests::the_writer_cuts_a_long_string_on_a_char_boundary ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests bmh_logic

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### Final output of content-check.sh (step 2)

`BMH_HORIZON_MOD_BIN=<scratchpad>/target-tools/debug/cargo-horizon-mod sh horizon/tools/content-check.sh` (exit 0):

```
content-check.sh: build in /Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/busy-visvesvaraya-4dfefc/horizon/content
Built ./target/horizon-mod/bmh.hmod
content-check.sh: check target/horizon-mod/bmh.hmod
bmh 1.9.0 (mod, API 0.2, both)
  731255 bytes, SHA-256 2ff906c26c2df965537892eedb21be5cc8fd8576bc85165b1c633bf1f2592069, unsigned
  may add blocks, items, mob effects and entity types to your worlds
  no code: content only, nothing to run
```

## Step 3. The core slice: respawn, settings, journey (coordinator)

Three areas are ported, each by one agent in its own worktree, then merged here:

| Area | Commit | Unit tests | Its log |
|---|---|---|---|
| respawn | 63d0592 | 31 | `docs/horizon/log/respawn.md` |
| settings | 073c2d9 | 24 | `docs/horizon/log/settings.md` |
| journey | 2001fc3 | 30 | `docs/horizon/log/journey.md` |

On the merged tree at 2001fc3: `sh horizon/tools/test-logic.sh` gives 106 passed, 0 failed, and
`sh horizon/tools/check.sh` gives `"check": "ok"` on Horizon fdc87ae. That is all the proof there is: the game
has not run with the mod. Each area's log has its list "To check in the game".

Two independent reviewers read the respawn area at 63d0592 (parity with the Java mod; use of the host game).
Their findings are in `docs/horizon/review/respawn-parity.json` (5) and `respawn-semantics.json` (7). They are
open. The three that matter most:

1. Once the mod moves the respawn point, Horizon skips vanilla's bed and anchor resolution. A Nether anchor
   never loses a charge, an anchor with no charge still counts as a centre, and a missing bed gives no message.
2. At death, a bed or anchor in a chunk that is not loaded counts as "still there". A bed that is gone then
   still centres the exile.
3. A player who disconnects while dead and comes back gets no exile: the search and the pending death are
   dropped at logout.

`sh horizon/tools/check.sh build` builds the game and `horizon-ctl` with the bundle (Horizon fast profile). It ran
at 2001fc3: `Finished fast profile [optimized] target(s) in 3m 13s`, a `horizon-client` of 82 MB and `horizon-ctl`.
The binaries were not started: a world needs the prepared game files.

## Step 4. The game runs (2026-11-17, coordinator)

- The tools were made again in the cache `~/Library/Caches/bmh-horizon` (a restart of the Mac had erased the first
  session's scratchpad). The game files are prepared in `resources/26.3`: the preparer needed Mojang's own
  `26.3.json` saved in the Nightly's tree as `.local/minecraft/26.3/manifest.json` (the owner said "continue" on
  2026-11-17 after the request for that download, as he had for the server jar on 2026-11-11).
- First run of the mod in the game: `docs/horizon/log/first-run.md`. The content mod was refused (a content item
  may not set `minecraft:lodestone_tracker`): fixed, `docs/horizon/log/content.md`.
- Respawn fix round 1: thirteen of fourteen items fixed, one deferred as a listed difference
  (`docs/horizon/review/respawn-*.json`, `docs/horizon/log/respawn.md`).
- On the merged tree at f2e263e, run by the coordinator: 117 unit tests pass, the compile passes, and
  `sh horizon/tools/test-slice.sh` gives 24 passed, 0 failed, 0 skipped in the real game.
- `sh horizon/tools/play.sh` starts the built game in a window for a person.
- The branch has its own `.claude/CLAUDE.md` and two skills (`horizon-build-test`, `horizon-port-area`).

## Resume here

State on 2026-11-18, 03:10. Branch `horizon-26.3`, local only (never pushed: the owner did not ask).

Done: the content mod (loads in the game: nine items with textures, a recipe, the advancement tab, the sounds),
the skeleton (66 hooks), the area respawn (verified in the game), the areas settings and journey (ported; the
slice test passes with them; settings had one fix round; not reviewed yet), the test system, the harness of the branch.

The owner played the built game on 2026-11-17 and 2026-11-18 (`play.sh`). His server log shows the Java distance
formula in the real game: 23 332 blocks with no Eternal Item, 35 555 blocks from his bed with two. He reported:
F3 shows the coordinates (row C14 of GAPS.md, area eternal, not ported), the amulet does nothing (area amulet,
not ported), and "many features are missing".

### The run of 06:00 on 2026-11-18 (the owner's order of 02:49)

His order, in short: when the usage window is new, start again and finish all that is left; check that every
feature works; use the right models (many Sonnet agents, Opus where it is needed); start at 06:00.

Who has the run. One coordinator only. The lock is a folder:
`mkdir ~/Library/Caches/bmh-horizon/run-2026-11-18.lock`. If `mkdir` fails, another session has the run: write
one line and stop. The scheduled task of the desktop app `bmh-horizon-run-0600` fires at
06:06 and starts a new session: that session takes the lock and is the coordinator of the run. The session "Mod
port to Horizon Minecraft" (the coordinator since 2026-11-11) does not take the run. Its context is above 300 000
tokens, and on 2026-11-18 from 02:50 to 03:11 about 45 of its calls cost 29 points of the 5-hour window, with no
sub-agent at work (earlier, with a small context, a call cost about 0.4 point). So a coordinator keeps its context
small: agents do the reading, and the run goes to a new session when the context passes 200 000 tokens. The coordinator writes one
line per step in `~/Library/Caches/bmh-horizon/logs/run-2026-11-18.md`.

Before the first wave: `git branch --show-current` must print `horizon-26.3` and `git status --short` must be
empty.

Usage. Read the usage before each wave and after each agent that ran long. Stop cleanly at 60 % of the 5-hour
window. Stop when the week passes 40 % (all models) or 50 % (Fable). Measured on this run: 21 to 28 points of the
window per million sub-agent tokens. A wave of two areas with its reviews and game checks is planned at about 50
points (an estimate, not measured). So the list below needs about four windows. The owner was asked on
2026-11-18 whether the run may start again by itself at each new window. Until he answers, it stops at 60 % and
waits for him.

Waves, in this order. The procedure of each step is in the skill `horizon-port-area`.

- Wave 1: amulet (Opus; rows A1 to A6; it also sets the default `minecraft:lodestone_tracker` of the amulet from
  the native hook, see `docs/horizon/log/content.md`) and eternal (Sonnet; rows C11, C13, C14, E10 to E12; C14 is
  the hidden coordinates of F3 that the owner asked about).
- Wave 2: homecoming (Sonnet) and sky (Opus). Before it starts, fix the shape of what they read from the amulet
  in `logic/src/contract.rs` and in HOOKS.md "Contracts".
- Wave 3: totem (Sonnet), ritual (Sonnet), tools (Sonnet).
- Wave 4: the open defects of respawn and journey (`docs/horizon/log/game-checks-1.md`: check n5, and the frame
  of the arrival world before the black); the review of settings and journey with the two lenses, then the fix.
- Then: one full `sh horizon/tools/test-slice.sh`, bring `HOOKS.md` and `GAPS.md` up to date with what the game
  showed, and ask the owner three things: lines in `CHANGELOG.md`, when to push, when to remove the Java sources.

One wave, six steps:

1. Port. One agent per area, each in its own worktree, all at the same time. No game in this step: the two gates
   only (`check.sh` has a lock, so the compiles queue by themselves).
2. Review. Two lenses (parity on Sonnet, use of the host game on Opus) for amulet, eternal, sky, homecoming and
   totem. One Sonnet reviewer with both lenses for ritual and tools, to save usage.
3. Fix. One Sonnet agent per area, in the area's worktree.
4. Merge. The coordinator brings each area into `horizon-26.3` and runs the two gates.
5. Game. One Sonnet agent with game rights, in the coordinator's tree, after the merges: it builds the game, adds
   the scenarios of the wave's areas to `test-slice.sh`, runs them, fixes what fails inside those areas, and
   keeps its evidence on disk.
6. Close. The coordinator opens the evidence, runs the full `test-slice.sh`, commits, updates this section and
   reads the usage.

### Done on 2026-11-18 before the run

- The settings fix is in `horizon-26.3`: commit f1e4803 (the commit ba75fd3 of the session "Fix the Create World
  button and settings defects", brought in with its own message; the owner said "Oui, fusionne") and commit
  27fa133 (the coordinator's correction: a visit to Data Packs put the choice back to ON; checks k5e and k5f; see
  `docs/horizon/log/settings.md`, "The coordinator's check of fix round 1").
- On that tip, run by the coordinator: 122 unit tests pass, the compile passes, and the full
  `sh horizon/tools/test-slice.sh --no-build` gives 51 passed, 1 failed, 0 skipped. The one failure is n5
  (`immediate_respawn`: no exile and no journey), an open defect of the respawn area.
- The inspector of the second healing was cut by a restart of the app. Its verdicts are in
  `~/Library/Caches/bmh-horizon/logs/healing-2-inquisitor.md` (31 confirmed, 10 wrong or not provable). The wrong
  ones are corrected in `.claude/CLAUDE.md`, the two skills, `docs/horizon/briefs/port.md` (its commands used
  `$R`, which no command set) and the owner's global skill `coordinate-sessions`. Not known: what the inspector
  would have said about the lines it did not reach.
- The briefs of wave 1 are written: `docs/horizon/briefs/area-amulet.md`, `area-eternal.md` (the part of a port
  agent's prompt that is about its area), `review.md` (a review agent, one lens or both) and `game.md` (the one
  agent with game rights). A port agent's prompt is: read `briefs/port.md` and `briefs/area-<area>.md`, BASE.
  Waves 2 and 3 need their own `area-<area>.md`: write them from the table of the skill `horizon-port-area`.

### In flight

Nothing on 2026-11-18, 03:10. The tree is clean and the next step is wave 1.

The tools: `~/Library/Caches/bmh-horizon` holds `horizon-mods` (with `native/target`: the Nightly's tree, cargo's
cache, the built game), `target-tools/debug/cargo-horizon-mod` (`BMH_HORIZON_MOD_BIN`), `resources/26.3`,
`minecraft-server-26.3/server.jar`, `play/` (the owner's own play folder: tests never touch it), `scratch/` and
the logs. An erased cache loses Mojang's version file inside the Nightly's tree: see the skill
`horizon-build-test`.

Traps of this run:

- When the app starts the session again, this worktree is back on the session's own branch
  (`claude/horizon-minecraft-mod-port-0812eb`, the Java `main`), with `horizon/` as untracked build folders.
  Nothing is lost: run `git switch horizon-26.3`.
- A restart of the app or a sleep of the Mac kills the background jobs and the sub-agents of the session (seen
  three times on 2026-11-17 and 2026-11-18: a wait, a build, an inspector). An agent that runs long keeps its log
  on disk from its first step. After a restart, read the logs before you start anything again.
- Do not build the game while the owner plays: the build writes the binary that `play.sh` runs and takes every
  core. Look for a `horizon-client` process first.

Rules of the run that the owner gave: stop cleanly when the 5-hour usage window reaches 60 %; stop when the
week passes 40 % (all models) and 50 % (Fable), and ask him before going on (on 2026-11-12 both were passed and
the run stopped until he said "continue" in the new week); use Sonnet for bounded work and Opus for new
structural code and for the host-game review lens.

## To check in the game

Nothing here is done until a run in the game shows it. Each area adds its rows.

| Area | What | How |
|---|---|---|
