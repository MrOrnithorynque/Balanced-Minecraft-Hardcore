# Port 26.1.2: items, tools, item models, data (agent "items")

Scope: `items/*`, `init/*`, `client/AmuletCompass`, the item mixins (B, G, I, L, the old H), `handlers/EternalWear`,
`TotemAwakeningHandler`, `TotemOfEternityHandler`, `assets/bmh`, `data/`, `pack.mcmeta`.
Started 2026-11-06 03:37 CET (`date`). Last update 2026-11-06 03:53 CET (`date`).
`SRC` = `$SCRATCH/src26` (decompiled 26.1.2), `$SCRATCH/items121/src` = classes of the 1.21.1 Mojang-named Loom jar
decompiled with Vineflower 1.10.1, `$SCRATCH/dl/neo` = NeoForge 26.1.2.126 userdev jar (its vanilla patches and ATs).

## Step 1: tools (BMHModTierInit, ItemInit, Eternal tool items)
- `Tier`, `TieredItem`, `SwordItem`, `DiggerItem`, `PickaxeItem` are gone. `BMHModTierInit.ETERNAL` is now a
  `ToolMaterial(INCORRECT_FOR_NETHERITE_TOOL, 33000, 15.0F, 0.0F, 80, bmh:eternal_tool_materials)` (`SRC/net/minecraft/world/item/ToolMaterial.java`).
  The class name stays: `handlers/KeepItemAfterDeath` imports it.
- Repair ingredient: `ToolMaterial` takes an item tag. New tag `data/bmh/tags/item/eternal_tool_materials.json` = `minecraft:gold_block`
  (1.21.1 used `Ingredient.of(Items.GOLD_BLOCK)`). The anvil checks `isValidRepairItem` against the REPAIRABLE component (`AnvilMenu.java:133`).
- Sword: plain `Item` with `properties.sword(material, 10, -1.4)`. Pickaxe: plain `Item` with `properties.pickaxe(material, 7, -2)`.
  Axe: still `AxeItem(material, 17, -3.2, properties)` (`AxeItem.java:55`), so stripping, scraping and unwaxing stay.
  Damage formula is the same as 1.21.1 (`base + attackDamageBonus`, `ToolMaterial.java` createToolAttributes vs
  `items121/.../DiggerItem.java:20`, `SwordItem.java:31`). Tool rules, durability per hit (sword 1, tools 2 via `Weapon`),
  per block (sword 2, tools 1) match 1.21.1. The axe disables shields 5 s (`Item.Properties.axe`, disableBlockingSeconds 5).
- Vanilla changes that reach our sword as they reach every sword: the sword tool also instantly mines `#sword_instantly_mines`.
- `ItemInit.register` now builds `new Item.Properties().setId(key)` before the item exists (the `Item` constructor
  throws "Item id not set" otherwise, `Item.java:134` and `Items.registerItem` at `Items.java:2379`). Public fields unchanged.
- Tooltips: `appendHoverText(stack, context, TooltipDisplay, Consumer<Component>, flag)`. `EternalTooltips` helpers take a
  `Consumer<Component>`. Vanilla still calls it first after the name (`ItemStack.java:872`), as 1.21.1 did.

## Step 2: other items
- `EternalBundleItem`: `InteractionResult`, `itemCopyStream()`, weight is `DataResult<Fraction>` (invalid weight counts as full,
  like vanilla `BundleItem.getWeightSafe`). Bar color `ARGB.color(102, 102, 255)` = 1.21.1 `Mth.color(0.4, 0.4, 1.0)`; the GUI
  forces it opaque (`GuiGraphicsExtractor.java:945`). Behaviour kept from 1.21.1: `use` drops everything at once
  (vanilla 26.1 bundles now drop one item at a time while used). `getFullnessDisplay` removed (the model uses vanilla's property).
- `NavigatorAmuletItem`: `use` returns `CONSUME` on the client (a `Success`, so `Minecraft.startUseItem` stops and the off hand
  is not used, `Minecraft.java:1758`), `SUCCESS_SERVER` on the server for a reading or the list (1.21.1 server `SUCCESS` swung
  from the server, `ServerGamePacketListenerImpl.java:1421`), `FAIL` otherwise. `useOn` returns `SUCCESS` on both sides
  (vanilla's mapping of `sidedSuccess`). Cooldowns take the stack (`ItemCooldowns.addCooldown(ItemStack, int)`).
  `inventoryTick` is server only now (`ServerLevel`, `EquipmentSlot`).
- `HintItem`, `TotemOfEternityItem`: tooltip signature only.

## Step 3: item models (`assets/bmh/items/*.json`, new) and the amulet needle
- Every item needs `assets/bmh/items/<id>.json` (`ClientItemInfoLoader.java:29`, `FileToIdConverter.json("items")`).
  Plain items: `minecraft:model`. `overrides` removed from `models/item/{eternal_sword,eternal_bundle,navigator_amulet}.json`.
- Eternal Sword: `minecraft:select` on `minecraft:display_context`; the four hand contexts give `eternal_sword_in_hand`,
  the fallback gives `eternal_sword`. 1.21.1 swapped exactly those four (`EternalSwordModel.isHand`, git `mc-1.21.1`); every
  1.21.1 draw went through `ItemRenderer.render` where the swap happened, so the choice depended on the context only.
  `on_shelf` (new) gets the 16x16. `client/EternalSwordModel.java` and `mixin/H_ItemRendererSwordMixin.java` deleted.
- Eternal Bundle: `range_dispatch` on vanilla `minecraft:bundle/fullness` (reads BUNDLE_CONTENTS of any item, no class
  check, `BundleFullness.java`), threshold 1e-7, as the 1.21.1 override on `filled`.
- Navigator's Amulet: `range_dispatch` on the new property `bmh:north`, scale 32, thresholds 0, 0.5, 1.5 ... 30.5, 31.5 to
  frames 00, 01 ... 31, 00: the 1.21.1 thresholds times 32 (1.21.1 picked the last override whose threshold <= angle;
  `RangeSelectItemModel.lastIndexLessOrEqual` picks the same). Exact in float (powers of two).
- `client/AmuletCompass` is now a `NeedleDirectionHelper` + `RangeSelectItemModelProperty` (codec field `wobble`, default true),
  the same maths as `CompassAngleState.calculate` (wobble 0.8, spin with hash 1327217883) with the target at the center of the
  block 1e6 north of the owner, Overworld only (1.21.1 `CompassItemPropertyFunction` with the same target).
- Registration: new client mixin `mixin/M_RangeSelectPropertiesMixin` (TAIL of `RangeSelectItemModelProperties.bootstrap()`,
  shadows the private static `ID_MAPPER`). Vanilla calls it from `ClientBootstrap.bootstrap()` (`client/main/Main.java:115`)
  before any resource load. NeoForge patches the same method only to post `RegisterRangeSelectItemModelPropertyEvent` at its
  end (`dl/neo/patches/.../RangeSelectItemModelProperties.java.patch`); TAIL runs after it, no conflict.
  `client/ClientItemProperties.java` deleted (nothing to register per item any more).

## Step 4: mixins
- `B_AnvilMenuMixin`: constructor now takes `ItemCombinerMenuSlotDefinition`. `createResult()V` HEAD/RETURN kept. On NeoForge
  `createResult` is a final wrapper around `createResultInternal` + AnvilUpdateEvent (NeoForge patch), HEAD/RETURN still frame it.
  `onTake`: 26.1 empties the left slot after the right slot (bytecode offset 193, the first `repairItemCountCost` read is at 20),
  so the 1.21.1 point (first field read) would clear the slot before vanilla reads its name. New point: INVOKE
  `ContainerLevelAccess.execute(BiConsumer)` (offset 208, the only call), right after vanilla empties it, before the anvil can break.
- `G_ItemEntityMixin`: `hurt` became `hurtServer(ServerLevel, DamageSource, float)` and `hurtClient(DamageSource)`, both final in
  ItemEntity (`ItemEntity.java:279,284`). Both cancelled for Eternal Items. `tick()V` unchanged. `/kill` (`Entity.kill` ->
  remove) and the void (`onBelowWorld` -> discard) do not go through hurt.
- `I_ContainerTotemMixin`: vehicles moved: `vehicle.minecart.AbstractMinecartContainer`, and `vehicle.boat.AbstractChestBoat`
  (now the parent of ChestBoat and ChestRaft; 1.21.1's ChestBoat covered the chest raft too). No class in the targets overrides
  `canPlaceItem` (`ShulkerBoxBlockEntity` only has `canPlaceItemThroughFace`).
- `I_SlotTotemMixin`, `L_ItemStackWearMixin`: targets unchanged and present (`Slot.java:44`, `ShulkerBoxSlot.java:12`,
  `ItemStack.java:445,491`).
- `bmh.mixins.json`: client list minus `H_ItemRendererSwordMixin`, plus `M_RangeSelectPropertiesMixin`.

## Step 5: data and pack
- Recipes: ingredients are now plain ids (`"minecraft:string"`), the `{"item": ...}` form is gone (`Ingredient.CODEC`,
  vanilla `data/minecraft/recipe/compass.json`). Results `{"id", "count"}` still valid (`ItemStackTemplate.MAP_CODEC`).
  Smithing: `template`, `base`, `addition` are ingredients. Every referenced id exists in 26.1.2 `Items`/`Blocks`.
- `data/bmh/loot_table/blocks/test_block.json` deleted: it named `bmh:test_block`, which never existed (the known
  "Couldn't parse element ... test_block" red of 1.21.1 boots).
- Tags: `tags/item/` and `tags/block/` folders unchanged; every block tag in `echo_sense_ores` exists.
- `pack.mcmeta`: `min_format` 84, `max_format` 101 (one file for both pack types; resource 84.0 and data 101.1 fall inside;
  `pack_format` alone above 64/81 is rejected by `PackFormat.IntermediaryFormat.validate`).

## Step 6: checks
- `TAG=items bash c26.sh`: 2 errors left in the whole tree, both `init/CreativeTabInit.java:21` "Output has protected access".
  `CreativeModeTab.Output` is `protected` in vanilla 26.1.2; NeoForge's AT makes it public (`ats/accesstransformer.cfg:207`).
  No lambda can implement `DisplayItemsGenerator` without naming it (probe `$SCRATCH/probe/p/T.java`). Needs a common AT.
- A scratch copy without that lambda compiles with 0 errors (`$SCRATCH/items-copy/javac.log`).

## Unverified
- Nothing ran in a game. The amulet needle, the sword sprite swap and the bundle sprite are checked against the codecs only.
- Forge 26.1.2 patches to AnvilMenu, ItemEntity, RangeSelectItemModelProperties: not read (only NeoForge's).
- NeoForge 26.1 transfer wrappers calling `Container.canPlaceItem` for hoppers of other mods: not read.
- Pre-existing, kept as is: `totem_of_eternity.png.mcmeta` uses the key `frame`, the codec reads `frames`, so the 20-tick hold on
  frame 0 never applied (1.21.1 read `frames` too). Fixing it would change the look.
