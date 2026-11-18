# Area brief: eternal (wave 1 of the run of 2026-11-18)

Read `docs/horizon/briefs/port.md` first and follow it exactly. This file is the part of your prompt that is about
your area. Your prompt gives BASE (the sha your worktree must be on).

No game in this step. Do not run `check.sh build`, `test-slice.sh`, `play.sh` or the game: another agent has the
game rights. Your proof is the two gates of port.md. What only a game can show goes in "To check in the game".

## Your area

The Eternal Items as things a player owns: they stay with the player through death, a player holds one of each
type, they never despawn on the ground, the Eternal Bundle, the rewards of a night of sleep, and the hidden
coordinates of the debug screen. The owner played the game on 2026-11-18 and reported that F3 shows the
coordinates: that is your row C14.

## Your files

- `<R>/horizon/native/mixins/bmh/bmh_eternal.rs` (the module).
- `<R>/horizon/native/mixins/bmh/logic/src/eternal.rs` (the logic, with its unit tests).
- `<R>/docs/horizon/log/eternal.md` (your log, from your first step).

## Your rows of GAPS.md

C11 (Eternal Items kept through death), C13 (the wake-up rewards; tier F: pending fragments, allowed), C14 (hide
the coordinates in survival with `reduced_debug_info`; tier F, with the tier N restore that the row names), E10
(Eternal Items on the ground), E11 (one Eternal Item of each type), E12 (the Eternal Bundle). Read also GAPS.md
section 3 for the substitutes and section 7 for the open questions that touch these rows.

## Audit sections

`<R>/docs/horizon/audit/1-respawn-server.md` and `verify-1.md`, rows 8a (sleep), 8b (kept through death), 8c (one
of each type), 8d (the debug screen). `<R>/docs/horizon/audit/3-items-tools.md` and `verify-3.md`, rows 5 (items
on the ground), 6 and 6b (kept items, one of each type), 7 (the bundle). Where an audit and its verification
differ, the verification wins. `<R>/docs/horizon/audit/6-single-player-owner.md` for the traps of `give_stack`
outside a player's own hook.

## Java sources

Under `<R>/common/src/main/java/github/mrornithorynque/bmh/`: `handlers/KeepItemAfterDeath.java`,
`handlers/EternalItemHandler.java`, `handlers/SleepHandler.java`, `handlers/DisableDebugInfo.java`,
`items/EternalBundleItem.java`, `items/IEternalItem.java`, `items/EternalTooltips.java`,
`mixin/G_ItemEntityMixin.java`, and the event wiring and its order in
`<R>/neoforge/src/main/java/github/mrornithorynque/bmh/neoforge/NeoForgeEvents.java`.

## Contracts (HOOKS.md, "Contracts": keep every signature)

- Which items are Eternal: `bmh_logic::ids::is_eternal` and `ids::ETERNAL_ITEMS`. The type of the rule "one of
  each type" is the item id.
- You READ `bmh_settings::rule(game, Rule::...)` and `bmh_settings::config(game)`. Each feature of your area that a
  Java game rule switches must read the same rule here: find the rule in the Java handler.
- You SHARE the hook `loot_drops` with the respawn area. Respawn counts the Eternal Items in the drops (row C1) and
  you take them out of the drops (row C11). The helper `hook_loot_drops.rs` calls the areas in a fixed order: the
  count must see the stacks before you remove them. Read the helper and HOOKS.md, keep that order, and write a
  unit test of your side that does not depend on it.
- You SHARE `player_respawned` with respawn and journey. You give the kept stacks back there. Read
  `hook_player_respawned.rs` for the order and do not change what another area's function receives.

## Points to settle yourself, by reading HN, and to record in your log

- C14 first, because the owner asked. Read `DisableDebugInfo.java` for the exact condition (which game modes, which
  rule, when it restores). Then read in HN what `reduced_debug_info` hides on the pinned Nightly's debug screen: if
  the Nightly's F3 does not read that game rule, say so plainly in your log and in your result, with the file and
  line. That would be a gap of the host game, not a thing to work around with an injection (D8).
- C11: where the kept stacks wait between the death and the respawn, so that a restart of the game while the
  player is dead does not lose them (the respawn area has the same case: read `docs/horizon/log/respawn.md`).
- C13: the pending fragments `player_food`, `player_status` and `player_effects`. Read each call site. The N
  fallback is not shipped (GAPS C13).
- E11: what the Java code does with the extra item (drop it, where, with which delay before pickup) and how
  `item_picked_up` refuses a pickup on the Nightly.
- E12: the bundle is the vanilla bundle template. Find how its contents are read and changed from a hook.

## Result

The structured result that port.md asks for. Add: the list of checks that the game agent of this wave must write
for your area (each one: what to do with `horizon-ctl` or a command, and what must be true), most important first,
C14 at the top.
