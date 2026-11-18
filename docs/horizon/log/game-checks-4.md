# Game checks 4 (wave 2 and 3, second part: m2, tools, ritual, sky)

Tip when I started: 8cda691 (clean tree). No horizon-client running.

## Steps

1. Read the brief, the part 1 log. Started this log.
2. Built (check.sh build). `--only m`: m1 to m3 pass (23333.0 blocks). `--only u,m`: pass. Full run at 8cda691: m2 red (21464.1).
3. Cause of m2, from the kept game log of the full run: the respawn point was (-25243, 93, 5227) (set by `spawnpoint @s` in
   scenario u, after t). The mod landed 23333 blocks from that point (`Homecoming record: ... 23333 blocks`). The test measured
   from `spawn_x`/`spawn_z`, which `ensure_point` set earlier and u did not update (POINT_SET stays 1). So the mod is right and
   the test measured from an old point. The distance in red changed from run to run (16674.6, 21464.1) because the old point differed.
   Fix: scenario u sets `spawn_x`/`spawn_z` to the player after its `spawnpoint @s`. No mod change.
4. Scenario `r` (tools). Traps: `horizon-ctl break` counts 5 extra ticks on every dig after the first (vanilla delay after a
   break), so r_dig takes 5 off and r0 calibrates it (wooden pickaxe: 23 ticks, Java 23). The reply of `break` can show the
   block as stone when the dig is very short (2 ticks), so the block is read again 20 ticks later (`horizon-ctl block`).
   Result: r0 to r7 pass (fresh eternal pickaxe 3 ticks, worn 12 ticks as Java, worn with Efficiency V 2 ticks as Java, the
   block stays gone, 32998 pickaxe after five blocks still there at 32999, death wear +2310, rule off none, creative none).
5. Scenario `s` (ritual). Trap: `horizon-ctl xp N` changes what `state` shows (level 40) but the mod's `native_acting_player()`
   reads 0 levels, so the ritual failed with `bmh.ritual.levels`. The command `xp add @s 40 levels` sets the real level. Used that.
   Result: s1 to s6 pass (see Checks).
6. Scenario `v` (sky). Reading from a slab 60 blocks up (the platform near trees hid the low stars). The log gives the azimuth
   and the last star's altitude of each line; the view is aimed from them. Two rounds on the spyglass name: the star sat at the
   centre of the scope but no name showed (screenshots below). Stopped there, as the brief says.
7. Full run: `test-slice.sh: 115 passed, 6 failed, 0 skipped`. Reds: n5 (immediate_respawn, known) and t1 to t5
   (the totem save, known from game-checks-3). m2 now passes in the full run. r0 to r7, s1 to s6, v1 to v5 pass.

## Checks

- tools (`r`): PASS r0 to r7. Fresh Eternal pickaxe 3 ticks, worn 12 (Java 12), worn with Efficiency V 2 (Java 2), the block is
  air 20 ticks later; pickaxe at 32998 after five blocks is at 32999 and not broken; the sword wears +2310 at a survival death,
  0 with the rule off, 0 in creative.
- ritual (`s`): PASS s1 to s6. By day: action bar "The moon is hidden: wait for night.", no start. At night aimed at the moon
  (aim error 1.7 to 3.7 degrees in the log): starts with the real level read; looking away gives `looked_away`; full ritual
  gives a Totem of Eternity, 40 levels become 10, the advancement. `native_acting_player()` answers (no refusal line).
- sky (`v`): PASS v1 to v3 (reading lines, no `does not draw` line, the self-check line once). Screenshots, all opened by me,
  in `docs/horizon/log/sky-shots/` (1280 px wide):
  - `v4-reading-toward-point.png`: night sky over a dark horizon; two thin violet chains of five stars, one with a star on
    each joint, running up from near the horizon. They draw over Horizon's stars and over the cloud shapes. In an earlier run
    (platform near trees) trees hid the low end of the chain: terrain hides the lines.
  - `v4-reading-toward-point-later.png`: the same chains a few seconds later, still there (they hold).
  - `v4-looking-at-horizon.png`: looking south, no chain (no reading in that direction), terrain and stars only.
  - `v5-spyglass-on-last-star.png` and `-early.png`: the scope (dark frame) with a bright star and a violet beam coming into
    it from above. No name text under the centre in either.

## Defects left

| Area | Severity | What happens | Expected | Evidence | Command |
|---|---|---|---|---|---|
| totem | major | The owner dies with a Totem of Eternity (`living_death` not called for him) | Saved at 1 health | t1 to t5 red | `sh horizon/tools/test-slice.sh --only t --no-build` |
| respawn | minor | `immediate_respawn` true: 0.7 blocks, WARN "landing search did not end" | A distant respawn | n5 red | `sh horizon/tools/test-slice.sh --only n --no-build` |
| sky | unknown | The spyglass name did not show with the last star at the scope centre (two runs) | The name of the line under the centre | v5 screenshots, no name | `sh horizon/tools/test-slice.sh --only v --no-build` |
| tools | unknown | E4 as a LAN guest or a dedicated server client is not tested: the owner case passes | no change after the crack | not tested | none |

`horizon-ctl xp N` shows a level in `state` that the mod's `native_acting_player().xp_level` does not read (0). Not a mod defect I could prove: the
real level comes with `xp add @s N levels`. Worth a look by whoever owns the Horizon side.

## Not tested

The sound (all runs are muted), the hand pose of the ritual, the ritual particles, the echo, the axe smash, the pickaxe glow (they need
hostile mobs or a person's eye), a hit during the ritual, a LAN guest or a dedicated server, the sky in rain or in the Nether, F5 and F1,
frame time, HDR, frame generation.
