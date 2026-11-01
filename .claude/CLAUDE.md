# Balanced Minecraft Hardcore (BMH)

Forge mod (mod id `bmh`): death sends you on a distant respawn instead of ending the run, the distance grows with the difficulty and the Eternal Items you carry. Author: MrOrnithorynque.

## One branch per Minecraft version

The same mod ships for several Minecraft versions. Each version lives on its own long-lived branch. Everything under `src` is identical on all of them; `build.gradle`, `gradle.properties` and `README.md` differ per branch.

| Branch | Minecraft | Forge |
|---|---|---|
| `main` | 1.20.4 | 49.0.31 (same sources as `dev-49.0.31`) |
| `dev-49.0.31` | 1.20.4 | 49.0.31 |
| `dev-49.0.2` | 1.20.3 | 49.0.2 |
| `dev-48.1.0` | 1.20.2 | 48.1.0 |
| `dev-47.2.0` | 1.20.1 | 47.2.0 |
| `dev-46.0.14` | 1.20 | 46.0.14 |

The table can drift: `gradle.properties` on each branch (`minecraft_version`, `forge_version`) is the truth.

**Every change goes to every branch, never to `main` alone.** Commit on `main`, then cherry-pick onto each `dev-*` branch (scratch worktrees work well), build each one, and push them together. `build.gradle`, `gradle.properties` and `README.md` can conflict: keep the branch's own version of every line the change did not touch and apply only the change's hunks. Never take `main`'s whole file (`dev-46.0.14`'s `build.gradle` has a different layout). After porting, `git diff --stat main <branch> -- src ':!src/main/resources/META-INF/mods.toml'` must be empty for every branch (`dev-46.0.14`'s `mods.toml` only lacks a commented-out template block).

## Building

- JDK 17 is not on the PATH; use Homebrew's: `JAVA_HOME=/opt/homebrew/opt/openjdk@17`.
- `gradlew` has no exec bit in git: run `sh ./gradlew build` (or `runClient`).
- The release jar lands in `build/libs/`. It must contain `bmh.refmap.json` and a `MixinConfigs` manifest entry, or the mixins will not load outside the dev environment.
- `runClient` opens a plain `java` window that computer use cannot drive: ask the user to click through, then read `run/logs/latest.log` (or the redirected output) for `[BalancedMcHardcoreMain]` lines.

## Code map

Paths under `src/main/java/github/mrornithorynque/bmh/` unless stated.

- `handlers/RespawnPositionHandler.java`: the distant respawn (distance formula, safe landing spot).
- `handlers/RespawnEffectHandler.java`: bell sound and "new journey" text on respawn.
- `utilities/BMHGameRules.java`: the six BMH game rules; `ALL_RULES` is what the world creation toggle flips.
- `mixin/CreateWorldGameTabMixin.java`: the "Balanced Hardcore: ON/OFF" button on the Create New World screen. Mixins are registered in `src/main/resources/bmh.mixins.json` and wired by MixinGradle in `build.gradle`. `mixin/ForgeGuiMixin.java` is dead (body commented out, not registered).
- `github/mrornithorynque/utilities/TextDrawer.java` (outside `bmh/`) draws on the client of the same JVM: on-screen text works in singleplayer and for a LAN host, not for players of a dedicated server.

## Changelog

`CHANGELOG.md` at the repo root, same file on every branch. Any change a player could notice adds a line under `## [Unreleased]` in the same commit, written for players (what changed in game, not which class). A release turns that section into the version's entry (see the `release` skill), and `publish.yml` posts exactly that entry as the changelog on Modrinth, CurseForge and the GitHub release.

## Releasing

Use the `release` skill (`.claude/skills/release/SKILL.md`). Short version: bump `mod_version` and the changelog on every branch, then push one `mc-<minecraft_version>-v<mod_version>` tag per version branch, one tag per `git push` (GitHub ignores pushes of more than three tags at once); `.github/workflows/publish.yml` publishes each tag to Modrinth and CurseForge.
