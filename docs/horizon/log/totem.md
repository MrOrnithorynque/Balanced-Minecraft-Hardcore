# Totem area: port log

Area: totem (GAPS.md rows T1, T2, T3, T4, T5, T6, T11, T12, T13, T14, E6). Files:
`horizon/native/mixins/bmh/bmh_totem.rs`, `horizon/native/mixins/bmh/bmh_totem_world.rs`,
`horizon/native/mixins/bmh/logic/src/totem.rs`, this log. No game was run: the proof is the two gates.

## Steps

1. The worktree was not on BASE c75751f (`merge-base --is-ancestor` exit 1). Branch
   `worktree-agent-a42f435fe4f113c16`, root
   `/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/agent-a42f435fe4f113c16`,
   not the coordinator's tree. Ran `git reset --hard c75751ff8fceedd6c3e9b9f4a87546f6bbcd9122` once.
2. Read port.md, ARCHITECTURE.md, HOOKS.md, GAPS.md rows, audit 2 and verify-2, the Java sources
   (TotemOfEternityHandler, TotemAwakeningHandler, A_LivingEntityTotemMixin, B_AnvilMenuMixin, I_*,
   TotemOfEternityItem, DormantTotemOfEternityItem, HintItem, EternalTooltips, EternalWear), the eternal log and
   `bmh_eternal.rs` as the worked example.
3. Read HN for each hook and helper (findings in "What HN says" below).
4. Wrote `logic/src/totem.rs` with 20 unit tests (`sh horizon/tools/test-logic.sh totem` passes).
5. Wrote `bmh_totem.rs` and `bmh_totem_world.rs` on it. Both gates passed on the first compile:
   `sh horizon/tools/test-logic.sh` (209 passed, 20 of them in `totem`) and `sh horizon/tools/check.sh`
   (`"check": "ok"`). Nothing was run in the game.
6. Self review (no sub-agents allowed). Paths reasoned through:
   - Save, normal: `living_death` (no player swapped in) asks the book. The book says yes when the last tick saw a
     Totem of Eternity and the damage type is not `out_of_world` or `generic_kill`. The hook cancels (Horizon sets
     health 1.0). The next `player_tick` swaps the first totem, sets health 1, replaces the effects, plays the sound and
     queues the advancement. `server_tick` end awards it.
   - Save, second hit before the swap: the book answers no and clears the wait, so the player dies (Java: the first
     save already spent the totem, but its Absorption would take the hit; difference 1).
   - Save, totem gone before the swap (dropped in that tick): the swap finds none and sets health 0, so the death
     comes back. No free save.
   - Extra totem in the inventory: the drop comes first, then the slot is emptied, so a failed drop loses nothing and a
     successful one cannot duplicate. A plain stack is removed by `take_items`; any other stack gets count 0.
   - Extra totem in a container: extract, give, then drop if the inventory is full. If both fail the totem is lost and
     a warning is logged. This needs a full inventory and a failed `summon`, the same window as in the eternal area.

## What HN says (findings that shaped the code)

- `living_death` (native_hooks.rs:6567, called from `guest_die`, lan_actions.rs:4309): no player swapped in, `cancel`
  sets health 1.0. `player_tick` (native_hooks.rs:2791) is the end of `ServerPlayer.tick`, after
  `native_inventory_ticks`. The player is swapped in, so `native_acting_player`, `native_carried_stack`,
  `native_take_items`, `native_give_stack`, `native_set_effects`, `native_set_player_status`, `native_summon` and
  `native_extract_item` all reach him.
- `native_set_effects` replaces the whole list (`/effect clear` then the adds, native_player_state.rs:295). It is the
  pending fragment `player_effects` (tier F), compiled into the Nightly. It runs in the player's own tick, so the
  effects come one tick after the cancel, as GAPS T3 allows, with no `run_command`.
- `native_set_inventory_slot` (F) fails inside a player's turn ("inside a player's turn", native_inventories.rs:165), so
  the swap writes the stack through `native_carried_stack`. A count of 0 is cleaned only by `native_inventory_ticks` at
  the next tick (native_hooks.rs:11555), so `take_items` is used for a plain stack.
- `native_container_slots` and `native_extract_item` (API 48, machines/native_items.rs) read the level in place and
  need a loaded container. A double chest answers with both halves.
- `inventory_move_item` gives block positions of the source and the destination. `hopper_pickup_item` is a method of
  `GameLevel` with the hopper position only (game.rs:1299 `allow_hopper_pickup_item`): nothing reads the hopper.
- `anvil_update` (stations.rs:702, lan_stations.rs:200): `output`, `xp_cost` and `material_cost` start as vanilla's; a
  hook may set all three; `answer_update` clamps a negative level cost to 0.
- `modify_default_components` receives the `items.json` components. The Totem of Undying holds
  `minecraft:death_protection.death_effects[].effects[]` with `id` keys (horizon-items items.rs:2377 test).

## The decisions of the area

- T2. The book knows who carries a totem from the last `player_tick` (a death hook cannot read a stack). The swap runs
  in the next tick of the player. A player whose tick never ran (just logged in) cannot be saved in that first tick.
- T3. Effects from `native_set_effects` in the swap tick (tier F helper; GAPS names `run_command` at tier N). The flags
  are those of the three-argument `MobEffectInstance`: not ambient, particles on.
- T4. Sound `minecraft:item.totem.use` (volume 1, pitch 1, source players) and a burst of 100 `totem_of_undying`
  particles about the middle of the body, for the players near. No item overlay (needs X). The count, spread and speed
  are chosen numbers, not Java values (vanilla runs a 30 tick emitter).
- T1. Tooltip lines for the four items, inserted under the name. The Hollow Totem and the Echo Heart get their hint. The
  Totem of Eternity and the Dormant Totem get eternity, their line and the gold heavy line.
- T11. `player_tick` drops every Totem of Eternity after the first, not for a spectator, whatever the game rule says.
  `item_picked_up` refuses a second one, as the eternal area does.
- T12. While a player has a container block open, each tick compares the totem slots with those at the open. A new
  totem that joins another leaves and goes to the player (inventory, else on the ground). Once more at the close. A pair
  that was already in the container is left alone (Java never touched a container that was already wrong).
- T13. `inventory_move_item` cancels a totem move into a container that holds a totem. `hopper_pickup_item` cancels
  every hopper pickup of a Totem of Eternity from the ground.
- T6, E6. See the code comment of `on_anvil_update`.

## Differences from the Java mod

1. T2, T3 (GAPS T2, T3): the swap and the effects land one tick after the death hook. A second hit in that tick finds no
   Absorption and kills the player. Java gave the effects at once. A player whose first tick has not run yet cannot be
   saved.
2. T2 (GAPS T2 "hard-coded list"): the damage types that bypass invulnerability are `out_of_world` and `generic_kill`
   (`respawn::BYPASSES_INVULNERABILITY`). A mod's damage type with that tag would not be known.
3. T2: the statistic `used` of the totem and the game event `item_interact_finish` are not made (no helper). Vibration
   sensors do not hear a save.
4. T4 (GAPS T4): no item animation on screen. A sound and a burst of particles for the players near, the player included.
5. T5 (GAPS T5): mobs that hold a Totem of Undying lose Fire Resistance too. The edit is in the default component,
   which mobs read.
6. T11: a dropped extra totem is thrown along the view with the spread of `Player.drop(stack, false)`. GAPS says "falls
   in place"; the code follows Java more closely. The pickup refusal is also in the eternal area (same answer).
7. T12 (GAPS T12): a second totem can sit in a container for up to one tick. The ender chest and a container that
   `native_container_slots` does not read are not enforced (UNVERIFIED which). A totem that goes into the player's own
   inventory slot of a menu is handled by T11, not by a slot rule.
8. T13 (GAPS T13): droppers are not stopped (no hook, needs the injection at `dropper_insert`). A hopper does not pick
   up a Totem of Eternity from the ground even when it holds none (Java accepts the first). Moves between containers
   follow Java.
9. T14 (GAPS T14): chest minecarts and chest boats are not enforced.
10. E6: none left after fix round 1. A repair that vanilla finds "Too Expensive" gets the result that Java's run
    gives with the penalty put aside (step 4 of "Fix round 1"). The max durability is the Java value 33000, not
    read from the item: a stack that a command gave another `max_damage` repairs by a quarter of 33000.
11. T1: the lang keys exist only in `en_us.json`; the other nine lang files do not have them (see "Changes outside").
12. T2, T3 (Horizon fault, semantics review, not fixed): a canceled `living_death` leaves the guest's body marked
    dead (`health.rs:993`, nothing clears it). So the swap tick cannot set the save effects or the health, and the
    player takes no damage until a new body (respawn, level change). The fallback "health 0" of a save with no
    totem left is refused too, so that save stands. No hook before `die()` can decide the save exactly (step 3 of
    "Fix round 1"). Horizon change needed: in `guest_die` (`lan_actions.rs:4316-4318`), when a mod cancels, also
    clear the body's flag (`motion.health.dead = false`), as the hook doc promises ("Cancelling keeps the entity
    alive at 1 health"). Or a hook with a `Game` at the death protection point of the hurt
    (`living_hurt.rs:187`, `checkTotemDeathProtection`), with the player swapped in: there the save is exact and
    immediate, as in Java.

## To check in the game

1. Save: with a Totem of Eternity in the inventory, `/damage @s 100 minecraft:generic` (or fall). Expected: alive at
   1 health after one tick, the totem became a Dormant Totem in the same slot, Regeneration II, Absorption II, Fire
   Resistance, the totem sound, particles, the advancement toast `eternity/saved`, and the log line
   `The Totem of Eternity saved <name> and is now dormant`.
2. `living_death` fires for the single-player owner and `player_tick` fires for him in the next tick (audit 6 says yes).
3. `/kill` and the void still kill a player with a totem; the totem stays a Totem of Eternity.
4. A Totem of Undying in a hand still saves first, with Regeneration and Absorption and no Fire Resistance (T5): look at
   the effect list after the save. If `items.json` has another shape than the test in horizon-items shows, the Fire
   Resistance stays.
5. `/give @s bmh:totem_of_eternity` twice: the second one drops at once; picking it up again is refused.
6. A chest that holds a totem: put a second in with a click and with shift click. Expected: it comes back to the
   inventory (or drops) within one tick. Check a double chest, a barrel, a shulker box and an ender chest (UNVERIFIED).
7. A hopper into a chest that holds a totem: the totem stays in the hopper. A hopper under a dropped totem does not take
   it.
8. Anvil: Hollow Totem (`bmh:dormant_totem`) left, Echo Heart right: result Totem of Eternity, 39 levels, one heart
   used. Typing a name changes nothing. Gold block on an Eternal tool: 15 levels, the repair cost component of the
   result equals the tool's.
9. Tooltips of the four items (Hollow Totem, Echo Heart, Totem of Eternity, Dormant Totem with "30" levels). If the tools
   or the ritual agent also add a heavy line or a Dormant Totem line, a line shows twice.
10. The sound source number 7 is the players' category (`eternal::PLAYER_SOUND_SOURCE`): the totem sound must be audible.
11. A count of 0 left in a slot for an extra totem that is not in the selected hotbar slot, or not plain: it must
    vanish at the next tick, and the totem in the hand must stay.
12. Fix round 1, the save on a dead body: after item 1, look for the log lines `The game refused the save effects
    of <name>` and `The game refused the save health of <name>`. If they show, the Horizon fault of difference 12
    is real: then `/damage @s 1` must also do nothing until a respawn.
13. Fix round 1, E6: give an Eternal tool a high penalty (`/give @s bmh:eternal_sword[repair_cost=40,damage=20000]`)
    and put it in an anvil with 2 gold blocks. Expected: a result with damage 3500, cost 15 levels, 2 blocks used,
    the result's penalty 40. Type a new name: the result has it, the cost stays 15. Check on a LAN guest too (the
    server answer).
14. Fix round 1, hoppers: with the mod on, a hopper under a dropped Totem of Eternity does not take it; turn the
    mod off (Options > Mods) and open another world: a hopper takes it.

## Changes outside my files

None. `horizon-mixin.toml`, the `hook_*.rs` helpers, `bmh.rs`, `lib.rs`, `ids.rs`, HOOKS.md and the lang files are as
they were. Report for the integrator: the lang keys `tooltip.bmh.eternity`, `totem_of_eternity_save`,
`heavy_with_eternity`, `sleeping_totem`, `dormant_totem` and `echo_heart` are in `en_us.json` only.

## Fix round 1

Fix agent of the totem area. Inputs: the reviews `docs/horizon/review/totem-parity.json` (1 major, 2 minor) and
`docs/horizon/review/totem-semantics.json` (1 major, 2 minor). No game rights: the proof is the two gates.

1. Guard: `git rev-parse --show-toplevel` printed this worktree. Read port.md, this log and both reviews.
2. Semantics major (dead body after a `living_death` cancel): HOLDS, by reading HN end to end.
   - `living_hurt.rs:186-193`: `hurt_server` calls `w.die(s)` when the health is 0 and `death_protection` is false.
     `health.rs:991-994`: the player's `die` sets `*self.dead = true`, and `dead` is `&mut h.dead` of the body
     (`health.rs:505`). `health.rs:394-395`: `dead_or_dying` is `health <= 0 || dead`.
   - The body is the guest's `motion` (`lan_actions.rs:1092` puts it back after each turn). Only
     `PlayerHealth::new` (`health.rs:372`) makes `dead` false: no other code clears it.
   - `guest_die` (`lan_actions.rs:4309-4319`): a cancel sets only the hub's `health = 1.0`. `enter_guest` copies that
     health into the body (`lan_actions.rs:532`, `set_health` does not touch `dead`).
   - `native_player_turn` (`native_player_state.rs:141-146`) returns None for an acting player whose body is
     `dead_or_dying`, so `native_set_effects` (`:296-306`) and `native_set_player_status` do nothing in the swap tick.
     `hurt_server` refuses every later hit (`living_hurt.rs:123`) until a new body (respawn, level change).
   - `native_living_death` is called only from `guest_die` for players (`lan_actions.rs:4309`).
3. Hooks before `die()`, checked for a move of the save:
   - `living_incoming_damage` v1 and v3 (`native_hooks.rs:9202`, `:7773`) run at `hurt_start`, before armour,
     enchantments, Resistance, absorption, the freezing factor and the helmet factor. The amount there does not
     tell whether the hit kills. No `Game`, no health.
   - `living_incoming_damage_v2` (`:9286`) is only for a player's weapon hit on a mob.
   - `living_damage_pre` (`:9237`, not wired) has the exact amount that leaves the health, but no `Game` and no
     health, and it cannot cancel. A save there needs the health at that moment: a value from the last player tick
     is wrong after any heal or loss between the tick and the hit. That is a guess, not a fix.
   - `living_damage_post` only hears. `player_totem` (`:8908`) only hears, and only for a totem in a hand.
   Result: no hook before `die()` can decide the save exactly. The save stays in `living_death` (finding "not
   fixed"). The swap now checks the answers of `native_set_effects` and `native_set_player_status` and logs a
   warning when the game refuses them, so a refused save is visible in the log.
4. Parity major E6: HOLDS (B_AnvilMenuMixin.java:89 sets the tool's penalty to 0 before vanilla's run; the old
   `on_anvil_update` returned at `let Some(result) = output.as_mut() else`). Fixed: when vanilla gives no result
   and a cost of 40 or more (`stations.rs:323`, "Too Expensive"; the server menu keeps that cost in `data[0]`,
   `menu.rs:1171`), the hook builds the result that Java's vanilla run gives with the penalty put aside: the
   repair loop of `createResult` (`logic::gold_repair`, 8250 a block), the rename (`logic::anvil_rename`), no
   result when the gold blocks' own penalty still makes it too expensive (`logic::gold_repair_refused`), the
   material cost = the blocks used. The penalty of the tool and the cost of 15 follow as before. Max durability
   33000 from BMHModTierInit.java:21. Unit tests: 5 new (`gold_repair` twice, rename, refusal, constants).
5. Parity minor (remove_totem): HOLDS (`native_server.rs:1059-1070`: `take_items` takes the selected slot first).
   Fixed: `take_items` only when the slot is the selected hotbar slot (`NativeServerPlayer.selected_slot`);
   any other slot gets a count of 0, so only the extra slot empties.
6. Semantics minor (ACTIVE never resets): HOLDS. Fixed: `ACTIVE_TICK` holds the start of a server tick that the
   gated `server_tick` hook opened (set at `ServerTickStart`, cleared at `ServerTickEnd`). Hoppers tick between
   the two (`levels.rs:509-597`). A world with the mod off never opens the window. A mark older than 60 s does not
   count (a mod turned off in the middle of a tick).
7. Semantics minor (pickup in the tick of a death): REFUTED. Every hurt that kills reaches `guest_die` before the
   pickups of a later turn or earlier in the same turn: nothing between `native_player_tick` and `guest_die`
   hurts (`lan_actions.rs:4249-4262`: map updates, progress, dig feedback, pickups, arrows, orbs). So a pickup
   there comes after the lethal hit. In Java the death is decided inside the hurt, and a player at 0 health picks
   nothing up (`Player.aiStep` touches entities only with health above 0; Horizon copies that guard for arrows,
   `creatures.rs:4210-4214` and `lan_actions.rs:5671`). A note in `item_picked_up` would let a totem picked up
   after the lethal hit save the player, which Java never does. No change.
8. Parity minor (hoppers refuse every totem): HOLDS, not fixed. `hopper_pickup_item` has no `Game` and no helper
   that reads the hopper (`game.rs:1299`); the hook gives only the level and the position. Needs a GAPS T13
   change (coordinator) or a Horizon helper that reads a container from that hook.
9. Set `status`, `outcome` and `note` of every finding in both review files.
10. Gates after the last code change: `sh horizon/tools/test-logic.sh` (214 passed, 25 of them in `totem`) and
    `sh horizon/tools/check.sh` (`"check": "ok"`). No game was run (no game rights). Changes outside my files: none.
