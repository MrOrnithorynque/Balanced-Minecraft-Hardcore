# Log: part B (b-totem), Totem of Eternity, recipes, tooltips

Session `b-totem`. Main side: branch `redesign/b-totem` (worktree `/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/b-totem-1ff280`),
cut from `origin/redesign/integration` at `432d0a2`. 1.21.1 side: branch `redesign/b-totem-1.21`, worktree
`/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/b-totem-1ff280-121`, cut from `origin/redesign/integration-1.21` at `a886d42`.
Steps are numbered, no clock times (I cannot see the clock).

## Resume here

Main side: written, built, boot-tested on the three loaders, committed (step 5). 1.21.1 side: code ported (step 6), build and boots pending (step 7).
Then: audit, merge `origin/redesign/integration` and `origin/redesign/integration-1.21`, rebuild, push, report (step 8).
The temporary self-test (see Verification) is NOT in the repo: its sources live in the session scratchpad. If you redo it, install the two files
(`BmhSelfTest.java` in `common/.../bmh/`, `B_SelfTestMixin.java` in `common/.../bmh/mixin/`) plus one `"B_SelfTestMixin"` entry in `bmh.mixins.json`,
never commit them, never `git add -A`.

## Steps

1. Checked the tree (`git rev-parse --show-toplevel` is my worktree), renamed the branch to `redesign/b-totem`, read `docs/redesign/BRIEF.md`
   (rules 1 to 13, part B) and `.claude/CLAUDE.md`.
2. Read the main-side sources (items, init, handlers, loaders) and the 1.20.4 and 1.21.1 vanilla sources for `AnvilMenu`,
   `SmithingTransformRecipe`, `ItemStack.transmuteCopy` (NeoForge-patched jars of rule 12). Checked every vanilla member I use with `javap` on the
   Loom jars of 1.20, 1.20.1, 1.20.2, 1.20.3, 1.20.4 and 1.21.1: `AnvilMenu.createResult()`, private `repairItemCountCost`,
   private final `cost` (DataSlot), protected `inputSlots`/`resultSlots`/`player` of `ItemCombinerMenu` and its public 4-argument constructor,
   `Item.appendHoverText` / `isFoil`, `Ingredient.of(ItemLike...)`: all identical on all five 1.20.x versions, so the main-side code compiles
   unchanged for G's ports. The anvil's "Too Expensive" test is `cost >= 40` on both sides (`AnvilScreen.renderLabels`, `AnvilMenu.createResult`).
3. Created the 1.21.1 worktree (rule 2) at `<main worktree>-121` on `redesign/b-totem-1.21`. The harness blocks the Write tool outside the session's own
   worktree, so I switched between the two worktrees with `EnterWorktree` (`path`), which the tool documents for a worktree created with `git worktree add`.
4. Main side written and built (`sh gradlew -p <worktree> build --max-workers=2`, JDK 17: BUILD SUCCESSFUL, forge/fabric/neoforge jars).
   - `ItemInit`: `DORMANT_TOTEM`, `ECHO_HEART` (both `HintItem`, stack 1, fire resistant, RARE); `CreativeTabInit`: both after the totem.
   - `B_AnvilMenuMixin` (common, `bmh.mixins.json` `mixins` list): HEAD of `AnvilMenu.createResult`, cancellable. `TotemAwakeningHandler.awaken(left, right)`
     returns a fresh Totem of Eternity for dormant totem (left) + echo heart (right); the mixin then sets cost 39, `repairItemCountCost` 1,
     the result slot, `broadcastChanges()` and cancels. Nothing else is touched, so every other anvil use goes to vanilla / the loader's patch.
   - `TotemOfEternityItem`: `isFoil` true, tooltip `Eternity` (gray) then the gold "Heavy with eternity" line. The sword, axe, pickaxe: gold line then
     gray `Each death: -7% durability`. The bundle: its fullness line, then the gold line. Shared lines in `EternalTooltips`.
   - `BMHModTierInit.ETERNAL.getRepairIngredient()` is `Ingredient.of(Items.GOLD_BLOCK)`.
   - Recipes (`data/bmh/recipes/`): deleted `eternal_sword.json`, `eternal_axe.json`, `eternal_pickaxe.json`, `totem_of_eternity.json`; added
     `dormant_totem.json` (the old totem pattern), `echo_heart.json`, `eternal_{sword,axe,pickaxe}_smithing.json`; replaced `eternal_bundle.json`.
     Kept `bundle.json` and `totem_of_eternity_blasting.json` untouched.
   - Models `dormant_totem.json`, `echo_heart.json`; placeholder textures `dormant_totem.png`, `echo_heart.png` = first 16x16 frame of
     `totem_of_eternity.png` (F replaces them under the same names).
   - `en_us.json`: one contiguous block of 7 keys after `itemGroup.bmh_creative_tab`. `CHANGELOG.md` `[Unreleased]`: Added and Changed.
   - Refmaps checked in the built jars: the Forge refmap has `B_AnvilMenuMixin.createResult -> AnvilMenu;m_6640_()V`, the Fabric one
     `class_1706;method_24928()V`; the `@Shadow` fields are renamed inside the mixin class by each loader's reobf (SRG `f_39000_`, `f_39002_`; intermediary
     `field_7776`, `field_7770`), NeoForge keeps official names: so the mixin applies outside the dev environment too (checked with `javap` on the
     built jars, not by running a production install).
5. Main side verified on three dedicated servers (see Verification), temporary self-test removed, committed.
6. 1.21.1 side ported by hand: same classes and keys; differences are only `appendHoverText(ItemStack, Item.TooltipContext, ...)`, recipes in
   `data/bmh/recipe/` with `"id"` in results (smithing result `{"id": ...}`, `ItemStack.STRICT_CODEC`), `compatibilityLevel` JAVA_21 left as is, the
   mixin listed before `EnchantmentMixin`. `B_AnvilMenuMixin` is byte-identical to the main side (same private members and constructor on 1.21.1, checked
   with `javap` on the 1.21.1 Loom jar).

## Calls made (the brief had no answer)

- **Common mixin for the anvil** (rule 13), not Forge's `AnvilUpdateEvent` plus a Fabric mixin: one class, same behaviour on three loaders. It skips
  Forge's event for this one combination only.
- **Hint tooltips on the Dormant Totem and the Echo Heart** (gray, one line each, via `HintItem`). Not in the brief; without them nothing in the game
  says the two items meet at an anvil (no recipe book entry for an anvil step). Remove by switching those two registrations to a plain `Item`.
- **The awakened totem keeps its display name "The Totem of Eternal Life"** (not in the brief, other languages say "Eternal Life" too). The Dormant Totem
  is "Dormant Totem of Eternity" as the brief says, so the pair reads slightly differently. Louis' call; one line in `en_us.json`.
- **No ability lines on the tools** ("Full-strength hits echo...", as the design page shows). The brief lists exactly two tooltip lines for B and
  forbids E to touch the tooltips, so nobody adds them; the page's lines describe an earlier version of the abilities. Flagged for Louis.
- On the awakened totem the gray `Eternity` line comes before the gold `Heavy with eternity` line (an enchantment line sits right under the name in
  vanilla); on the tools the gold line comes first, as on the design page.
- Smithing recipe files are named `eternal_*_smithing.json` (new ids) instead of reusing the deleted crafting ids.
- README not touched: its crafting-grid pictures (`resources/*_crafting_grid.png`) are obsolete after this change. A README refresh needs new images and
  belongs to the release, not to this part. Flagged.
- Pre-existing, not mine: `CreativeTabInit` uses the title key `itemGroup.BMH_tab` but the lang files define `itemGroup.bmh_creative_tab`, so the
  creative tab title shows the raw key. One-word fix for whoever owns it.

## Verification

- Main side, `sh gradlew -p <main worktree> build --max-workers=2` (JDK 17): green, 43 tasks, three jars.
- Temporary in-server self-test (sources only in the session scratchpad, never committed): a common mixin on `MinecraftServer.tickServer` runs a class once,
  60 ticks after start. A fake `ServerPlayer` opens real `AnvilMenu` and `SmithingMenu` instances and the `RecipeManager`, and logs `[BMH-SELFTEST] PASS/FAIL`.
  Checks: anvil Dormant Totem + Echo Heart gives the Totem of Eternity, cost 39, name field ignored, 38 levels cannot take it and 39 can, both inputs
  consumed and 39 levels paid, wrong order / wrong item give nothing, a vanilla repair still works through the mixin; smithing for sword, axe and
  pickaxe keeps Sharpness V + Mending, damage 123 of 10000 and a custom name, and refuses a diamond tool, a diamond as addition and the dormant totem as
  template; the dormant totem, echo heart and eternal bundle recipes assemble, the four old crafting recipes do not match and their ids are gone, the
  blasting and vanilla bundle recipes remain; gold block repairs an eternal sword by 25% at the anvil and the totem does not; the tooltip lines, their
  colours and their order on all seven items; `hasFoil` only on the awakened totem; the pickup rule; the seven lang keys; models and textures on the classpath.
  - NeoForge 1.20.4 (`:neoforge:runServer`): `Done`, `pass=73 fail=0`.
  - Forge 1.20.4 (`:forge:runServer`) and Fabric 1.20.4 (`:fabric:runServer`): `Done`; the first 15 checks passed (everything about the anvil, plus the sword's
    smithing); the run then crashed inside my harness, not the mod: `SmithingMenu.onTake` calls `awardUsedRecipes`, which needs the player's network
    connection, and a fake player has none. I replaced that check by `mayPickup` and did not rerun Forge and Fabric (the remaining checks are common code
    and data, and NeoForge ran them all). Forge logged `Mixing B_AnvilMenuMixin from bmh.mixins.json into net.minecraft.world.inventory.AnvilMenu`.
  - No `invalid dist` line on any of the three, no recipe parse error, no mixin error. Red that is not mine: Fabric logs `Couldn't parse element
    loot_tables:bmh:blocks/test_block` (the test loot table names a block that does not exist) and `No key layers in MapLike[{}]` (my flat-world
    `server.properties` without `generator-settings`); Forge and NeoForge log the dev-only `Reference map 'bmh.refmap.json' ... could not be read`.
- Em-dash / en-dash check on every line I added (global rule): none.
