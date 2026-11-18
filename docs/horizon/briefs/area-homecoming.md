# Area brief: homecoming (wave 2 of the run of 2026-11-18)

Read `docs/horizon/briefs/port.md` first and follow it exactly. This file is the part of your prompt that is about
your area. Your prompt gives BASE (the sha your worktree must be on).

No game in this step. Do not run `check.sh build`, `test-slice.sh`, `play.sh` or the game: another agent has the
game rights. Your proof is the two gates of port.md. What only a game can show goes in "To check in the game".

## Your area

The end of an exile. After a distant respawn the server keeps a record on the player: the exile center, the
amulet points of that moment (Overworld exile only), the dimension, the game time of the arrival and the
distance. Once a second it checks the player. Within 24 blocks of a home, in the same dimension, the player gets
a title ("Home" or the point name) with a subtitle (blocks and nights away), the BMH bell plays, two
advancements can come, and the record goes. All of it is server side. Today `bmh_homecoming.rs` is a skeleton:
the record is never kept and nothing happens at home.

## Your files

- `<R>/horizon/native/mixins/bmh/bmh_homecoming.rs` (the module).
- `<R>/horizon/native/mixins/bmh/logic/src/homecoming.rs` (the logic, with its unit tests).
- `<R>/docs/horizon/log/homecoming.md` (your log, from your first step).
- Content files that HOOKS.md ("Content files") gives to homecoming: `data/bmh/advancement/journey/home_at_last.json`
  and `long_way_home.json`. Both exist. The four lang keys `bmh.homecoming.*` are already in
  `<R>/horizon/content/assets/bmh/lang/en_us.json` (lines 77-80).

## Your row of GAPS.md

C10 (homecoming; tier N): attachment with `copy_on_death`, `server_tick` at 1 Hz, `server_players`,
`level_game_time`, `run_command` for `title`, `playsound` and `advancement grant`. GAPS lists no difference for
C10, so every difference you make needs a strong reason in your log. Read also GAPS.md section 7: question 1
(`run_command` from `server_tick`) and question 8 (the toast of `advancement grant`). The respawn area already
awards from `server_tick` (`bmh_respawn::on_server_tick`, docs/horizon/log/respawn.md lines 34-36 and 101), and the
amulet award "Reading the sky" was seen in the game (docs/horizon/log/game-checks-2.md lines 36-38).

## Audit sections

`<R>/docs/horizon/audit/1-respawn-server.md`, section 3 "Homecoming" (behaviour spec, Horizon mapping, parity
risks), and `<R>/docs/horizon/audit/verify-1.md`, row 3 (the corrected verdict: it wins). The audits cite Horizon
main's line numbers: find each function again in HN. `<R>/docs/horizon/audit/6-single-player-owner.md` for what
fires for the single-player owner and the traps (`server_players()` is empty inside a player hook; `run_command`
refuses where a player is swapped in).

## Java sources

Under `<R>/common/src/main/java/github/mrornithorynque/bmh/`:

- `handlers/HomecomingHandler.java` (all of it: the record, `onExile`, `tick`, `reached`, `nights`, `arrive`,
  `showTitle`, the codec).
- `mixin/C_ServerPlayerMixin.java` (lines 33-44 and 59-78: the record saved with the player, loaded, and copied by
  `restoreFrom` across death and the End exit).
- `mixin/E_ServerPlayerMixin.java` (lines 43-52: the check at `ServerPlayer.tick` HEAD when
  `tickCount % 20 == 0`).
- `handlers/RespawnEffectHandler.java` (line 55: `onExile` is the last step of the arrival effects).
- `handlers/BmhAdvancements.java` (`FAR_AWAY_BLOCKS`, `award`) and `init/SoundInit.java` (line 15, `bell_sfx`).
- The event order in `<R>/neoforge/src/main/java/github/mrornithorynque/bmh/neoforge/NeoForgeEvents.java`.

## Contracts (HOOKS.md, "Contracts": keep every signature)

- You READ the exile: `bmh_homecoming::on_exile(game, player, exile: &bmh_logic::contract::Exile)`. The helper
  `hook_player_respawned.rs` calls it after `bmh_respawn::on_player_respawned`, only for a distant respawn. Keep
  the signature. `Exile { dimension, center, distance }`.
- You READ the amulet points: `bmh_amulet::points(game, player) -> Vec<bmh_logic::contract::AmuletPoint>` (row
  "Amulet points of a player", fixed for this wave). Call it in `on_exile` only when `exile.dimension` is
  `minecraft:overworld` (`bmh_logic::respawn::OVERWORLD`), as Java does (HomecomingHandler.java:104-108). The
  skeleton calls it for every exile: change that. The points come in list order; the center goes first, with an
  empty name, so the center wins a tie (HomecomingHandler.java:102 and 130-150).
- You READ `bmh_settings::rule(game, Rule::DisplayTextOnRespawn)`: the title follows this rule, the bell and the
  advancements do not (HomecomingHandler.java:164-177). Server side only.
- You GIVE nothing to another area.
- Reuse, do not copy: `bmh_logic::respawn::FAR_AWAY_BLOCKS` (30 000, for `journey/long_way_home`),
  `bmh_logic::respawn::BELL_SOUND` (`bmh:bell_sfx`), `crate::game::bmh::award(game, name, path)` (from a hook where
  no player is swapped in), and the respawn pattern that skips an advancement the player already has
  (`native_player_advancements`, review respawn semantics-5).

## Points to settle yourself, by reading HN, and to record in your log

- The record. An attachment on `Holder::Player(uuid)` with a codec and `copy_on_death: true`, as the amulet keeps
  its points (`bmh_amulet.rs`, `static POINTS`). Keep the Java field names in the compound: `dimension`,
  `arrival`, `distance`, `homes` (a list of `{name, x, y, z}`, every field always written). No record writes
  nothing (Java `write`). A new exile replaces the old record.
- The End exit. Java copies the record in `restoreFrom` for the End exit too (C_ServerPlayerMixin.java:73-78).
  `player_respawned` does not fire for the End exit (native_hooks.rs:8454-8458). Find in HN whether the End exit
  keeps the `copy_on_death` attachments of the player. If it does not, say so in your log.
- The check once a second. `server_tick` is wired for you (HOOKS.md). It has no player swapped in, so read the
  players with `native_server_players()` (uuid, dimension, position) and the record with the attachment host.
  Pick one phase, keep your own tick counter or use the game time, and write why. Java counts `tickCount` per
  player (it starts at 0 for each new player object); the 1 Hz rhythm is what matters, not the phase.
- Where each effect runs. `run_command` works only where no player is swapped in. `/playsound` reaches only the
  players of the level that is in place: the amulet area plays its chime to the player at once with
  `native_play_sound_to` and queues the `/playsound` for the others to `level_tick_post` of that level; the
  respawn area queues its sounds there too. The bell of
  a homecoming plays to everybody near, the player too (HomecomingHandler.java:176-177: volume 1.0, pitch 0.8,
  category players). `level_tick_post` does not call your area yet: wiring it (one line in
  `hook_level_tick_post.rs` and the HOOKS.md row) is an allowed change outside your files.
- The title. Java sends the times (10, 70, 20), then the subtitle, then the title (HomecomingHandler.java:202-205).
  Two routes: `run_command` with `title <name> times 10 70 20`, `title <name> subtitle {json}` and
  `title <name> title {json}`, or a `title` helper if the Nightly has one (audit 1 names a pending
  `native_title`; check its call site and tier in HN). A point name is player text: build the JSON with escapes
  for `"` and `\` in the logic crate, with a unit test, so a name can never break or extend the command. The title
  of the center is the key `bmh.homecoming.home`; a named point shows its name as literal text.
- The nights. `nights = max(0, (gameTime - arrival) / 24000)` with the game time of the level
  (`native_level_game_time`). The three subtitle keys follow nights >= 2, == 1, else.
- Java worlds. Java saved the record under `bmh_homecoming` in the player data. The amulet brings the points of a
  Java world in at `level_tick_post` from `native_player_compound` (`import_java_points` in `bmh_amulet.rs`,
  docs/horizon/log/amulet.md step 10). Do the same for the record, or list the difference with its reason.
- A player who leaves, a player who dies again before home (a new exile replaces the record), a player in another
  dimension (no check), a record whose dimension no longer exists.

## Worked examples

- `<R>/docs/horizon/briefs/area-amulet.md` and `<R>/docs/horizon/log/amulet.md`: an attachment on the player with
  `copy_on_death`, the Java import at `level_tick_post`, awards and sounds queued to a hook with no player swapped
  in.
- `<R>/docs/horizon/log/respawn.md`: awards from `server_tick`, sounds from `level_tick_post`, the skip of an
  advancement the player already has.
- `<R>/docs/horizon/briefs/area-eternal.md` and `<R>/docs/horizon/log/eternal.md`: the shape of a brief and of a
  log with its three closing sections.

## Open review findings that touch your area

None open. The respawn findings are fixed, except semantics-2 (deferred: the `Departure` message is one tick
late), which does not touch the exile record (`docs/horizon/review/respawn-*.json`). The amulet findings are
fixed (`docs/horizon/review/amulet-*.json`). The settings area is not reviewed yet: if `bmh_settings::rule` gives
a value you do not expect, note it in your log for the integrator.

## To check in the game (start your log's list with these)

1. A distant respawn in the Overworld keeps a record with the exile center and the amulet points of that moment.
   Read it back (a log line at `on_exile` with the home count). A point saved after the respawn is no home.
2. Walk back within 24 blocks of the exile center: the title "Home" with the subtitle of blocks and nights, the
   bell (also heard by another player within 16 blocks), the advancement "Home at last", and the record is gone
   (no second title). With an exile of 30 000 blocks or more, also "The long way home".
3. Within 24 blocks of a named amulet point: the title is the point name. A name with `"` or `\` shows as typed.
4. `/bmh rule display_text_on_respawn false`: no title, the bell and the advancements still come.
5. A Nether exile ignores the amulet points; only the center counts, and only in the Nether.
6. The record survives a restart of the world, a second death (a new exile replaces it) and the End exit (or the
   log says why not).
7. The subtitle keys: `/time add 24000` once gives "1 night", twice gives "2 nights", none gives no nights.
8. The single-player owner and a LAN guest both get the title (the `title` command reaches each by name).
9. A Java world with a `bmh_homecoming` record (if you port the import): the homecoming still ends the exile.
