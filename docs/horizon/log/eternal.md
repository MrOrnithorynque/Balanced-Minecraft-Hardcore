# Eternal area: port log

Area: eternal (GAPS.md rows C11, C13, C14, E10, E11, E12). Files: `horizon/native/mixins/bmh/bmh_eternal.rs`,
`horizon/native/mixins/bmh/logic/src/eternal.rs`, this log. No game was run in this step: the proof is a compile and
35 unit tests. What only the game can show is in "To check in the game".

## Steps

1. The worktree was not on BASE 72f0fc8 (`merge-base --is-ancestor` exit 1, no `horizon/native/mixins/bmh`).
   Branch `worktree-agent-ac59114b17226f1ed`, root `/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/agent-ac59114b17226f1ed`,
   not the coordinator's tree. Ran `git reset --hard 72f0fc8aded3e4ca92139e16f8024820c010995d` once. Guard file present.
2. Read port.md, area-eternal.md, ARCHITECTURE.md, HOOKS.md, GAPS.md (rows and sections 3, 4, 7), audit 1 section 8,
   verify-1 rows 8a to 8d, audit 3 sections 5 to 7, verify-3 rows 5 to 7, audit 6, the respawn log, and the Java
   sources (KeepItemAfterDeath, EternalItemHandler, SleepHandler, DisableDebugInfo, EternalBundleItem,
   IEternalItem, G_ItemEntityMixin, NeoForgeEvents).
3. C14 first. Read HN for the debug screen (next section).
4. Read the rest of HN for each row (paths in `crates/horizon-client/src`), with these findings:
   - Death order: `game.rs:4294-4318` `drop_inventory` runs inside the dying guest's turn, calls `native_loot_drops` with
     `entity` the player, `player` set, `table` empty, and does nothing with `keep_inventory` (nothing is kept, nothing is
     worn: GAPS C11). API 405 is in this Nightly (`game/native_hooks.rs:21`). The stacks that stay in `drops` are thrown with a
     pickup delay of 40.
   - `player_respawned` (`game/lan_actions.rs:4083`) runs `as_guest`, so the player is swapped in and `give_stack` reaches
     them (`game/native_server.rs:1110-1142`, the `uuid == native_acting_id()` branch).
   - Persistent data: `native_player_persistent_data` and `native_set_player_persistent_data` (`game/native_player_state.rs:385-408`)
     use `level.player_extra` for the swapped-in player, and `lan_actions.rs:630` and `:1021` swap it with the guest's `saved`.
   - `player_bed_leave` has two call sites. `game.rs:15154` (`wake_up`) runs in the player's own turn. `game/lan_actions.rs:4662`
     (`wake_guest_from`) runs with the hub taken out and the guest not swapped in (`lan.rs:2755`: `self.lan` is None there). In
     that case the helpers `player_food`, `set_player_status` and `add_effect` find nothing (`native_place`,
     `game/native_player_state.rs:109-123`). The morning skip and any wake from a packet take that path. So C13 queues the
     player and gives the reward at the end of the next `server_tick`, where `native_place` finds a placed guest.
   - `inventory_tick` (`game/native_hooks.rs:11542-11571`): an empty slot is skipped, so there is no "first slot" marker. The
     order is slots 0 to 35, armour 103 down to 100, off hand -106. A copy is found by looking at the earlier slots.
   - `item_picked_up` (`game.rs:4788`, `game/lan_actions.rs:5619`, `game/native_hooks.rs:11461`): the player is swapped in, so
     `native_carried_stack` reads their slots. A refused item waits 10 ticks (`item.pickup_delay = 10`).
   - `item_expire` (`game.rs:4700-4710`): `age = 6000 - extra_life.min(32766)`. The item asks again at age 6000.
   - `item_spawn` (`game.rs:4337-4378`) is called from `spawn_item`, the one place that creates a dropped item (a drop, a death,
     a break). `native_entity_merge_nbt` loads `Health` as a short (`entity_nbt.rs:2481`, `:2606-2611`). `apply_entity_nbt`
     replaces the item entity (`game/commands/data.rs:387-393`).
   - `use_item` (`game.rs:7208`): the hook has no hand. The main hand goes first, so the area claims the click for the main hand
     when it holds an Eternal Bundle, else for the off hand.
   - `native_summon` of `minecraft:item` (`game/native_entities.rs:206-250`) needs the player's own dimension and a loaded chunk.
     Both hold in a player's hook. It does not call `item_spawn`, so the area puts `Health` in the summoned NBT itself.
   - Horizon's bundle: `horizon-items/src/bundle.rs:94-97` refuses only shulker boxes. `StackData.bundle_contents` is public and
     `bundle::destroyed` empties a bundle and resets its selection.
   - Content: all six Eternal items have a stack size of 1 and `damage_resistant` `#minecraft:is_fire`
     (`horizon/content/data/bmh/horizon/item/*.json`).
5. Wrote `logic/src/eternal.rs`: the constants with their Java lines, `take_eternal`, `keep_after_death`, the one-of-each rules
   (`rule_applies`, `slot_order`, `earlier_slots`, `is_copy`, `copy_drop`, `blocks_pickup`), the drop position and velocity,
   the numbers of an item on the ground, the bundle test and sound, `sleep_reward` and `after_reward`, and `DebugForce` (the
   state machine of DisableDebugInfo). 35 unit tests.
6. Wrote `bmh_eternal.rs` on it (the flow is at the top of the file). Both gates passed on the first compile.

7. Fix round 1. One line per finding (reviews `docs/horizon/review/eternal-semantics.json` and `eternal-parity.json`):
   - semantics, major, `SERVER.kept` duplicates across worlds: fixed. `bmh_eternal.rs` `waiting_for`, `store_waiting`,
     `write_persisted`, `read_persisted`, `on_server_tick`; `logic::Fallback`. The compound is the truth, the memory list exists
     only after a refused write, and `Fallback::new_world(context.epoch)` empties it when the world epoch changes (the same signal
     as the debug state). Paths reasoned through:
     - Death then respawn: `loot_drops` writes the compound, the respawn reads it, gives each stack, then clears the key. One give.
     - Restart while dead: the compound is on disk (if the game saves it for a dead player: check 4), the memory is empty, the
       respawn reads the compound. One give.
     - Change of world while dead: world A keeps its compound, so the stacks come back in world A only. World B reads the compound of
       world B (none) and a memory list from A is gone (new epoch). No stack crosses over, and none is given twice. If A's write had
       failed, the stacks were in memory only and are lost with the world change (logged at the death as "wait in memory only").
     - Two deaths in a row: the second `loot_drops` reads the waiting list and appends the new stacks (`keep_after_death`). A death
       with nothing to keep leaves the list as it is. Java replaced the list.
     - Full inventory at respawn: `give_stack` returns the rest, `drop_stack` throws it at the feet. If that fails, the stack is
       written back to the compound and waits for the next respawn.
     - Not covered: a restart between the gives and the clear of the compound would give the stacks twice. The window is inside one
       hook call.
   - semantics, minor, a failed stack is in memory only: fixed. The compound is cleared once, after the gives, with the failed
     stacks in it. A new death appends to them. They return at the next respawn. There is no retry on a player tick.
   - parity, minor, E12 empty bundle blocks the other hand: fixed in the part the hook controls. `on_use_item` now takes the first
     hand whose bundle holds something. An empty main hand bundle no longer hides a loaded off hand bundle. What the host does
     with the click is fixed: a claim ends in `swing_arm` and `Ok(true)` (HN game.rs:7210-7213). A click that is not claimed reaches the
     template bundle use, which also ends in `start_using_item`, `swing_arm` and `Ok(true)` for every bundle (HN game.rs:7321-7325). So with
     two empty bundles, or an empty bundle in the main hand and a shield in the off hand, the other hand does not act, and the arm
     swings (Java: FAIL, no swing, the off hand acts). The hook cannot return FAIL (`use_failed` is set only by the host). The area
     claims the click in that case, so the hand does not rise for the template use. Listed as difference 9.
   - parity, minor, E12 bundles outside the player inventory: not fixed, no wired hook gives the moment (`inventory_tick` sees only
     the carried slots). Listed as difference 10.
   - parity, minor, E10 items not made through `spawn_item`: not fixed. `item_expire` is wired and fires at age 6000, but it would
     merge NBT into an entity during its own tick, which the area did not verify and the brief does not allow to guess. Listed as
     difference 11. Proposal for a later round: in `on_item_expire` call `native_entity_merge_nbt` with `Health` 32767 after a game
     check that the item keeps its age and position.
   - parity, minor, E10 cactus and lightning: not fixed, the content file is not the area's. Difference 5 is extended. Proposal for the
     integrator below.

## C14: what the Nightly's F3 reads

Read in HN (`crates/horizon-client/src`):

- `game/debug_keys.rs:603-607`, `reduced_debug_info()`: true when the option is set, or `self.ui.rules.bool("reduced_debug_info")`,
  or `self.remote.reduced_debug_info`.
- `debug_entries.rs:68`, `:144`, `:339-346`: the entry `player_position` is registered with `reduced = false`, and
  `enabled(path, overlay, reduced)` answers `shown && (e.reduced || !reduced)`. So with the rule on, the XYZ line of F3 is hidden.
  The entries `looking_at_block_*`, `biome`, `heightmap`, `light_levels` and the others registered with `false` hide too.
- `game/debug_lines.rs:42` and `:46` draw only the enabled entries. `game/debug_keys.rs:311` and `:631`: F3+C (copy the location)
  needs `!reduced`. `game/pvp.rs:265` hides the PvP HUD coordinates when reduced.
- How the window learns the rule in single player: `game/lan.rs:2729-2730` copies the server's `ui.rules.bool("reduced_debug_info")`
  to the hub each server tick. `lan/hub.rs:836-846` sends entity event 22 (reduced) or 23 to every placed guest, and the owner is
  a guest. `game/remote.rs:4584-4588` sets `remote.reduced_debug_info` on the window's game. The Login packet carries it too
  (`lan/hub.rs:1418`).

Verdict: the Nightly's F3 does read `reduced_debug_info`. It is not a gap of the host game, and no injection into a game function is
needed (decision D8). At BASE the `on_server_tick` body of this area was empty, so nothing forced the rule. That is the most likely
reason why the owner saw the coordinates on 2026-11-18. This is not proven: the game was not run in this step (check 1 below).

## The decisions of the area

- C11, where the stacks wait (changed in fix round 1). The compound that the mod keeps on the player, under the key
  `bmh_eternal_kept`, is the one source of truth. It is written at the death and read at the respawn. `SERVER.kept` (a
  `logic::Fallback`) holds a list only when the game refused the compound, and it is forgotten at the first server tick of a new
  world epoch. A stack that `give_stack` cannot place is dropped at the feet. If that fails too, the stack is written back to the
  compound (and to memory if that fails), and the respawn clears the compound only after all the gives.
- C11, the wear. The 7 percent death wear belongs to the tools area (C12). It runs before this area in `loot_drops`. This area only
  moves the stacks out of `drops`. The unit test `the_split_does_not_depend_on_what_other_areas_did_to_the_drops` pins that the
  split does not depend on the order.
- C14, the restore. The area reads the rule with the pending helper `level_game_rule` (tier F, the fragment `level_world` that the
  respawn area already compiles against). So the value before is the real one, as in Java. When the helper answers nothing, the
  area takes the last value it set, else false (the tier N restore that GAPS C14 names).
- C14, the world. The state is forgotten when the world epoch changes (Java: the weak reference to the server).
- C13, the timing. The reward comes at the end of the server tick that follows the wake-up, not at the wake-up. Health and food are
  read then.
- E11, the Totem of Eternity. Java refuses a second one always. This area does the same (`blocks_pickup`). The totem area may do it
  as well: both only set `canceled`.
- E12, the empty bundle (changed in fix round 1). The hook has no FAIL. The area picks the hand whose bundle holds something
  (`logic::first_loaded`, main hand first). When no hand has one, the click is claimed and nothing else happens. See step 7.

## Differences from the Java mod

1. C11 (GAPS C11): a Vanishing curse Eternal Item is gone before `loot_drops`. With `keep_inventory` the hook is not heard: the
   items stay, with no 7 percent wear (Java wore them in survival and moved them out and back).
2. C11, allowed by the brief ("a restart ... does not lose them"): the stacks survive a restart (Java lost them). A stack that
   does not fit at the respawn is dropped (Java lost it).
3. C13 (GAPS C13): the reward comes at the end of the tick of the wake-up, not at the wake-up. The player must be placed in the
   world then. The heal is `health + heal`, capped by the game at the maximum health (vanilla `heal` does the same).
4. C14 (GAPS C14): `reduced_debug_info` is set by the console command `gamerule reduced_debug_info <value>`, at most once a tick.
   An operator who changes the rule by hand is overruled on the next tick (Java too). The restore skips the command when the
   value is already right (Java sets it without a test, which changes nothing). Where the read helper answers nothing, the restore
   sets false.
5. E10 (GAPS E10): explosions still take health away from a dropped Eternal Item (about 6500 blasts). The content mod gives the six
   items `#minecraft:is_fire` only, so a dropped Eternal Item that lies in a cactus loses 1 health a tick and lasts about 27
   minutes (Java: it never dies). Lightning is not covered either (Java ignores every damage). Content change for the integrator: add a damage type tag
   `bmh:eternal_resistant` (file `horizon/content/data/bmh/tags/damage_type/eternal_resistant.json`, or the folder that the
   content mod uses for tags) with the values `#minecraft:is_fire`, `#minecraft:is_explosion`, `#minecraft:is_lightning` and
   `minecraft:cactus`. Then set `"minecraft:damage_resistant": { "types": "#bmh:eternal_resistant" }` in the six files
   `horizon/content/data/bmh/horizon/item/eternal_*.json`, `totem_of_eternity.json` and `sleeping_totem.json` (line 6 of
   `eternal_bundle.json`). Also correct the GAPS.md row E10 ("fire and cactus"): until then it names cactus and the content does not. This would
   remove the blast gap too. Whether Horizon reads a mod tag in `damage_resistant` is not checked.
6. E11: the copy is found by comparing with earlier slots in the hook's order (armour from head to feet). Java scans the armour from
   feet to head. An Eternal Item does not go in an armour slot, so the result is the same. A copy drops in front of the player with a
   pickup delay of 40 ticks, as `Player.drop(copy, false)` does. For a stack of more than one, Java drops a copy of the whole stack and
   takes one off the original (`copy_drop`). All six items have a stack size of 1, so that case does not happen.
7. E12 (GAPS E12): a refused stack sits in the bundle for up to one tick. It comes back through `give_stack`, so it goes to the first
   free slot, not to where it came from (Java: it never left its slot or the cursor). The `used` statistic of the bundle does not
   count when it drops its contents (no public helper). The drop has the same sound, volume, pitch and scatter as Java.
8. E12: with a loaded bundle in both hands, the main hand is used (as Minecraft does).
9. E12, the empty bundle: the hook has no FAIL. The arm swings, and the other hand does not act after an empty bundle (a shield,
   a bucket, an ender pearl, a second empty bundle). Java gave the click to the other hand. A loaded off hand bundle is used when the
   main hand bundle is empty. HN game.rs:7210-7213 and :7321-7325.
10. E12: the refusal of an Eternal Item and an Ender Chest works only for a bundle in the player's inventory slots (`inventory_tick`).
    A bundle in a chest, an Ender Chest, a shulker box or on the cursor keeps a refused stack until the bundle comes back into a slot
    of the player, so an Eternal Item can be stored in such a bundle. Java refused it at the insert in every menu. Needs a hook for a
    container click (not wired).
11. E10: only an item made by `spawn_item` (or summoned by this area) gets the health of 32767. An Eternal Item on the ground from a
    save made before the mod, or made by a path that skips `spawn_item`, keeps the vanilla health and a blast or a cactus can destroy
    it. The life time is fixed for every Eternal Item at age 6000.
12. C11: a list kept in memory because the game refused the compound is lost when the world changes or the game restarts.

## To check in the game

Seventeen entries. The three that matter most are 1 (C14), 3 (C11) and 5 (C13). "Type" means the chat line or a `horizon-ctl`
command that does the same.

1. C14, the owner's report. New survival world, mod on, rule on (the default). Press F3: the XYZ line must be gone, and "Looking at
   block", "Biome" and "Local Difficulty" with it. Type `/gamemode creative` for the only player: within a tick the coordinates come
   back (the rule goes back to its value before). Type `/gamemode survival`: they go again. Type
   `/bmh rule remove_coordinates_in_survival false`: the coordinates come back. The log has no WARN line "did not run".
2. C14, `run_command` works from the end of `server_tick` on the integrated server (GAPS section 7, question 1) and `level_game_rule`
   answers there. Type `/gamerule reduced_debug_info`: it must say `true` in survival with the rule on. If `level_game_rule` answers
   nothing, the first tick sets the rule and the restore sets false (check with an operator value of true: it is lost).
3. C11. Type `/give @s bmh:eternal_sword`, then the axe, the pickaxe, `bmh:eternal_bundle` (put a stone in it),
   `bmh:totem_of_eternity` and `bmh:sleeping_totem`, then `/kill`. After the respawn all six are in the inventory, the bundle still
   holds its stone, and no Eternal Item lies at the death spot. A `bmh:dormant_totem` and a `bmh:navigator_amulet` in the same
   inventory drop on the ground.
4. C11 after a restart. As check 3, then `quit` the game on the death screen, start it again on the same world and respawn: the six
   items come back, and a later death gives no second copy. Read `players/data/<uuid>.dat` of the world before the respawn
   (`bmh_eternal_kept` must be in `NeoForgeData`) and after it (gone). If it is missing, the compound is not saved for a dead
   player and a restart loses the items as Java did.
5. C13. Survival, health 20 and food 20, `/time set night`, sleep in a bed. After the morning skip: health 20, food 16 (the
   exhaustion of 4 makes it fall a little more), and Haste II and Resistance II in the effects list for 3 minutes, with no particles.
   With health 14: health 20 and no effect. With food 12: heal 4, food 10. Food 7: heal 3, food 6. Food 3: heal 1, food 2. Food 1: nothing.
   Leave the bed by hand (Escape): the same reward, once.
6. C13. Type `/bmh rule food_and_health_regen_on_wake_up false`: no reward.
7. E11. Survival: `/give @s bmh:eternal_sword` twice. The second sword appears on the ground in front of the player at once and
   the player does not pick it up. Throw the first one with Q, pick it up: it works, and the other one is refused. Creative and the
   rule off (`/bmh rule only_carry_one_eternal_item_type false`): two swords are kept.
8. E11. A Totem of Eternity and a Dormant Totem can be carried together (two types).
9. E10. Throw an Eternal Sword on the ground. `/data get entity @e[type=item,limit=1]` shows `Health: 32767s`, and the sword keeps its
   damage and name (the NBT round trip of `native_entity_merge_nbt`). Wait past 5 minutes (or `/tick sprint 6100`): it is still
   there, `Age` went back to about -26766. Drop it in lava and on a fire: it stays. Let TNT explode next to it: it stays.
10. E12. Put an Eternal Sword and an Ender Chest into the Eternal Bundle (right click with the item on the cursor, and with the
    bundle on the cursor onto the item): after one tick both are in the inventory, not in the bundle. A stack of dirt stays in. With
    a full inventory the refused stack drops on the ground.
11. E12. Right click with the bundle holding dirt and a stone: all stacks drop at once, scattered, with the bundle sound, and the
    bundle is empty. An empty bundle: nothing happens (no one by one drop). The same from the off hand.
12. The order of `loot_drops`. Die with two Eternal Items: the exile distance counts both (respawn area), worn tools come back worn
    (tools area) and all of them come back (this area).
13. Two players (LAN): the reward, the copy rule and the pickup rule work for a guest as for the owner. `reduced_debug_info` is one
    rule for the world.
14. `give_stack` at the respawn, from `player_respawned`, works for the single-player owner (audit 6 trap): check 3 shows it.
15. C11, change of world while dead (fix round 1). Single player. In world A: `/give @s bmh:eternal_sword`, `/kill`, and on the death
    screen quit to the title. Open world B (or create it), die there with no Eternal Item and respawn: no sword. Open world A again,
    respawn: one sword, and `players/data/<uuid>.dat` of A has no `bmh_eternal_kept`. Quit and open A again: still one sword in total,
    no second one. Also die twice in A before the first respawn is possible (Horizon kills a player who logs in dead): the
    sword is given once.
16. C11, a full inventory at the respawn. Die with an Eternal Sword and a full inventory of items (kept by `keep_inventory` off:
    the others drop). After the respawn the inventory is empty, so the sword fits. To test the full case, fill the slots in a world
    where a mod gives items at the respawn, or read the log line "did not fit and could not drop" in a forced failure.
17. E12, the empty bundle. Right click with an empty Eternal Bundle in the main hand: the arm swings and nothing drops. With a loaded
    Eternal Bundle in the off hand and an empty one in the main hand: the off hand bundle drops its stacks. With a shield in the off
    hand: note whether it rises (expected: no, see difference 9).

## Changes outside my files

None. No hook helper, manifest, `bmh.rs`, `lib.rs`, `ids.rs`, `wire.rs`, `contract.rs`, HOOKS.md, GAPS.md or content file was
changed. No lang key was added (the area shows no text).

## Gates

- `sh horizon/tools/test-logic.sh`: all unit tests pass (35 are the area's).
- `BMH_HORIZON_CACHE=/Users/louisobenicherousse/Library/Caches/bmh-horizon sh horizon/tools/check.sh`: `"check": "ok"`.
