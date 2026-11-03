# BMH redesign: master brief

Owner: Louis. Coordinator: the Claude session that wrote this file. Decided on 2026-11-03 with Louis; every design call
below is final for this run. Do not ask Louis anything: if a call is missing, make it, write it in your log with its
reason, and keep going. Only a real blocker (something you cannot build or decide) ends your turn with `BLOCKED:`.

Design page (for context, not a spec): `docs/redesign/assets/bmh-item-redesign.html`. Where the page and this brief
differ, this brief wins (the page shows an earlier version).

## Rules for every session

1. **Tree guard.** Every command and every sub-agent you run uses absolute paths and starts by checking
   `git rev-parse --show-toplevel` prints your own worktree. Never `cd` into a shared folder. Never touch
   `/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore` itself (Louis's checkout) or another session's worktree.
   The harness can reset your shell to Louis's checkout after a `cd`, so never run a relative command: always
   `sh <abs worktree>/gradlew -p <abs worktree> ...`, and absolute paths for `run/` folders and logs.
2. **Two sides.** The mod has one 1.20.x codebase (`main`, Minecraft 1.20.4) and a separate 1.21.1 codebase
   (`dev-52.1.20`, different sources: data components, registry enchantments, Java 21). Each part session does its part
   on BOTH:
   - main side: your branch `redesign/<name>`, cut from `origin/redesign/integration` (your session starts there);
   - 1.21.1 side: your branch `redesign/<name>-1.21`, in a second worktree you create with
     `git -C <your worktree> worktree add <your worktree>-121 -b redesign/<name>-1.21 origin/redesign/integration-1.21`.
   The five other 1.20.x branches (`dev-49.0.31`, `dev-49.0.2`, `dev-48.1.0`, `dev-47.2.0`, `dev-46.0.14`) are NOT
   yours: session G cherry-picks the main side onto them. This overrides the repo CLAUDE.md's "cherry-pick onto every
   branch yourself" rule for this run. Everything else in `.claude/CLAUDE.md` applies (read it first).
   - The app names your worktree's branch; rename it first: `git -C <abs worktree> branch -m redesign/<name>`.
   - Names: A `a-fixes`, B `b-totem`, C `c-amulet`, D `d-sky`, E `e-abilities`, F `f-sprites`, G `g`. So session C
     pushes `redesign/c-amulet` and `redesign/c-amulet-1.21`, logs to `docs/redesign/log-c-amulet.md`, and so on.
   - **Main-side code must also compile on Minecraft 1.20 to 1.20.3**, unchanged, because G cherry-picks it there. Before
     you use a vanilla API that may have moved, check it with `javap` against the Loom jars
     `~/.gradle/caches/fabric-loom/minecraftMaven/net/minecraft/minecraft-merged/<version>-*/*.jar` (1.20, 1.20.1,
     1.20.2 and 1.20.4 are there). Known trap: `SavedData.Factory` does not exist before 1.20.2.
3. **Build and boot.** JDK 17 on the main side (`JAVA_HOME=/opt/homebrew/opt/openjdk@17`), JDK 21 on 1.21.1
   (`/opt/homebrew/opt/openjdk@21`). `sh ./gradlew build --max-workers=2`, one Gradle build at a time per session. Both
   sides must build with zero errors before you report. Boot a dedicated server on each loader of each side
   (`:forge:runServer`, `:fabric:runServer`, `:neoforge:runServer`; NeoForge exists on both sides here) to prove
   registration and the client/server split: a client class loaded on a dedicated server fails Forge at startup.
   - Use your own port: put `server-port=<yours>` in `<loader>/run/server.properties` (A 25611, B 25612, C 25613,
     D 25614, E 25615, G 25617) and `eula=true` in `<loader>/run/eula.txt`.
   - The Mac has 24 GB and every Gradle run asks for 3 GB: **at most two Gradle runs machine-wide.** Take a slot with
     `mkdir /private/tmp/bmh-gradle.1` (or `.2` if 1 is taken), write `<name> <pid> <epoch>` into a file inside it,
     remove the slot when Gradle ends. If both are taken, do other work and retry.
   - Server boots are serialized machine-wide. Build first, then take `mkdir /private/tmp/bmh-runserver.lock`, write
     `<name> <pid> <epoch> <port>` into `owner` inside it, boot with a 5-minute timeout, stop the server once it logs
     `Done`, `rm -r` the lock. Anyone may clear a lock older than 20 minutes whose port is closed. Never leave it.
   - Forge and NeoForge dev servers get no console input, so `stop` and console commands do not work. Set
     `enable-rcon=true`, `rcon.port=<your port + 100>`, `rcon.password=bmh` in `server.properties` and use a tiny RCON
     client (a few lines of Python) for commands and `stop`; otherwise kill only `lsof -ti tcp:<your port>`.
   - Known harmless red on 1.21.1 boots: `Couldn't parse element minecraft:loot_table/bmh:blocks/test_block`. Do not chase it.
   - Redirect Gradle and server output to files in your worktree, read only the lines you need.
4. **No game client.** Nobody can drive `runClient` here. Write every check that needs eyes in the game into
   `docs/redesign/checklist-<name>.md` on your main-side branch: numbered steps, what Louis should see, commands to give
   himself items (`/give @p bmh:...`). Session G merges them into one checklist for Louis.
5. **Durable log.** Keep `docs/redesign/log-<name>.md` on your main-side branch from your first step: what you did, the
   calls you made and why, the commands that verified it, and a `Resume here` line, updated after every milestone and
   pushed. A fresh session must be able to resume from it alone.
6. **Git.** Push only your own branches (`redesign/<name>`, `redesign/<name>-1.21`). Never push, merge into, or reset
   `main`, any `dev-*` branch, or the `redesign/integration*` branches. Before your final report, merge
   `origin/redesign/integration` into your main-side branch and `origin/redesign/integration-1.21` into your 1.21 branch,
   rebuild, push. Commit messages: what changed in game, end with your harness's attribution lines.
   G merges each part as soon as it reports, so `origin/redesign/integration` moves during the run: merge it whenever
   you start a new piece. Before reporting, run `git merge-tree --write-tree HEAD origin/redesign/<sibling>` against
   every sibling branch already pushed and fix on your side whatever conflicts you can avoid.
7. **Changelog and lang.** Player-facing lines go under `## [Unreleased]` in `CHANGELOG.md` on both sides. New strings
   go through `assets/bmh/lang/en_us.json`; add your keys as one contiguous block (others add theirs; G merges).
8. **Shared files** (expect merge conflicts; G resolves them): `ItemInit.java`, `en_us.json`, `bmh.mixins.json`, the
   loaders' client setup classes (`forge/.../ForgeClientEvents.java`, `fabric/.../BalancedMcHardcoreFabricClient.java`,
   `neoforge/.../NeoForgeClientEvents.java`), the loaders' event classes (`ForgeEvents`, `NeoForgeEvents`,
   `BalancedMcHardcoreFabric`), `bmh.fabric.mixins.json`, `CHANGELOG.md`. Keep your edits there additive and small: put
   your logic in new classes of your own and add one registration line to the shared file. Every new mixin class name
   starts with your part letter (`A_LivingEntityTotemMixin`, `E_PlayerAttackMixin`), so two parts never create the same
   class. Item class
   ownership: B owns the Totem of Eternity, the eternal tool tier and every tooltip on eternal items; E adds behaviour in
   its own handler classes and does not edit the item classes' tooltips; C owns `NavigatorAmuletItem`.
9. **Usage guard.** Call `get_usage` after each finished piece. If the weekly all-models window is above 60%: finish the
   piece, push, write `Resume here` in your log, end your turn with `PAUSED`.
10. **Sub-agents.** At most 2 at once. Run audits in the foreground (a background agent's report would reach the
    coordinator, not you). A read-only agent is read-only outside the filesystem too: no servers, no network writes.
11. **Report.** End your final turn with this first line, nothing before it:
    `DONE redesign/<name> <sha> | redesign/<name>-1.21 <sha>` (or `BLOCKED: <question>`, or `PAUSED <branches> <shas>`).
    Then at most 10 lines: what changed, what you verified (builds, server boots per loader and side), what is left for
    Louis's checklist, and a `Learnings:` line. Never message the coordinator; ending your turn is the report.
12. **Vanilla facts.** Check behaviour in the decompiled sources before relying on it: 1.21.1 in
    `/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/neoforge/build/moddev/artifacts/neoforge-21.1.265-sources.jar`,
    1.20.4 in whichever `/Users/louisobenicherousse/.gradle/caches/neoformruntime/intermediate_results/sourcesAndCompiledWithNeoForge_*_output.jar`
    has `VERSION_STRING = "1.20.4"` in `net/minecraft/SharedConstants.java` (read with `unzip -p`). Your own worktree
    gains its own after its first NeoForge build. These jars are NeoForge-patched: Forge and NeoForge move some vanilla
    code (for example `Player.attack` resets attack strength first in vanilla, last in Forge and NeoForge). Fabric runs
    plain vanilla: check Fabric behaviour in the Loom jars of rule 2.
13. **Hooks.** Prefer one common mixin over three loader hooks when it works on all three; this overrides line 17 of
    the `add-mixin` skill for this run (use the skill for everything else). Plain Mixin only: no Forge jar here ships
    MixinExtras, so no `@Local`, `@WrapOperation` or `LocalCapture`. Forge and NeoForge client setup
    (`FMLClientSetupEvent`, item properties) is a **mod-bus** event: on main, `ForgeClientEvents` is registered on the
    game bus, where `FMLClientSetupEvent` never fires. Register client setup with `modEventBus.addListener(...)` in the
    `@Mod` constructor behind `FMLEnvironment.dist.isClient()`, item properties inside `enqueueWork` (part A creates this
    listener on both sides, part C adds its line to it). Fabric item properties: `FabricModelPredicateProviderRegistry`.
    Never register a game-bus event (a client command, a tick) in a class registered on the mod bus: Forge 1.21.1
    rejects it at startup, and a dedicated server never shows it.

## The parts

### A · Fixes and rule tweaks (model: Sonnet)

1. **1.21.1 only: Eternal tools can take enchantments again.** Since 1.21 an enchantment applies only to items in its
   supported-items tags, and the mod ships no tags. Add `bmh:eternal_sword` to `data/minecraft/tags/item/swords.json`,
   `bmh:eternal_axe` to `axes.json`, `bmh:eternal_pickaxe` to `pickaxes.json` (`"replace": false`). Check in the
   1.21.1 data that `#minecraft:enchantable/*` reach those tags. Verify on a dedicated server per loader, over RCON:
   `summon zombie` holding the eternal sword, then `enchant @e[type=zombie,limit=1] minecraft:mending` must succeed
   (it fails before the fix).
2. **All sides: the Eternal Bundle's filled texture shows.** The model switches on the `filled` item property, but
   vanilla registers `filled` for `Items.BUNDLE` only. Register it for `bmh:eternal_bundle` on each loader's client,
   reading `EternalBundleItem.getFullnessDisplay`. Client-only code behind the existing client hooks.
3. **All sides: anchor exile at world scale.** In `RespawnPositionHandler`, divide the exile distance by the respawn
   level's `dimensionType().coordinateScale()` (8 in the Nether, 1 elsewhere), so an anchor exile is the same distance
   in world terms as an Overworld one. Log the scaled distance.
4. **All sides: Totem of Undying nerf.** When a totem saves a player, keep its Regeneration and Absorption, remove its
   Fire Resistance (Louis's "invincible effect"). Confirm in the sources first what vanilla applies
   (`LivingEntity.checkTotemDeathProtection`). Players only.
5. Changelog lines for all four.

### B · Totem of Eternity, recipes, tooltips (model: Sonnet)

1. **The Totem of Eternity is made in steps.**
   - New item `bmh:dormant_totem` ("Dormant Totem of Eternity"): stack 1, fire resistant, rarity rare, NOT an
     `IEternalItem` (it drops on death like any item). Its recipe is the old Totem of Eternity recipe, unchanged
     (pattern `nAn`, `DET`, ` n `), now producing the dormant totem.
   - New item `bmh:echo_heart` ("Echo Heart"): stack 1, fire resistant, rarity rare. Recipe `CEC`, `ESE`, `CEC`
     (C crying obsidian, E echo shard, S nether star).
   - **Anvil:** dormant totem in the left slot + echo heart in the right slot gives `bmh:totem_of_eternity`, cost 39
     levels (the most a survival anvil allows: 40 and above is "Too Expensive"), consuming both. Name field ignored.
   - The awakened Totem of Eternity keeps everything it has today (eternal, one per player pickup rule, repair role
     replaced below) and gains the enchantment glint (`isFoil`) and a tooltip line `Eternity` styled like an
     enchantment (gray). It is not a real enchantment: the item itself is what survives death, so a grindstone cannot
     strip it. Remove the old `totem_of_eternity.json` crafting recipe; keep `totem_of_eternity_blasting.json` untouched.
2. **Eternal tools at the smithing table.** `smithing_transform` recipes: template `bmh:totem_of_eternity`, base
   `minecraft:netherite_sword` / `_axe` / `_pickaxe`, addition `minecraft:echo_shard`, result the eternal tool. Smithing
   copies the base item's data, so enchantments, name and wear carry over (checked in the 1.20.4 and 1.21.1 sources).
   Delete the old crafting recipes `eternal_sword.json`, `eternal_axe.json`, `eternal_pickaxe.json` (the dragon egg
   goes with them). Recipe JSON formats differ between the sides (`item` vs `id` in results): write each side's own.
3. **Eternal Bundle recipe:** `SGS`, `LTL`, `LNL` (S string, G gold ingot, L leather, N netherite scrap, T
   `bmh:totem_of_eternity`). Replace `eternal_bundle.json`. Keep `bundle.json` (the vanilla bundle recipe) untouched.
4. **Repair:** the eternal tier's repair ingredient becomes `minecraft:gold_block` (was the Totem of Eternity).
5. **Tooltips on every Eternal Item:** a gold line `Heavy with eternity: you wake farther while you carry it`, and on
   the three tools a gray line `Each death: -7% durability`. Static text (a client cannot read the server's config).
6. Models for the two new items. Use placeholder textures (copies of the current totem texture, first frame) named
   `dormant_totem.png` and `echo_heart.png`; session F delivers the final ones under the same names.
7. Changelog lines.

### C · Navigator's Amulet mechanics (model: Sonnet)

1. **Needle points north.** The amulet becomes a compass: a compass-style `angle` item property (like vanilla's
   compass, `CompassItemPropertyFunction` with a target lambda) whose target is always due north of the holder, on each
   loader's client. Overworld only: in other dimensions the target is null and the needle spins, as vanilla's does. The
   item model gets 32 overrides like vanilla's `compass.json`. Textures: `docs/redesign/assets/amulet/navigator_amulet_00.png`
   to `_31.png` (index i = needle at 360*i/32 degrees clockwise from screen-up) plus `navigator_amulet.png`. Vanilla's
   `compass.json` maps angle 0 to `compass_16`, its needle-up frame, while our `_00` is needle-up: override k uses
   `navigator_amulet_{(vanilla index - 16) mod 32}`. Copying the names 1:1 would point the needle south. Delete the
   old `navigator_amulet.png.mcmeta`.
2. **Home.** Sneak + use the amulet on a bed or a lodestone in the Overworld sets that block as the player's Home
   (action bar confirmation, a chime). Store it on the player, in their saved NBT, through a common mixin on
   `ServerPlayer` `addAdditionalSaveData` / `readAdditionalSaveData` plus `restoreFrom` (copied across death); those
   signatures are the same from 1.20 to 1.21.1, unlike `SavedData`. Until a player sets one, Home is their respawn point
   if it is in the Overworld, else world spawn.
3. **Star reading (use).** Server side. Works only in the Overworld, at night (the level's star brightness up, i.e.
   `isNight()`), not raining, with open sky above the player's head. Otherwise the action bar says the stars are hidden
   and nothing is spent. A reading:
   - spends one charge, plays an amethyst chime, writes `X <x> · Y <y> · Z <z>` to the action bar (this also fixes
     today's dedicated-server bug, where the on-screen text never shows);
   - writes the reading contract below into the amulet stack, which session D's sky renderer reads;
   - never shows Home's coordinates or distance, and never the exile radius.
   Charges: 3, stored on the stack, refilled to 3 at each nightfall (track the last night index on the stack). Show them
   in the tooltip (`Starlight 2/3`) and as a violet item bar. Short cooldown (2 s). The amulet has no durability any
   more: remove `.durability(10)` and `LocalPlayerText` use; since it can no longer take Unbreaking, simplify or remove
   its enchanting hooks (`canApplyAtEnchantingTable`, `supportsEnchantment`, the amulet branch of
   `mixin/EnchantmentMixin.java` on 1.21.1) and document what you did.
4. **Recipe:** ` S `, `CKC`, ` Y ` (S string, C copper ingot, K compass, Y spyglass). Replace `navigator_amulet.json`.
5. **Reading contract** (C writes, D reads; keep the names exact):
   - main side: the stack's NBT tag (`getOrCreateTag()`), compound `bmh_reading` with `azimuth` (float, degrees
     clockwise from north, from the player to Home) and `time` (long, `level.getGameTime()` of the reading);
   - 1.21.1 side: the same compound inside `DataComponents.CUSTOM_DATA`.
   Charges live next to it as `bmh_starlight` (int) and `bmh_night` (long).
   Azimuth convention (C and D both use it): `azimuth = toDegrees(atan2(homeX - playerX, playerZ - homeZ))`, taken
   mod 360, so 0 is north (-Z) and 90 is east (+X). Both log a self-check: Home at (0, -100) seen from (0, 0) gives 0,
   Home at (100, 0) gives 90.
6. Changelog lines, lang keys for every message.

### D · Sky constellation (model: Opus)

Client only, both sides. When the local player holds a Navigator's Amulet (either hand) whose `bmh_reading.time` is one
this client has not shown yet and is less than 40 ticks old, draw lines through real stars toward
`bmh_reading.azimuth`:

1. Inject right after vanilla draws the stars in `LevelRenderer.renderSky` (a common client mixin, in the `client` list
   of `bmh.mixins.json`), so your geometry shares the star field's transform. On 1.20.x the stars are drawn with the
   method's `PoseStack` argument; on 1.21.1 with a local `PoseStack` you cannot capture (no MixinExtras, rule 13), so
   rebuild that pose yourself: the `frustumMatrix` argument, then `YP(-90)`, then `XP(getTimeOfDay * 360)`. Replicate vanilla's star field exactly
   (seed 10842, 1500 tries, the same filter and scale as `drawStars`; read both versions' code) to know where every real
   star is.
2. Pick 5 to 7 stars forming a chain from high in the sky down toward the horizon in the azimuth's direction, as seen at
   the moment of the reading (convert with the current sky rotation), within about 10 degrees of the vertical plane
   through the azimuth, preferring bigger stars, roughly evenly spaced. The last star, nearest the horizon, marks the
   direction.
3. Animate: trace one segment at a time (about 0.3 s each), brighten the chain's stars, hold about 8 s, fade over 2 s.
   Amethyst-white lines (core `#f0e4ff`, edge `#b58cff`), a little wider than a star. Multiply alpha by
   `(1 - rainLevel)` and by the star brightness, like vanilla's stars. No text, no distance. Only this player sees it.
4. Build against the contract without waiting for C: a development trigger (for example a client command firing a
   reading at a given azimuth), disabled or removed before you report DONE.
5. Before any in-game look: render the chosen chains offline (a small script or test that reuses your star code) for
   azimuths 0, 90, 180, 270 at midnight and at a late-night time, as PNGs in `docs/redesign/sky-previews/`, and list
   them in your report. Note in your log how it behaves with shader mods (Iris/Oculus may replace sky rendering).
6. The two sides render differently (1.21.1 changed `Tesselator`/`BufferBuilder`); port by hand. Boot dedicated servers
   on every loader on both sides: none of your classes may load there.

### E · Echo abilities (model: Sonnet)

Server side, in new handler classes (do not edit the eternal items' tooltips; B owns them). Hook through one common
mixin where it works on all loaders, else loader events, per side.

1. **Sword, Echo Strike.** Read the attack strength at the HEAD of `Player.attack` (vanilla, so Fabric, resets it
   before damage is dealt; Forge and NeoForge reset it last), then: a hit with the Eternal Sword at full strength
   (attack strength 0.9 or more) echoes about 5
   ticks later onto the nearest other living enemy within 5 blocks of the target (not the attacker, not players, not
   the attacker's tamed animals, not villagers, not armor stands): half the hit's damage, as the attacker's damage, with
   a cyan particle line between the two. The echo never echoes. It targets another mob on purpose: a second hit on the
   same target would land inside its invulnerability frames and add nothing.
2. **Axe, smash.** No tree felling. A hit with the Eternal Axe while falling (fall distance above 1.5, not on ground,
   not gliding, not in water or climbing) multiplies the hit's damage by `1 + vertical speed` (blocks per tick), capped
   at x3. The server does not track a player's velocity reliably on 1.20.x, so derive the speed from the fall:
   `speed = sqrt(0.16 * fallDistance)` (free fall under 0.08 blocks/tick^2 gravity). After a smash, halve the player's
   fall distance (about half the fall damage). Sound and particles on a smash.
   Base damage stays 18. Write the expected damage for 3, 10 and 30-block falls in your log.
3. **Pickaxe, Echo Sense.** Breaking an ore with the Eternal Pickaxe makes the same ore within 8 blocks glow through
   walls for 5 seconds: spawn glowing block-display entities (1.19.4+, so every branch has them) with a cyan glow
   override, at most 64, removed after 100 ticks. "Ore" is a new tag `bmh:echo_sense_ores` listing the vanilla ore tags
   (coal, copper, iron, gold, redstone, lapis, diamond, emerald) plus nether quartz ore, nether gold ore and ancient
   debris; "same ore" means the same vanilla ore tag, so deepslate and stone variants light up together.
   - The tag lives in `data/bmh/tags/blocks/` on the main side and `data/bmh/tags/block/` on 1.21.1: the wrong folder
     gives an empty tag and no error, so check it loads (log its size once).
   - Display entities are saved with their chunk: a player who quits within 5 s would leave the ore glowing forever.
     Tag each display `bmh_echo` with its expiry game time, and discard expired ones in a common mixin on `Display.tick`
     (which also catches displays reloaded from disk).
   - `Display.BlockDisplay.setBlockState` and `Display.setGlowColorOverride` are private on every version: use an
     `@Invoker` accessor mixin.
4. Changelog lines, lang keys.

### F · Sprites (model: Opus)

16x16 textures, on both sides (same files). Work from Louis's own originals in
`common/src/main/resources/assets/bmh/textures/item/`. Louis rejected a full repaint of the tools and the bundle: keep
his style, change as little as possible.

1. **Eternal Bundle** (`eternal_bundle.png`, `eternal_bundle_filled.png`): his originals plus one gold accent, a small
   gold ring charm (the "eternity ring" motif) hanging from the drawstring. Nothing else changes.
2. **Dormant Totem** (`dormant_totem.png`): his Totem of Eternity (first frame of its 8-frame strip), asleep: gems and
   eyes unlit, no sparkle, static.
3. **Echo Heart** (`echo_heart.png`): new. A heart-shaped crystal holding a Nether Star glint in echo cyan, with
   crying-obsidian violet facets. At 1x it must read as a precious crafted heart, not as a vanilla echo shard or nether
   star.
4. Iterate visually: render review sheets (1x, 2x, 4x on an inventory-grey slot, plus dark and light grounds), then a
   blind read by a fresh sub-agent in the foreground before you report. A small toolkit is in
   `docs/redesign/assets/px/px.py`. Commit the sheets to `docs/redesign/sprites/`.
5. No code, no models (B owns the models). No servers to boot.

### G · Integration and ports (model: Sonnet, starts parked)

Read this brief, set up your worktree (branch `redesign/g`) and a second one for the 1.21.1 side (branch
`redesign/g-1.21` from `origin/redesign/integration-1.21`), then end your turn with `READY redesign/g <sha>` and wait.
The coordinator messages you each time a part reports DONE, then once more for the final port. The
`redesign/integration*` branches are checked out in the coordinator's worktrees, so never check them out: work on
`redesign/g` / `redesign/g-1.21` and publish with `git push origin HEAD:redesign/integration` (fast-forward only).

1. On each "merge <part>" message: fetch, merge that part's main-side branch into `redesign/g` and its `-1.21` branch
   into `redesign/g-1.21` with `--no-ff`, resolve the shared files (rule 8; where two parts add the same texture path,
   F's file wins), build both sides, push both to `origin/redesign/integration` and `origin/redesign/integration-1.21`,
   report `MERGED <part> <sha> | <sha-1.21>`, and wait again.
2. On the "port" message: port the main side to the five other 1.20.x branches. For each, a branch
   `redesign/port-<dev branch>` from `origin/<dev branch>`, then
   `git cherry-pick -m 1 $(git rev-list --reverse --first-parent origin/main..origin/redesign/integration)`, following
   the repo CLAUDE.md's rules for `forge/build.gradle`, `gradle.properties`, `README.md` and `mods.toml`. For the
   empty-diff check of the repo CLAUDE.md, diff against `origin/redesign/integration`, not `main`.
3. Build all seven with the right JDK; boot a dedicated server on every loader each branch has (NeoForge only where the
   branch builds it; on 1.20.1 note that NeoForge runs the Forge jar).
4. Merge the parts' `checklist-*.md` into one `docs/redesign/CHECKLIST.md`, ordered for a single `runClient` session per
   loader.
5. Push only `redesign/*` branches. Report `DONE` with every branch and sha, the build and boot results per branch and
   loader, and the conflicts you resolved.

The release (version bump, tags, pushing `main` and `dev-*`) is not part of this run: it waits for Louis's word.
