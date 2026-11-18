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

## Step 5. Wave 1: eternal and amulet (2026-11-18, the scheduled session)

Each area: one port agent in its own worktree, two reviewers (parity on Sonnet, use of the host game on Opus), one
fix agent, then the merge. The counts of the fix round come from the field `round1` of the review files.

| Area | Commit | Review findings | Fix round 1 (`round1`) | Its log |
|---|---|---|---|---|
| eternal | 86d2dbd | 1 major, 5 minor | 3 fixed (one in part: the empty bundle), 3 not fixed (listed as differences; one is a content change for the integrator) | `docs/horizon/log/eternal.md` |
| amulet | c260f94 | 0 major, 5 minor | 4 fixed, 1 not fixed (the command level, kept on purpose: GAPS.md A5) | `docs/horizon/log/amulet.md` |

- The major finding of eternal: the kept Eternal Items waited in a list of the whole process, so a single-player
  owner could get them in two worlds. Fixed: the saved player data is the source of truth.
- On the merged tree: 191 unit tests pass and the compile passes.
- In the game (`docs/horizon/log/game-checks-2.md`): the game agent wrote the scenarios `p` (eternal, 20 checks)
  and `q` (amulet, 16 checks) in `test-slice.sh`. It changed no file of the mod. The coordinator then ran the full
  script on that tree: 87 passed, 1 failed, 0 skipped (`~/Library/Caches/bmh-horizon/logs/coordinator-slice-2.log`).
  The one failure is `n5`, as before.
- The coordinator opened two of the ten screenshots: `eternal-shots/c14-f3-survival.png` (F3 in survival: no XYZ
  line, and the chat line says that `reduced_debug_info` is true) and `amulet-shots/naming-screen-bed.png` (the
  naming screen on a bed, "Point 1 of 5", the amulet in the hand). The other eight are on disk and were not opened
  by the coordinator.
- The two things that the owner reported have their checks in the game: F3 hides the coordinates in survival
  (p1 to p4), and the amulet reads the stars, opens its screens and keeps its points (q1 to q16). The star lines
  in the sky belong to the sky area, which is not ported.

## Waves 2 and 3: homecoming, ritual, totem, tools, sky (2026-11-18, the second coordinator)

Same method as step 5: one port agent per area in its own worktree, reviewers, one fix agent, then the merge. Before
the port of homecoming and sky, an Opus helper fixed the contract that both read (commit 9773ae7): the points of the
amulet come from `bmh_amulet::points` inside `player_respawned`, and the sky reading travels as `wire::SkySignal` on
the message `bmh:sky` (HOOKS.md, "Contracts"). The areas are listed in commit order. The unit test count is the one
on the merged tree. The counts of the fix round come from the review files and the area logs.

## Step 6. Homecoming (54a6f86)

213 unit tests, compile passes. Review: parity 1 minor (no record kept when no game time can be read), semantics 0
major and 1 minor (the bell queue is not cleaned at logout); the five load-bearing claims held in the Nightly's
tree. Fix round 1: 2 of 2 fixed. Integrator change: `hook_player_logged_out.rs` and the HOOKS.md row. Log:
`docs/horizon/log/homecoming.md`.

## Step 7. Ritual (344a366)

243 unit tests, compile passes. One Sonnet reviewer with both lenses: 1 major (the window start hook never cancels, so
a refused start keeps the raised totem while the key is held), 4 minor (no `use_cooldown` in `sleeping_totem.json`, no
`ITEM_USED` stat, no BOW pose, a hit rule wider than `hurtTime`). The six claims held. Fix round 1: the major is
fixed and `use_cooldown` (1.0 s) is added to `sleeping_totem.json`, so 2 fixed; the 3 parity minors stay as
differences (T7). Integrator change: one call in `hook_living_incoming_damage.rs` and the HOOKS.md row. T10 (the
glint while it charges) is not built (tier X). Log: `docs/horizon/log/ritual.md`.

## Step 8. Totem (dfe0b38)

266 unit tests, compile passes. Review: parity 1 major (E6: a gold repair gives no result with a high prior work
penalty) and 2 minor (`remove_totem` takes the main hand first; hoppers refuse every totem pickup); semantics 1 major
(a `living_death` cancel leaves the body dead: from reading the Nightly only) and 2 minor (a pickup in the death tick;
the ACTIVE flag is never cleared). Fix round 1 (Opus): E6, `remove_totem` and the ACTIVE flag fixed (3), the pickup in
the death tick refuted (1), the hopper pickup not fixed (no helper; T13), and the dead-body fault confirmed in the
Nightly and not fixable from the mod (it needs a Horizon change). No integrator file changed. Log:
`docs/horizon/log/totem.md`.

## Step 9. Tools (ca5a2a2)

314 unit tests, compile passes. One Opus reviewer with both lenses: parity 1 major (the Efficiency order of E4) and 5
minor; semantics 3 major (E4 client and server disagree; the E7 sweep empties other dimensions; the E4 worn flag
comes from any drawn stack) and 1 minor. The first claim was partly refuted: the hook `break_speed` is needed, but the
client guess was wrong. Fix round 1 (Opus): 9 of 10 fixed, PvP kept as a difference (the hooks hear mob hits only).
Integrator changes: new `hook_break_speed.rs`, one `[[injection]]` in `horizon-mixin.toml`, `hook_player_died.rs` and
the HOOKS.md rows. The `bmh:tools` message lives in `tools.rs`, not in `wire.rs` (open, Resume here). Log:
`docs/horizon/log/tools.md`.

## Step 10. Sky (6cd1e3e)

345 unit tests, compile passes, the WGSL validated with naga 29, the Java `StarField`, `StarChain` and `StarTrace`
pinned in tests. Review: parity 0 major and 2 minor (the spyglass stamp uses the server-fed game time; a world change
also drops the pending reading); semantics 2 major (the pipeline is built outside a wgpu error scope, so a failure
panics the game; the spyglass name draws over the pause menu) and 4 minor (frame generation blink, splitscreen
statics, HDR alpha, an old device kept). Claims 1, 2, 3 and 6 held; 4 and 5 were partly refuted. Fix round 1 (Opus):
6 of 8 fixed, 2 kept as differences (frame generation, splitscreen). Integrator change: one line in `hook_frame.rs`
and the HOOKS.md frame row. Log: `docs/horizon/log/sky.md`.

## Step 11. Game checks (8cda691, a2ec127)

None of the five areas had run in the game when it was merged. One Sonnet game agent ran in two parts (logs
`docs/horizon/log/game-checks-3.md` and `game-checks-4.md`); it added the scenarios t, u, r, s and v to
`test-slice.sh`.

- Homecoming (u1 to u4) pass. Totem t6 to t10 pass (the Totem of Undying first, the second totem dropped, the anvil
  awakening, E6, `/kill`). Totem t1 to t5 fail: `living_death` is never called for the single-player owner, so the
  save does not happen. The game log has no call, and the Nightly's only callers are `creatures.rs:1569` (mobs) and
  `lan_actions.rs:4309` (`guest_die`, a LAN guest). The dead flag after a cancel for a LAN guest is not tested.
- Tools r0 to r7 pass (the Java numbers for the dig time, the wear at death, the rule off and creative). Ritual s1 to
  s6 pass. Sky v1 to v3 pass. The coordinator opened `sky-shots/v4-reading-toward-point.png`: a violet chain of stars
  over the night sky. The spyglass name was not seen in two runs (v5).
- `m2` went red after scenario u: a test fault (a stale spawn point), fixed in the script, no change to the mod.
- Full run at a2ec127: 115 passed, 6 failed, 0 skipped. The six reds are t1 to t5 (the totem save) and n5
  (`immediate_respawn`, known since wave 1).

## Step 12. Review of settings and journey (522a84f, 9e847b8)

Four reviewers read the tree at cd2e53f. Settings parity (Sonnet): 0 major, 2 minor. Settings semantics (Opus): 2 major,
1 minor. Journey parity (Sonnet): 1 major, 2 minor. Journey semantics (Opus): 2 major, 4 minor. Files:
`docs/horizon/review/{settings,journey}-{parity,semantics}.json`.

- Settings semantics: the Java rules are read too late on a dedicated server (the save runs first), and `/bmh rule`
  takes its level from the command root, so it also blocked `/bmh point`.
- Journey semantics diagnosed two open defects. Check n5: the client respawns before `Departure` reaches it, the server
  cannot hold `ClientCommand`, and the search had not ended, so the player got the vanilla point. The arrival frame: the
  black started only at `Arrival`.

## Step 13. Fix round 1 of settings and journey, with n5 (22d804d, e3320a2)

Both fixes started from BASE 9e847b8, each in its own worktree, and were merged one after the other.

- Settings (Sonnet), merged as 22d804d. The Java rules are read at the Overworld level load, before the dedicated
  server's save (new hook `level_load`). `/bmh rule` refuses a player below level 2 by itself (`run_rule_command` in
  `bmh_settings.rs`, rule `may_run_rule`, tested at 0, 1, 2 and 4), so `/bmh point` is back at level 0, as in Java. The
  console and RCON can run `/bmh rule` (new hook `perform_console_command`). Not fixed: Re-Create (the host gives no
  flag, so a re-created world starts with every rule ON). The one-time import of Java rules stays, as a listed difference.
- Journey with respawn n5 (Opus), merged as e3320a2. An early respawn keeps the search, the server moves the player at its
  end and then runs the arrival steps once. A `Departure` that comes while the world shows starts a black journey. The
  arm belongs to one death. A screen over the death screen keeps the hold. A press in the first second is ignored. Not
  fixed: "any Other screen counts as the loading screen" (the host has no kind for it; journey difference 15).
- Gates after both merges: 353 unit tests pass, `check.sh` ok.

## Step 14. Game checks of wave 4 (a03e0ab)

One Sonnet game agent (game rights) added the scenarios w (journey and early respawn), x (a LAN guest below operator
level) and y (Java rules import) to `test-slice.sh`. Log: `docs/horizon/log/game-checks-5.md`.

- n5 passes. The arrival world no longer shows before the black. One frame of the level-loading screen still shows
  (w2 sheet, frame 3 at 0.4 s; the coordinator read the sheet, and the agent's log said black before it was corrected).
- Pass: a stale arm, a screen over the death screen, an early press. A LAN guest gets the refusal for `/bmh rule` and can
  save a point. The rules of a Java world are imported (single player).
- New defect w1d (major): with `immediate_respawn` and an old exile record, the early respawn at the old home fired a
  homecoming for a death.

## Step 15. The w1d fix (2fcfaa1)

One Sonnet agent with game rights. The homecoming tick skips a player while an early respawn waits for its landing
(`bmh_respawn::early_pending`, rule `may_check` in `logic/src/homecoming.rs`, with a unit test). 354 unit tests pass,
`check.sh` ok, `--only w` gives 9 of 9 in three runs, `--only u,n` gives 9 of 9. Full run: 129 passed, 6 failed. The
reds are t1 to t5 (they wait for the owner) and w1c (timing: 2 black frames at a capture cadence of 0.18 s under a full
load; it passes alone, and it was not weakened). The coordinator opened `w1d-fixed-sheet.png`: no Home title.

Also seen, not investigated: the ritual checks s2, s4, s5 and s6 failed once in one full run and passed alone and in
the next full run.

## Resume here

State on 2026-11-18, about 22:40. Branch `horizon-26.3`, local only (never pushed: the owner did not ask). HEAD: see
git log.

Done: all ten areas are ported, reviewed and fixed (respawn, settings, journey, eternal, amulet, homecoming, ritual,
totem, tools, sky). Checked in the game: every area as listed in steps 5, 11, 14 and 15, and the totem except its
save. The full `test-slice.sh` gives 129 passed, 6 failed (t1 to t5 and the timing flake w1c). 354 unit tests pass.

Open, in order:

a. The Totem of Eternity save (t1 to t5). `living_death` is never called for the single-player owner (callers:
   `creatures.rs:1569` for mobs, `guest_die` for LAN guests), and for a LAN guest a cancel may leave the body's dead
   flag set (not tested). This needs a Horizon change. The owner has to decide: inject into `guest_die` (breaks
   D8), ask Horizon for a fix or a hook, or accept the difference.
b. New after wave 4. The flakes: w1c (capture cadence under a full load; the game agent already lowered its own new threshold from 5 to 3 black frames before the first commit, and a full run then saw 2) and the ritual checks s2, s4, s5, s6 (failed
   once in one full run). One frame of the level-loading screen still shows before the black. Re-Create gives every
   rule ON. No tool runs a dedicated server, so the Java rules import on a dedicated server and the console `/bmh rule`
   are not checked in a game. The first-frame signal on a slow link is not tested.
c. Open in the game: the spyglass name (not seen in v5), E4 for a LAN guest or a dedicated-server client, echo,
   smash and glow with mobs, the bell and every sound (all runs are muted), a LAN guest for every area.
d. GAPS.md lines that the fix agents proposed and nobody wrote yet (do not edit GAPS.md before the owner agrees):
   ritual T7 (three lines: the `ITEM_USED` stat, the third person BOW pose, the hit rule wider than `hurtTime`);
   tools E7 and E8 (PvP: no echo, no smash and no halved hit against a player) and the other differences of
   `tools.md`; sky V1 (frame generation: the reading blinks at the display rate) and a line for all areas
   (splitscreen is not supported); totem T13 (a hopper does not pick up a dropped totem: it needs a ground pickup
   helper). GAPS.md A5 is now stale: `/bmh point` is at level 0 since step 13. Wave 4 left new differences to propose:
   journey difference 15 (any Other screen counts as the loading screen), settings Re-Create (a re-created world
   starts with every rule ON) and the one-time import of Java rules.
e. Integrator cleanups: the `bmh:tools` message lives in `tools.rs`, not in `wire.rs` (D6); totem and ritual each
   copy a private helper of another area (`drop_stack` of eternal, `sky` of amulet); `horizon-ctl xp N` shows a level
   that `native_acting_player().xp_level` does not read, while `xp add @s N levels` does (seen by the ritual check).
f. Three questions for the owner: lines in `CHANGELOG.md`, when to push, when to remove the Java sources.

Usage at the close of wave 4: 5-hour 11 %, week 40 %, Fable 47 %. The owner raised the weekly stop to 70 % for this run ("go to
70"). The 5-hour stop stays at 60 %.

### The next run

Who has the run. One coordinator only, and a new session: the coordinator of this run ended above 220 000 tokens
of context. The lock is a folder with the date of the run: `mkdir ~/Library/Caches/bmh-horizon/run-<date>.lock`.
If `mkdir` fails, another session has the run: write one line and stop. The lock of 2026-11-18 stays in the
cache; its file `owner` says that the run stopped. The prompt of the scheduled task `bmh-horizon-run-0600` names
the lock of 2026-11-18, so a later firing of that task stops at the lock: give it the new date first. The
coordinator writes one line per step in `~/Library/Caches/bmh-horizon/logs/run-<date>.md` (this run:
`run-2026-11-18.md`).

Before the first wave: `git branch --show-current` must print `horizon-26.3` and `git status --short` must be
empty. Run the two gates.

Usage. Read the usage before each wave and after each agent that ran long. Stop cleanly at 60 % of the 5-hour
window. Stop when the week passes 70 % (the owner raised it from 40 % for this run, 2026-11-18: "go to 70"). The Fable stop stays at
50 %; the week of Fable was at 47 % on 2026-11-18.

Waves left, in this order. The procedure of each step is in the skill `horizon-port-area`.

- Next, in this order: the owner's decision on the totem save (open item a); then `HOOKS.md` and `GAPS.md` up to date
  (items d and e); then one full `sh horizon/tools/test-slice.sh`; then the questions of item f.

One wave, six steps:

1. Port. One agent per area, each in its own worktree, all at the same time. No game in this step: the two gates
   only (`check.sh` has a lock, so the compiles queue by themselves).
2. Review. Two lenses (parity on Sonnet, use of the host game on Opus) for sky, homecoming and totem. One Sonnet
   reviewer with both lenses for ritual and tools, to save usage. The reviewers read the port agent's worktree.
3. Fix. One Sonnet agent per area, in the area's worktree.
4. Merge. The coordinator brings each area into `horizon-26.3` and runs the two gates.
5. Game. One Sonnet agent with game rights, in the coordinator's tree, after the merges: it builds the game, adds
   the scenarios of the wave's areas to `test-slice.sh`, runs them, fixes what fails inside those areas, and
   keeps its evidence on disk.
6. Close. The coordinator opens the evidence, runs the full `test-slice.sh`, commits, updates this section and
   reads the usage.

The prompts of this run worked as written in `docs/horizon/briefs/` (`port.md` with `area-<area>.md`, `review.md`,
`game.md`). A fix agent got: the worktree, the two review files, "check the evidence of each finding first", the
files it may change, and what the coordinator expected for each finding.

### Open after wave 1


Eternal (`docs/horizon/log/eternal.md`, "Differences from the Java mod" and "To check in the game"):

- A dropped Eternal Item resists fire only. Cactus, lightning and explosions damage it. The fix is a content
  change for the integrator: a damage type tag (fire, explosion, lightning, cactus) in `minecraft:damage_resistant`
  of the six items. Not checked: whether Horizon reads a tag of a mod there. GAPS.md row E10 names cactus although
  the content does not cover it.
- A bundle in a chest or on the cursor can keep an Eternal Item or an Ender Chest: no wired hook sees a click in a
  container.
- A dropped Eternal Item that `spawn_item` did not make keeps the vanilla health.
- An empty Eternal Bundle still stops the other hand (the hook cannot return the FAIL of Java).
- A restart between the gives at respawn and the clear of the saved data would give the stacks twice. Not tested.
- Not exercised in the game: an Eternal Item put into the bundle by clicks, a full inventory at the respawn,
  `keep_inventory`, fire, lava, cactus, TNT and lightning on a dropped item.
- Seen by the game agent, not investigated: after a world opens again, creative keeps `reduced_debug_info` true
  until a person sets it false (the game agent says that Java does the same); an Eternal Item on the cursor at
  death falls and does not come back; the first click on the Title Screen button of the death screen does nothing.

Amulet (`docs/horizon/log/amulet.md`):

- `/bmh point` needs operator level 2 (GAPS.md A5 says why). To give Java's level 0, the settings area must first
  check the level of the player for `/bmh rule` by itself.
- The main hand had no client route to the reading outside first person: solved by `wire::SkySignal` on `bmh:sky` (commit 9773ae7).
- "Is it night" uses the 1.21.1 formula, not the value of the host game. Dusk and dawn can differ by some ticks.
- Not tested in the game: the import of the points of a Java world (check 16 of the log says how to make such a
  world), the refusals in the Nether and with the eyes in water, two players.
- Seen once: a roof of one block read as open sky 50 ticks after the block was set (light lag).

Never tested on this branch: sound (all runs are muted), the Nether and the End, a dedicated server, a LAN guest.

### Traps and tools

The tools: `~/Library/Caches/bmh-horizon` holds `horizon-mods` (with `native/target`: the Nightly's tree, cargo's
cache, the built game), `target-tools/debug/cargo-horizon-mod` (`BMH_HORIZON_MOD_BIN`), `resources/26.3`,
`minecraft-server-26.3/server.jar`, `play/` (the owner's own play folder: tests never touch it), `scratch/` and
the logs. An erased cache loses Mojang's version file inside the Nightly's tree: see the skill
`horizon-build-test`. The built game in the cache is the tree of commit 8cda691 (the commit after it changes only
`test-slice.sh`).

Traps of the runs:

- When the app starts the session again, this worktree is back on the session's own branch
  (`claude/horizon-minecraft-mod-port-0812eb`, the Java `main`), with `horizon/` as untracked build folders.
  Nothing is lost: run `git switch horizon-26.3`.
- A restart of the app or a sleep of the Mac kills the background jobs and the sub-agents of the session (seen
  three times on 2026-11-17 and 2026-11-18: a wait, a build, an inspector). An agent that runs long keeps its log
  on disk from its first step. After a restart, read the logs before you start anything again.
- Do not build the game while the owner plays: the build writes the binary that `play.sh` runs and takes every
  core. Look for a `horizon-client` process first.
- A full `test-slice.sh --no-build` with 135 checks takes about 10 minutes and ends with exit 1 while t1 to t5 are red.
  Run it in the background.
- A full `test-slice.sh` run overwrites the tracked PNGs under `docs/horizon/log/sky-shots`. Restore them with
  `git checkout -- docs/horizon/log/sky-shots`.
- The capture cadence slows under a full run (0.18 s against 0.12 s alone), so a check that counts frames can fail
  there and pass alone (w1c).
- An agent's summary can differ from its own list (the fix agent of eternal wrote "three fixed, one fixed in
  part, two not fixed" for six findings whose `round1` fields say three fixed and three not fixed). Count from
  the list or from the `round1` fields.

Rules of the run that the owner gave: stop cleanly when the 5-hour usage window reaches 60 %; stop when the
week passes 40 % (all models) and 50 % (Fable), and ask him before going on (on 2026-11-12 both were passed and
the run stopped until he said "continue" in the new week; on 2026-11-18 he raised the weekly stop to 70 % for this run); use Sonnet for bounded work and Opus for new
structural code and for the host-game review lens.

## To check in the game

Nothing here is done until a run in the game shows it. Each area adds its rows.

| Area | What | How |
|---|---|---|
