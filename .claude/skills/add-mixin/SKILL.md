---
name: add-mixin
description: Add or change a Mixin in BMH (inject into vanilla Minecraft classes such as screens or the HUD), on Forge, Fabric, NeoForge or all of them. Use when a feature needs to change vanilla UI or behavior that loader events do not expose, when the user asks to add a button or option to a vanilla screen, or when a mixin fails to apply or crashes on load.
---

# Adding a Mixin to BMH

Mixin is already wired on every loader. Two configs, one refmap name (`bmh.refmap.json`), written by each loader's build for its own jar (except NeoForge, and Forge on 1.21.1, which run on the official names and need none; NeoForge lists the common config in its `mods.toml`, `neoforge.mods.toml` on 1.21.1):

- `common/src/main/resources/bmh.mixins.json`: mixins in `common/src/main/java/github/mrornithorynque/bmh/mixin/`, applied on Forge, Fabric and NeoForge.
- `fabric/src/main/resources/bmh.fabric.mixins.json`: mixins in `fabric/src/main/java/github/mrornithorynque/bmh/fabric/mixin/`, Fabric only.

The reference mixin is `common/src/main/java/github/mrornithorynque/bmh/mixin/CreateWorldGameTabMixin.java`: read it first, it uses most patterns below. `fabric/.../fabric/mixin/` holds the simple `HEAD` injections.

## 1. Pick where it goes

- **Forge has an event for the hook**: do not mixin on Forge. Put the logic in a common handler (`common/.../handlers/`) and call it from `forge/.../forge/ForgeEvents.java` and `neoforge/.../neoforge/NeoForgeEvents.java` (same handlers, same priorities). On Fabric, use a Fabric API event when it fires at the same point as Forge's (respawn, ticks and HUD do: `BalancedMcHardcoreFabric`, `BalancedMcHardcoreFabricClient`), otherwise a Fabric-only mixin where Forge fires its event (death, wake-up and item pickup do; read the Forge-patched code, step 2). Check before trusting a Fabric API event: `ServerLivingEntityEvents.ALLOW_DEATH` runs before the totem check, `AFTER_DEATH` after the drops.
- **No loader event** (a vanilla screen, say): a common mixin, listed once in `bmh.mixins.json`.
- **Client-only target** (screens, rendering): list it under `client`, never `mixins`. A dedicated server must never load a client class, and Forge refuses to start if it does; common code that runs on the server reaches client code only behind `Services.PLATFORM.isPhysicalClient()` (see `client/LocalPlayerText.java`).

## 2. Read the code you target

There are no sources in the repo, but ForgeGradle leaves a decompiler and the mapped Minecraft jar in the Gradle cache (built by `sh ./gradlew :forge:build`). Decompile only the package you need, into the scratchpad (paths for 1.20.4 / Forge 49.0.31; on another branch use its `minecraft_version` and `forge_version`):

```bash
#!/bin/bash
S=<absolute scratchpad path>/mc-src
JAR=$(find ~/.gradle/caches/forge_gradle/minecraft_user_repo/net/minecraftforge/forge -name 'forge-1.20.4-49.0.31_mapped_official_1.20.4.jar' | head -1)
FF=$(find ~/.gradle/caches/forge_gradle/maven_downloader/net/minecraftforge/forgeflower -name 'forgeflower-*.jar' | head -1)
mkdir -p "$S/cls" "$S/dec"
( cd "$S/cls" && unzip -o -q "$JAR" 'net/minecraft/client/gui/screens/worldselection/*' )
/opt/homebrew/opt/openjdk@17/bin/java -jar "$FF" -dgs=1 "$S/cls/net/minecraft/client/gui/screens/worldselection" "$S/dec"
```

That jar carries Forge's patches: Fabric runs the vanilla code without them. NeoForge has its own patches, and they differ (NeoForge 1.21.1's anvil calls the item's `supportsEnchantment` and never reaches `Enchantment.canEnchant`): after `sh ./gradlew :neoforge:build`, its patched sources are in `neoforge/build/moddev/artifacts/neoforge-<version>-sources.jar`. A common mixin can silently do nothing on NeoForge when its patch routes around the injection point. For a common mixin, check that the injection point is not inside a Forge patch (a `ForgeHooks` or `ForgeEventFactory` call around it); for a Fabric mixin that mirrors a Forge event, the patch shows exactly where the event fires.

`javap -p -cp "$JAR" 'pkg.Outer$Inner'` lists members, including synthetic ones, without decompiling. Before writing a mixin, look for a public getter (`CreateWorldScreen.getUiState()` existed, so no accessor was needed). Common code compiles against vanilla only (`sh ./gradlew :common:compileJava`): Forge's access transformers widen some vanilla members (`GameRules.register` is private in vanilla), so a member visible from `forge/` may not be from `common/`.

## 3. Write the mixin

In the package of its config (listed at the top), then list the class name in that config: under `client` for client-only targets, under `mixins` for code that runs on both sides.

- **Package-private or inner class:** target by string, `@Mixin(targets = "net.minecraft.client.gui.screens.worldselection.CreateWorldScreen$GameTab")`.
- **Injecting into an inner class constructor:** the outer instance is the constructor's first parameter, so the handler is `(CreateWorldScreen screen, CallbackInfo ci)` with `@Inject(method = "<init>", at = @At("TAIL"))`.
- **Protected fields of the target's superclass** (`GridLayoutTab.layout`): make the mixin class `extends` that superclass, with a private constructor that only calls `super(...)` (never run, it just has to compile). Otherwise use `@Shadow` for the target's own fields (private ones included); an `@Accessor` interface is only needed when code outside the mixin reads the field.
- Prefix handler names with `bmh$` so they cannot collide with other mods' mixins. `private static final` fields and lambdas are fine in a mixin.
- `injectors.defaultRequire` is 1: an injection that finds no target crashes the game when the target class loads, which is what you want in development.

## 4. Check it

1. `JAVA_HOME=/opt/homebrew/opt/openjdk@17 sh ./gradlew build` (`openjdk@21` on `dev-52.1.20`), then the Forge and Fabric jars (`forge/build/libs/`, `fabric/build/libs/`) must still contain `bmh.refmap.json` (the Forge one only on 1.20.x), and the Forge jar a `MixinConfigs` manifest line. `unzip -p <fabric jar> bmh.refmap.json` lists every injection target resolved to intermediary names: a mixin missing there found no target at compile time.
2. `sh ./gradlew :forge:runClient` and `sh ./gradlew :fabric:runClient` (and `:neoforge:runClient` on 1.20.4 and 1.21.1), and have the user open the screen or trigger the code (computer use cannot drive the dev `java` window). A mixin is applied lazily, when its target class is first loaded; on Forge the proof is a line logged at DEBUG at that moment, so it is in `forge/run/logs/debug.log` (and the console), never in `latest.log`: `Mixing <YourMixin> from bmh.mixins.json into <target>`.
3. For anything that runs on the server, boot a dedicated server on each loader too (`:forge:runServer`, `:fabric:runServer`, `:neoforge:runServer` where it is built, with `eula=true` in `<loader>/run/eula.txt`).
4. Harmless in the Forge logs: `Reference map 'bmh.refmap.json' ... could not be read` (dev uses unobfuscated names, and so does a 1.21.1 release) and the DEBUG line about `JAVA_17` (`JAVA_21` on 1.21.1) being higher than the supported compatibility level.

## 5. Every Minecraft version

The mixin goes to every version branch like any change (see `.claude/CLAUDE.md`). Vanilla internals move between versions: a branch that compiles proves the signatures exist, not that the injection point still means the same thing. When the target is a method body (not a constructor `TAIL` or `HEAD`), decompile the same class on the oldest branch (1.20 / Forge 46.0.14) and compare before porting. On `dev-52.1.20` (1.21.1) the cherry-pick may not even compile: port the mixin by hand there, against the 1.21.1 class.
