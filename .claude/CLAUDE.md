# Balanced Minecraft Hardcore (BMH)

Minecraft mod (mod id `bmh`) for Forge and Fabric: death sends you on a distant respawn instead of ending the run, the distance grows with the difficulty and the Eternal Items you carry. Author: MrOrnithorynque.

## Layout: one codebase, two loaders

Three Gradle modules (MultiLoader-Template style, all on official Mojang mappings):

- `common/`: the gameplay, vanilla code only. Its own build compiles it against vanilla Minecraft (VanillaGradle), so a Forge or Fabric import there fails the build. It is not a jar of its own: `forge` and `fabric` compile its sources and resources into their jars (root `build.gradle`), so each loader's mixin processor writes `bmh.refmap.json` for the common mixins too.
- `forge/`: ForgeGradle 6 + MixinGradle, the `@Mod` class, `Config` (`config/bmh-common.toml`) and `forge/ForgeEvents` (Forge events, with the priorities the handlers rely on).
- `fabric/`: fabric-loom, the entrypoints, `FabricConfig` (`config/bmh.properties`) and `fabric/mixin`.

Common code reaches the loader through `platform/IPlatformHelper` (registration, creative tab builder, game rules, physical side, config), found with `ServiceLoader` (`META-INF/services` in each loader). Vanilla keeps `GameRules.register` private, so game rules go through it too.

Fabric hooks death, wake-up, item pickup and the anvil's enchant check with its own mixins rather than Fabric API events, to fire where Forge fires its events: `ALLOW_DEATH` runs before the totem check and `AFTER_DEATH` after the drops; the enchant mixin mirrors Forge's `canApplyAtEnchantingTable` hook (without it a Mending book would go on the Navigator's Amulet).

A dedicated server must never load a client class: on Forge it refuses to start ("Attempted to load class ... for invalid dist DEDICATED_SERVER"; 1.1.0 and earlier did exactly that). Client code lives in `client/LocalPlayerText`, `TextDrawer`, the client mixin and the loaders' client hooks, and is only reached behind `Services.PLATFORM.isPhysicalClient()`. `javap -v` on the built classes shows which ones reference `net/minecraft/client`.

## One branch per Minecraft version

The same mod ships for several Minecraft versions. Each version lives on its own long-lived branch. Everything under `common/src`, `forge/src` and `fabric/src`, the Gradle files other than `forge/build.gradle`, and the wrapper are identical on all of them; `forge/build.gradle`, `gradle.properties` and `README.md` differ per branch.

| Branch | Minecraft | Forge | Fabric API |
|---|---|---|---|
| `main` | 1.20.4 | 49.0.31 (same sources as `dev-49.0.31`) | 0.97.3+1.20.4 |
| `dev-49.0.31` | 1.20.4 | 49.0.31 | 0.97.3+1.20.4 |
| `dev-49.0.2` | 1.20.3 | 49.0.2 | 0.91.1+1.20.3 |
| `dev-48.1.0` | 1.20.2 | 48.1.0 | 0.91.6+1.20.2 |
| `dev-47.2.0` | 1.20.1 | 47.2.0 | 0.92.12+1.20.1 |
| `dev-46.0.14` | 1.20 | 46.0.14 | 0.83.0+1.20 |

The table can drift: `gradle.properties` on each branch (`minecraft_version`, `forge_version`, `fabric_api_version`) is the truth.

**Every change goes to every branch, never to `main` alone.** Commit on `main`, then cherry-pick onto each `dev-*` branch (scratch worktrees work well), build each one, and push them together. `forge/build.gradle`, `gradle.properties` and `README.md` can conflict: keep the branch's own version of every line the change did not touch and apply only the change's hunks. Never take `main`'s whole file (`dev-46.0.14`'s `forge/build.gradle` has a different layout). After porting, `git diff --stat main <branch> -- . ':!forge/build.gradle' ':!gradle.properties' ':!README.md' ':!forge/src/main/resources/META-INF/mods.toml' ':!.vscode'` must be empty for every branch (`dev-46.0.14`'s `mods.toml` only lacks a commented-out template block).

## Building

- JDK 17 is not on the PATH; use Homebrew's: `JAVA_HOME=/opt/homebrew/opt/openjdk@17`.
- `gradlew` has no exec bit in git: run `sh ./gradlew build` (both jars), or one loader: `sh ./gradlew :fabric:build`, `:forge:runClient`, `:fabric:runServer`.
- The release jars land in `forge/build/libs/` and `fabric/build/libs/`. Both must contain `bmh.refmap.json`, and the Forge jar a `MixinConfigs` manifest entry, or the mixins will not load outside the dev environment.
- `runServer` needs `eula=true` in `<loader>/run/eula.txt`. Booting a dedicated server on each loader is the cheap check that registration, game rules, config and the client/server split hold.
- `runClient` opens a plain `java` window that computer use cannot drive: ask the user to click through, then read `<loader>/run/logs/latest.log` (or the redirected output) for `[BalancedMcHardcoreMain]` lines.

## Code map

Paths under `common/src/main/java/github/mrornithorynque/bmh/` unless stated.

- `handlers/RespawnPositionHandler.java`: the distant respawn (distance formula, safe landing spot).
- `handlers/RespawnEffectHandler.java`: bell sound and "new journey" text on respawn.
- `utilities/BMHGameRules.java`: the six BMH game rules; `ALL_RULES` is what the world creation toggle flips.
- `mixin/CreateWorldGameTabMixin.java`: the "Balanced Hardcore: ON/OFF" button on the Create New World screen, registered in `common/src/main/resources/bmh.mixins.json` (both loaders). Fabric-only mixins are in `fabric/src/main/resources/bmh.fabric.mixins.json`. `forge/.../mixin/ForgeGuiMixin.java` is dead (body commented out, not registered).
- `client/LocalPlayerText.java` and `github/mrornithorynque/utilities/TextDrawer.java` (outside `bmh/`) draw on the client of the same JVM: on-screen text works in singleplayer and for a LAN host, not for other players.

## Changelog

`CHANGELOG.md` at the repo root, same file on every branch. Any change a player could notice adds a line under `## [Unreleased]` in the same commit, written for players (what changed in game, not which class). A release turns that section into the version's entry (see the `release` skill), and `publish.yml` posts exactly that entry as the changelog on Modrinth, CurseForge and the GitHub release.

## Releasing

Use the `release` skill (`.claude/skills/release/SKILL.md`). Short version: bump `mod_version` and the changelog on every branch, then push one `mc-<minecraft_version>-v<mod_version>` tag per version branch, one tag per `git push` (GitHub ignores pushes of more than three tags at once); `.github/workflows/publish.yml` publishes each tag to Modrinth and CurseForge, as two versions (Forge, Fabric) per tag.
