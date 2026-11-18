# Port log: homecoming

Area: the end of an exile (GAPS.md row C10). Briefs: `docs/horizon/briefs/port.md` and
`docs/horizon/briefs/area-homecoming.md`. No game rights: the proof is the two gates only.
Files: `horizon/native/mixins/bmh/bmh_homecoming.rs`, `horizon/native/mixins/bmh/logic/src/homecoming.rs`, this log.

## Steps

1. The worktree was not on BASE 9773ae7 (the briefs were missing). Branch `worktree-agent-a1fcd36a96b03a718`, so
   the brief allows the reset: `git reset --hard 9773ae7306e156eb7706d8b386a5cbc3fec6c2d4`. Guard file present.
2. Read port.md, the area brief, HomecomingHandler.java, audit 1 section 3, verify-1 row 1e, the amulet module
   (attachment, Java import, queues) and the respawn module (awards from `server_tick`, sounds from
   `level_tick_post`).
3. Read HN. Findings:
   - `native_title(uuid, &Text, &Text, fade_in, stay, fade_out)` exists (native_hooks.rs:10170), tier F, sends the
     three packets in the order of `/title` (times, subtitle, title). It works from `server_tick` for any online
     player. So no `run_command title` and no JSON in the logic crate: a point name goes in as `Text::literal`,
     which cannot break out of anything.
   - `NativeLifecycleContext.tick` counts non-paused server ticks since the world opened (mod_api.rs:802). The
     once-a-second check uses `tick % 20 == 0` at `ServerTickEnd`. Java uses `tickCount % 20` per player; only the
     rhythm matters.
   - The End exit does no `copy_on_death` copy but also loses nothing (verify-1 row 1e: server_travel.rs:376
     `native_player_clone(uuid, &name, false)` keeps the attachments as they are), so the record survives it.
4. Wrote `logic/src/homecoming.rs`: constants with their Java lines, `new_record` (center first, points only for an
   Overworld exile), `reached` (24 blocks on x and z from the block center, first home wins a tie, the Java
   `squared - 1.0E-9` rule), `nights`, `subtitle` and `title`, `awards`, `bell_position`, `bell_command`,
   `someone_hears`, `is_check_tick`. 19 unit tests, Java values worked by hand (24.0 blocks in, 24.01 out; 17+17
   blocks out, 16+17 in; ties both ways; nights at 23 999, 24 000, 48 000 and below zero).
5. Wrote `bmh_homecoming.rs`:
   - The record is the attachment `bmh:homecoming` (`copy_on_death: true`) with the Java field names `dimension`,
     `arrival`, `distance`, `homes` (`name`, `x`, `y`, `z`, every field written) plus a flag `imported`. No record
     writes only the flag. The decode is strict like the Java codec: a missing field, or one bad home, gives no
     record.
   - `on_exile` reads the amulet points only for an Overworld exile (the skeleton read them always), builds the
     record with the game time and keeps it. A new exile replaces the old record.
   - `on_server_tick` at `ServerTickEnd`, when `context.tick % 20 == 0`: for each online player with a record in
     the same dimension, `reached` on the player's x and z. The phase comes from the server tick count (non-paused
     ticks since the world opened); Java counts `tickCount` per player, only the rhythm matters.
   - `arrive` clears the record first (if that fails, nothing else happens, so no title once a second), then the
     advancements (`home_at_last`, plus `long_way_home` from 30 000 blocks; skipped when already done, as the
     respawn area does), then the title when the rule `display_text_on_respawn` is on, then the bell.
   - The title uses `native_title` (tier F, step 3): times 10, 70, 20, then subtitle, then title. The subtitle is
     `Text::translate` with the blocks and the nights as literal text. The title of the center is the key
     `bmh.homecoming.home`; a point name is `Text::literal`, so no name can break or extend anything.
   - The bell: `native_play_sound_to` to the player at once (category `player`, volume 1.0, pitch 0.8) and a
     `/playsound` line for the others, queued with the player's dimension and run at `level_tick_post` of that
     level when someone else is within 16 blocks. `/playsound` has `@a[name=!<name>]`; the name is checked as a
     player name in the logic crate.
   - The Java record: `import_java_records` at `level_tick_post` reads `bmh_homecoming` from
     `native_player_compound`, once per player (flag). A new exile and a homecoming set the flag too, so the Java
     record of a player who already had an exile in Horizon never comes back.
6. Changes outside my files: `hook_level_tick_post.rs` (one line) and its HOOKS.md row.
7. Both gates pass: `sh horizon/tools/test-logic.sh` (213 passed, 0 failed), `check.sh` (`"check": "ok"`).

## Not done

No game ran (no game rights). Nothing here is shown to work in the game; the list below says what to check.

## Differences from the Java mod

1. The bell sounds at the center of the block (x + 0.5, y + 0.5, z + 0.5). Java calls
   `level.playSound(null, BlockPos, ...)`, which vanilla centers the same way. Not a GAPS.md row; a check for the
   position only. The other players hear it through `/playsound` (range 16 blocks, volume 1), not through the
   level's sound broadcast; GAPS.md C7 allows this for the arrival sounds and C10 uses the same substitute.
2. The check phase is the server tick count, not each player's `tickCount`. A player hears the title up to one
   second after crossing the line, as in Java.
3. The title goes through `native_title` (tier F), not `run_command title`. The brief names both routes. The
   result is the same three packets in the same order.
4. The attachment key is `bmh:homecoming` (a Horizon save), not `bmh_homecoming` in the player data. The Java
   record of a Java world comes in once (step 5). No GAPS.md row; the audit already says the key cannot be the same.
5. `/playsound` for the other players needs a player name that is safe in a selector. A name that is not (it
   cannot happen with vanilla names) gets no bell for the others; the player hears it.

## To check in the game

1. A distant respawn in the Overworld keeps a record: the log line "Homecoming record: N homes (M amulet points)".
   A point saved after the respawn is no home.
2. Walk back within 24 blocks of the exile center: title "Home" and the subtitle of blocks and nights, the bell
   (a second player within 16 blocks hears it too), the advancement "Home at last", no second title. With an
   exile of 30 000 blocks or more, also "The long way home". Type `/advancement revoke @s everything` before.
3. Within 24 blocks of a named amulet point: the title is the point name. A name with `"` or `\` shows as typed.
4. `/bmh rule display_text_on_respawn false`: no title; the bell and the advancements still come.
5. A Nether exile ignores the amulet points; only the center counts, and only in the Nether.
6. The record survives a restart of the world, a second death (a new exile replaces it) and the End exit
   (verify-1 row 1e says the End exit keeps the attachment).
7. `/time add 24000` once gives "1 night", twice "2 nights", none gives no nights. The count uses the game
   time (`/time add` moves the day time; if it does not move the game time, wait or use `/tick sprint`).
8. The single-player owner and a LAN guest both get the title and the awards (the title goes by uuid, the
   awards by name).
9. `server_tick` gives `stored` / `set` on an attachment of a player with no player swapped in (the brief says
   the same host works from `level_tick_post` for the amulet). If the log shows "The homecoming record is not
   saved", the clear fails and the player gets no title.
10. The bell for the player alone plays from `server_tick` at the tick's end; the second player hears
    `/playsound` at the next level tick of the level, up to one tick later.
11. A Java world with a `bmh_homecoming` record: the log line "Homecoming record read from the Java player
    data", and the homecoming still ends the exile.

## Changes outside my files

- `horizon/native/mixins/bmh/hook_level_tick_post.rs`: one line, `bmh_homecoming::on_level_tick_post`, after the
  amulet. Reason: the hook did not call homecoming (brief: allowed).
- `docs/horizon/HOOKS.md`, row `level_tick_post`: "homecoming" added to the areas and one sentence on what it
  runs there.
- Observation for the amulet owner (not changed): `encode_points` in `bmh_amulet.rs` inserts the `imported` entry
  twice in its array. It is harmless (the same value under the same key).

## Fix round 1

Two minor findings, both with evidence found in the code.

1. Parity, `bmh_homecoming.rs` `on_exile`: with no game time the record was not saved and the exile could never
   end. Fixed: the record is saved with arrival 0, as Java always saves it. The title then shows no nights.
   Hardening only. New difference from Java: the nights are not counted in this case.
2. Semantics, bells of logged-out players: fixed. New `on_player_logged_out` removes the bell lines of the
   player. Change outside my files (allowed integrator change): `hook_player_logged_out.rs` calls it after the
   amulet, and the `player_logged_out` row of `docs/horizon/HOOKS.md` lists homecoming.

No new unit test: both fixes sit in Game-bound code. The review files carry the outcomes.
