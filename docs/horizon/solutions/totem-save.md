# Totem of Eternity save (GAPS T2): routes and verdicts

Session `gap-totem-save` (branch `claude/gap-totem-save-d5104c`, base `5da909a`), 2026-11-19. Source read: the
Nightly tree at `~/Library/Caches/bmh-horizon/horizon-mods/native/target/horizon/fdc87aec.../crates`. Paths below are
relative to `crates/`.

Status: research done, route chosen, NOT built and NOT run in the game. The session stopped at 57 % of the 5-hour
window (the stop is 60 %, and three other sessions use the same account). Checks `t1` to `t5` are still red.

## 1. The real root cause (new)

GAPS T2 says "Horizon never calls `living_death` for the owner". That is true, but the cause is not the owner. It is
the build:

- In single player the owner is a guest of the integrated server, like a LAN guest. Proof: `native_player_tick` has
  one caller, the guest loop (`horizon-client/src/game/lan_actions.rs:4249`), and the mod's `player_tick` fires for
  the owner (t7 passes). `Game::tick` hands the window to `tick_remote` (`horizon-client/src/game.rs`, "Only a server
  ticks its own world from here").
- `guest_die` (`lan_actions.rs:4292-4319`) calls `native_living_death` only when
  `!horizon_client::mod_api::mods().is_empty()`.
- `mods()` is the list of trait mods (`horizon-client/src/mod_api.rs:857-861`). Only `entry::main_with` fills it
  (`horizon-client/src/entry.rs:12`). The BMH game binary calls `entry::main()` (stage
  `crates/horizon-client/src/main.rs`), which passes `Default::default()`: an empty list.
- So `living_death` never fires for ANY player in a BMH build, owner or LAN guest. The mod is a mixin bundle with no
  trait mod (decision D2), so every gate `mods().is_empty()` is closed. There are 39 such gates in
  `horizon-client/src` (for example `native_player_tick_pre`, `lan_actions.rs:4156`, and
  `native_damage::note_player`, `game/native_damage.rs:21`). Other areas can hit the same wall.

## 2. How Horizon's own death protection works

- `living_hurt.rs:186-193` (horizon-world): at 0 health, `death_protection()` decides. True means no `die()`.
- `horizon-client/src/health.rs:969-979`: `death_protection` is true when `totem` is true and the damage does not
  bypass invulnerability. `totem` is `self.health.totem_ready` (`health.rs:493`, `:514`). On a save it sets
  `totem_used` (`health.rs:534-537`).
- `totem_ready` is written only by `Game::update_totem_ready` (`horizon-client/src/game/totem.rs:52-61`): true when
  the selected hotbar stack or the off hand stack has `minecraft:death_protection`, whatever the item. For a guest it
  runs at each turn entry (`lan_actions.rs:671`, `enter_guest`).
- `Game::apply_totem` (`game/totem.rs:66-117`), called in `leave_guest` (`lan_actions.rs:753`): calls the mod hook
  `native_player_totem` BEFORE the use-up (`totem.rs:71-73`), sets health 1, takes one from the main hand when it has
  the component, else from the off hand, counts the stat `used`, fires `minecraft:used_totem`, posts the game event
  `item_interact_finish`, and applies the `death_effects` of the used stack's own component. Entity event 35 (sound,
  particles, animation) follows (`lan_actions.rs:1141-1148`).
- The component is read from the stack's patch, else from the item's default (`game/totem.rs:243-264`). A content
  item can declare it: the loader accepts a component type that some vanilla item has by default
  (`horizon-client/src/mod_content.rs:822-838`), and `minecraft:totem_of_undying` has it.
- The vanilla Totem of Undying in the prepared `items.json` gives clear-all, Regeneration II 900, Absorption II 100,
  Fire Resistance I 800. These are exactly the Java Totem of Eternity effects
  (`TotemOfEternityHandler.java:44-47`).

## 3. Routes

| Route | What | Verdict |
|---|---|---|
| A | Give `bmh:totem_of_eternity` the `minecraft:death_protection` component in the content item; finish the swap to the Dormant Totem from the mod | CHOSEN for a totem in a hand. No injection. Not built yet |
| B | Make the hand check see a totem that is in another slot (swap it into the off hand around `update_totem_ready`) | REFUTED. The check runs at every turn entry, so the totem would have to stay in the hand, which the player sees |
| C | Patch the component onto whatever the player holds | REFUTED. The use-up takes one of that item (a sword vanishes), the patch stops stacking, and an empty hand cannot be patched |
| D | Revive the body after `die()` in `player_tick` (it runs before `guest_die`) | REFUTED. `die()` sets the body's `dead` flag (`health.rs:991-994`); `native_player_turn` then refuses the health and effect helpers (`game/native_player_state.rs:141-146`); only a new body clears the flag |
| E | Any slot, through the engine | Needs tier X (D8). See section 5. Proposal only |

## 4. Route A, ready to build

1. Content (`horizon/content/data/bmh/horizon/item/totem_of_eternity.json`): add the component, a copy of the
   Totem of Undying's, with the three Java effects:
   `"minecraft:death_protection": {"death_effects": [{"type": "minecraft:clear_all_effects"}, {"type":
   "minecraft:apply_effects", "effects": [{"id": "minecraft:regeneration", "amplifier": 1, "duration": 900,
   "show_icon": true}, {"id": "minecraft:absorption", "amplifier": 1, "duration": 100, "show_icon": true},
   {"id": "minecraft:fire_resistance", "duration": 800, "show_icon": true}]}]}`. The T5 edit in
   `bmh_totem_world.rs` touches only `minecraft:totem_of_undying`, so it keeps Fire Resistance here.
   `horizon/content/NOTES.md` line 28 chose `echo_shard` to avoid this component: update that note.
2. Logic (`logic/src/totem.rs`): a pure function for the hand that vanilla uses, main first, with unit tests:
   main = Totem of Undying gives "Undying"; main = Totem of Eternity and off = Totem of Undying gives "swap" (Java
   saves with the Undying first, `A_LivingEntityTotemMixin.java:46` runs only after vanilla's check); main = Totem of
   Eternity gives "Eternity, main"; off = Totem of Eternity gives "Eternity, off"; off = Undying gives "Undying";
   else none. Slot numbers: carried off hand -106; `native_set_inventory_slot` off hand 40 (`file_slot`,
   `game/native_inventories.rs:52-59`, `lan/menu.rs:130-141`); main = `selected_slot`.
3. New injection `hook_player_totem` (HEAD of `horizon_client::game::native_hooks::Game<'_>::native_player_totem`,
   bindings `game`, `player: u128`, `name: &str`), helper `hook_player_totem.rs` with the `native_on` check, calling
   `bmh_totem::on_player_totem`. In it (player swapped in, body at 0 health):
   - read the two hand items with `native_carried_stack` (it has no dead check, `native_hooks.rs:10399`) and
     `native_acting_player().selected_slot` (no dead check, `native_server.rs:661`);
   - "swap": exchange the two hand stacks now (both are non-empty, so `native_carried_stack` can write them). Then
     the use-up takes the Undying from the main hand with the Undying's effects. Record a "swap back";
   - "Eternity": record (uuid, name, inventory slot). Do not write the slot here: the use-up reads the hand after
     the hook, and a Dormant there would make it take the other hand.
4. `on_server_tick`, phase `ServerTickEnd` (no player swapped in, so `native_set_inventory_slot` works,
   `game/native_inventories.rs:140-143`): for each record, put `bmh:sleeping_totem` in the emptied slot with
   `expected = None`. If the slot is not empty any more, `native_give_stack`, else a warning. For a swap back: main
   slot `expected None` gets the Totem of Eternity, then slot 40 `expected Some(totem)` gets None. Then push the
   name to `awards` (advancement `eternity/saved`) and log `The Totem of Eternity saved <name> and is now dormant`.
   Clear the records on a new epoch and on logout.
5. Keep the old `living_death` path: it is dead code in this build, and it is the any-slot path the day the gate of
   section 1 opens.

What the player sees with route A, compared with Java: the same health, effects, sound, particles, stat and game
event, at once (no tick of delay). The Dormant Totem lands in the same slot at the end of the same server tick. The
animation shows a Totem of Undying, as in Java (the client's `findTotem` finds no totem left in the hands).
Differences: (1) a Totem of Eternity that is NOT in a hand does not save; (2) a mob that picks up a Totem of
Eternity is saved by it once (Java: never).

Test changes: t1 to t5 give the totem with `/give`, so it lands in the selected slot when the hotbar is empty, which
is a hand: check the scenario. Add a case with the totem in the off hand, a case with the totem in slot 20 (expected
red until section 5), and the swap case (Totem of Eternity in the main hand, Totem of Undying in the off hand:
expected Undying used, Fire Resistance absent, Totem of Eternity kept in the main hand).

## 5. Any slot: proposal for the owner (tier X, D8)

No route without an injection into the game's own functions. Three options, smallest first:

1. Upstream (best): ask Horizon to call `native_living_death` in `guest_die` whatever `mods()` holds
   (`lan_actions.rs:4293-4296`), and to clear the body's `dead` flag on a cancel (`lan_actions.rs:4316-4318`, totem.md
   difference 12). The existing `living_death` path of the mod then works for every slot.
2. Two injections, `RETURN` of `Game::update_totem_ready` (`game/totem.rs:52`: if not ready and the player carries a
   Totem of Eternity, set `totem_ready`) and `HEAD` of `Game::apply_totem` (`game/totem.rs:66`: when no hand has the
   component, spend the Totem of Eternity from its slot and apply its effects, and return the body). Ties the mod to
   commit `fdc87aec`.
3. A trait mod for BMH (an `[api]` crate passed to `entry::main_with`). It reverses D2 and opens all 39 gates at once,
   with unknown side effects (for example `hub.mods_loaded`, `lan.rs:2389`). Not recommended.

## Evidence still to make

None of section 4 is proven in the game. Next session: build route A, run `sh horizon/tools/test-slice.sh --only t`
under `game.lock`, keep the log lines and a screenshot of the Dormant Totem in the hand on disk, and append them here.
