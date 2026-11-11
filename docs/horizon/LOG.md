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

## Resume here

State on 2026-11-11, 20:25. Branch `horizon-26.3`, local only. The run stopped here on purpose, at 54 % of the
5-hour usage window (the owner's line is 60 %). The tree is clean and every step above is committed.

Done: the content mod, the skeleton (66 hooks), the areas respawn, settings and journey, the game binary with
the mod inside (`check.sh build`), the game preparation script and the in-game test script (not run).
Not done, in this order:

1. Fix the open findings of `docs/horizon/review/respawn-*.json` (one agent in a worktree; check each finding's
   evidence first; both gates must pass).
2. Review settings and journey the same way (two lenses each: parity with the Java mod, use of the host game),
   then fix.
3. Run the in-game test of the slice. It is written and has never run against a game:
   `sh horizon/tools/prepare-game.sh --server-jar <path>` once (it says what is missing), then
   `sh horizon/tools/test-slice.sh`. `docs/horizon/TESTING.md` lists its checks and its open questions for the
   first real run (the flags after `launch --wait`, the JSON field names, the `damage` command). Expect to fix
   the script on that first run. It needs prepared game files.
4. Port the seven other areas: homecoming, eternal, totem, ritual, tools, amulet, sky. Amulet before sky and
   homecoming (they read its points and its reading). The brief of an area agent is `docs/horizon/briefs/port.md`
   (its absolute paths name the first session scratchpad: change them); its rules are the ones of ARCHITECTURE.md and of the first three area logs.
5. Skills and a CLAUDE.md for this branch; remove the Java sources when the port is complete.

Blocked on the owner: Minecraft 26.3 must be installed and played once, and the 26.3 server jar downloaded.
Without them no game runs. `/Applications/Horizon.app` is still refused by Gatekeeper (only the launcher needs
it; the build from source does not).

Rules of the run that the owner gave: stop cleanly when the 5-hour usage window reaches 60 %; stop when the
weekly usage passes 40 % (all models) and 50 % (Fable); use Sonnet for bounded work and Opus for new structural
code.

## To check in the game

Nothing here is done until a run in the game shows it. Each area adds its rows.

| Area | What | How |
|---|---|---|
