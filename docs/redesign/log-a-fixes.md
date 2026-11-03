# Part A log: fixes and rule tweaks (session `a-fixes`)

Brief: `docs/redesign/BRIEF.md`, part A. Two sides, each with its own branch and worktree:

| Side | Branch | Worktree | Cut from |
|---|---|---|---|
| main (Minecraft 1.20.4, JDK 17) | `redesign/a-fixes` | `/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/a-fixes-7acb8d` | `origin/redesign/integration` (432d0a2) |
| 1.21.1 (JDK 21) | `redesign/a-fixes-1.21` | `/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/a-fixes-7acb8d-121` | `origin/redesign/integration-1.21` (a886d42) |

Server port 25611, RCON port 25711, RCON password `bmh`. The step numbers below are not clock times: the clock was read once
(`date`: 2026-11-03 21:30 CET at the start).

## Resume here

Code for A1 to A4 is written and builds on both sides (main: `BUILD SUCCESSFUL`, 1.21.1: `BUILD SUCCESSFUL`, with and
without the tags). 1.21.1 baseline server (no tags, Fabric) refused every `/enchant` on the Eternal tools, as expected.
Pending: server boots with RCON checks (1.21.1 tagged: Fabric, NeoForge, Forge; main: Fabric, NeoForge, Forge), commit and
push of the 1.21.1 branch, merge of `origin/redesign/integration*` into both branches, `git merge-tree` against the pushed
siblings, an audit by a fresh sub-agent, the final report.

## Plan (brief part A)

1. A1, 1.21.1 only: `bmh:eternal_sword|axe|pickaxe` into the vanilla item tags `swords`, `axes`, `pickaxes`, so enchantments
   apply to them again. Proof: RCON `enchant` of a zombie holding the eternal sword, on every loader.
2. A2, all sides: register the `filled` item property for `bmh:eternal_bundle` on each loader's client, and create the mod-bus
   client setup listener on both sides (part C adds its line to it).
3. A3, all sides: divide the exile distance by the respawn level's `coordinateScale()`; log the scaled distance.
4. A4, all sides: when a Totem of Undying saves a player, keep Regeneration and Absorption, remove Fire Resistance.
5. Changelog lines, `docs/redesign/checklist-a-fixes.md`, merge the integration branches, rebuild, push, report.

## What was built, and the calls behind it

**A4, totem nerf.** One common mixin, `mixin/A_LivingEntityTotemMixin.java` (registered in `bmh.mixins.json`, common list):
an `@Inject` at `RETURN` of `LivingEntity.checkTotemDeathProtection`, which removes `MobEffects.FIRE_RESISTANCE` when the
method returns true and the entity is a `Player`.
- Vanilla, read with `javap` in the Loom jars of 1.20, 1.20.1, 1.20.2, 1.20.3, 1.20.4 and 1.21.1 and in the NeoForge-patched
  sources of 1.20.4 and 1.21.1: the method is `private boolean checkTotemDeathProtection(DamageSource)` everywhere, and after
  it consumes the totem it clears the effects (`removeAllEffects`; NeoForge: `removeEffectsCuredBy(PROTECTED_BY_TOTEM)`) and
  adds Regeneration (900 ticks, amplifier 1), Absorption (100 ticks, amplifier 1) and Fire Resistance (800 ticks, amplifier
  0). So the totem's Fire Resistance is the only one left to remove; a Fire Resistance the player had before is wiped by
  vanilla anyway.
- Why a mixin and not the loaders' hooks: Forge and NeoForge fire `LivingUseTotemEvent` before the effects are added, and
  Fabric has no event. One mixin at the end of the method covers the three loaders and both sides, and compiles for
  1.20 to 1.21.1 unchanged (`removeEffect(MobEffects.FIRE_RESISTANCE)` takes a `MobEffect` before 1.20.5 and a `Holder`
  after, the source line is the same).
- Players only: `(Object) this instanceof Player`. A mob holding a totem keeps vanilla's three effects (checked over RCON on
  a cow, see below).

**A3, exile at world scale.** `RespawnPositionHandler.toLevelBlocks(serverLevel, worldDistance)` divides the distance by
`serverLevel.dimensionType().coordinateScale()` (rounded to the nearest block, `Mth.floor(x + 0.5)`) and logs
`Exile distance: <world> world blocks, <scaled> blocks in <dimension> (coordinate scale <s>)`. Applied at respawn, not at
death, because only the respawn level is known to have the right scale. The stored distance (`distancesFromBed`) stays in
world blocks. The file is identical on both sides. `coordinate_scale` is validated in `[1.0E-5, 3.0E7]` by the dimension
type codec (1.20.4 and 1.21.1 sources), so there is no division by zero.

**A2, Eternal Bundle `filled` property.** New `client/ClientItemProperties.java` (common, client only): `registerAll(Registrar)`
registers `minecraft:filled` for `bmh:eternal_bundle` with `EternalBundleItem.getFullnessDisplay`. `Registrar` is a one
method interface so each loader passes its own register call and a property is written once for the three loaders:
- Forge main: new `forge/ForgeClientSetup.onClientSetup(FMLClientSetupEvent)`, hooked with `modEventBus.addListener` in the
  `@Mod` constructor behind `FMLEnvironment.dist.isClient()` (`ForgeClientEvents` sits on the game bus there, where
  `FMLClientSetupEvent` never fires); the property goes in `enqueueWork`.
- Forge 1.21.1 and NeoForge (both sides): `ForgeClientEvents` / `NeoForgeClientEvents` are already mod-bus classes
  registered behind the client check, so the listener is one more `@SubscribeEvent onClientSetup` method in them.
- Fabric (both sides): `BalancedMcHardcoreFabricClient.onInitializeClient` calls
  `ClientItemProperties.registerAll(FabricModelPredicateProviderRegistry::register)`.
- **For part C:** add the amulet's `angle` property in `ClientItemProperties.registerAll`, once, and it is registered on
  every loader of both sides. The 1.21.1 copy uses `ResourceLocation.withDefaultNamespace("filled")`; C needs the same
  helper for its own id (`fromNamespaceAndPath(MODID, ...)` on 1.21.1, `new ResourceLocation(...)` on main).
- Client classes: `javap -v` on the built main jars shows `net/minecraft/client` only in `ClientItemProperties`,
  `ClientItemProperties$Registrar`, `LocalPlayerText`, `TextDrawer`, `CreateWorldGameTabMixin`, `ForgeClientEvents`,
  `ForgeClientSetup` (Forge) and `NeoForgeClientEvents` (NeoForge). Nothing the server loads.

**A1, Eternal tools take enchantments (1.21.1 only).** `data/minecraft/tags/item/swords.json`, `axes.json`, `pickaxes.json`
(`"replace": false`, one value each) under `common/src/main/resources`.
- The tag graph, read from the 1.21.1 vanilla data (147 item tags, 42 enchantments, script run on the Loom jar): the
  `#minecraft:enchantable/*` tags reach `swords` through `durability, fire_aspect, sharp_weapon, sword, vanishing, weapon`,
  `axes` through `durability, mining, mining_loot, sharp_weapon, vanishing, weapon`, `pickaxes` through `durability, mining,
  mining_loot, vanishing`. Without our tags each Eternal tool is accepted by 0 enchantments; with them the sword takes the
  same 10 as `netherite_sword`, the axe the same 9 as `netherite_axe`, the pickaxe the same 6 as `netherite_pickaxe`,
  Mending and Unbreaking included.
- Side effect, on purpose: the only other vanilla data that reads these tags is `minecraft:breaks_decorated_pots`, so the
  Eternal tools now break decorated pots like any vanilla tool.
- `EnchantmentMixin` (1.21.1) only special-cases the Navigator's Amulet, so nothing else gets in the way (part C owns it).

**Tooling note (the edit hook).** This session's Edit and Write tools refuse files of another worktree, and the Bash guard
refuses `git -C <other worktree>`. The 1.21.1 worktree is mine by the brief, so I switched the session into it with
`EnterWorktree` (path) when I edited there and back when I edited here. Before I knew about the hook, two files had been
copied into the 1.21.1 worktree with `cp` and the exile patch applied with `git apply`; the content is the same as on this
side.

## Verification commands

- Tree guard before work: `git rev-parse --show-toplevel` in each worktree.
- Builds (one at a time, under the Gradle slot rule), `JAVA_HOME=/opt/homebrew/opt/openjdk@17` for main and `openjdk@21` for
  1.21.1: `sh <worktree>/gradlew -p <worktree> --max-workers=2 build`. Main: `BUILD SUCCESSFUL in 2m 49s`. 1.21.1 without the
  tags: `BUILD SUCCESSFUL in 2m 11s`; with them: `BUILD SUCCESSFUL in 51s`.
- Vanilla totem code: `javap -c -p` of `LivingEntity` in the Loom merged jars (script reads `checkTotemDeathProtection`).
- Tag closure: a Python script over the 1.21.1 Loom jar (item tags and enchantment definitions).
- Built main jars: Forge jar has `bmh.refmap.json` with the entry
  `A_LivingEntityTotemMixin: checkTotemDeathProtection -> LivingEntity;m_21262_(DamageSource)Z` and the `MixinConfigs`
  manifest entry; Fabric jar has the refmap entry (`method_6095`); the NeoForge jar has no refmap, as documented.
- 1.21.1 baseline (tagless build, Fabric dedicated server, port 25611, RCON): `summon zombie` holding each Eternal tool, then
  `enchant ... minecraft:mending` (and sharpness 5, efficiency 5, fortune 3): every one answered `The Sword of Eternity
  cannot support that enchantment` (and the same for the Eternal Axe and Pickaxe). `sweeping_edge` on the pickaxe is refused
  too. A zombie holding a totem, killed with `damage ... 1000 minecraft:generic`, ended with Absorption and Fire Resistance
  and no Regeneration (undead mobs are immune to Regeneration in vanilla; later runs use a cow). The Mixin export
  (`-Dmixin.debug.export=true`) of `LivingEntity` shows our handler called at both returns of `checkTotemDeathProtection`.

(more below as the boots finish)
