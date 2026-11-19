# One Totem of Eternity per container: workarounds for T12, T13, T14

Branch `claude/gap-totem-containers-d99671`, cut from `horizon-26.3` at `5da909a`. Status: both gates pass (`test-logic.sh`: 357 tests; `check.sh`: `"check": "ok"`). Scenario `z` of `test-slice.sh` ran in
the game on 2026-11-19 (01:25 to 01:35): **z3, z4, z5, z6 pass; z1, z2, z7 fail.** See "Game results" first: the
click route of the table below does not work in the game, and the table is the design as written, not the verdict.

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

## Game results (2026-11-19, scenario z, evidence in the kept log of the run)

| Check | Result | What it shows |
|---|---|---|
| z3 | PASS | An empty chest takes the first totem by a shift click (control). |
| z4 | PASS | A hopper that holds a totem leaves a loose Totem of Eternity on the ground. |
| z5 | PASS | An empty hopper takes the first loose totem. Together with z4 this is exact Java parity for the hopper pickup: the old "refuse every totem" limit of T13 is gone. Route: the hook binds `level` and reads the hopper's block entity. |
| z6 | PASS | A dropper pushed its totem into a chest that held one. Within the scan period the chest was back to 1 totem and the other lay on the ground as an item. Log line `The scan took a second Totem of Eternity out of a container`. T13 droppers: working substitute, delay up to one second. |
| z1, z2 | FAIL as written | The totem did not stay in the chest (chest 1, player 1), but the log shows no refusal line from the new hook: that is the older after-the-fact scan of T12 (`scan_open_container`) giving it back within a tick. The new click hook was heard (`DEBUG click window 1 slot 56 mode 1` in the log) and found `game.lan` empty. |
| z7 | FAIL, not understood | The scenario did not find the minecart id (`entities 8` answer not matched), so the cart menu never opened. The T14 click route has the same limit as z1. Not a verdict on the vehicle read. |

### Cause of z1, z2 and the verdict for T12

`lan_inventory_click_hooks(&mut self, host: &mut LanHost, conn, p)` (lan_actions.rs:1865 on the pin) runs with the
host taken out of `game.lan` (the call site at lan_actions.rs:2101 passes `host` as a parameter). In the hook
`game.lan` is `None`, so the guest, his cursor, his inventory and his open menu cannot be read. Only the world
(`game.world.block_entities`, the container blocks and the vehicle placeholder keys) is readable.

- **T12 verdict: no exact route without an injection.** The click hook gives no cursor and no inventory. What is left
  without an injection is the existing after-the-fact scan (T12 in `bmh_totem.rs`, up to one tick). The player cannot
  see the totem sit in the chest, but the slot shows it for a moment.
- **Injection proposal (rule D8, not done):** target `horizon_client::game::lan_actions::Game<'_>::lan_inventory_click_hooks`,
  kind HEAD (or just after `native_inventory_click`), bindings `host: &mut LanHost`, `conn: u32`, `p: &mut play::ContainerClick`.
  The helper then runs `bmh_totem::on_inventory_click` logic with `host.hub.guest(conn)` (menu, `carried`,
  `inventory`, `offhand`) and `Slots::read(&self.world, guest)`; to cancel it sets `(p.slot, p.button, p.input) = (-999, 12, 5)`.
  The functions `logic::click_refused`, `drag_refused` and the reading code in `menu_totems` are written and unit tested for
  that day. A drag is the same with `native_inventory_drag`.
- **A mirror without injection** (not built): keep per player, from `player_tick`, which inventory slots held a totem,
  and from the clicks a "cursor holds a totem" flag. It covers shift click and number keys, but the cursor flag
  drifts on drags and server resyncs. Not worth the risk.
- **T14 verdict:** the open-menu case has the same limit as T12. The hopper-into-vehicle read (`vehicle_holds_totem_at`)
  and the scan of `game.carts` and `game.boats` are written; the scan is not yet checked in the game (z7 needs a fixed
  scenario: take the id from `ctl entities` with the `type` field).

## Next steps (updated)

1. Fix z7 (find the cart) and add a hopper-into-chest-minecart check and a scan check for a cart.
2. Decide on the injection above. Without it T12 stays "cleanup within one tick".
3. Update `GAPS.md` (T13 hopper: exact; T13 dropper: scan substitute; T12, T14: as above) and `HOOKS.md` after the owner
   reads this note.

## Integration check

1. The pin takes the LAN host out before it handles a container click. The packet hook receives the window and button only.
   The cursor helper reads the missing host for vanilla menus. The inventory click handlers therefore cannot read the server cursor.
2. The control command `entities` lists mobs, loose items and experience orbs. It omits vehicles (`game/agent.rs`, `entities_value`).
   A vehicle check must aim at its known position and use the normal interaction.
