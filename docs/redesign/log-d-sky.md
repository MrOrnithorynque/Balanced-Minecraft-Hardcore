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

## Resume here

Step 5 done (plain-Java code and previews tuned, not yet regenerated into the repo). Next: the renderer
(`StarReadingRenderer`) and the client mixin (`D_LevelRendererSkyMixin`) on main, build, server boots, then the 1.21.1
port. Regenerate the PNGs into `docs/redesign/sky-previews/` once the look is final:
`javac -d <tmp> docs/redesign/sky-previews/SkyPreview.java common/src/main/java/github/mrornithorynque/bmh/client/sky/Star*.java`
then `java -Djava.awt.headless=true -cp <tmp> SkyPreview docs/redesign/sky-previews`.
