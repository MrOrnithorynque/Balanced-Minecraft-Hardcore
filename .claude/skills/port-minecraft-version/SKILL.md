---
name: port-minecraft-version
description: Port Balanced Minecraft Hardcore to a new Minecraft version (a new version branch, or a version jump such as 1.21.1 to 26.1): measure the jump, pin the toolchain, split the code port, check every mixin target against the loaders' patches, and prove the jars on production servers and with an old world. Use when asked to port to, support or add a new Minecraft version, or to backport to an older one.
---

# Porting BMH to a new Minecraft version

The method that took `mc-1.21.1` to `mc-26.1` (2026-11-06). Its full record, with the evidence of every claim, is `docs/port-26/LOG.md` on `mc-26.1`: read it first, it also lists the measured steps for 26.2, 26.3 and a 1.21.11 backport. Project rules are in `.claude/CLAUDE.md` (one branch per version, every change on every branch, the changelog).

## 1. Measure before you code

- Start from the newest branch with the closest code (`mc-26.1` for 26.x, for example). Name the new branch `mc-<minecraft version>`.
- `bash .claude/skills/verify-loaders/scripts/compile_check.sh <version>` compiles the common code against that version. On 26.x it downloads the Mojang client jar (unobfuscated, official names) and needs no Loom; an obfuscated version (1.21.11 and older) needs one Gradle build of a branch on that version first, for its Loom jar. Count the errors per file: that is the size of the jump. javac stops some checks after resolution errors, so the count is a lower bound, and it never sees mixin target strings.
- Facts to pin, each with its source (Maven metadata, the Mojang version JSON): the Java version (`javaVersion.majorVersion`), whether the version JSON still has `client_mappings` (no mappings means no refmap anywhere), the Forge, NeoForge, Fabric Loader and Fabric API versions, and the Gradle plugins. The best references: jaredlll08/MultiLoader-Template has one branch per Minecraft version (Loom and ModDevGradle versions, `neoFormVersion` for common) and the Forge MDK at `MinecraftForge/MinecraftForge` tag `<mc>` (`mdk/build.gradle`: the ForgeGradle range, the Gradle wrapper). `gh api repos/<owner>/<repo>/contents/<path>?ref=<branch>` reads any of them.
- Write the measurement into a log in the repo (`docs/port-<version>/LOG.md`) before coding.

## 2. References on disk

- Decompile the client jar with Vineflower (`java -jar ~/.gradle/caches/modules-2/files-2.1/org.vineflower/vineflower/<v>/*/vineflower-<v>.jar -only=net/minecraft <client.jar> <out dir>`, about five minutes). Every API answer comes from there, not from memory.
- The loaders' patches decide whether a mixin still hits: `gh api repos/MinecraftForge/MinecraftForge/contents/patches/minecraft/net/minecraft/<path>.java.patch?ref=<mc>` and `gh api repos/neoforged/NeoForge/contents/patches/net/minecraft/<path>.java.patch?ref=<mc>.x` (base64 in `.content`). After one Gradle configure, the full patched sources sit in the Gradle cache (see the `add-mixin` skill).

## 3. Build first, then code

- Change the build until `sh ./gradlew projects` configures all three loaders in one build. The root `build.gradle` declares every plugin once, so they share a classloader: check that first, nothing else matters if it fails.
- Split the code port by area to agents when it is large (the 26.1 port: client rendering; items with their models and data; the coordinator kept game rules, handlers, server mixins and the loader modules). Each agent: absolute paths, a tree guard, its own files only, no Gradle, no commits, a running log in `docs/port-<version>/`, a compile loop with its own output folder. Mechanical renames across the whole tree go first, done once by the coordinator, so agents do not trample each other.
- For every mixin: find its target in the decompiled source AND in both loaders' patches, with the exact descriptor. A boot proves only the classes it loads; the self-tests and bots cover the rest.

## 4. Prove it

In this order, each step on every loader:
1. `sh ./gradlew build`, then `jarcheck.py` (extend its version rules first: refmap expected or not, data folders, item model files).
2. A dedicated server boot per loader. The first boot of 26.1 died on a removed field (`spawnInvulnerableTime`): read the first error of every loader.
3. A real death in a client (`death_client.sh`): the far position right after the respawn packet.
4. The verify-loaders matrix (bots, clients, self-tests). Port the hooks first (`MockPlayers_<variant>.java`, `hook/compat/ClientCompat_<variant>.java`), and teach the bots the version (`bot_version`, `rule_names.js` when rules or commands change). Check that mineflayer's minecraft-data knows the protocol (`dataPaths.json`); without it no bot can join.
5. `prod_server.sh` on every loader version the jar claims (each patch release of the Minecraft line, each Forge major): the shipped jar, outside the dev environment.
6. `cross_upgrade.sh` from the previous branch: an old world, its items, points and game rules.

## Traps seen on the 26.1 port

- zsh does not split `$c` in `for c in "a b c"; do bash x.sh $c`: put such loops in a bash script (all nine production runs once booted nothing because of it).
- A removed field behind an `@Accessor` crashes every loader at boot, and javac does not see it.
- `optionalFieldOf(name, default)` leaves out a value equal to the default: a save format must write every field.
- NeoForge 26.1.0 ships `sponge-mixin` 0.17.0 (`@ModifyArg.at` is one `@At`), 26.1.2 ships 0.17.4 (`At[]`): the jar built on 26.1.2 still runs on 26.1.0 (proved on a production server), but check the class files of builds against each loader version (`unzip` both jars, `diff -r`) and read any difference.
- Fabric API 26.x refuses vanilla clients when a mod adds to a synced registry: the bots need the verify hook (`lib.sh fabric_bot_hook`).
- Vanilla's data fixers rename only vanilla data: BMH's own saved names need a migration (`N_SavedDataGameRulesMixin` for the game rules of 26.1).
- ForgeGradle 7 does not add `-XstartOnFirstThread` to the macOS client run: `forge/build.gradle` must.
- A new branch keeps the README title of the branch it came from: fix line 2 before the first release.

## 5. Finish

- Changelog lines for what a player can notice, in Simplified Technical English (`.claude/CLAUDE.md`), plus the version in `README.md` (availability line) and the CurseForge listing (`release_game_versions` in `gradle.properties` when one jar covers several versions).
- The harness changes (`.claude/`) and `CHANGELOG.md` go to every branch: apply the same diff on each, check `compile_check.sh --selftest` and `--client` on each branch's versions, and keep the 1.20.x branches equal to `main` outside their own files.
- Add the branch to `.claude/CLAUDE.md` (table and a section for what differs) and to `scripts/branches.txt`, then release with the `release` skill.
