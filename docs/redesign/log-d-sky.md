# Session D (`d-sky`): sky constellation, durable log

Brief: `docs/redesign/BRIEF.md`, part `D · Sky constellation`. Branches: `redesign/d-sky` (main side, this worktree
`/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/d-sky-690d62`) and
`redesign/d-sky-1.21` (1.21.1 side, worktree `/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/d-sky-690d62-121`,
cut from `origin/redesign/integration-1.21`, upstream unset so a push never lands on the integration branch).
Server port 25614, RCON 25714.

Steps are numbered, not timed. Started 2026-11-03 21:28 CET (read from `date`).

## Steps

1. Tree guard checked, branch renamed `claude/d-sky-690d62` to `redesign/d-sky`, brief and repo `.claude/CLAUDE.md` read.
   Shared notes folder was empty.
2. Created the 1.21.1 worktree (`git worktree add <wt>-121 -b redesign/d-sky-1.21 origin/redesign/integration-1.21`).
3. Read vanilla's star code on every version (javap on the Loom jars 1.20 to 1.21.1, the ForgeGradle `_mapped_official_`
   jars 1.20 to 1.21.1, the NeoForge 1.20.4 and 21.1.265 decompiled sources):
   - `LevelRenderer.drawStars` is bytecode-identical on 1.20, 1.20.1, 1.20.2, 1.20.3, 1.20.4 (normalized javap hash
     `83fb48190755`), so one main-side replica covers every branch G ports to. 1.21.1 differs (floats, JOML quaternion,
     and the corners are accumulated with a mutating `Vector3f.add`, so each 1.21 star is a diamond offset by its
     half-size from its nominal direction).
   - `LevelRenderer.renderSky` is bytecode-identical on vanilla 1.20 to 1.20.4 (hash `dfc6f1ac8e88`). On every Forge,
     NeoForge and vanilla version, the star section is `getfield starBuffer`, `bind`, `getfield starBuffer`,
     `drawWithShader`, `VertexBuffer.unbind()`, `Runnable.run()` (fog setup). Injection point: the first
     `VertexBuffer.unbind()` after the first `starBuffer` read, shift AFTER.
   - Sky transform: pose = camera rotation, then `YP(-90)`, then `XP(timeOfDay * 360)`. For a star at (x, y, z) in the
     star field's frame, the world direction is `(-(y sin t + z cos t), y cos t - z sin t, x)` (x east, y up, z south).
     Checked against the sun: noon at the zenith, sunset (dayTime 12000) in the west.
   - `getStarBrightness` is 0.5 at full night; vanilla adds the stars with shader colour (b, b, b, b) and blend
     SRC_ALPHA + ONE, multiplied by (1 - rain).
4. Wrote the plain-Java star code in `common/src/main/java/github/mrornithorynque/bmh/client/sky/`:
   `StarField` (vanilla's field, its own copy of `LegacyRandomSource`), `StarChain` (the picker), `StarTrace` (the
   animation as triangles in the star field's frame). No Minecraft import, so the offline previews run the same code.
   Verified:
   - `StarField.Random` vs Minecraft's `RandomSource.create(10842L)` (real class from the 1.20.4 jar): 20000 mixed
     `nextFloat`/`nextDouble` calls, 0 bit mismatches.
   - `StarField` vs vanilla 1.20.4 `drawStars` (the decompiled body turned mechanically into a vertex recorder, run
     with the real `RandomSource`): 780 stars, 3120 corners, 0 bit mismatches.
5. Offline preview tool `docs/redesign/sky-previews/SkyPreview.java` (software rasterizer with GL's SRC_ALPHA + ONE
   blend, draws vanilla's stars and exactly the triangles `StarTrace` emits; player view and all-sky map; sweeps
   every azimuth over the night into `stats.txt`). Tuned the picker on it (see Calls 1 to 5).

6. In-game code, main side (commit `74d0b71`): `client/sky/StarReadingRenderer` (reads the held amulet's
   `bmh_reading`, starts a reading this client has not shown and that is under 40 ticks old, animates it, draws
   `StarTrace`'s triangles with vanilla's sun-and-moon path: CPU-transformed vertices, `BufferUploader`, position-colour
   shader, SRC_ALPHA + ONE, previous shader and shader colour restored) and `mixin/D_LevelRendererSkyMixin` (client
   list of `bmh.mixins.json`; `@Inject` into `renderSky` with its full descriptor, slice from the `starBuffer` field,
   first `VertexBuffer.unbind()` after it, shift AFTER; `@Shadow level`). Logs once
   `Sky azimuth self-check` (from (0, 0): azimuth 0 aims at (0, -100), 90 at (100, 0)) and per reading
   `Star reading toward <az>: <n> stars, the last at azimuth <a>, altitude <h>`. Changelog line, checklist
   `docs/redesign/checklist-d-sky.md`.
7. Plain `javac` of the whole common tree against the Loom jars of 1.20, 1.20.1, 1.20.2, 1.20.3, 1.20.4: all compile
   (script: scratchpad `compile_all.sh`, libraries from the Gradle cache).
8. 1.21.1 side (commit on `redesign/d-sky-1.21`, pushed): `StarField` rewritten on 1.21.1's code (floats, JOML,
   accumulated corners), checked bit-exact against vanilla 1.21.1 `drawStars` (780 stars, 3120 corners, 0 mismatches,
   real `RandomSource` replaced by our copy, `Mth.lengthSquared` inlined from its source); same 780 stars as the main
   side (centres within 0.143 degrees: the diamond offset), 86.0 percent of the 16200 sweep readings pick the very
   same chain, the rest differ by tie-breaks. `StarChain` and `StarTrace` identical to main. Renderer: custom data
   (`DataComponents.CUSTOM_DATA`, `contains` then `copyTag`), `Tesselator.begin` / `addVertex` / `setColor` /
   `build()` (null when empty). Mixin: rebuilds the star pose from `frustumMatrix`, `YP(-90)`, `XP(timeOfDay * 360)`.
   `javac` against the 1.21.1 jar: compiles.
9. C's branches already write the contract exactly (`AmuletReading`: `bmh_reading`, `azimuth` float from
   `atan2(toX - fromX, fromZ - toZ)`, `time` = `level.getGameTime()`), on both sides.

10. Main-side `gradle build` (all loaders): OK. Refmaps: Forge maps `renderSky` to `m_202423_`, `starBuffer` to
    `f_109471_`, `VertexBuffer.unbind` to `m_85931_`; Fabric to `method_3257`, `field_4113`, `method_1354`. The
    `@Shadow level` field is renamed in the class itself (Forge `f_109465_`, Fabric `field_4085`, NeoForge `level`);
    the Forge manifest has `MixinConfigs: bmh.mixins.json`.
11. Fabric dedicated server (development build, `-verbose:class`): ready, RCON world prep (time 18000, no daylight
    cycle, clear weather), clean stop, port closed; 33 mod classes loaded, none under `client.sky` nor the mixin.
12. Non-interactive Fabric client run (`tools/client-shots.sh`, quick play into a copy of that world): the mixin
    applied, `Sky azimuth self-check: from (0, 0), azimuth 0 aims at (0, -100), azimuth 90 at (100, 0)`, eight
    readings, eight screenshots, clean exit. Every logged chain (`Star reading toward ...`) matches its offline preview
    star for star. What the frames showed:
    - the chain looks about 12 percent smaller than in the preview: a spectator flies, and flying multiplies the FOV
      by 1.1 (70 to 77; tan 35 / tan 38.5 = 0.88, the measured ratio was 0.87). Not a bug.
    - the real moon is a small bright square: only the middle 8 of 32 texels of a moon phase (10 of its 40 blocks) are
      bright, the sun's core is 10 of 32 texels. The exclusion boxes shrank to the bright discs plus 3 blocks (moon 8,
      sun 13 half-size): mean last-star error 2.07 degrees (was 2.20), worst 11.8 (was 17.2), 4 readings of 16200 at
      10 degrees or more (was 56), sharp bends 226 (was 326). Only the 270 / 22000 case changed: now 270.1 degrees,
      just under the moon, its line clipping the moon's corner.
    - BUG found and fixed: the halos around the chain's stars were missing. The sky is drawn with back faces culled
      and the halo fans wound outward. They now wind like the ribbons and vanilla's star quads (counter-clockwise from
      the centre); the preview rasterizer culls back faces like GL, and still shows vanilla's stars, so its convention
      matches the game's.

## Calls made (and why)

1. **Picker = dynamic programming, not greedy.** Candidates within the off-plane limit, sorted high to low; the DP
   keeps the last two stars so it can price bends. Cost: small stars, off-plane distance (x2), last star off-plane
   (x4 more: it marks the direction), uneven gaps (preferred 11 degrees), bends (3 x (1 - cos)). Greedy picks made
   zigzags. Score = cost / stars so 5, 6 and 7 compete fairly.
2. **Relaxation passes** (max off-plane / lowest top / highest bottom / max gap / max last-star off-plane):
   10/50/18/20/4, then 12/45/20/22/6, 14/40/24/26/10, 18/35/28/30/18. Always 5 to 7 stars. On the vanilla field the
   sweep (16200 readings: azimuth 0 to 359, dayTime 12500 to 23500 step 250) never comes back empty.
3. **Chain spans 5 to 72 degrees of elevation.** Above 72 the chain no longer fits one look up at FOV 70; below 5 the
   first hill hides it.
4. **Stars behind the sun or the moon are never picked.** Both turn with the stars, so in the star field's frame they
   are fixed squares (sun 60 blocks at y = 100, moon 40 at y = -100): an exact box test, same stars at every hour, 3
   blocks of margin. A cone around the moon made chains detour 14 degrees. A line may still cross the moon between two
   stars (accuracy first): it stays visible, whiter.
5. **Look:** line = white core (#f0e4ff, half-width 0.10), violet edge (#b58cff, 0.34, alpha 0.6), faint violet glow
   (0.9, alpha 0.14): about 3 px of line at 1080p, a little wider than a star (0.3 to 0.5 units). Lines stop 0.6 short
   of each star; each chain star is redrawn in the core colour plus a soft halo (bigger on the last star); a spark rides
   the head of the segment being traced. Timing: 0.3 s per segment, each star lights in 0.15 s as the line reaches it,
   hold 8 s after the trace, fade 2 s.
6. **Sky alpha = starBrightness x (1 - rain) / 0.5**, clamped to 1: the brief's "multiply alpha by (1 - rainLevel) and
   by the star brightness, like vanilla's stars", normalized so a reading at full night is at full strength (vanilla's
   star brightness tops out at 0.5).

7. **Once started, a reading plays to the end** even if the amulet leaves the hand (it spent a charge; cutting it
   on a hotbar scroll would look like a bug). It stops early only on a level change. A newer reading replaces it.
8. **Development trigger = `DevSkyShots`**, an environment variable (`BMH_DEV_SKYSHOTS=1`) read by the client, not a
   command: no loader code to add and remove on six loader/side pairs. It runs one non-interactive singleplayer
   client (quick play into a world the dedicated server made), fakes readings toward 0/90/180/270 at dayTime 18000 and
   22000, faces the camera that way, saves screenshots, quits. Brief rule 4 says nobody can *drive* `runClient`; this
   needs no driving, and only a real frame proves the GL state and the pose, which the offline previews cannot. One
   window on Louis's screen for a few minutes, sound muted in `options.txt`. Removed before DONE (brief, part D,
   item 4).
9. **Self-check log** for the azimuth convention (brief, part C: "both log a self-check"), from the very helper the
   picker aims with (`StarChain.toward`).
10. **Harness:** the Write/Edit tools refuse files in the `-121` worktree while the session sits in the main one;
    switching with `EnterWorktree` (path) works (session A's note). After a switch, the Bash guard refuses scripts and
    variables around git: plain commands with literal paths only.

## Resume here

Steps 1 to 9 done; both branches pushed (main `74d0b71`, 1.21.1 committed with the same content ported). In progress:
main-side Gradle `build` queued for a slot (scratchpad `wait_build.sh`, log `build/d-sky-logs/build-main-1.log`). Next:
Fabric server boot that leaves a night world (`tools/boot-test.sh` + `tools/cmds-fabric-world.txt` in the scratchpad,
copies of session A's tools with `-verbose:class`), the client screenshot run (`tools/client-shots.sh`), then remove
`DevSkyShots` and its one call line on both sides, final builds, three server boots per side, audit, report.
Regenerate the PNGs into `docs/redesign/sky-previews/` once the look is final:
`javac -d <tmp> docs/redesign/sky-previews/SkyPreview.java` plus `StarField.java`, `StarChain.java`, `StarTrace.java`
from `common/src/main/java/github/mrornithorynque/bmh/client/sky/` (not `Star*.java`: that also catches the
Minecraft-bound renderer), then `java -Djava.awt.headless=true -cp <tmp> SkyPreview docs/redesign/sky-previews`.
