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

### 2.4 Integration (commit 8eb1721 on `mc-26.1`)
- Both agents finished. `port-client` and `port-items` logs: `docs/port-26/agent-client.md`, `docs/port-26/agent-items.md`.
- `CreativeModeTab.Output` is protected in vanilla 26.1.2; the Forge and NeoForge access transformers make it public (`ats/accesstransformer.cfg` in both userdev jars), Fabric API too. So the common code no longer fills the tab: `IPlatformHelper.creativeTabBuilder(List<Supplier<? extends ItemLike>>)` and each loader calls `displayItems`. This keeps `compile_check.sh` and the NeoForm build of `common` free of an access transformer.
- The item properties are gone from the loaders: `ClientItemProperties`, `EternalSwordModel`, `H_ItemRendererSwordMixin` and Fabric's client entrypoint are deleted. The needle property `bmh:north` registers through `M_RangeSelectPropertiesMixin` (TAIL of `RangeSelectItemModelProperties.bootstrap`, client list). Forge has no patch for that class (no file in its patches folder at tag 26.1.2).
- Item mixin targets checked against the Forge 26.1.2 patches (`facts/patches/forge-{AnvilMenu,ItemEntity,Slot,ItemStack}.patch`): one `ContainerLevelAccess.execute` in `AnvilMenu.onTake`, `createResult` gets an extra early return on Forge (RETURN injections run there too, as on 1.21.1).
- `sh ./gradlew build`: BUILD SUCCESSFUL, jars `bmh-{forge,fabric,neoforge}-26.1.2-1.7.0.jar`. No jar has `bmh.refmap.json`; the Forge manifest has `MixinConfigs: bmh.mixins.json`. Fabric's manifest says `Fabric-Mixin-Version: 0.17.5+mixin.0.8.7`.

### 2.5 First server boots (`$SCRATCH/boot3.sh`, shared verify locks, logs `$SCRATCH/logs/boot1`, `boot2`)
- boot1: all three loaders failed at once: `InvalidAccessorException: No candidates were found matching spawnInvulnerableTime:I in net/minecraft/server/level/ServerPlayer`. 26.1 removed that field: vanilla now protects a respawned player only until the client sends `ServerboundPlayerLoadedPacket`, or 60 ticks (`ServerGamePacketListenerImpl.hasClientLoaded`, line 2207). The mixins themselves applied, which shows the jars work with no refmap in dev.
- Fix (commit 513fb5a): `F_ServerPlayerJourneyMixin` keeps its own 200-tick counter, counted down at the head of `ServerPlayer.tick`, and cancels `ServerPlayer.hurtServer(ServerLevel, DamageSource, float)` while it runs, except for damage in `#bypasses_invulnerability` (the 1.21.1 rule).
- boot2: `Done (` on forge (22 s), fabric (16 s) and neoforge (16 s). Each log has `[BalancedMcHardcoreMain] Loading...`, the azimuth self-check, and `Echo Sense: tag bmh:echo_sense_ores lists 19 blocks.` No mixin error. Every loader logs `No key layers in MapLike[{}]`: vanilla cannot read the empty `generator-settings` of the verify `server.properties` and falls back to the default flat preset (`DedicatedServerProperties.java:269-277`), so the world is still flat.
- Not covered by a boot: mixins on classes that load only on use (anvil, displays, item entities, the client). The verify self-tests and the client runs cover those.

### 2.6 verify-loaders on 26.1 (commit 0a36018)
- Agent `port-hooks` (log `docs/port-26/agent-hooks.md`) ported the Java hooks: `MockPlayers_26.1.java`, shared helpers in every MockPlayers copy, `hook/compat/ClientCompat_{1.21.1,26.1}.java` picked by `client_variant` in `lib.sh`. Checked by the coordinator: `compile_check.sh --selftest 26.1.2` gives `26.1.2: OK (107 classes)`, `--client 26.1.2` gives `OK (97 classes)`. The agent reports OK on 1.21.1 and 1.20 to 1.20.4 with scratch trees of `mc-1.21.1` and `main` (not rerun by the coordinator).
- `jarcheck.py`: on 26.x no jar may ship a refmap, and every item needs `assets/bmh/items/<item>.json` whose models and textures exist. On the build of 8eb1721+: `JARS 163/163`.
- Bots: `rule_names.js` turns the old rule names into 26.1 ids for 26.x; `bot_version` announces 26.1 to mineflayer.

### 2.7 Respawn with a real death (death_client.sh, logs `$SCRATCH/verify/death-d26-*`)
- Fabric: respawn packet t+889ms, then the first position packet at -10824,-60,-2503; overlay gone at t+5857ms; `final respawned=true alive=true screen=- overlay=-`. Frames show the fade, the night sky with the star line and "A new journey begins for you", then the arrival.
- NeoForge: first position packet after the respawn packet at 9149,-60,6306; overlay gone at t+5841ms; same final line.
- Forge: the first run failed before the hook (`GLFW may only be used on the main thread ... -XstartOnFirstThread`): ForgeGradle 7 does not add that flag on macOS. Fixed in `forge/build.gradle` (commit 8af7f90). Rerun (`death-d26b-forge`): position packet at -9167,-60,6277 right after the respawn packet, overlay gone at t+5924ms, same final line.

### 2.8 The next versions, measured with `compile_check.sh` (Mojang jars, `$SCRATCH/logs/javac-26.{2,3}.log`)
- 26.1: `26.1: OK (91 classes)`. The common code compiles against the 26.1 jar too (signatures only; the jar was not run on 26.1 or 26.1.1).
- 26.2: 45 errors in 6 files: `StarReadingRenderer` (13: pipeline `withUniform`, `RenderPass.draw`, `getMainRenderTarget`, `getMainCamera`, `getGuiTicks`), `JourneyOverlay` (14), `JourneyCinematic` (6: `Minecraft.getOverlay/setOverlay`), `EchoSenseHandler` (6: the vanilla ore block tags and `EntityType.BLOCK_DISPLAY`), `PointSignal` (5), `ConfigScreen` (1: `setScreen`).
- 26.3: 79 errors in 13 files: the same plus the blaze3d packages moved (`GpuBuffer`, `RenderPipeline`, `RenderPass`, `VertexFormat`), `AxeItem` gone (`EternalAxeItem`), `BundleContents.Mutable` and `itemCopyStream` changed (`EternalBundleItem`).
- These are lower bounds (javac stops some checks after resolution errors) and say nothing about mixin targets.

## Step 3: verify-loaders (matrix.sh mc-26.1 26.1.2, BMH_LOCAL=1, outputs `$SCRATCH/verify/mc26`)

First run (branch.sh at 8af7f90, selftest.sh at b76cd12), the summary lines verbatim:
`mc26 8af7f90 build=ok jars=163/163 | forge up=1 amulet=40/40 smoke=44/46 srvErr=0 tag=1 client=0 shots=0 cliErr=0 | fabric up=1 amulet=0/0 smoke=0/1 srvErr=0 tag=1 client=1 shots=4 cliErr=0 | neoforge up=1 amulet=40/40 smoke=44/46 srvErr=0 tag=1 client=1 shots=4 cliErr=0`
`mc26 b76cd12 selftest | forge up=1 selftest=71/71 end=1 fail=0 | fabric up=1 selftest=71/71 end=1 fail=0 | neoforge up=1 selftest=71/71 end=1 fail=0`

The red cells, one by one:
- smoke 44/46 on Forge and NeoForge: two bot bugs, not mod bugs. `Echo Sense: ores light up` read `Test passed. Count: 2` (26.1 capitalises Count; the regex wanted lowercase). `cow totem` summoned the cow with `HandItems`, which 26.1 ignores (`equipment.mainhand` since 1.21.5), so the cow had no totem and died. Fixed in `smoke_bot.js` (commit aab701f). Rerun with `server_only.sh mc26 26.1.2 <loader> smoke_bot.js smoke2`: `SMOKE 46/46` on Forge and on NeoForge; the cow line now reads `PASS cow totem: still has Fire Resistance [Cow has ... id: "minecraft:fire_resistance"...]`.
- Fabric bots 0/1: the server kicks a vanilla client: `This server requires Fabric Loader and Fabric API installed on your client! The following registry entry namespaces may be related: bmh` (`server-fabric.log:113`). Fabric API 0.116 (1.21.1) skipped the registry sync for a client that could not receive it (`RegistrySyncManager.configureClient` in the 5.3.2 sources jar returns early); the 26.1 Fabric API (registry sync 7.1.1) disconnects it unless every modded synced registry is optional (`javap` of `RegistrySyncManager`). So the mineflayer bots cannot test Fabric on 26.1, and a vanilla client cannot join a Fabric 26.1 server that runs BMH. Fabric coverage on 26.1: the 71 self-tests, the quick-play client, the real death. Which registry triggers it is not confirmed (the game rules registry is new since 1.21.1; the items are modded in 1.21.1 too).
- Forge client=0: `IllegalStateException: Can't find a primary monitor` at 04:44, the Mac display was off (memory `bmh-forge-1201-display-off`). The Forge death client ran fine at 04:34. Rerun with `earlyWindowControl = false` in `verify-mc26/forge/run/config/fml.toml`: see 3.1.
- Visual check of the four screenshots of Fabric and NeoForge: the star reading draws in the night sky, the totem animation shows the Totem of Eternity. Client log: `sword textures: inventory bmh:item/eternal_sword, hand bmh:item/eternal_sword_in_hand, GUI bmh:item/eternal_sword; sword in-hand model differs from the inventory one: true, inventory model kept in the GUI: true`, `after the totem: health=7.0 slot5=bmh:dormant_totem`.
- Harness review (inquisitor, no blocker): CLAUDE.md fixes (commit b76cd12), version JSON cache in compile_check (aab701f). The coordinator reran the hooks on the older versions: `--selftest` OK on 1.21.1 (108 classes) and 1.20 to 1.20.4 (108 each), `--client` OK on 1.21.1 and 1.20 to 1.20.4 (98 each), with fresh detached worktrees of `mc-1.21.1` and `main`, removed after.

## Steps left for the next versions
- 26.2 (Forge 26.2-65.1.6, NeoForge 26.2.0.93, Fabric API 0.161.0+26.2, Java 25): branch `mc-26.2` from `mc-26.1`, set `gradle.properties`, then fix the 45 compile errors of 2.8 (sky pipeline API, journey overlay, `Minecraft.getOverlay/setOverlay`, the vanilla ore block tags and `EntityType.BLOCK_DISPLAY` in Echo Sense, `PointSignal`, `setScreen`). Recheck every mixin target and the Forge and NeoForge patches the same way as here. The bots cannot run: mineflayer's minecraft-data 3.119.0 has no 26.2 data (`dataPaths.json` lists only 26.1 for 26.x), and 26.2 is protocol 776.
- 26.3 (Forge 26.3-66.1.5, NeoForge 26.3.0.101, Fabric API 0.165.2+26.3): from `mc-26.2` rather than `mc-26.1`. 79 compile errors from 26.1: the 26.2 set plus the blaze3d package move (`GpuBuffer`, `RenderPipeline`, `RenderPass`, `VertexFormat`), `AxeItem` gone, `BundleContents.Mutable` and `itemCopyStream`. Same bot problem (protocol 777).
- 1.21.11 backport (Forge 1.21.11-61.2.5, NeoForge 21.11.48, Fabric API 0.141.6+1.21.11, Java 21): the jar is still obfuscated, so the build goes back to remapping: `fabric-loom` (remap plugin, MultiLoader-Template 1.21.11 uses `1.13-SNAPSHOT`) with `officialMojangMappings()`, a Fabric refmap again, and a Loom jar for `compile_check.sh 1.21.11` (build once, then the script works as for 1.21.1). Not measured: which 26.1 APIs (GUI extraction, game rule registry, `Identifier`, `ValueOutput`) already exist in 1.21.11. Start from `mc-26.1` and expect to undo the 26.1-only parts, or from `mc-1.21.1` and redo the 1.21.2 to 1.21.11 parts: measure both with one compile before choosing.

### 3.1 Reruns and the final matrix (`date` 2026-11-06 05:51 CET)
- Forge cell rerun with `earlyWindowControl = false` (05:35): `amulet=39/40 smoke=46/46 client=1 shots=4`. The amulet failure was real: `bed point kept on the head of the bed [... {name: "Bed", x: -8, y: -60}]`, no z. `optionalFieldOf("z", 0)` leaves out a value equal to the default, so a point at z = 0 was saved without z. Fixed in f04f162 (every field always written); probe `$SCRATCH/probe26/ProbeCodec.java` prints `encoded: {name:"Bed",x:-8,y:-60,z:0}` and reads a save without z as z = 0.
- Final matrix on f04f162, verbatim:
`mc26 f04f162 build=ok jars=163/163 | forge up=1 amulet=40/40 smoke=46/46 srvErr=0 tag=1 client=1 shots=4 cliErr=0 | fabric up=1 amulet=0/0 smoke=0/1 srvErr=0 tag=1 client=1 shots=4 cliErr=0 | neoforge up=1 amulet=40/40 smoke=46/46 srvErr=0 tag=1 client=1 shots=4 cliErr=0`
`mc26 f04f162 selftest | forge up=1 selftest=71/71 end=1 fail=0 | fabric up=1 selftest=71/71 end=1 fail=0 | neoforge up=1 selftest=71/71 end=1 fail=0`
- Everything is green except the Fabric bot cells, which cannot run (Fabric API refuses vanilla clients on 26.1, see Step 3).

## Not verified (as of step 3; step 4 closed most of it, see 4.7)
- 26.1 and 26.1.1 at runtime: closed by 4.5 and 4.6.
- The Fabric bot scenarios: closed by 4.2.
- A world upgraded from 1.21.1: closed by 4.4 (values now kept).
- Looks and feel judged by a human: still open, see 4.7.
- Hoppers versus the one-totem rule: closed by 4.3 (code read).

## Step 4: release readiness (owner: "finish to fix and verify what is needed, we aim for a release soon"; `date` 2026-11-06 09:26 CET)

### 4.1 Tag name (commit 71765d7)
`mc-26.1.2-v<version>`: the tag takes `minecraft_version`, as on every branch, so the release skill and `publish_local.py` need no change.

### 4.2 Fabric bots (commit 7ad8448)
`hook/fabric/Z_VanillaBotsMixin.java`, copied by `lib.sh fabric_bot_hook` into the verify tree for Fabric on 26.x only and removed at the end, makes `RegistrySyncManager.areAllRegistriesOptional` return true: the server then lets a vanilla client in without a sync, as Fabric API 0.116 did on 1.21.1 (bytecode of `configureClient`: no sync channel and not all optional means disconnect). jarcheck runs before the hook. Results on the verify tree at f04f162: Fabric `SMOKE 46/46` (`smoke3-fabric.log`), amulet `40/40 checks passed` (`amulet3-fabric.log`).

### 4.3 Hoppers and the one-totem rule
NeoForge's transfer API inserts into a vanilla container through `VanillaContainerWrapper`, whose `isValid` calls `container.canPlaceItem` (NeoForge 26.1.2.126 sources, `transfer/item/VanillaContainerWrapper.java:174-176`), which `I_ContainerTotemMixin` adds. Code read only.

### 4.4 Upgrade of a 1.21.1 world (scratch `up26/`, NeoForge 21.1 then NeoForge 26.1.2)
First run (f04f162): 10/10, but the game rules file logged `Failed to parse saved data for 'SavedDataType[minecraft:game_rules]': Not a valid resource location: displayTextOnRespawn ...` and the BMH rules went back to ON. Fix (73d93c9): `N_SavedDataGameRulesMixin` renames the six old keys with their values at the return of `SavedDataStorage.readTagFromDisk` (no loader patch touches that call: `facts/patches/*-SavedDataStorage.patch`). Rerun on 73d93c9: `UPGRADE26 new 13/13`: items with their damage and bundle contents, the amulet point, `keep_inventory=true`, `bmh:random_death_spawn_point=false` and `bmh:damage_eternal_item_when_dead=false` kept, an unchanged rule still true, a distant respawn of 35552 blocks, no death wear with the rule off. No error line in the new server log; six `Game rule <old> is now <new> (<value>)` lines. The changelog now says the values are kept.

### 4.5 26.1 and 26.1.1 (scratch trees `t26.1`, `t26.1.1`: the same sources with that version's loaders)
- Loaders: 26.1: Forge 62.0.9, NeoForge 26.1.0.19-beta, Fabric API 0.145.1+26.1. 26.1.1: Forge 63.0.2, NeoForge 26.1.1.15-beta, Fabric API 0.145.4+26.1.1. NeoForge had only beta builds before 26.1.2.
- Build: OK on both. Classes against the 26.1.2 build: Forge and Fabric identical (26.1: 96 and 98 classes, 0 differ). NeoForge: three mixin classes differ only in how `at` is stored: NeoForge 26.1.0 ships `sponge-mixin 0.17.0`, where `@ModifyArg.at` and `@ModifyVariable.at` are one `@At`; 26.1.2 ships 0.17.4, where they are `At[]`.
- Self-tests: `p26.1` and `p26.1.1`: 71/71 on forge, fabric and neoforge.
- Production servers (official installers, the shipped jars in mods/, `$SCRATCH/prod/`): see 4.6.

### 4.6 Production servers (`$SCRATCH/prod/prod_test.sh`, `all.sh`, results `all.out`)
The official installer of each loader (maven.minecraftforge.net, maven.neoforged.net, meta.fabricmc.net with Fabric API from maven.fabricmc.net), the jars built at 73d93c9 (with `N_SavedDataGameRulesMixin`, without any verify hook) in `mods/`, a fresh flat world, Java 25. Result lines verbatim:
```
[26.1 forge 62.0.9] RESULT up=1 mixinErrors=0 SMOKE 46/46 bmh=2
[26.1 fabric 0.20.1] RESULT up=1 mixinErrors=0 - bmh=2
[26.1 neoforge 26.1.0.19-beta] RESULT up=1 mixinErrors=0 SMOKE 46/46 bmh=2
[26.1.1 neoforge 26.1.1.15-beta] RESULT up=1 mixinErrors=0 SMOKE 46/46 bmh=2
[26.1.1 forge 63.0.2] RESULT up=1 mixinErrors=0 SMOKE 46/46 bmh=2
[26.1.1 fabric 0.20.1] RESULT up=1 mixinErrors=0 - bmh=2
[26.1.2 neoforge 26.1.2.126] RESULT up=1 mixinErrors=0 SMOKE 46/46 bmh=2
[26.1.2 forge 64.1.6] RESULT up=1 mixinErrors=0 SMOKE 46/46 bmh=2
[26.1.2 fabric 0.20.1] RESULT up=1 mixinErrors=0 - bmh=2
```
The Fabric cells are a boot only: the shipped jar has no verify hook, so the 26.1 Fabric API refuses the vanilla bot. Fabric on 26.1 and 26.1.1 rests on that boot, the 71 self-tests of 4.5, and its classes being byte-identical to the 26.1.2 build. NeoForge 26.1.0 runs Mixin 0.17.0 (single `at`), and the jar built against 0.17.4 (`at` arrays) still passes there: the Mixin version gap of 4.5 does not break it. The first attempt of this run passed each line as one unsplit word (a zsh loop) and booted nothing; `all.sh` is the bash rerun.
CurseForge dry run (`publish_local.py --dry-run --only curseforge`, read only): every name resolves: `26.1` 15933, `26.1.1` 16021, `26.1.2` 16082, `Java 25` 14454, Client 9638, Server 9639, Forge 7498, Fabric 7499, NeoForge 10150. `release_game_versions=26.1,26.1.1,26.1.2` (commit 2e0b311) lists each file for the three versions.

### 4.7 Still open before the release
- Looks and feel by a human (star reading, cinematic and its frame rate, screens, needle, bundle sprite).
- The `.claude/` harness changes of this branch (compile_check, hooks, bots, jarcheck, release script) exist only on `mc-26.1`. CLAUDE.md wants every change on every branch: port them to `main` and the other `mc-*` branches before a release runs the matrix from another checkout.
- Bump `mod_version` and the changelog section, push, tag `mc-26.1.2-v<version>`, publish: the release skill, with the owner's go.

