# One Totem of Eternity per container: workarounds for T12, T13, T14

Branch `claude/gap-totem-containers-d99671`, cut from `horizon-26.3` at `5da909a`. Status: **compiled and unit tested,
not yet run in the game.** The session stopped at 61 % of the 5-hour window (rule: stop at 60 %), and another session held
the game lock (`game.lock`, owner `gap-rules-createworld`) when it wanted to run. Both gates pass
(`test-logic.sh`: 357 tests, 34 in `totem`; `check.sh`: `"check": "ok"`). Nothing below is called "works" until
scenario `z` of `test-slice.sh` has run.

## What was found in the Nightly tree (fdc87ae)

1. `native_inventory_click` and `native_inventory_drag` (pending fragments `inventory_click`, `inventory_drag`, tier F)
   have a `Game` and are heard before the click lands. Cancel makes the click a no-op and the client resyncs. They give
   only the window, slot, button and mode.
2. The missing data is in the game's own fields, which a module of the mod can read because the module is a child
   of `game` (the same way `native_container_slots` is used). No injection into a game function is needed:
   - the guest and his open menu: `game.lan.hub.guests[..].menu` (`Menu.slots`, `Menu.window`), the cursor
     `guest.carried`, the inventory `guest.inventory`, the off hand `guest.offhand`;
   - the container contents in window order: `Slots::read(&game.world, guest)`. It covers one block, a double chest,
     the player's ender chest, and the items of a chest minecart or chest boat (the game keeps them under the key
     `(i32::MAX - 15, VEHICLE_KEY_Y, id)` while a menu shows them).
3. `GameLevel` (the receiver of `hopper_pickup_item`) holds `world`, so the hook can bind `self` and read the
   hopper's block entity. The "no helper reads the hopper" limit of GAPS T13 was a limit of the old binding.
4. `game.carts` and `game.boats` are readable and writable (`Minecart.items`, `Boat.items`, `position`, `id`).
5. `game.world.block_entities` iterates (it derefs to a `HashMap`), so a periodic scan needs no list of droppers.

## Routes

| Gap | Route | Code | Verdict |
|---|---|---|---|
| T12 vanilla menus (chest, double chest, barrel, shulker, hopper, dropper, ender chest) | Cancel the click before it lands: `logic::click_refused` for pickup, quick move and number key swap, `drag_refused` for drags. Same cases as `Slot.mayPlace` in `I_SlotTotemMixin`. | `bmh_totem::on_inventory_click`, `on_inventory_drag`, `hook_inventory_click.rs`, `hook_inventory_drag.rs`, two injections in the manifest | Compiled, unit tested. In-game check `z1`, `z2`, `z3` written, not run. Expected verdict: the second totem never lands, the refusal looks like Java's. |
| T14 chest minecart, chest boat, menu open | The same click rule: `Slots::read` reads the vehicle's live items. | same code | Compiled. Check `z7` written, not run. |
| T14 hopper into a vehicle | `inventory_move_item` reports the block the vehicle is in. `vehicle_holds_totem_at` reads `game.carts` and `game.boats`. | `on_inventory_move_item` | Compiled. Not checked in the game (needs a hopper over a chest minecart). |
| T14 and T13 other paths (dropper, hopper minecart picking up an item, commands) | A scan once a second (`scan_all`, 20 server ticks). A container or vehicle that grew past one totem since the last scan gives the new one back as an item entity. A container new to the scan (chunk just loaded) is only counted, so an old pair stays, as in Java. | `scan_all`, `scan_blocks`, `scan_vehicles`, `logic::scan_ejects` | Compiled. Check `z6` (dropper into a chest) written, not run. The totem can sit in the container for up to one second. A hopper minecart that stands over a loose totem may pick it up again after the scan gives it back (not checked). |
| T13 hopper pickup from the ground | The hook binds `level` and reads the hopper's slots: refuse only when the hopper holds a totem. Exact Java behaviour (the first totem is accepted by an empty hopper). | `on_hopper_pickup_item`, `hook_hopper_pickup_item.rs` binding `level` | Compiled. Checks `z4`, `z5` written, not run. |
| T13 dropper, exact | Still needs an injection. Proposal, not done: target `horizon_client::game::machines::Game<'_>::dispense_from` (machines.rs:1331 on the pin), at the `Some(into) =>` arm of the dropper branch, before `self.add_item(None, &into, one, Some(facing.opposite()))`: refuse when `into` holds a totem and the stack is one. Until then the scan above is the substitute. | none | Proposal only (rule D8). |

## Risks to check when the game runs

- `Slots::read` for a vehicle: the placeholder key must exist while the menu is open (z7 shows it).
- The window slot mapping assumes the container first, then 27 main slots, then the 9 of the hotbar.
- `native_dimension()` is the level in place: the scan sees only that level's block entities and vehicles.
- A double chest: the other half uses its neighbour's count as "previous" (a neighbour that is another chest could
  give a wrong count; the effect is one spurious drop at most).
- Reading `game.lan`, `game.world`, `game.carts` ties the mod to the pinned commit, as the use of
  `native_container_slots` already does.

## Next steps

1. Take `game.lock`, run `sh horizon/tools/check.sh build`, then `sh horizon/tools/test-slice.sh --only z`.
   The slot ids of `click-slot` are 0 to 35 inventory and 300 and up container. The scenario uses `305` and the
   `ctl container` and `ctl entities` JSON shapes without having seen them: fix `z_chest_totems` first if it reads 0.
2. Keep the scan from the open-container scan in `scan_open_container` only if `z1` to `z3` show a hole.
3. Update `GAPS.md` T12, T13, T14 and `HOOKS.md` (new hooks `inventory_click`, `inventory_drag`, and the `level`
   binding of `hopper_pickup_item`) after the checks pass. Not done here to keep the claim honest.
