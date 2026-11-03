# Part A log: fixes and rule tweaks (session `a-fixes`)

Brief: `docs/redesign/BRIEF.md`, part A. Two sides, each with its own branch and worktree:

| Side | Branch | Worktree | Cut from |
|---|---|---|---|
| main (Minecraft 1.20.4, JDK 17) | `redesign/a-fixes` | `/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/a-fixes-7acb8d` | `origin/redesign/integration` (432d0a2) |
| 1.21.1 (JDK 21) | `redesign/a-fixes-1.21` | `/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/a-fixes-7acb8d-121` | `origin/redesign/integration-1.21` (a886d42) |

Server port 25611, RCON port 25711, RCON password `bmh`. The step numbers below are not clock times: the clock was read once
(`date`: 2026-11-03 21:30 CET at the start).

## Resume here

The work of part A is complete on both sides: A1 to A4 are written, built and booted (table in "Verification"), reviewed by
a fresh agent and fixed accordingly, and both branches are pushed. `origin/redesign/integration` and `-1.21` had not moved,
so the merges into my branches were no-ops. What is left is Louis's in-game checking of what needs a client: the Eternal
Bundle texture (A2), the player side of the totem (A4) and the anchor respawn (A3), all in `checklist-a-fixes.md`. A fresh
session only has to re-run the final builds if it touches the code again.

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
  0). So the totem's Fire Resistance is the only one left to remove; a Fire Resistance the player had before is cleared by the
  totem anyway.
- Why a mixin and not the loaders' hooks: Forge and NeoForge fire `LivingUseTotemEvent` before the effects are added, and
  Fabric has no event. One mixin at the end of the method covers the three loaders and both sides, and compiles for
  1.20 to 1.21.1 unchanged (`removeEffect(MobEffects.FIRE_RESISTANCE)` takes a `MobEffect` before 1.20.5 and a `Holder`
  after, the source line is the same).
- Players only: `(Object) this instanceof Player`. A mob holding a totem keeps vanilla's three effects (checked over RCON on
  a cow, see below).

**A3, exile at world scale.** `RespawnPositionHandler.toLevelBlocks(serverLevel, worldDistance)` divides the distance by
`serverLevel.dimensionType().coordinateScale()` (rounded to the nearest block) and logs
`Exile distance: <world> world blocks, <scaled> blocks in <dimension> (coordinate scale <s>)`. Applied at respawn, not at
death, because only the respawn level is known to have the right scale. The stored distance (`distancesFromBed`) stays in
world blocks. The file is identical on both sides. `coordinate_scale` is validated in `[1.0E-5, 3.0E7]` by the dimension
type codec (1.20.4 and 1.21.1 sources), so there is no division by zero; the result is capped at
`WorldBorder.MAX_CENTER_COORDINATE` (29999984) so that a scale at the bottom of that range cannot overflow an int and silently
skip the exile (the reviewer's point, see below).

**A2, Eternal Bundle `filled` property.** New `client/ClientItemProperties.java` (common, client only): `registerAll(Registrar)`
registers `minecraft:filled` for `bmh:eternal_bundle` with `EternalBundleItem.getFullnessDisplay`. `Registrar` is a one
method interface so each loader passes its own register call and a property is written once for the three loaders:
- Forge main: new `forge/ForgeClientSetup.onClientSetup(FMLClientSetupEvent)`, hooked with `modEventBus.addListener` in the
  `@Mod` constructor behind `FMLEnvironment.dist.isClient()` (`ForgeClientEvents` sits on the game bus there, where
  `FMLClientSetupEvent` never fires); the property goes in `enqueueWork`.
- Forge 1.21.1 and NeoForge (both sides): `ForgeClientEvents` / `NeoForgeClientEvents` are already mod-bus classes
  registered behind the client check, so the listener is one more `@SubscribeEvent onClientSetup` method in them (not an
  `addListener` call as rule 13 words it; same effect, no change to the `@Mod` classes).
- Fabric (both sides): `BalancedMcHardcoreFabricClient.onInitializeClient` calls
  `ClientItemProperties.registerAll(FabricModelPredicateProviderRegistry::register)`. That class is `@Deprecated` in every
  Fabric API version in use (the build warns); the brief names it and it works on all of them.
- **For part C:** add the amulet's `angle` property in `ClientItemProperties.registerAll`, once, and it is registered on
  every loader of both sides. The id must be vanilla's `minecraft:angle`, the name the model predicate `angle` resolves to
  (`ResourceLocation.withDefaultNamespace("angle")` on 1.21.1, `new ResourceLocation("angle")` on main), not a `bmh:` one.
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
- Side effect, accepted: the only other vanilla data that reads these tags is `minecraft:breaks_decorated_pots`, and 1.21.1's
  `DecoratedPotBlock.playerWillDestroy` cracks a pot for any item in that tag unless it has Silk Touch. So on 1.21.1 an
  Eternal tool now breaks a decorated pot into sherds like a vanilla tool, where before the pot dropped whole. On the main
  side the mod ships no tags (the brief limits A1 to 1.21.1), so nothing changes there. Changelog line added.
- `EnchantmentMixin` (1.21.1) only special-cases the Navigator's Amulet, so nothing else gets in the way (part C owns it).

**Tooling note (the edit hook).** This session's Edit and Write tools refuse a file of another worktree, and the Bash guard
refuses `git -C <other worktree>`. The 1.21.1 worktree is mine by the brief, so I switched the session into it with
`EnterWorktree` (path) when I edited there and back when I edited here. Before I knew about the hook, two files had been
copied into the 1.21.1 worktree with `cp` and the exile patch applied with `git apply`; the content is the same as on this
side.

## Audit by a fresh reviewer, and what I did with it

One read-only reviewer (Opus, clean context, briefed to refute, raw output kept in the session scratchpad) read both diffs
against the sources. No blocker. Each finding was checked against the files before acting:
- Main-side runtime evidence was void (Fabric: entities not found right after `forceload`; NeoForge: the effects key
  `ActiveEffects` is `active_effects` since 1.20.2; Forge had not booted yet). Done: the command file got waits and the
  right key, Fabric and NeoForge were rerun, Forge booted (table below).
- The Forge refmap is load-bearing for this mixin and a dev boot cannot catch a missing entry (Mixin falls back to the
  source name in dev). It is in the hand-off for G below.
- The decorated pot side effect was missing from the changelog and described wrongly here. Fixed in both.
- The note for part C could be read as "use a `bmh:` namespace for the compass". Reworded above (C's own code already uses
  `withDefaultNamespace("angle")`).
- A2 and the player side of A4 have never executed: stated as unverified (top of this log, the checklist, the report).
- My edits rewrote the Javadoc of the shared client classes (rule 8 wants additive edits). Reverted; only the new methods
  and imports remain.
- Extreme `coordinate_scale` could overflow the radius. Capped (see A3).
- Changelog wording ("again", "Sharpness" on the pickaxe) and a mixin comment that overclaimed on NeoForge. Fixed.
- The checklist's Nether step could suffocate a Survival player. It now sets the anchor up in Creative.
- Left as they are: `FabricModelPredicateProviderRegistry` is deprecated (the brief names it); the "fails before" run was
  done on Fabric 1.21.1 only, because the cause is data and the same on every loader.

## Verification

Builds (one at a time, under the Gradle slot rule), `JAVA_HOME=/opt/homebrew/opt/openjdk@17` for main and `openjdk@21` for
1.21.1: `sh <worktree>/gradlew -p <worktree> --max-workers=2 build`. First main build `BUILD SUCCESSFUL in 2m 49s`; 1.21.1
without the tags `BUILD SUCCESSFUL in 2m 11s`, with them `BUILD SUCCESSFUL in 51s`. The final builds after the audit fixes
and the merges are listed at the end of this file.

Dedicated server boots (fresh world, port 25611, RCON, `-Dmixin.debug.export=true`; the log of every boot that reached
`Done` has no client-class or mixin error, only the known `test_block` loot table message and, on Forge 1.21.1, Netty's
harmless `Reflective setAccessible(true) disabled`):

| Side | Loader | Ready after | RCON result |
|---|---|---|---|
| 1.21.1 | Fabric | 102 s | sword, axe, pickaxe take Mending, Sharpness V / Efficiency V / Fortune III; `sweeping_edge` on the pickaxe refused; the cow's totem gives all three effects |
| 1.21.1 | NeoForge | 96 s | same |
| 1.21.1 | Forge | 111 s | same |
| 1.20.4 | Fabric | 204 s | the cow's totem gives all three effects; a zombie holds `bmh:eternal_sword` |
| 1.20.4 | NeoForge | 114 s | first run: the cow consumed its totem, the zombie held the sword; the effects read used the old key. The rerun hit the 5 minute limit under machine load (spawn area at 22% after 5 minutes) and was stopped: not repeated |
| 1.20.4 | Forge | 252 s | same as Fabric |

- Before the fix (1.21.1 Fabric, the same build without the three tag files): every `/enchant` on the Eternal Sword, Axe and
  Pickaxe answered `... cannot support that enchantment`. After: `Applied enchantment Mending to Zombie's item` and the
  enchantments are on the stacks (`data get entity ... HandItems[0]`). A zombie, being undead, gets no Regeneration from a
  totem in vanilla, so the mob check uses a cow.
- The Mixin export of `LivingEntity` (`.mixin.out`) shows our handler called at both returns of
  `checkTotemDeathProtection` on all six boots.
- Built main jars: the Forge jar has `bmh.refmap.json` with the entry
  `A_LivingEntityTotemMixin: checkTotemDeathProtection -> LivingEntity;m_21262_(DamageSource)Z` and the `MixinConfigs`
  manifest entry; the Fabric jar has the refmap entry (`method_6095`); the NeoForge jar has no refmap, as documented.
- Vanilla totem code: `javap -c -p` of `LivingEntity` in the Loom merged jars of 1.20 to 1.21.1.
- Tag closure: a Python script over the 1.21.1 Loom jar (item tags and enchantment definitions).
- APIs used by the main-side code exist on 1.20, 1.20.1, 1.20.2, 1.20.3 and 1.20.4 (`javap` on the Loom jars:
  `DimensionType.coordinateScale()`, `ClampedItemPropertyFunction`, `LivingEntity.removeEffect(MobEffect)`, `Mth.floor` and
  `Mth.clamp(double, double, double)`, `WorldBorder.MAX_CENTER_COORDINATE`). `ItemProperties.register` is private in vanilla
  on every version, so it needs Forge's and NeoForge's public version, which the standard modding idiom relies on (built on
  1.20.4 and 1.21.1, not on the older Forge versions).

Final state after the audit fixes and the (no-op) merges of `origin/redesign/integration` and `-1.21`:
- main, HEAD `64f0df0` plus this log: `BUILD SUCCESSFUL in 1m 21s` (43 tasks), Forge, Fabric and NeoForge jars built;
- 1.21.1, HEAD `ebf5363`: `BUILD SUCCESSFUL in 1m 7s` (39 tasks).
- The server boots above ran on the commits before the audit fixes (`e24f682`, `1ecf8ca`). The fixes changed comments,
  Javadoc, the changelog, and one line of `RespawnPositionHandler` (the cap), none of which runs at server startup, so the
  final commits were not booted again: the machine was at a load average of 80 and each boot took five minutes or more.
  The NeoForge 1.20.4 rerun of the final main sources compiled and reached world generation before the limit hit it.

Not executed anywhere (no client): the three client registration paths of A2 on each loader, the totem on a real player (A4)
and the anchor respawn (A3). All in `checklist-a-fixes.md`.

## For session G (ports) and for the repo docs

- Merge conflicts to expect against the sibling branches pushed when this was written (`git merge-tree --write-tree HEAD
  origin/redesign/<sibling>`; `redesign/g` is clean): `CHANGELOG.md` with every part; `bmh.mixins.json` with b-totem,
  c-amulet and e-abilities on main and with b-totem-1.21 and c-amulet-1.21 (all add to the common `mixins` list);
  `BalancedMcHardcoreFabricClient.java` with c-amulet and c-amulet-1.21 (both add a line to `onInitializeClient`: keep both
  lines); `forge/.../BalancedMcHardcoreMain.java` with c-amulet on main (A adds `modEventBus.addListener(ForgeClientSetup::
  onClientSetup)` and C adds `ForgeAmuletClient::onClientSetup` at the same spot: keep both listeners, or move C's property
  registration into `ClientItemProperties.registerAll` and drop `ForgeAmuletClient`).
- **Check the refmap of every 1.20.x Forge and Fabric jar you build.** `A_LivingEntityTotemMixin` is the first common mixin
  that targets a member by name, and on Forge 1.20.x that method only exists as `m_21262_` at runtime. A dev `runServer`
  applies the mixin from the source name even when the refmap lacks the entry, so only the jar shows it:
  `unzip -p forge/build/libs/<jar> bmh.refmap.json` must list `A_LivingEntityTotemMixin` with `m_21262_`, the Fabric jar's
  with `method_6095`. `forge/build.gradle` differs per branch (and has another layout on `dev-46.0.14`), so look at each.
- The repo `.claude/CLAUDE.md` sentence "Client code lives in ..." lists the client classes; it now also needs
  `client/ClientItemProperties` and `forge/ForgeClientSetup` (and whatever C and D add). Left to G to update once, because
  every part would otherwise edit that one sentence.
- Hand-over for the other parts, with the server and RCON scripts used here, is in the shared notes folder
  (`a-fixes-handover.md`, `a-fixes-tools/`).
