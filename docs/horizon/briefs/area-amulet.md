# Area brief: amulet (wave 1 of the run of 2026-11-18)

Read `docs/horizon/briefs/port.md` first and follow it exactly. This file is the part of your prompt that is about
your area. Your prompt gives BASE (the sha your worktree must be on).

No game in this step. Do not run `check.sh build`, `test-slice.sh`, `play.sh` or the game: another agent has the
game rights. Your proof is the two gates of port.md. What only a game can show goes in "To check in the game".

## Your area

The Navigator's Amulet: the item itself, the star reading at night, the clicks on beds and lodestones, the named
points of a player, the two screens, the `/bmh point` command. The owner played the game on 2026-11-18 and
reported "the amulet does nothing": today the item is content only and its needle spins.

## Your files

- `<R>/horizon/native/mixins/bmh/bmh_amulet.rs` (the module, where the hooks have a `Game`).
- `<R>/horizon/native/mixins/bmh/bmh_amulet_world.rs` (the module of `modify_default_components`, a hook with no
  `Game`).
- `<R>/horizon/native/mixins/bmh/logic/src/amulet.rs` (the logic, with its unit tests).
- `<R>/docs/horizon/log/amulet.md` (your log, from your first step).
- Content files that HOOKS.md ("Content files") gives to the amulet: the three advancements under
  `data/bmh/advancement/navigation/`.

## Your rows of GAPS.md

A1 (the item: needle to the north, charge bar, tooltip), A2 (the star reading at night; tier X: build the tier N
substitute that the row names, the spawn read from `level.dat` plus `spawn_change`, and no injection), A3 (the
clicks), A4 (the cooldown; tier G: the mod keeps its own timer for each player, and the white sweep on the icon is a
listed difference), A5 (the named points), A6 (the naming screen and the point list screen). Read also GAPS.md
section 3 for the substitutes and section 7 for the open questions that touch these rows.

## Audit sections

`<R>/docs/horizon/audit/4-amulet-client.md` (features 4, 5a, 5b, 5c) and `<R>/docs/horizon/audit/verify-4.md` (the
corrected verdicts: they win). For the points on the player and the command: `verify-1.md`, row 6.
`<R>/docs/horizon/audit/6-single-player-owner.md` for what fires for the single-player owner.

## Java sources

Under `<R>/common/src/main/java/github/mrornithorynque/bmh/`: `items/NavigatorAmuletItem.java`,
`handlers/AmuletReading.java`, `handlers/AmuletPoints.java`, `handlers/AmuletPointCommand.java`,
`handlers/AmuletPointSignal.java`, `handlers/BmhAdvancements.java`, `client/AmuletCompass.java`,
`client/amulet/PointScreen.java`, `client/amulet/PointListScreen.java`, `client/amulet/PointSignal.java`,
`mixin/M_RangeSelectPropertiesMixin.java`, `mixin/J_CommandsMixin.java`, and the event wiring in
`<R>/neoforge/src/main/java/github/mrornithorynque/bmh/neoforge/`.

## Contracts (HOOKS.md, "Contracts": keep every signature)

- You GIVE `bmh_amulet::points(game, player) -> Vec<bmh_logic::contract::AmuletPoint>`. Homecoming reads it in the
  next wave.
- You GIVE the reading of a player to the sky area (next wave). No shape is fixed yet. Propose it: add the type to
  `<R>/horizon/native/mixins/bmh/logic/src/contract.rs` and a row to HOOKS.md "Contracts", as a change outside your
  files, and say in your log what the sky area needs from it (read `client/sky/` in the Java source to see what the
  sky draws from a reading).
- You READ `bmh_settings::rule(game, Rule::...)` and `bmh_settings::config(game)` (the cap of points is
  `max_amulet_points`). Server side only.
- You OWN the `/bmh point` subcommands: HOOKS.md, "The /bmh command root", says how. Do not touch `bmh rule`.
- Advancements: award with `crate::game::bmh::award` (in `bmh.rs`), as the respawn area does.
- A message between your server side and your screens goes in `logic/src/wire.rs`, in a section named amulet, on
  the channel `bmh:amulet`, as a change outside your files. The server checks all that a client sends (D7).

## Points to settle yourself, by reading HN, and to record in your log

- The needle. The content item has no `minecraft:lodestone_tracker` now: the game's content loader refused the
  component (`<R>/docs/horizon/log/content.md`). Set the default tracker from `modify_default_components` in
  `bmh_amulet_world.rs` (overworld, pos [0, 64, -30000000], tracked false), as the content notes say, and check in
  HN that the compass model of the pinned Nightly reads that component for an item that is not
  `minecraft:compass`.
- Where the points live. They must survive death and a restart (GAPS A5: attachments on the player, copied on
  death). Read how MODS/native/smp stores data on a player.
- The screens. Read how MODS/native/recipes and minimap open a mod screen, draw it and take keys and clicks. The
  text field is hand written (GAPS A6).
- The star reading. Which hook gives you the right click in the air with the amulet for the single-player owner,
  and what the sky rules of the Java code need (night, weather, sky above the player): find each helper in HN.
- The charges and the nightly refill: where the count lives (the stack or the player) so that it is the same after
  a restart.

## Result

The structured result that port.md asks for. Add: the list of checks that the game agent of this wave must write
for your area (each one: what to do with `horizon-ctl` or a command, and what must be true), most important first.
