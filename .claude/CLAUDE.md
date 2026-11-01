# Balanced Minecraft Hardcore (BMH)

Minecraft mod (mod id `bmh`) for Forge, Fabric and NeoForge: death sends you on a distant respawn instead of ending the run, the distance grows with the difficulty and the Eternal Items you carry. Author: MrOrnithorynque.

## Layout: one codebase, three loaders

Four Gradle modules (MultiLoader-Template style, all on official Mojang mappings):

- `common/`: the gameplay, vanilla code only. Its own build compiles it against vanilla Minecraft (VanillaGradle), so a loader import there fails the build. It ships no jar of its own (`common/build/libs` is a by-product, never published): each loader module compiles its sources and resources into their jars (root `build.gradle`), so each loader's mixin processor writes `bmh.refmap.json` for the common mixins too (except NeoForge, and Forge on 1.21.1, see Building).
- `forge/`: ForgeGradle 6 + MixinGradle, the `@Mod` class, `Config` (`config/bmh-common.toml`) and `forge/ForgeEvents` (Forge events, with the priorities the handlers rely on).
- `fabric/`: fabric-loom, the entrypoints, `FabricConfig` (`config/bmh.properties`) and `fabric/mixin`.
- `neoforge/`: ModDevGradle, `neoforge/BalancedMcHardcoreNeoForge` (`@Mod`), `NeoForgeConfig` (`config/bmh-common.toml`, same as Forge), `NeoForgeEvents` (same handlers and priorities as `ForgeEvents`) and the mixins config listed in its mods.toml (`neoforge.mods.toml` on 1.21.1). NeoForge runs on the official names since 1.20.2: no reobf, no refmap. It is built on 1.20.4 and 1.21.1: on the 1.20.x branches `settings.gradle` includes it only when `gradle.properties` has `neoforge_version` (only 1.20.4 does), and `dev-52.1.20` always includes it. NeoForge has no 1.20, its 1.20.1 is a Forge fork that loads the Forge jar as is (`publish.yml` lists that jar for NeoForge too), and its 1.20.2 and 1.20.3 lines died young, so `neoforge/` sits unbuilt on those branches.

Common code reaches the loader through `platform/IPlatformHelper` (registration, creative tab builder, game rules, physical side, config), found with `ServiceLoader` (`META-INF/services` in each loader). Vanilla keeps `GameRules.register` private, so game rules go through it too.

Fabric hooks death, wake-up, item pickup and the anvil's enchant check with its own mixins rather than Fabric API events, to fire where Forge fires its events: `ALLOW_DEATH` runs before the totem check and `AFTER_DEATH` after the drops; the enchant mixin mirrors Forge's `canApplyAtEnchantingTable` hook (without it a Mending book would go on the Navigator's Amulet). On 1.21.1 the anvil reads only the enchantment's item tags on Forge and Fabric, so that enchant mixin is a common one (`mixin/EnchantmentMixin.java`) there; without it the amulet would take no book at all. NeoForge 1.21.1's anvil skips `Enchantment.canEnchant` and asks the item's `supportsEnchantment(ItemStack, Holder<Enchantment>)`, which the amulet overrides (no `@Override`, like the Forge hook); NeoForge 1.20.4 reaches `canApplyAtEnchantingTable` like Forge.

A dedicated server must never load a client class: on Forge it refuses to start ("Attempted to load class ... for invalid dist DEDICATED_SERVER"; 1.1.0 and earlier did exactly that). Client code lives in `client/LocalPlayerText`, `TextDrawer`, the client mixin and the loaders' client hooks, and stays out of the server's way: common code reaches it only behind `Services.PLATFORM.isPhysicalClient()`, the loader hooks are gated by `FMLEnvironment.dist` (Forge and NeoForge) and the `client` entrypoint (Fabric), and the mixin by the `client` list of its config. `javap -v` on the built classes shows which ones reference `net/minecraft/client`.

## One branch per Minecraft version

The same mod ships for several Minecraft versions. Each version lives on its own long-lived branch. Everything under `common/src`, `forge/src`, `fabric/src` and `neoforge/src`, the build scripts other than `forge/build.gradle`, and the wrapper are identical on all the 1.20.x branches; `forge/build.gradle`, `gradle.properties`, `README.md` and `.vscode/settings.json` differ per branch (and `mods.toml` on `dev-46.0.14`).

`dev-52.1.20` (1.21.1) is the exception: it has its own sources and build scripts, because 1.20.5 moved item data from NBT to data components, made enchantments registry data, made `ResourceLocation`'s constructor private (`fromNamespaceAndPath`), renamed the data pack folders (`recipe/`, `loot_table/`) and went to Java 21, and Forge 1.20.6+ runs on the official names (no reobf, no refmap). Its port commit summarizes the differences; `git diff main dev-52.1.20` is the full list.

| Branch | Minecraft | Forge | Fabric API | NeoForge |
|---|---|---|---|---|
| `main` | 1.20.4 | 49.0.31 (same sources as `dev-49.0.31`) | 0.97.3+1.20.4 | 20.4.253 |
| `dev-49.0.31` | 1.20.4 | 49.0.31 | 0.97.3+1.20.4 | 20.4.253 |
| `dev-49.0.2` | 1.20.3 | 49.0.2 | 0.91.1+1.20.3 | none |
| `dev-48.1.0` | 1.20.2 | 48.1.0 | 0.91.6+1.20.2 | none |
| `dev-47.2.0` | 1.20.1 | 47.2.0 | 0.92.12+1.20.1 | the Forge jar |
| `dev-46.0.14` | 1.20 | 46.0.14 | 0.83.0+1.20 | none |
| `dev-52.1.20` | 1.21.1 | 52.1.20 | 0.116.17+1.21.1 | 21.1.265 |

The table can drift: `gradle.properties` on each branch (`minecraft_version`, `forge_version`, `fabric_api_version`, `neoforge_version`) is the truth, except the 1.20.1 NeoForge cell: that is `publish.yml` listing the Forge jar for NeoForge when `minecraft_version` is 1.20.1.

**Every change goes to every branch, never to `main` alone.** Commit on `main`, then cherry-pick onto each `dev-*` branch (scratch worktrees work well), build each one, and push them together. `forge/build.gradle`, `gradle.properties` and `README.md` can conflict: keep the branch's own version of every line the change did not touch and apply only the change's hunks. Never take `main`'s whole file (`dev-46.0.14`'s `forge/build.gradle` has a different layout). After porting, `git diff --stat main <branch> -- . ':!forge/build.gradle' ':!gradle.properties' ':!README.md' ':!forge/src/main/resources/META-INF/mods.toml' ':!.vscode'` must be empty for every 1.20.x branch (`dev-46.0.14`'s `mods.toml` only lacks a commented-out template block).

On `dev-52.1.20` the cherry-pick applies cleanly only where the change does not touch the code 1.21 changed (items and their data, enchantments, `ResourceLocation` construction, data pack files, the loader hooks, the build scripts). Elsewhere, port the change by hand to the 1.21.1 API, build, and boot a server on each loader. The empty-diff check does not apply to it.

## Building

- No JDK is on the PATH; use Homebrew's: `JAVA_HOME=/opt/homebrew/opt/openjdk@17` on the 1.20.x branches, `JAVA_HOME=/opt/homebrew/opt/openjdk@21` on `dev-52.1.20`. The branch's `build.gradle` (`JavaLanguageVersion.of(...)`) says which; `publish.yml` reads it there too.
- `gradlew` has no exec bit in git: run `sh ./gradlew build` (every jar), or one loader: `sh ./gradlew :fabric:build`, `:forge:runClient`, `:neoforge:runServer`.
- The release jars land in `forge/build/libs/`, `fabric/build/libs/` and `neoforge/build/libs/`. The NeoForge jar never has a refmap (official names). The other two must contain `bmh.refmap.json`, and the Forge jar a `MixinConfigs` manifest entry, or the mixins will not load outside the dev environment. On `dev-52.1.20` the Forge jar has no refmap, on purpose: Forge 1.21.1 runs on the official names, and the annotation processor's refmap would point at SRG names that no longer exist at runtime (Mixin then logs that `bmh.refmap.json` could not be read, which is harmless).
- `runServer` needs `eula=true` in `<loader>/run/eula.txt`. Booting a dedicated server on each loader is the cheap check that registration, game rules, config and the client/server split hold.
- `runClient` opens a plain `java` window that computer use cannot drive: ask the user to click through, then read `<loader>/run/logs/latest.log` (or the redirected output) for `[BalancedMcHardcoreMain]` lines.

## Code map

Paths under `common/src/main/java/github/mrornithorynque/bmh/` unless stated.

- `handlers/RespawnPositionHandler.java`: the distant respawn (distance formula, safe landing spot). Its targets are thousands of blocks from any player, in chunks that are not loaded: `Level.getHeight` then answers `minBuildHeight` and every spot looks like void, so load the chunk first (`serverLevel.getChunk(...)`). This broke every respawn once and compiled fine: verify respawn changes with a real death in `runClient` and its `[BalancedMcHardcoreMain]` log lines.
- `handlers/RespawnEffectHandler.java`: bell sound and "new journey" text on respawn.
- `utilities/BMHGameRules.java`: the six BMH game rules; `ALL_RULES` is what the world creation toggle flips.
- `mixin/CreateWorldGameTabMixin.java`: the "Balanced Hardcore: ON/OFF" button on the Create New World screen, registered in `common/src/main/resources/bmh.mixins.json` (every loader; NeoForge lists it in its mods.toml). Fabric-only mixins are in `fabric/src/main/resources/bmh.fabric.mixins.json`. `forge/.../mixin/ForgeGuiMixin.java` is dead (body commented out, not registered; gone on `dev-52.1.20`). To add one, use the `add-mixin` skill.
- `client/LocalPlayerText.java` and `github/mrornithorynque/utilities/TextDrawer.java` (outside `bmh/`) draw on the client of the same JVM: on-screen text works in singleplayer and for a LAN host, not for other players.

## Changelog

`CHANGELOG.md` at the repo root, same file on every branch. Any change a player could notice adds a line under `## [Unreleased]` in the same commit, written for players (what changed in game, not which class). A release turns that section into the version's entry (see the `release` skill), and `publish.yml` posts exactly that entry as the changelog on Modrinth, CurseForge and the GitHub release.

## Releasing

Use the `release` skill (`.claude/skills/release/SKILL.md`). Short version: bump `mod_version` and the changelog on every branch, then push one `mc-<minecraft_version>-v<mod_version>` tag per version branch, one tag per `git push` (GitHub ignores pushes of more than three tags at once); `.github/workflows/publish.yml` publishes each tag to Modrinth and CurseForge, as two versions (Forge, Fabric) per tag.
