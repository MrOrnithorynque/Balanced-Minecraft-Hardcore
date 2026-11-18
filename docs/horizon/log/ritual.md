# Ritual area: port log

Area: ritual (GAPS.md rows T7, T8, T9, T10, T15). Files: `horizon/native/mixins/bmh/bmh_ritual.rs`,
`horizon/native/mixins/bmh/logic/src/moon.rs`, this log. No game was run in this step (no game rights): the proof is the
two gates. What only the game can show is in "To check in the game".

## Steps

1. The worktree was on an old commit (no `docs/horizon`). Branch `worktree-agent-a2c7f7b6c3f3b6884`, root
   `/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/agent-a2c7f7b6c3f3b6884`, not the
   coordinator's tree. Ran `git reset --hard c75751ff8fceedd6c3e9b9f4a87546f6bbcd9122` once. Guard file present.
2. Read port.md, ARCHITECTURE.md, HOOKS.md, GAPS.md rows T7 to T15 (they match the prompt: T7 F, T8 X, T9 N, T10 X, T15 N),
   audit 2 sections 4, 5 and 7, verify-2 rows 4, 5a, 5b and 7, and the Java sources (MoonRitualHandler,
   DormantTotemOfEternityItem, RitualHandRenderer, RitualParticles, AmuletReading.skyHiddenReason).
3. Read HN for the use hooks (`item_use.rs`, `native_hooks.rs:7995-8260`), the hand pose (`game.rs:14418`, `hand_pose_matrix`),
   the helpers (`native_acting_player`, `native_action_bar`, `native_give_levels`, `native_play_sound_to`,
   `native_send_particles_to`) and the moon: the Overworld timeline (`resources/26.3/data/minecraft/timeline/day.json`)
   gives `moon_angle = 180 + 360 * ease(((t - 6000) mod 24000) / 24000)` with the cubic bezier `[0.362, 0.241, 0.638, 0.759]`
   (`horizon-client-render/src/environment.rs` samples it). The mod re-implements that sample.
4. Wrote `logic/src/moon.rs`: the constants with their Java lines, the moon (`moon_angle` is Horizon's own sample of the
   timeline track, with the same bezier solver; a test checks it against the old vanilla `timeOfDay` curve within 1.5
   degrees), `view_vector`, `aims_at_moon`, `blocked_reason` (uses `amulet::hidden_reason`, as Java shares
   `AmuletReading.skyHiddenReason`), `start`, `tick` (Java's order: blocked, hurt, level), the cooldown, the places of the
   totem and of the puff, `Rng` and `rune`, and the first person pose (`totem_target`, `hand_pose`). 29 unit tests pass.
5. Wrote `bmh_ritual.rs` on it. First compile: one error (`_` arm after the two phases of `ServerTickPhase`, unreachable
   inside the game crate); fixed. Then both gates passed (`test-logic.sh`: 220 passed, 29 of them in `moon`; `check.sh`: `"check": "ok"`).
6. Wired the hit: `hook_living_incoming_damage.rs` calls `bmh_ritual::on_living_incoming_damage` after the respawn area
   (HOOKS.md row updated). Nothing else outside my files changed.

## The design in one page

- T15 and T7, start. `use_duration` answers 300 for `bmh:sleeping_totem` on both sides. HN `mod_use` then holds, so a click
  starts a use and `start_mod_use` consumes it (even when the start hook cancels). The start hook is decided on the server only:
  cooldown (silent), then `blocked_reason` (message on the action bar), then `begin` (record, start sound, log line). The window
  does not decide: it notes the use (hand, elapsed 0) for the pose.
- T7, tick. All of Java's `tick` runs in `living_entity_use_item_tick` on the server, because only that hook can end the use:
  `canceled = true` makes the host finish the use (HN `using_item.rs:99-103`, `complete_using_item`), and the finish hook finds the
  record gone and leaves the Dormant Totem. The order is Java's: blocked reason (the aim is renamed `looked_away`), hit, then on
  every 10th tick the level check, the level (`give_levels(-1)`, not in creative) and the chime (`0.5 + 1.4 * elapsed / 300`).
  `remaining` is the `duration` the hook gives: 300 on the first tick, 1 on the last, as Java's `onUseTick`.
- T7, fail. `fail` removes the record, starts the 20 tick timer of the player, queues the puff (own place in front of the eyes,
  others at the raised totem) and the fizzle, and puts the reason on the action bar. A release is the same fail with
  `bmh.ritual.released`, and sets `used` so that a content `use_cooldown` starts too.
- T7, wake. The finish hook replaces the result with `bmh:totem_of_eternity`, puts `bmh.ritual.woken` on the action bar and queues
  the resonate sound, the advancement `eternity/moon_ritual` (needs `run_command`, so it waits for the end of the tick) and a
  sound and particles in place of entity event 35.
- The hit (Java `hurtTime > 0`). `living_incoming_damage` notes the server tick of each hit on a player by entity id (a hit that was
  not canceled and has an amount above 0). A ritual tick is hurt when the last hit is less than 10 ticks old. This also catches a
  hit just before the click, as `hurtTime` does.
- T9. At the end of each server tick, for each running ritual (a record whose use ticked in the last 3 server ticks) and each viewer in
  the level within 32 blocks: one `enchant` particle by `send_particles_to` with count 0 (a directional particle: the offset is its
  motion), at the totem plus 1.2 blocks up. The player gets the first person place, the others the raised hand place.
- T8. `render_hand_pose` adds an offset, a yaw, a roll and a scale to the hand that uses the Dormant Totem. `hand_pose` solves the
  offset so that the item of the hand lands where Java draws the totem (a unit test checks it for both hands). The scale is the
  ratio of the `fixed` and the first person display of a flat item (0.16 * 0.5 / 0.68), which also shrinks the arm to almost nothing.
- T10 is not built (tier X): the Dormant Totem shows no glint while it charges.

## Differences from the Java mod

1. T7 (GAPS T7, "a client that predicts a start the server refuses shows the ritual for a moment"): the window never decides, so
   when the server refuses, the window still shows the raised totem as long as its own use ticks. How the window leaves a use the
   server never started is not known (audit 2 open question 7). The player sees the reason on the action bar.
2. T7: sounds, puffs, wake effects and the advancement come out at the end of the server tick, one tick late (the use hooks run
   inside a player's turn, where `server_players` is empty and `run_command` refuses). Sounds go to each player within 16 blocks of
   the place with `play_sound_to`, not through `Level.playSound`.
3. T7, the hit: `living_incoming_damage` fires "past its guards" for every hit, also one that the invulnerable time of the player
   would refuse later. Java sets `hurtTime` only for a hit that lands. A weak hit inside the invulnerable time can fail a ritual
   here. The precise route is the pending `living_damage_pre` hook; it is not wired (it needs a new injection).
4. T7, the cooldown: the mod keeps a 20 tick timer per player (silent refusal, as Java never reaches `use` on a cooling item).
   The client cannot see this timer, so a held use key makes the window predict a start every 4 ticks and the pose comes back.
   The content item `bmh:sleeping_totem` has no `minecraft:use_cooldown` component (see "Content the area needs").
5. T7, the moon: the angle is the mod's copy of the timeline track (`timeline/day.json`, same easing as Horizon's sampler). A
   datapack that changes the timeline is not followed. "Dark outside" and "sees the sky" are the amulet's rules (GAPS A2).
6. T7: the main arm is taken as the right one (no helper for it). A left-handed player gets the mirrored place for the runes of
   others and the puff, and the first person pose on the wrong hand side.
7. T7: Java counts `awardStat(ITEM_USED)` for the totem. The mod does not award it. Whether the host counts a finished use is not
   checked.
8. T7: entity event 35 (the totem animation on screen) is a GAPS T4 row (tier X, the totem area). The wake plays
   `item.totem.use` and 30 `totem_of_undying` particles instead, with no item overlay.
9. T7: `UseAnim.BOW` (both arms raised for other players and in third person) is not set: the `use_animation` hook is not wired,
   and its doc says it is for the first person hand only. Other players hold the totem as any other item.
10. T8 (GAPS T8): the arm is drawn, shrunk with the item, and the light is the hand's. The place and size are an estimate (the
    display transforms of the model are assumed to be the vanilla ones of `item/generated`).
11. T9 (GAPS T9): one tick late, and the server cannot see the third person camera, so the player always gets the first person place.
12. T10 (GAPS T10): no glint while the ritual charges.
13. The fail ends the use through the host's `complete_using_item` (Java: `stopUsingItem`). It runs `send_use_item_complete`,
    `apply_use_cooldown` and the finish hook on the way. With no `use_cooldown` on the item, the cooldown step does nothing.
14. A use with no acting player, or with no record of the start, is ended with no wake (the mod fails closed).

## To check in the game

1. The hand pose: the raised totem is under the crosshair, small, facing the camera, with the arm almost invisible; it rises in 10
   ticks and trembles more at the end. Tune `FACE_YAW` (90 or -90), `HAND_SCALE` and the assumed item place in `moon.rs`
   (`START_*`). Command: `/give @s bmh:sleeping_totem`, stand under the night sky, look at the moon, hold use, take screenshots.
2. `native_acting_player()` is `Some` inside `living_entity_use_item_start`, `_tick`, `_finish` and `release_using` for the
   single-player owner and for a LAN guest. If it is `None`, the ritual never starts (log line "A Dormant Totem use started with
   no acting player").
3. A fail in the tick hook (`canceled = true`) ends the use on the server and the Dormant Totem stays in the hand: look away from
   the moon during the ritual. Expect the action bar `You looked away from the moon...`, the puff, the fizzle, and the same item.
   Check also that the client leaves the use and the pose ends.
4. The 30 levels: `/xp set @s 40 levels`, run a full ritual; expect level 10 left, 30 chimes rising from 0.54 to 1.9, then
   `The Totem of Eternity wakes.`, the Totem of Eternity in the hand, the advancement `Moon-touched`, and the log lines
   `Moon ritual started`, `Moon ritual done`. Creative: no levels taken.
5. `native_play_sound_to` and `native_send_particles_to` reach players from `ServerTickEnd` (no player swapped in), and
   count 0 particles of type `enchant` fly into the totem (not away from it) and are not drawn in front of the camera.
6. A hit during the ritual fails it (`The pain breaks the ritual`), also a hit up to 9 ticks before the click.
7. The release (let go of the use key) fails with `You let go too soon`, and a held key does not restart the ritual for 20 ticks
   (the mod timer); and what the window shows then (differences 1 and 4).
8. The commands `time set 18000` (midnight) and `time set 13000` (the moon low in the east): the aim test follows the moon.
   Compare the log line "aimed N degrees from the moon" with where the moon is drawn.
9. A second player near hears the start, the chimes, the fizzle and the wake, and sees the puff and the runes at the raised hand.
10. Whether `use_cooldown` in the content item shows the cooldown sweep on the icon and stops the window's prediction.

## Content the area needs (not edited: not my files)

- `horizon/content/data/bmh/horizon/item/sleeping_totem.json`: add to `components`
  `"minecraft:use_cooldown": { "seconds": 1.0 }` (20 ticks, `FAIL_COOLDOWN_TICKS`). The host then starts the cooldown after a
  failed tick (through the finish) and after a release (`used`), and the window sees it. It also starts after a successful wake, on
  the id of the replaced item: harmless. The mod's own timer stays as the authority.
- No lang key was added: the Java keys `bmh.ritual.*` are in the content lang files.

## Changes outside my files

- `horizon/native/mixins/bmh/hook_living_incoming_damage.rs`: one call added after the respawn call,
  `bmh_ritual::on_living_incoming_damage(entity, kind, *amount, *canceled)`. Reason: the hit that breaks a ritual (Java `hurtTime`).
- `docs/horizon/HOOKS.md`: the row of `living_incoming_damage` lists ritual after respawn.
- Not changed, but a proposal: `bmh_amulet.rs` has a private `sky(game, me)` and this file has a copy. A shared `pub(super)` one
  would remove the copy.

## Fix round 1

1. Checked the evidence of the five findings against the files: the window branch of the start hook set no `canceled` (line 409 before the fix), the item had no `use_cooldown`, no stat or `use_animation` route exists, and the hit rule is the quoted one. All five hold.
2. Major (semantics): the window start branch now runs the same test as the server. `logic::start_test(cooling, sky, look)` in `moon.rs` joins `start` and `blocked_reason`; the server and the window both call it. The window builds the sky from `native_camera()` (dimension, camera position, yaw, pitch) with `sky_at`, and sets `canceled` when the answer is not `Go`. With no camera frame yet the start passes and the server decides. New unit test `the_start_test_is_the_same_for_both_sides` (aim, cooldown, rain, dimension, roof).
3. Window limits: it cannot read Blindness or Darkness (no local player id), so `blinded` is false there; its cooldown state comes from the item's `use_cooldown`, not from the mod timer. The camera yaw is assumed to be the head yaw (a third person front view may differ): to check in the game.
4. Integrator change (coordinator allowed): `"minecraft:use_cooldown": { "seconds": 1.0 }` added to `horizon/content/data/bmh/horizon/item/sleeping_totem.json`. The key shape is the one the Nightly reads (`UseCooldown::from_json`, `seconds`). `content-check.sh` builds and checks the mod.
5. Parity minors (ITEM_USED stat, BOW pose, hit rule): not fixed. No Nightly helper awards a stat (`award_stat` only inside `finish_consuming`), the `use_animation` hook is first person only, and the exact hit route (`native_living_damage_pre`) is not in the pinned hook list. Each stays a difference (see "Differences from the Java mod" 3, 7, 9). Proposed GAPS.md T7 additions: the stat, the third person BOW pose, and the hit rule counting hits that vanilla's invulnerable time refuses later.
6. Gates: `test-logic.sh` 221 passed, 0 failed. `check.sh` ok (`"check": "ok"`). No game run (no game rights).
