# Solution: the amulet's cooldown sweep (GAPS.md A4)

Verdict: solved for the player's eye. The Navigator's Amulet shows the white cooldown sweep on its icon after a
reading (40 ticks) and after a screen or list opens (20 ticks), on the hotbar and in every menu slot. The server
keeps the timer as before. The sweep is drawn by the mod with `item_decorations`, not by the game's cooldown table.
Proven in the real game with screenshots on disk (below). Status of GAPS.md A4: G changes to N.

## The gap

Java calls `ServerPlayer.getCooldowns().addCooldown(stack, ticks)` (`NavigatorAmuletItem.java:81,106,134`). The
vanilla client then shades the icon white from the top down until the time ends. Horizon gives a native mod no
setter for that table (GAPS.md section 3.1).

## Routes tried

### Route 1: the content component `minecraft:use_cooldown` (the idea of the ritual area)

Verdict: does not work for the amulet. Evidence is from the Nightly source, not from a game run.

- Horizon starts a `use_cooldown` only in `Game::apply_use_cooldown` (Nightly `crates/horizon-client/src/game.rs:6930`).
  Its callers are the game's own item paths: a thrown item, a finished consumable, a mod use that runs over time
  (`complete_using_item`, `item_use.rs:128`), a mod release with `used` set (`item_use.rs:196`), a goat horn.
- A click the mod claims in `native_use_item` or `native_use_item_on` ends `use_hand` at once with `return Ok(true)`
  (`game.rs`, the lines after `native_use_item`, "if claimed { self.swing_arm(); return Ok(true); }"). No call to
  `apply_use_cooldown` comes first. The amulet must claim, because its reading is the mod's own work.
- The cooldown time is fixed by the component (seconds in the content file or the stack's patch). The amulet needs
  two lengths (20 and 40 ticks) and only after a successful action. A refused click (rain, day, no charges) must
  not start a cooldown. The hook `player_item_cooldown` (API 262) can only cancel a cooldown, not start or resize one.
- Not tested: a variant where the mod does not claim and answers `native_use_duration` with 1 tick, so that
  `start_mod_use` and `complete_using_item` start the component. It would need a per-use component patch and it
  changes the hand pose and the use state for one tick on every reading. Not worth the risk when route 3 works.
- The `sleeping_totem` already has this component. Its sweep was never proved in the game (ritual log, check 10).
  The same source reading says it applies at the end of a ritual that ran to its end or when `used` is set.

### Route 2: a cooldown helper, packet or hook in the Nightly tree

Verdict: none exists. Searched `fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca` for `ItemCooldowns`, `cooldown`,
`CooldownSet`, `native_player_item_cooldown`, `add_cooldown`. `add_cooldown` and the `Cooldowns` struct
(`crates/horizon-items/src/cooldowns.rs`) are private to the game (`pub(super)`). The only mod hook is
`native_player_item_cooldown` (cancel only). `native-hooks.json` and `docs/native-mods.md` list nothing else. This
matches GAPS.md 3.1 and the upstream ask in section 6 stays.

### Route 3: the mod draws the sweep (chosen)

`native_item_decorations(stack, x, y, batch)` (API 89, `docs/native-mods.md`) runs for every icon the game draws
(hotbar, menu slots, tooltips) after the game's own bar and cooldown shade. The mod fills the same rectangle as
vanilla `GuiGraphicsExtractor.itemCooldown`: white, `0x7FFFFFFF`, from `y + floor(16 * (1 - left))` and
`ceil(16 * left)` rows tall, where `left` runs from 1 to 0 over the cooldown.

How the timer gets to the screen:

1. The server still owns the timer and the refusal (`bmh_amulet.rs`, `start_cooldown`). Behaviour of a click on a
   cooling amulet is unchanged.
2. `start_cooldown` also sends the new message `AmuletSignal::Cooldown { ticks }` (tag 3 on `bmh:amulet`,
   `logic/src/wire.rs`) to the player.
3. Each window starts the cooldown on its own clock. It clears the timer on expiry, disconnect or a change of world.
4. `on_item_decorations` draws the shade on every amulet icon while the cooldown runs.
   The hook is wired in `horizon-mixin.toml` (`hook_item_decorations`) and `hook_item_decorations.rs`, HOOKS.md row added.

Same as Java: all amulets of the player share one cooldown, the shade shows in the hotbar and in the inventory,
the length is 20 or 40 ticks, a refused click starts nothing.

Differences from vanilla (small):

- The sweep steps once per game tick. Vanilla also uses the part of the tick that has passed (the frame's partial
  tick); `native_clock` gives whole ticks only. At 20 ticks a second the step is 0.4 pixel per tick at 16 px, so it is
  hard to see.
- The game's own cooldown table stays empty, so other code that reads it (the vanilla "item on cooldown" check, the
  cooldown sound of other mods) does not see the amulet. The mod's server timer does the check.
- A respawn in the same world can keep the shade until the timer expires. Changing worlds clears it.

## Evidence

The original standalone check is now `sh horizon/tools/test-slice.sh --only cooldown` (run 2026-11-19, 1 passed,
0 failed). It gives an amulet at night under open sky, uses it, and takes three screenshots: 4 ticks after the use,
24 ticks, and 80 ticks (over). A script measures the pixels that are brighter than the last frame: the width gives the
GUI scale (96 px = 16 px at scale 6) and the height gives what is left of the icon: 0.88 at 4 ticks, 0.38 at 24
ticks, none at 80. Expected from a 40 tick cooldown: 0.90 and 0.40.

I opened the files and looked at them:

- `docs/horizon/solutions/amulet-cooldown-shots/z1-sweep-4-ticks.png`: the amulet in the first slot is washed out.
- `docs/horizon/solutions/amulet-cooldown-shots/z2-sweep-24-ticks.png`
- `docs/horizon/solutions/amulet-cooldown-shots/z3-sweep-over.png`: the icon in full colour.
- `docs/horizon/solutions/amulet-cooldown-shots/hotbar-slot-4-24-over.png`: the three slots side by side, zoomed.
  Left: nearly all of the icon is shaded. Middle: the lower part only, the upper part is in colour. Right: no shade.

Gates after the change: `test-logic.sh` 355 passed, `check.sh` `"check": "ok"`.

## Not proven

- A LAN guest. The message goes to any player and the window uses its own clock, but no guest run was made (scenario
  `x` runs a guest; it does not check the sweep).
- The 20 tick sweep after the naming screen opens (same code path, `start_cooldown`; only the reading is in scenario `cooldown`).
- Smoothness of the step by eye in a live window. The screenshots are single frames.
