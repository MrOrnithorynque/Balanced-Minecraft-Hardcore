# BMH content mod for Horizon 26.3

Mod id `bmh`, version 1.9.0 (`mod_version` in `gradle.properties`:41). The mod holds data and assets only. A native Rust mod adds the behaviour later.

## What is in the folder

- `horizon-mod.toml`: manifest. `api = "0.2"`, `side = "both"`, `capabilities = ["registries"]`.
- `data/bmh/horizon/item/*.json`: nine item declarations (new, see the table).
- `data/bmh/horizon/creative_tab/bmh_tab.json`: the creative tab (new).
- `assets/` and `data/` otherwise: copied with `rsync -a` from `common/src/main/resources/`. The copy covers recipes (8), tags (3 in `bmh`, 3 in `minecraft`), advancements (20), 10 lang files, 9 item model definitions, 43 models, 44 texture files, `sounds.json` and 11 ogg files. An earlier run copied the same files. This run copied them again and checked with `diff -rq`: the only difference is `assets/bmh/items/navigator_amulet.json` (below).

## Items

All stats come from `common/src/main/java/github/mrornithorynque/bmh/init/ItemInit.java` (IT), `init/BMHModTierInit.java` (TIER) and the vanilla 26.x template. "fire" is `minecraft:damage_resistant` with `#minecraft:is_fire`, which is what `fireResistant()` sets in 26.1.

| Id | Java source | Template | Components set (besides the top-level `max_stack_size` and `rarity`) |
|---|---|---|---|
| `bmh:eternal_sword` | IT:47-55 (stack 1 :53, fire :54, EPIC :55; damage 10.0 :50, speed -1.4 :51), TIER:20-21, `items/EternalSwordItem.java`:21 (`properties.sword(...)`) | `minecraft:netherite_sword` | `max_damage` 33000 (TIER:21), `enchantable` 80 (TIER:21), `repairable` `#bmh:eternal_tool_materials` (TIER:21), fire, `attribute_modifiers` attack_damage +10.0 and attack_speed -1.4 (mainhand). The tool rules and the weapon component stay the template's: a 26.1 sword has the same ones whatever the material. |
| `bmh:eternal_axe` | IT:27-35 (damage 17.0 :30, speed -3.2 :31), TIER:20-21, `items/EternalAxeItem.java`:17-21 | `minecraft:netherite_axe` | same as the sword with damage +17.0 and speed -3.2, and `tool`: deny drops on `#minecraft:incorrect_for_netherite_tool` (TIER:21), mine `#minecraft:mineable/axe` at speed 15.0 (TIER:21, mining speed), `damage_per_block` 1. Stripping, scraping and unwaxing come from the template's block transformer component (not checked, see open questions). |
| `bmh:eternal_pickaxe` | IT:37-45 (damage 7.0 :40, speed -2.0 :41), TIER:20-21, `items/EternalPickaxeItem.java`:19 | `minecraft:netherite_pickaxe` | as the axe with damage +7.0, speed -2.0 and `#minecraft:mineable/pickaxe`. |
| `bmh:eternal_bundle` | IT:19-25 (stack 1 :22, empty `BUNDLE_CONTENTS` :23, fire :24, EPIC :25), `items/EternalBundleItem.java`:44 (weight 64, as vanilla) | `minecraft:bundle` | fire. The empty `bundle_contents` is the template's default. |
| `bmh:navigator_amulet` | IT:57-61 (stack 1 :60, COMMON :61), `items/NavigatorAmuletItem.java`:36-50 (no durability, no enchantment) | `minecraft:compass` | None. The default `lodestone_tracker` was removed (the game refuses it, see the needle section). The compass has no durability and no enchantability, so the amulet has none. |
| `bmh:totem_of_eternity` | IT:63-68 (stack 1 :65, fire :66, EPIC :67), `items/TotemOfEternityItem.java`:26 (`isFoil` true) | `minecraft:echo_shard` | fire, `enchantment_glint_override` true. |
| `bmh:sleeping_totem` | IT:74-79 (stack 1 :77, fire :78, EPIC :79), `items/DormantTotemOfEternityItem.java`:69 (glint only during the ritual: code) | `minecraft:echo_shard` | fire. No glint. |
| `bmh:dormant_totem` (Hollow Totem) | IT:83-89 (stack 1 :86, fire :87, RARE :88) | `minecraft:echo_shard` | fire. |
| `bmh:echo_heart` | IT:91-97 (stack 1 :94, fire :95, RARE :96) | `minecraft:echo_shard` | fire. |

Why `echo_shard` for the plain items: it has no behaviour and no special component. A copy of `totem_of_undying` would carry `death_protection` and could save the player in Horizon's own way. Why `compass` for the amulet: it is the vanilla item that owns the needle component.

### The amulet needle (data only, tracker still missing)

The Java needle is the client item property `bmh:north` (`client/AmuletCompass.java`). Horizon reads a foreign property as 0, so `assets/bmh/items/navigator_amulet.json` uses the vanilla `minecraft:compass` range property with `target` `lodestone` (32 frames, same thresholds).

The first run in the game (2026-11-17) refused the item file's default `minecraft:lodestone_tracker`: "mod bmh content not loaded: bmh:navigator_amulet: minecraft:lodestone_tracker is not a known component type". The loader (`horizon-client/src/mod_content.rs`, `patch_components`) accepts a component type only when some vanilla item in `items.json` has it by default or when it is a Horizon custom component. No vanilla item has a default tracker. So the key is gone from `data/bmh/horizon/item/navigator_amulet.json`, and the game now loads the mod.

What the needle shows now: `item_model.rs` `compass()` reads the stack's `lodestone_tracker`. With none, the target is empty and the needle spins at random (the "no target" state, as a vanilla compass in the wrong dimension). So the amulet looks alive but does not point north.

What a native area must do: set the tracker at run time, with the same value as before (dimension `minecraft:overworld`, pos [0, 64, -30000000], `tracked` false). The hook `native/mixins/bmh/hook_modify_default_components.rs` (which calls `bmh_amulet_world`) is the place. Difference from Java: Java aims at the block 1.0E6 north of the holder, the data aims at a fixed block. The error is 0.04 degrees at x = 20000.

## Files changed after the copy

| File | Change | Why |
|---|---|---|
| `data/bmh/horizon/item/navigator_amulet.json` | The `components` block (`minecraft:lodestone_tracker`) was removed. | The game refuses the type in a content item (game log, first run). |
| `assets/bmh/items/navigator_amulet.json` | `"property": "bmh:north"` became `"property": "minecraft:compass"` plus `"target": "lodestone"`. The scale (32.0), the 32 thresholds and the models are unchanged. | Horizon cannot read `bmh:north`. |

Nothing else was edited. In particular the lang files keep `%%` in `tooltip.bmh.death_wear` (see open questions), and `data/minecraft/tags/item/{swords,axes,pickaxes}.json` are unchanged: Horizon's sweep reads the `minecraft:swords` tag (audit 3, feature 1).

## Java resources not carried over

- `bmh.mixins.json`: lists Java mixins, no meaning in Horizon.
- `pack.mcmeta`: not needed in a content mod.
- Java code behind the items: tooltips (ability line, worn-out line, heavy line, death wear line, the hint lines, the starlight charges), the item bar of the tools (one pixel left) and of the amulet (charges), the worn-out state (half damage, stone mining speed, no ability), tools that never break, the flat 15-level gold block repair, the Totem of Eternity save, the moon ritual (use time 300 ticks, bow pose, `MoonRitualHandler.java`:38), the Dormant Totem glint while it charges, one totem per container, the bundle's drop-all use and its refusal of Eternal Items and ender chests, the Eternal Items staying through death. All of this needs the native mod.
- `SoundInit.java` (the sound events `bmh:end` and `bmh:bell_sfx`): the event names exist through `sounds.json`, which is carried over.

## Open questions

1. SETTLED by the game (2026-11-17): `minecraft:lodestone_tracker` is refused in a content item. The amulet has no default tracker; the native hook must set it (see the needle section).
2. The netherite templates carry the block transformer component (axe stripping) and the `weapon` component. Not checked: the prepared `items.json` is not in the Horizon repository.
3. The exact 26.3 shape of `tool` (`default_mining_speed`, `damage_per_block`, `can_destroy_blocks_in_creative`) is written from the 26.1 codec. The checker does not read it.
4. `minecraft:compass` as a template (open: no click on a lodestone was tried): if Horizon ties "use on a lodestone" to the compass id, the amulet might not get it (that is wanted). If it ties it to the component, a click on a lodestone could retarget the needle. Test in game.
5. `crafting_remainder`, tags and recipes of `minecraft:bundle`: the Java recipe `bundle.json` still crafts the vanilla bundle. Not checked in game.
6. Lang `%%` in `tooltip.bmh.death_wear`: Java needs `%%` to print "%". Horizon's lang loader may print it twice.
7. `totem_of_eternity.png.mcmeta` (animation): not checked that Horizon animates it.
8. The tooltips are code in the Java mod. I did not put them in `minecraft:lore`, so that the native mod owns all of them. Say if you want the static lines as lore.
9. Unused sounds: the nine bundle ogg files are copies of vanilla sounds that Java never plays. They are carried over unchanged.

## Validation

The checker does not run the game. It does not prove that an item looks right, has the right stats or behaves right in a world. It proves the manifest, the file names, the JSON and the content checks the registry makes.

Commands, run in a subshell from the mod folder:

```
$ cargo-horizon-mod horizon-mod build
Built ./target/horizon-mod/bmh.hmod
exit 0
$ cargo-horizon-mod horizon-mod check target/horizon-mod/bmh.hmod
bmh 1.9.0 (mod, API 0.2, both)
  731255 bytes, SHA-256 2ff906c26c2df965537892eedb21be5cc8fd8576bc85165b1c633bf1f2592069, unsigned
  may add blocks, items, mob effects and entity types to your worlds
  no code: content only, nothing to run
exit 0
```

No error and no warning. The `.hmod` holds 162 files. The folder `target/` is ignored by `.gitignore`.

## Game check (2026-11-17)

The game loads the mod (log: `Horizon: content from 1 mod(s) in ...`, no `content not loaded`). All nine items can be given and show their own texture, the creative tab "BMH Tab" lists them, the recipes unlock, the advancements and the sounds exist. Evidence and commands: `docs/horizon/log/content.md`.
