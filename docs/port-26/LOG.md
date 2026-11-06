# Port of BMH from 1.21.1 to Minecraft 26.1 (built against 26.1.2)

Running log of the port. A fresh agent must be able to resume from this file alone.
Worktree: `/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/respawn-distance-art-848c97`.
Base: `mc-1.21.1` at 7a77d48. Target branch name: `mc-26.1` (renamed from `claude/port-26-1-adef77` at the first commit).
Scratch evidence (not in git, may be wiped): `$SCRATCH/facts/` where
`SCRATCH=/private/tmp/claude-501/-Users-louisobenicherousse-Projects-Balanced-Minecraft-Hardcore--claude-worktrees-respawn-distance-art-848c97/b8692706-cd97-4dbf-ba41-71963975f78d/scratchpad`.
Rules: no push, no tag, no release, no other branch. Steps are numbered; times come from `date` only.

## Step 1: measure (started 2026-11-06 03:26 CET by `date`)

### 1.1 Facts and their sources

| Fact | Value | Source |
|---|---|---|
| Java for 26.1, 26.1.1, 26.1.2, 26.2, 26.3 | 25 (`java-runtime-epsilon`) | Mojang version JSONs, `javaVersion.majorVersion` (`facts/ver-26.1.2.json`) |
| Java for 1.21.11 | 21 | same, `facts/ver-1.21.11.json` |
| Mappings on 26.x | none: the version JSON has only `client` and `server` downloads, no `client_mappings`. The client jar holds Mojang names (`net/minecraft/server/level/ServerPlayer.class`). The game ships unobfuscated. | `facts/ver-26.1.2.json`, `unzip -l facts/mc26/client.jar` |
| Mappings on 1.21.11 | obfuscated, `client_mappings` and `server_mappings` exist | `facts/ver-1.21.11.json` |
| Forge for 26.1.2 | `26.1.2-64.0.0` to `26.1.2-64.1.6` (newest 64.1.6) | `maven.minecraftforge.net/.../forge/maven-metadata.xml` (`facts/forge-meta.xml`) |
| NeoForge for 26.1.2 | `26.1.2.126` newest, no `-beta` suffix (70 earlier 26.1.2 builds are `-beta`) | `facts/neoforge-meta.xml` |
| NeoForm for 26.1.2 | `26.1.2-1` | `maven.neoforged.net/.../neoform/maven-metadata.xml` |
| Fabric Loader | `0.20.1` (stable, listed for 26.1.2) | `meta.fabricmc.net/v2/versions/loader/26.1.2` |
| Fabric API for 26.1.2 | `0.155.3+26.1.2` newest | `facts/fapi.xml` |
| Fabric Loom | plugin id `net.fabricmc.fabric-loom` (not `fabric-loom`), 1.15.5 in MultiLoader-Template 26.1.2 | `facts/mlt/build.gradle` (jaredlll08/MultiLoader-Template, branch 26.1.2) |
| ModDevGradle | 2.0.141 in MultiLoader-Template 26.1.2; 2.0.148 newest (BMH uses it already). Common compiles with `neoForm` mode (vanilla only). | `facts/mlt/common/build.gradle` |
| ForgeGradle | 7, `[7.0.17,8)` in the Forge 26.1.2 MDK; 7.0.40 newest. New DSL: `minecraft.dependency(...)`, `minecraft.mavenizer(it)`, `fg.forgeMaven`. | `facts/forgemdk/build.gradle` |
| VanillaGradle | 0.3.2 newest. No evidence of 26.x support found. MultiLoader-Template replaced it with ModDevGradle `neoFormVersion`. | plugin portal metadata, `facts/mlt/common/build.gradle` |
| Gradle | 9.2.0 (MultiLoader-Template 26.1.2), 9.5.0 (Forge 26.1.2 repo). BMH uses 8.8. | the two wrapper files |
| Mixin on Forge 26.1.2 | Forge bundles Mixin 0.8.7 and MixinExtras 0.5.4. The `MixinConfigs` manifest attribute is still read (JarInJarDependencyLocator on the default branch). | Forge `settings.gradle` at tag 26.1.2, `gh search code` |
| MultiLoader-Template has no Forge module from 26.1 on | no reference build mixes ForgeGradle 7, Loom and ModDevGradle | tree of branch 26.1.2 |

Not confirmed (I do not know yet):
- Whether ForgeGradle 7, Loom 1.15 and ModDevGradle 2.0 load together in one Gradle build (BMH declares all three in the root `build.gradle`). No reference build does it.
- How ForgeGradle 7 dev runs pick up the mixin configs (MixinGradle 0.7 belongs to ForgeGradle 6).
- Whether any loader still needs `bmh.refmap.json`. Expected: no (unobfuscated game, official names at runtime everywhere), to be checked on the built jars and in boots.
- 1.21.11: not measured. No Loom jar of 1.21.11 is in `~/.gradle`, and its jar is obfuscated, so the direct download used for 26.1.2 does not work for it.

### 1.2 Compile check

`compile_check.sh 26.1.2` and `compile_check.sh 1.21.11` both stop with "no Loom merged jar": nothing on this machine built those versions with Loom. Because 26.x is unobfuscated, the Mojang client jar itself is a valid compile target. I downloaded `client.jar` and the 55 non-native libraries of the 26.1.2 version JSON into `facts/mc26/` and ran
`javac -proc:none --release 25` (JDK `/opt/homebrew/opt/openjdk@25`) on the 79 files of `common/src/main/java`, with Mixin 0.8.7 from the Gradle cache.
Result (`facts/mc26/javac.log`): **350 errors in 47 of 79 files**. This is a lower bound: javac stops some checks after resolution errors, and it does not see mixin target strings or the loader modules (934 lines in `forge/`, `fabric/`, `neoforge/`).

Main causes, by count of "cannot find symbol" and other errors:
- `javax.annotation.Nonnull/Nullable` gone from the classpath (49): mechanical.
- Game rules rewritten (`GameRules` package and its inner types: 34): `utilities/BMHGameRules.java` (21), `platform/IPlatformHelper`, `CreateWorldGameTabMixin`.
- GUI rewrite: `GuiGraphics` is gone, `Renderable` now has `extractRenderState(GuiGraphicsExtractor, ...)` (13+): journey overlay, point screens, config screen, spyglass mixin.
- Render pipeline rewrite: `ShaderInstance`, `BufferUploader`, `RenderSystem.setShader*`, `GlStateManager` gone: `client/sky/StarReadingRenderer.java` (33).
- Item model rewrite: `BakedModel`, `ClampedItemPropertyFunction`, `CompassItemPropertyFunction`, `ItemRenderer` gone: `client/AmuletCompass`, `client/ClientItemProperties`, `client/EternalSwordModel`, `mixin/H_ItemRendererSwordMixin`.
- Tools rewrite: `Tier`, `TieredItem`, `SwordItem`, `PickaxeItem`, `DiggerItem` gone: `init/BMHModTierInit`, the Eternal tool items.
- Renames: `ResourceLocation` (13), `InteractionResultHolder` (11), `DimensionTransition` (4), `serverLevel()` (22), `displayClientMessage` (13), `Util`.
- NBT getters return `Optional` (`CompoundTag.getInt`, `getList`, `contains`): `handlers/AmuletPoints`, `EternalBundleItem`.
- Respawn API: `getRespawnPosition`, `getRespawnDimension`, `getSharedSpawnPos`.

Code map parts touched (errors per file in `facts/mc26/javac.log`): every one. Respawn (12), journey cinematic (16), Navigator's Amulet and its screens and command (100), sky and spyglass (39), Totem of Eternity (6), Eternal Items and tools and their abilities (about 100), game rules and world creation toggle (31), config screen (10), anvil mixin (1).

### 1.3 Compile check script

`.claude/skills/verify-loaders/scripts/compile_check.sh` now handles 26.x: it downloads the Mojang client jar and the libraries of the version JSON once into `~/.gradle/caches/bmh-compile-check/<version>` and compiles with JDK 25. With no argument, a 26.x `minecraft_version` gives that version. 1.21.1 still compiles there (`1.21.1: OK (92 classes)` on the unchanged tree).

## Step 2: port (started 2026-11-06 03:30 CET by `date`)

### 2.1 Build scripts (done, `sh ./gradlew projects` BUILD SUCCESSFUL in 3m 30s, log `$SCRATCH/logs/g1-projects.log`)
- Gradle 9.5.0 (the Forge 26.1.2 repo's), foojay resolver 1.0.0, Java 25 toolchain.
- Root plugins: ForgeGradle 7.0.40, `net.fabricmc.fabric-loom` 1.15.5, ModDevGradle 2.0.148, all in one classloader: configure works.
- `common`: ModDevGradle `neoFormVersion = 26.1.2-1` replaces VanillaGradle (vanilla only, as before), `compileOnly org.spongepowered:mixin:0.8.7`.
- `fabric`: no mappings line, `implementation` instead of `modImplementation`, no refmap setting, jsr305 dropped (common uses `org.jspecify.annotations.Nullable`, which Minecraft ships).
- `forge`: ForgeGradle 7 DSL (`minecraft.dependency`, `minecraft.mavenizer(it)`, `fg.forgeMaven`), runs pass `--mixin.config bmh.mixins.json`, the jar manifest keeps `MixinConfigs`. MixinGradle is gone. `net.minecraftforge.gradle.merge-source-sets=true` in gradle.properties.
- Mixin configs: `refmap` removed, `compatibilityLevel` stays `JAVA_21` (Mixin 0.8.7 of Forge knows no `JAVA_25`; `javap` of its `CompatibilityLevel` ends at `JAVA_21`).
- `fabric.mod.json`: `"minecraft": "~26.1"`, `"java": ">=25"`. NeoForge range `[26.1,)`, Minecraft range `[26.1,26.2)`. Only 26.1.2 is tested.

### 2.2 Split of the code port
- Agent `port-client` (log `docs/port-26/agent-client.md`): sky, spyglass, journey overlay, point screens, config screen, client mixins.
- Agent `port-items` (log `docs/port-26/agent-items.md`): items, init, item models and data pack, item mixins, totem handlers, EternalWear.
- Coordinator (this log): game rules, handlers, server mixins, loader modules, build.
- Compile loop: `TAG=<name> bash $SCRATCH/c26.sh` (javac against the 26.1.2 client jar), decompiled reference in `$SCRATCH/src26` (Vineflower 1.12), Forge sources in `$SCRATCH/forge26`, loader patches in `$SCRATCH/facts/patches`.

### 2.3 Coordinator changes (all compile in c26.sh)
- Mechanical: `ResourceLocation` to `Identifier`, `serverLevel()` to `level()`, `displayClientMessage(x, true)` to `sendOverlayMessage(x)`, `isClientSide` to `isClientSide()`, `javax.annotation` dropped, `getGameProfile().getName()` to `name()`.
- Game rules: 26.1 makes them a registry (`BuiltInRegistries.GAME_RULE`, ids like `minecraft:keep_inventory`). `BMHGameRules` builds each `GameRule<Boolean>` the way vanilla's private `GameRules.registerBoolean` does and registers it through `IPlatformHelper.register(Registries.GAME_RULE, ...)`; `registerBooleanGameRule` is gone from the platform interface. New ids: `bmh:random_death_spawn_point`, `bmh:display_text_on_respawn`, `bmh:food_and_health_regen_on_wake_up`, `bmh:only_carry_one_eternal_item_type`, `bmh:damage_eternal_item_when_dead`, `bmh:remove_coordinates_in_survival`. Player-visible: the `/gamerule` names change. Vanilla's `GameRuleRegistryFix` renames only vanilla rules, so a 1.21.1 world's BMH rule values are not carried over (they go back to the default, true). Unverified: that the unknown old keys do not break the load of the other rules.
- Respawn: `PlayerList.respawn` in 26.1.2 still stores `serverPlayer.findRespawnPositionAndUseSpawnBlock(boolean, PostTeleportTransition)` into a local (decompiled `PlayerList.java:387`). The Forge patch does not touch that line; the NeoForge patch fires `PlayerRespawnPositionEvent` after it (`facts/patches/neo-PlayerList.patch:60-66`). `F_PlayerListRespawnMixin` keeps its `@ModifyVariable` `INVOKE_ASSIGN`, retargeted to `TeleportTransition`, and returns `transition.withPosition(...)`. The bed comes from `getRespawnConfig().respawnData()`, the world spawn from `level.getRespawnData().pos()`.
- `Player.attack`: one `Entity.hurtOrSimulate(DamageSource, float)` and one `causeFoodExhaustion(0.1F)` (decompiled `Player.java:985, 996`); the Forge and NeoForge patches add neither (checked in `facts/patches/*-Player.patch`). `E_PlayerAttackMixin` retargeted from `hurt` to `hurtOrSimulate`.
- Player save: `addAdditionalSaveData(ValueOutput)` and `readAdditionalSaveData(ValueInput)`. `AmuletPoints` writes and reads with codecs that keep the 1.21.1 NBT layout (`bmh_points` list of `{name, x, y, z}`, legacy `bmh_home`), missing fields default as before.
- Time: `isNight()` is `isDarkOutside()` (same test), `getDayTime()` is `getOverworldClockTime()`. Unverified: that the Overworld clock counts like the old day time after `/time set`.
- Other mixin targets checked present with the same descriptors: `LivingEntity.checkTotemDeathProtection(DamageSource)Z`, `ServerPlayer.restoreFrom(ServerPlayer, Z)`, `Display.tick`, `ServerPlayer.tick`, `MinecraftServer.loadLevel()V`, `ServerPlayerGameMode.destroyBlock(BlockPos)Z`, field `ServerPlayer.spawnInvulnerableTime`, `Commands(CommandSelection, CommandBuildContext)`, Fabric's `ServerPlayer.die`, `ItemEntity.playerTouch`, `Player.stopSleepInBed(ZZ)`, Display invokers `setTransformation`, `setGlowColorOverride`, `BlockDisplay.setBlockState`.
- Forge module: EventBus 7 (`LivingDeathEvent.BUS.addListener(Priority.HIGHEST, ...)`, cancellable listeners return a boolean), `FMLJavaModLoadingContext.getModBusGroup()`, `FMLCommonSetupEvent.getBus(group)`. The player tick listener runs at `PlayerTickEvent.Post` only (1.21.1 ran `DisableDebugInfo` at both phases; it does nothing twice in a tick).
- verify-loaders bots: mineflayer 4.39.0 lists 26.1 as tested; 26.1, 26.1.1 and 26.1.2 are all protocol 775 in its minecraft-data. The scripts must pass `26.1` as the bot version.
