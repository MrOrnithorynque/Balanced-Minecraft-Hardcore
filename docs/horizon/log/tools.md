# Area tools: log

Branch: a throwaway worktree branch, reset to BASE c75751ff8fceedd6c3e9b9f4a87546f6bbcd9122. No game rights: the proof is the two gates
(`sh horizon/tools/test-logic.sh`: 233 passed; `sh horizon/tools/check.sh`: `"check": "ok"`). Nothing here was run in the game.
Rows: C12, E2, E3, E4, E5, E7, E8, E9. E6 (the gold block repair at the anvil) is not in my list: ARCHITECTURE.md gives the anvil to the totem
area (T6, E6). I wrote nothing for it and I did not touch `hook_anvil_update.rs`.

## Steps

1. Read port.md, ARCHITECTURE.md, HOOKS.md, the GAPS rows, audit 3, verify-3, the Java sources and the worked examples (eternal, amulet, respawn).
2. Facts found in the Nightly (HN) that change the design:
   - The dig speed code of the Nightly (`Items::mining_speed`, horizon-items items.rs:1135) reads the default rules of the item only. It does
     NOT read a `minecraft:tool` patch on the stack (verify-3 3c says it does; at this commit it does not). So E4 uses the pending hook
     `break_speed` (tier F, compiled in), not a stack patch.
   - `native_hurt_entity`, `native_send_particles_in_place` and `native_play_sound_in_place` work for the level in place only. So the
     Echo Strike echoes land from `player_tick` (the level of the player is in place), as Java did at the head of the player's tick.
   - `native_player_attack_cooldown_reset` fires on the Punch packet (lan_actions.rs:2368), after `reset_attack_strength` of the same swing.
     The estimate of the attack strength keeps the last two resets of a player (`Resets`).
   - `NativeServerPlayer` has `on_ground`, `position`, `entity_id`, `selected_slot` and `game_mode`. No helper reads the fall distance.
3. `logic/src/tools.rs`: wear rules, tooltip lines, bar, break speed, attack strength, fall tracker and Axe Smash numbers, Echo Strike (queue,
   victim, line), Echo Sense (ore tag, families, sphere, glow book, budget of 64), the `bmh:tools` message. 42 unit tests pin the Java values.
4. `bmh_tools.rs`: the hook bodies. `bmh_tools_lib.rs`: the blocks of fall that a smash took, for `living_fall`.
5. New hook `break_speed`: `hook_break_speed.rs`, its `[[injection]]`, its row in HOOKS.md (E4).
6. Gates run. Both pass.

## Decisions

- C12. `on_loot_drops` wears each Eternal tool in the drops of a player's own death by `logic::death_wear` (7 percent of 33000 = 2310, down
  to the last point), only in survival and only when the world rule `damage_eternal_item_when_dead` is on (KeepItemAfterDeath.java:38-46). It runs
  before the eternal area takes the stacks out of the list (HOOKS.md order: respawn, tools, eternal). The eternal area owns "kept through death";
  the wear is mine, as eternal's log says. The bundle and the totems have no wear (their `reduceDurability` does nothing).
- E2. `item_damaged` caps `amount` by `logic::cap_damage` on the stack of the main hand (else the off hand) with the same item id. The tool in the
  hand is read with `native_carried_stack(selected_slot)`, which works in a hook that swaps the player in. `inventory_tick` also puts a tool that is past
  its last point back on it: Java reaches the same state at the next wear (negative amount).
- E3. `living_incoming_damage_v2` halves the amount for a worn out tool (all three tools). Java scales only the one hit on the main target
  (`hurtOrSimulate`), and a sweep hit uses `hurtServer`. The hook is also called for the sweep victims, so only the first call of a player in a tick is scaled.
- E8. The fall is followed in `player_tick` (`logic::Fall`): it grows with each block of descent and goes back to 0 on the ground, in water, on a
  climbable block (tag `minecraft:climbable`), while gliding (pose `fall_flying`), while flying (`me.flying`) and under Slow Falling or Levitation
  (`native_effects`; the last two since fix round 1). The multiplier is `min(3, 1 + sqrt(2 * 0.08 * fall))` for a
  fall above 1.5. A smash that landed (`hurt_enemy` confirms it) halves the tracked fall, gives the removed blocks to
  `horizon_client_lib::bmh_tools_lib` and `living_fall` takes them off the landing. The entry is cleared when the player is on the ground again.
- E7. `living_incoming_damage_v2` notes a sword hit (damage after the scale, attack strength). `hurt_enemy` (the hit landed, Java's
  `causeFoodExhaustion` branch) queues the echo: 8 at most, 5 ticks, half the damage. `player_tick` of the attacker delivers the echoes that are due,
  drops the ones of another dimension, picks the nearest hostile mob within 5 blocks of the target (not the target itself), hurts it with
  `native_hurt_entity` (type `minecraft:player_attack`, attacker = the player), and only if that hurt worked: the cyan dust line, 6 electric sparks,
  the resonate sound and the advancement. A flag keeps the hurt of an echo from echoing.
  The hostile mobs come from `entity_join_level` (kind in the `Enemy` list of `logic::is_enemy`); their position is read when needed; a sweep every 100 ticks
  per dimension, in a player's tick of that dimension, drops the mobs that left (fix round 1). A dying mob is no candidate. A new main-hand item resets the
  attack strength (`note_held_item`, fix round 1). A death clears the echoes, the hit and the fall (`on_player_died`, fix round 1).
- E9. `destroy_block` (before the break) with the Eternal Pickaxe that is not worn out, on an ore of `bmh:echo_sense_ores` (since fix round 1 the
  server reads this tag and the family tags; the written copy answers only when the tags cannot be read), at most every 4 ticks per
  player: scans the sphere of radius 8 with `native_block(pos)` (an unloaded block reads as air, so no chunk loads), keeps a `GlowBook` of what shines
  (64 around a spot, a glow that is met again lasts longer), plays the chime and awards, and at the end of the tick sends `bmh:tools` `Glow` to the
  players of the dimension within 72 blocks. The client keeps the positions for 100 ticks of real time (50 ms each), drops a block that changed since the glow came (fix round 1: it cannot read the ore tag),
  and draws a box 0.96 wide through walls, colour 0x00F0FF, in `world_primitives`.
- E5. `tooltip` inserts the lines after the name (as the amulet does): ability (dark aqua) or worn out (red), heavy (gold), death wear (gray).
  `item_bar` keeps a bar of 1 pixel at least. The bar also tells the client whether the tool in the hand is worn out (see E4).
- E4 (fix round 1). `break_speed` (both sides): a worn out axe or pickaxe has its dig speed multiplied by `(4 + bonus) / (15 + bonus)` on the blocks
  of its tag (`minecraft:mineable/axe` or `pickaxe`), where bonus is the Efficiency level squared plus 1. That gives the Java value: Java caps the tool
  speed at 4 and then adds the bonus. Both sides read the main-hand stack with `native_equipment(MainHand)`. The server reads the tag and sends each
  answer once per player, tool and block (`bmh:tools` `Mineable`, at the end of the server tick). A client keeps the answers; with no answer it scales
  the speed too, so it never digs faster than its server.
- Awards. `run_command` refuses inside a player's hook, so an award is queued and given at the end of the server tick (the check "not done yet"
  is the one of the amulet).

## Differences from the Java mod

1. E3, E7, E8 (GAPS E3 lists the first part only): against a player target, a worn out tool does not halve the hit, the Eternal Sword gives no echo and
   the Eternal Axe gives no smash. The v2 damage hook and `hurt_enemy` hear hits on mobs only (native_hooks.rs:9092, 9279). Proposed GAPS line for E7 and
   E8: "Echo Strike and Axe Smash work on mob targets only: no hook hears a melee hit on a player with its weapon."
2. E4 (GAPS E4 allows the stack patch; this is another build, fix round 1): the numbers are the Java numbers (worn, Efficiency V: 30; with no enchantment:
   4; times Haste, Mining Fatigue, water and air). The bonus is computed from the Efficiency level of the stack (level squared plus 1, efficiency.json).
   Another source of `minecraft:mining_efficiency` (a data pack that changes Efficiency, a modifier of another mod) makes the scaled speed differ.
   A remote client with no answer from its server for a block scales any block: the first dig of a block that is not of the tool, with a worn tool, is
   slower than in Java for about one round trip (at most `(4 + bonus) / (15 + bonus)` of the speed), then exact. The answers stay in the client for the
   session: after a join to another server whose data packs change the mineable tags, an old answer counts until that server's answer comes.
3. E7 (GAPS E7): the attack strength is estimated from the resets of the punch packet (the sword's speed 2.6 only; Haste is ignored). A player with no
   known reset hits at full strength. "Enemy" is a list of hostile kinds. Java also spares tamed pets, the ridden mob, allies, invulnerable and
   unattackable mobs: none of these is tested (they are not hostile mobs, with rare exceptions). A dying mob is left out, as in Java (fix round 1). A mob
   that refuses the hurt (invulnerability frames) loses the echo, as in Java. Java reads the target's UUID; here the entity id.
4. E7: the echo is delivered in `player_tick`, which is the end of the tick of the player. Java delivers at its head.
5. E8 (GAPS E8): the fall distance is the mod's own. It resets on the ground, in water, on a climbable block, while gliding, while flying and under
   Slow Falling or Levitation (fix round 1). A passenger is not excluded (no helper tells it). A teleport down (an ender pearl, `/tp`) counts as a fall
   until the player lands; vanilla resets the fall for an ender pearl. A death clears it. The halving takes blocks off the landing; Java halves the value
   at the time of the hit. The result is the same.
6. E9 (GAPS E9): a box outline on the clients that have the mod, in place of a glowing block display. The server does not spawn any entity, so a client
   with no mod sees nothing. The glow starts before the break (the hook is before it) and a cancel by another mod does not undo it. The positions in a
   message are cut to 256. The server reads the ore tags as Java does (fix round 1). The client cannot read them: a glow ends when the block at its
   position changes, where Java ends it when the block leaves `bmh:echo_sense_ores` (the same for a mined ore).
7. E2: the tool in the hand is the one that wears. A wear of the other tools (a tool in the inventory) is not capped, and it does not happen.
8. C12 (GAPS C12): with `keep_inventory` the hook is not heard and the tools are not worn (the eternal area has the same note).

## To check in the game

1. `item_damaged` fires for the owner and `native_carried_stack(selected_slot)` answers there. Check: `/give @s bmh:eternal_pickaxe` with the
   `minecraft:damage` component at 32998 (or mine many blocks), mine one block, read the damage: it stays at or below 32999 and the pickaxe never breaks.
2. Death wear: `/give` an Eternal sword, die in survival, read the damage after the respawn: 2310 more. Die with the world rule off
   (`/bmh rule damage_eternal_item_when_dead false`): no change. Die in creative with `/kill`: Java wears only in survival.
3. `native_acting_player()` answers inside `loot_drops` for the dying owner (the survival test). If it does not, the test takes survival.
4. Sweep order. The first `living_incoming_damage_v2` call of a tick for a player must be the main target. Hit three zombies with the Eternal sword
   (a sweep): the log line `Echo Strike` must name a victim, and the damage dealt to the main target must be the unscaled damage (worn out: halved).
5. The attack strength: swing the Eternal sword twice quickly (under 7 ticks apart): no echo. Wait one second: an echo lands 5 ticks later on another
   hostile mob within 5 blocks (cyan line, spark burst, chime).  Check that the punch packet does not come before the attack in the same tick (if it does, `Resets` handles it).
6. `native_hurt_entity` works from `player_tick` for the owner (the level in place, the attacker named by its entity id). If it returns false every time, no echo shows.
7. The echo does not echo: the damage hooks hear the hurt, the flag `ECHOING` stops them. No second line in the log.
8. Axe Smash: jump from 10 blocks onto a zombie with the Eternal axe: the damage is about 2.26 times higher, the anvil and knockback sounds play, the landing
   hurts about half as much as after a free fall of the same height. Check `[BalancedMcHardcoreMain] Smash` in the log. Also check that the landing
   hook (`living_fall`) fires for the owner so that the softening applies.
9. Fall tracking: `native_entity_pose` returns `fall_flying` while gliding, `native_block_name_in_tag` answers for `minecraft:climbable` in a `player_tick` (an Err is read as "not climbing").
10. Echo Sense: mine an iron ore with the Eternal pickaxe next to other iron ore: boxes through walls for 5 seconds, a chime, the advancement
    `eternity/echo_sense`. `world_primitives` works with the single-player owner and the message reaches them (`bmh:tools`). Mine a glowing ore: its box ends.
    A second player sees it too.
11. E4, the most important one. As a LAN guest or a client of a dedicated server, with an Eternal Pickaxe at 32999 (`/give @s bmh:eternal_pickaxe[damage=32999]`):
    dig stone on the ground, while jumping, under water, with Mining Fatigue (`/effect give @s mining_fatigue`) and with Efficiency V (`/enchant @s efficiency 5`).
    In each case the block never comes back after the crack ends, and the dig time matches Java (stone, hardness 1.5, on the ground: worn with no
    enchantment, speed 4, 4 / 1.5 / 30 = 0.089 a tick, 12 ticks; worn with Efficiency V, speed 30, 0.67 a tick, 2 ticks). Dig dirt with the worn pickaxe just after joining: slower for a moment, then normal. Hold a worn and a
    fresh pickaxe in the hotbar and dig with each: only the worn one is slow. Test after a repair: the speed is back. Checks inside it:
    `native_equipment(MainHand)` gives the window's own stack in the client's prediction and the guest's stack on the server; `native_acting_player()`
    answers in `break_speed` on the server (else no answer is sent and the client stays slow on blocks that are not of the tool); the `Mineable` message
    reaches the client (no `Mineable answer not sent` line in the debug log).
15. E7, main-hand switch: hold another item, switch to the Eternal Sword and hit a zombie at once (another zombie within 5 blocks): no echo. Wait one
    second and hit: an echo.
16. E7, two dimensions: one player in the Overworld and one in the Nether for more than 100 ticks. Each hits a zombie or a piglin next to another mob:
    each gets its echo (the mob list is not emptied).
17. E7, dying mob: kill a zombie with a sweep next to a second zombie: the echo lands on the second one, not lost on the dying one.
18. E8: drift down 20 blocks under Slow Falling (`/effect give @s slow_falling`) onto a zombie with the Eternal Axe: no smash (no anvil sound). Fly down
    in creative, switch to survival in the air above a zombie and hit it while still falling less than 1.5 blocks: no smash.
19. Death: hit a zombie with the sword next to another one and die within 5 ticks (`/kill @s` right after the hit): no echo after the respawn.
    `player_died` fires for the owner and for a guest.
20. E9 tags: with a data pack that adds a block to `minecraft:iron_ores`, mine iron ore next to that block: it glows with the iron. Mine a glowing ore: its
    box ends.
12. Tooltips: the three lines show after the name; the red line shows for a tool at 32999 and the ability line returns after a repair. `Loses 7%% of its
    durability per death` prints `7%` (the open question 6 of content NOTES).
13. The bar of a worn out tool is one pixel.
14. `native_play_sound_in_place` and `native_send_particles_in_place` answer true from `player_tick`, `hurt_enemy` and `destroy_block` (return values are ignored).

## Changes outside my files

- `horizon/native/mixins/bmh/hook_break_speed.rs` (new): the helper of the new hook `break_speed`, one call to `bmh_tools::on_break_speed`.
- `horizon/native/mixins/bmh/horizon-mixin.toml`: one `[[injection]]` `hook_break_speed` at the end, on `native_break_speed` (pending fragment, compiled into the Nightly). Bound: `game`, `pos`, `speed`.
- `docs/horizon/HOOKS.md`: one row for `break_speed` after `world_primitives`, and the counts line (67 hooks, 4 pending).
- Fix round 1: `horizon/native/mixins/bmh/hook_player_died.rs`: one line, `crate::game::bmh_tools::on_player_died(game, player, cause);` after the
  respawn call. Reason: Java keeps the echo queue and the fall on the ServerPlayer, which a respawn replaces (review tools-semantics, finding 4). It changes
  nothing that another area's function receives.
- Fix round 1: `docs/horizon/HOOKS.md`: the `player_died` row names "respawn, tools".
- Not changed: `wire.rs` (the `bmh:tools` message and its tests are in `logic/src/tools.rs`, with `wire::Reader` and `wire::Writer`), `lang` files (every key of the area exists in `en_us.json`), `bmh.rs`.
- The other hook helpers already called my functions; their signatures are unchanged.

## Fix round 1

1. Tree guard passed (git rev-parse printed the worktree). Read port.md, the log, tools-parity.json (1 major, 5 minor) and tools-semantics.json (3 major, 1 minor).
2. Evidence check, all ten findings: every quoted line is in the file at the cited place (ours, HN and Java). None is refuted. Facts found for the fixes:
   - `hand.rs:410-443` (`destroy_progress`) runs on both sides. The client's `native_block_name_in_tag` gives `Err(NotServer)` (`native_world_read.rs:30`). No helper reads a tag or the tool speed on a client.
   - `native_equipment(EquipmentSlot::MainHand)` (`native_host_bridge.rs:97`, `&self`) gives the full main-hand stack on both sides: on a client the window's own, on a server the acting player's. `ItemStack::enchantment_level` (horizon-items `item_data.rs:317`) reads Efficiency from it.
   - Efficiency is the attribute `minecraft:mining_efficiency`, `levels_squared` plus 1 (resources 26.3 `enchantment/efficiency.json`), added by `modify_destroy_speed` (`enchant_effects.rs:41-50`) when the tool speed is above 1, before the other factors.
   - The server calls `destroy_progress` for a guest at START (`lan_actions.rs:2183`) and at STOP (`lan_actions.rs:5442`). A client that digs slower than its server is safe: the server breaks the block at STOP when its own count is at least 70 percent. A client that digs faster gets the block back (`StopDig::Delay`).
   - `native_mob(dimension, id)` (`native_mobs.rs:222`) gives only a living mob of the level in place. `native_effects(uuid)` (`native_player_state.rs:252`, pending `player_effects`, compiled in) lists the effects. `NativeServerPlayer.flying` exists.
3. `logic/src/tools.rs`: E4 `worn_break_speed(speed, mineable, efficiency_level)` scales by `(4 + bonus) / (15 + bonus)` (bonus = level squared plus 1), and scales also when `mineable` is None; `MineableAnswers` (the client's copy of the server's tag answers, capped at 4096); message `ToolsSignal::Mineable` (tag 2: tool byte, block name, bool). E8 `FallInput` has `flying` and `floating` (Slow Falling, Levitation). E9 `FAMILY_TAGS`, `ORE_TAG`, `listed_in_tag`, `same_ore(broken, other, in_tag)`, `find_same_ore(center, block_at, same)`. Unit tests: 239 passed.
4. `bmh_tools.rs`: E4 `on_break_speed` reads the main-hand stack with `native_equipment` on both sides, the Efficiency level from it, and the tag on the server; the server queues one `Mineable` answer per player, tool and block (`tell_client`, sent in `send_answers` at the end of the server tick); the client reads its `MineableAnswers`. `item_bar` sets no flag any more (`worn_in_hand` removed). E7: `note_held_item` resets the attack strength on a new main-hand item; `sweep_mobs` runs in the player's tick for that dimension; `is_alive` leaves a dying mob out of the candidates. E8: `flying` and `floating` fed from `me.flying` and `native_effects`. E9: the server reads `bmh:echo_sense_ores` and the family tags (`in_tag`, the written copy when the tags cannot be read); the client ends a glow when the block at its position changes. State: `on_player_died` and `forget_body`.
5. Integrator change: `hook_player_died.rs` calls `bmh_tools::on_player_died` after respawn; HOOKS.md row `player_died` says "respawn, tools".
6. Gates: `sh horizon/tools/test-logic.sh`: 239 passed, 0 failed. `sh horizon/tools/check.sh`: `{"mods": ["bmh"], "horizon": "fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca", "check": "ok"}`, no warning in the full log.
7. Outcomes written in both review files (`outcome` and `note` per finding): parity 5 fixed, 1 not fixed (PvP: difference 1, a GAPS line proposed); semantics 4 fixed. The sections "Decisions", "Differences", "To check in the game" (item 11 rewritten, items 15 to 20 new) and "Changes outside my files" above are updated. No game rights: nothing here ran in the game.
