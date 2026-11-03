# Log: part C, Navigator's Amulet mechanics (session `c-amulet`)

Two sides, two worktrees:

| Side | Branch | Worktree | Cut from |
|---|---|---|---|
| main (1.20.4 sources, JDK 17) | `redesign/c-amulet` | `/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/c-amulet-675a6c` | `origin/redesign/integration` (432d0a2) |
| 1.21.1 (JDK 21) | `redesign/c-amulet-1.21` | `/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/c-amulet-675a6c-121` | `origin/redesign/integration-1.21` (a886d42) |

Brief: `docs/redesign/BRIEF.md`, part C. Server port 25613, RCON 25713, password `bmh`. Steps are numbered; the only clock times
here were read from `date`.

## Resume here

DONE. Both sides implemented, audited, fixed, built at the final commits (main `8687369` plus docs, 1.21.1 `660976a`), boot-tested on
every loader (6 of 6, 28 of 28 checks each) and pushed. `origin/redesign/integration*` had not moved at the last fetch (no part merged
yet), so the merge of integration into my branches was a no-op; if a part has been merged since, merge it, rebuild, and move my three
loader registrations into part A's `ClientItemProperties.registerAll` (optional cleanup, G can do it). Left for Louis:
`checklist-c-amulet.md` (everything the client draws). `get_usage` at the end of the run: see the report.

## Steps

1. Tree guard OK, branch renamed `claude/c-amulet-675a6c` to `redesign/c-amulet`, brief and repo CLAUDE.md read. Second worktree
   created with `git worktree add ...-121 -b redesign/c-amulet-1.21 origin/redesign/integration-1.21`, upstream unset (a push goes
   only with an explicit refspec).
2. Vanilla facts checked in the decompiled sources (1.20.4 and 1.21.1) and with `javap` on the Loom jars (1.20, 1.20.1, 1.20.2,
   1.20.3, 1.20.4, 1.21.1): `CompassItemPropertyFunction` (public constructor taking a `CompassTarget`, the target returns a
   nullable `GlobalPos`), `ItemProperties.register` (the `ClampedItemPropertyFunction` overload is private in vanilla, the
   `ItemPropertyFunction` one is public in the Forge and NeoForge patches, Fabric has `FabricModelPredicateProviderRegistry`),
   `Level.isNight`/`isRaining`, `canSeeSky`, `ServerPlayer.addAdditionalSaveData`/`readAdditionalSaveData`/`restoreFrom`, `Item.useOn`/`use`/
   bar methods/`inventoryTick`/`appendHoverText` (1.21.1 takes an `Item.TooltipContext`).
3. Vanilla's `compass.json` is byte-identical in 1.20.4 and 1.21.1: 33 overrides (angle 0.0, then 0.015625 + k * 0.03125, the last
   0.984375 wraps back to the first frame). The amulet's model is generated from it with the shift of the brief
   (`gen_models.py`, scratchpad): frame = (vanilla index - 16) mod 32; plain `item/compass` is `compass_16`, so it becomes `_00`.
   I rendered the 32 given textures on a contact sheet: red tip up in `_00`, right in `_08`, down in `_16`, left in `_24`, as the
   brief says.
4. Main side written: `NavigatorAmuletItem` (rewritten), `handlers/AmuletReading`, `handlers/AmuletHome`, `mixin/C_ServerPlayerMixin`,
   `client/AmuletCompass`, `forge/ForgeAmuletClient`, `neoforge/NeoForgeAmuletClient`, one line in `BalancedMcHardcoreFabricClient`;
   resources: 33 textures, 33 models, recipe, 12 lang keys, changelog; `.durability(10)` removed; Fabric `EnchantmentMixin` and the
   old `navigator_amulet.png.mcmeta` deleted.
5. 1.21.1 side ported by hand (custom data instead of NBT tags, `ResourceLocation.withDefaultNamespace`, `Item.TooltipContext`,
   recipe `id`), and its common `EnchantmentMixin` deleted (it only existed for the amulet).
6. Main build: `sh gradlew build --max-workers=2` with JDK 17: `BUILD SUCCESSFUL in 3m 44s` (log in the worktree's `build/logs/`).
7. Main jars: the Fabric and Forge refmaps resolve the three injection targets of `C_ServerPlayerMixin`, the Forge manifest has
   `MixinConfigs: bmh.mixins.json`, NeoForge has no refmap and lists the config in `mods.toml`. `javap -v` on the built Forge classes:
   only `AmuletCompass` and `ForgeAmuletClient` reference `net/minecraft/client`; the item, `AmuletReading`, `AmuletHome`, the mixin
   and `BalancedMcHardcoreMain` do not.
8. Compatibility with 1.20 to 1.20.3 (rule 2): the five common classes of the part compile unchanged with plain `javac` against the
   Loom merged jars of 1.20, 1.20.1, 1.20.2, 1.20.3 and 1.20.4 (`compile_check.sh`, scratchpad). The loader classes
   (`ForgeAmuletClient` etc.) only use APIs that exist on every 1.20.x Forge, and G's port builds will prove them.
9. 1.21.1 build: `BUILD SUCCESSFUL in 3m 43s` (JDK 21).
10. Server boots with a scripted vanilla client (the scripts are kept in `docs/redesign/c-amulet-tools/`, with a README).
    `boot_server.sh` takes a Gradle slot and the server lock, writes
    `server.properties` (flat world, peaceful, port 25613, RCON 25713) and boots `:<loader>:runServer`; once the log says `Done` it runs
    `amulet_scenario.js`, a mineflayer 4.39 bot (`online-mode=false`) that sends only "use item" and "use item on
    block" packets and reads the action bar, with RCON (`rcon.py`) for `give`, `time`, `weather`, `setblock`, `tp`, `kill` and `data get
    entity`. It checks: a reading by day is refused with nothing written; at midnight under open sky a reading shows
    `bmh.amulet.reading` with the player's block coordinates, spends one charge and writes `bmh_reading`, `bmh_starlight`, `bmh_night`;
    a second use within 2 s does nothing; 3 readings give 0 and the 4th says `bmh.amulet.empty`; `time add 24000` refills to 3 with night
    2 (inventoryTick); rain and a roof refuse; sneak + use on a lodestone and on a bed sets Home (`bmh_home` x, y, z of the block);
    in creative the azimuth from south, west, north and east of Home is exactly 0, 90, 180 and 270; Home survives `kill` (restoreFrom)
    and a relog (save and read); in the Nether the stars refuse. A vanilla client can join the Forge and Fabric dev servers.
    The default Home was checked numerically too: a new player at (9.5, -1.5) got azimuth 257.4712 on Fabric 1.21.1, which is
    atan2(-9, -2) toward the world spawn (0.5, 0.5).

    | Side | Loader | Boot | Scenario | Notes |
    |---|---|---|---|---|
    | 1.20.4 | Fabric | `Done` after 160 s | 28 of 28 | |
    | 1.20.4 | Forge | 195 s | 28 of 28 | debug log: `Mixing C_ServerPlayerMixin from bmh.mixins.json into net.minecraft.server.level.ServerPlayer` |
    | 1.20.4 | NeoForge | 207 s | 28 of 28 | |
    | 1.21.1 | Forge | 156 s | 28 of 28 | same `Mixing C_ServerPlayerMixin` debug line |
    | 1.21.1 | NeoForge | 160 s | 28 of 28 | |
    | 1.21.1 | Fabric | 174 s | 28 of 28 | its first run (106 s) was 27 of 28: the "empty" check lost a race in my script (the server answers in a few ms, before my wait window started); fixed in the script, not in the mod |

    Every boot logs the item's self-check: `Azimuth self-check: Home (0, -100) seen from (0, 0) gives 0.0 (expected 0), Home (100, 0)
    gives 90.0 (expected 90)`. The needle was also checked by a simulation (`needle_sim.py`): vanilla's angle for a non-local holder
    in every yaw, the model overrides resolved the way `ItemOverrides` does, and the frame's needle direction: worst error 5.6 degrees
    (half a frame is 5.625), north gives frame 00, east 24, south 16, west 08.
11. Audit. A fresh read-only sub-agent was briefed to refute 11 claims (click flow, charges, needle, Home, dedicated server safety,
    1.21.1 port, compatibility, spec) with citations, and its 5 million azimuth samples, bytecode comparisons across 1.20 to 1.21.1
    and constant pool scan of the six jars all came back clean. One real flaw, read myself in `Minecraft.startUseItem` before acting:
    see "Client side of a click" below (fixed on both sides). Left as they are and written under "Known limits": adventure mode,
    spawn protection, held right click spending the charges, glass. The other findings were doc corrections (the stale enchantment
    glint of old amulets, the pending boots).
12. Fix and commits: main `72f03bd` and 1.21.1 `0977005` (client `use()` returns `consume`). The log and the checklist were updated.
13. Conflicts with the siblings (`git merge-tree --write-tree HEAD origin/redesign/<sibling>` for a, b, d, e, f and their `-1.21`).
    Avoidable ones removed: the Fabric client entrypoint now carries one line (`FabricAmuletClient.register()`, before the HUD line, no new
    import: same package) and the Forge constructor registers the client setup before the common one, so part A's edits next to them
    merge cleanly (main `8687369`, 1.21.1 `660976a`). What is left is only what rule 8 expects G to resolve: `CHANGELOG.md` (every part)
    and the `mixins` array of `bmh.mixins.json` (a, b, e: each part adds its mixin line to the same array; on 1.21.1 keep the others and
    `C_ServerPlayerMixin`, and drop `EnchantmentMixin`, which part C deleted). Part A's `ClientItemProperties.registerAll(Registrar)` is
    the better home for the amulet line (`registrar.register(ItemInit.NAVIGATOR_AMULET.get(), AmuletCompass.ANGLE, AmuletCompass.angle())`);
    with it, `ForgeAmuletClient`, `NeoForgeAmuletClient` and `FabricAmuletClient` and their three call lines can go. Left to G on purpose:
    folding needs A merged first.
14. Final builds at the final commits: see the Verification section.

## Calls made (with reasons)

- **Night.** The server cannot ask the star brightness (`getStarBrightness` is `ClientLevel` only), so "night" is `Level.isNight()`
  (sky darkness 4 or more: about 12542 to 23459 in clear weather) plus `!isRaining()`, Overworld only, and `canSeeSky` at the eye
  position (sky light 15: a roof, leaves, water or a one block overhang all hide the stars). Rain is refused first because a daytime
  thunderstorm also makes `isNight()` true.
- **Nightfall index.** `floorDiv(dayTime + 12000, 24000)` goes up at dusk (12000) and stays through the next day, so sleeping through the
  night or waking up does not refill; only the next dusk does. `/time set` can move it back; any change refills, which is harmless.
- **Charges.** Stored as `bmh_starlight` (int) and `bmh_night` (long), absent means full. Refilled on the server once a second in
  `inventoryTick` (so the bar and the tooltip, which have no level, read the stored value) and again at the start of a reading.
  A stack sitting in a chest across a nightfall shows its old count until it is carried again.
- **Creative.** Readings in creative spend nothing (testing), the 2 s cooldown still applies.
- **Cooldown.** Vanilla item cooldown (the sweep shows, held right click is throttled by the game itself): 40 ticks after a reading,
  20 ticks after setting Home. A refused reading sets no cooldown.
- **Client side of a click.** The client `use()` returns `consume` and decides nothing; the server answers `success` (which swings the
  arm) or `fail`. Found by the audit (read in `Minecraft.startUseItem`, 1.20.4 and 1.21.1): after `useItem` the loop over both hands only
  stops on a consuming result, so a client `pass` would use the other hand in the same click (shield, bucket, pearl) while the
  server does the reading. The old amulet had that flaw too. Price: while the amulet is in the main hand, a right click that nothing
  else wanted is the amulet's (an off-hand item still works during the 2 s cooldown, as for any vanilla item on cooldown).
  `useOn` decides on both sides (sneak and a bed or lodestone) so that the client does not also send a `use()` for the click that set
  Home. In other dimensions the same click says Home is Overworld only, so the bed never explodes in the Nether.
- **Known limits, left as they are.** Adventure mode never reaches `useOn` (vanilla returns early), so there a sneak click on a bed
  reads the stars instead of setting Home. A bed inside the server's spawn protection, or out of reach, gives the arm swing and no
  Home (vanilla does the same for sleeping). A held right click repeats every 4 ticks, so holding it for 4 s spends the 3 charges
  (an ender pearl does the same). Glass lets the stars through (skylight 15), leaves and water do not.
- **Messages.** One message per reason (`bmh.amulet.hidden.dimension/rain/day/roof`) instead of one generic line: with four conditions
  a player cannot guess which one fails. All strings are lang keys; only `en_us.json` has them (other languages fall back to it).
- **Bar.** Always visible (3 states are the information), violet `0xB58CFF` (the amethyst edge colour part D draws with).
- **Home storage.** Player NBT `bmh_home: {x, y, z}` through `C_ServerPlayerMixin` (`@Unique` field, the holder interface lives in
  `handlers/AmuletHome` because Mixin forbids referencing anything from the mixin package). Copied in `restoreFrom` for death and the
  End return. No dimension stored: Home is Overworld only.
- **Azimuth.** From the exact player position to the centre of Home's block, `atan2(dx, -dz)` taken mod 360 and kept below 360. The
  item constructor logs the self-check of the brief once per game start (see the verification section).
- **Enchanting hooks.** Removed `canApplyAtEnchantingTable`, `supportsEnchantment`, the Fabric `EnchantmentMixin` and its config line
  (main side) and the common `EnchantmentMixin` (1.21.1). Without a durability the vanilla rules refuse every enchantment, so
  nothing is needed. An amulet from an older version keeps its stale tags: `Damage` is ignored, an `Enchantments` entry (Unbreaking)
  still gives the glint and the tooltip line but does nothing (left alone on purpose). The two docs
  that described these hooks (`.claude/CLAUDE.md`, `add-mixin` skill) were corrected on both sides and a Code map bullet added.
- **Client registration.** Part A has not delivered the shared mod-bus client listener yet, so each loader has a small class of its
  own (`ForgeAmuletClient`, `NeoForgeAmuletClient`) registered with one `addListener` line behind `FMLEnvironment.dist.isClient()`,
  and one line on Fabric. When `origin/redesign/integration` carries A's listener (final merge) I move these lines into it.
- **README.** The amulet paragraph was wrong after this change and is rewritten; the crafting grid picture
  `resources/navigator_amulet_crafting_grid.png` still shows the old recipe and needs a new in-game screenshot (checklist item 14).
- **Tooling.** The harness blocks Write/Edit outside the session's worktree, and the brief needs a second one: I switched the session
  into the `-121` worktree with `EnterWorktree path=...` for its edits and back for the main side. Git commands in a worktree session
  have to be plain (no loops or variables around `git`).

## Verification commands

- `JAVA_HOME=/opt/homebrew/opt/openjdk@17 sh <main worktree>/gradlew -p <main worktree> build --max-workers=2` (via `gradle_run.sh`,
  slot in `/private/tmp/bmh-gradle.N`): `BUILD SUCCESSFUL in 3m 44s` at the first commit and `BUILD SUCCESSFUL in 2m 15s` at the final
  one (`8687369`), Forge, Fabric and NeoForge jars each.
- Same with `openjdk@21` in the `-121` worktree: `BUILD SUCCESSFUL in 3m 43s` at the first commit and `BUILD SUCCESSFUL in 1m 40s`
  at the final one (`660976a`), Forge, Fabric and NeoForge jars each.
- `unzip -p <fabric jar> bmh.refmap.json`, `unzip -p <forge jar> META-INF/MANIFEST.MF`, `javap -v -cp <extracted forge classes> ...`.
- `compile_check.sh <version> <jdk> <worktree> <out>` for 1.20 to 1.20.4.
- `boot_server.sh <worktree> <jdk> <loader> <mc version> <log dir> <during.sh>` for the six boots of step 10, logs in each worktree's
  `build/logs/` (`server-<loader>-<version>.log` and `scenario-<loader>-<version>.log`); `needle_sim.py <navigator_amulet.json>`.
