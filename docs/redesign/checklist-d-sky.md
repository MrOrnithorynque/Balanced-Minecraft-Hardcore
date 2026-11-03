# Checklist, part D: the sky constellation

What a star reading of the Navigator's Amulet draws in the sky. The reading itself (charges, conditions, the action
bar) is part C; this list only checks the drawing. Same steps on every loader (Forge, Fabric, NeoForge) and on 1.20.4
and 1.21.1. Reference pictures of what to expect: `docs/redesign/sky-previews/` (made offline from the same star code).

## Setup (singleplayer, cheats on)

1. `/time set 18000`, `/weather clear`, `/gamerule doDaylightCycle false` (the sky holds still while you compare).
2. `/give @p bmh:navigator_amulet`.
3. Stand outside, open sky above your head, in the Overworld.

## Steps

1. **Use the amulet** (right click, main hand). Within a moment a white line with a violet edge starts high in the sky
   and traces down toward the horizon, one segment about every 0.3 s, through 5 to 7 real stars. Each star lights up
   (white, soft violet glow) as the line reaches it. The lowest star has a slightly bigger glow.
2. **It points home.** Turn toward the lowest star: the amulet's needle (part C) points the same way, give or take a
   few degrees. The log (`run/logs/latest.log`) has a line
   `[BalancedMcHardcoreMain] Star reading toward <azimuth>: <n> stars, the last at azimuth <a>, altitude <h>`, and
   `<a>` is within a few degrees of `<azimuth>` (up to about 15 when the moon sits low in that direction).
3. **It holds, then fades.** About 8 s after the trace ends it fades out over 2 s. No text, no number, nothing on the
   HUD.
4. **It is pinned to the stars.** The line starts and ends exactly on stars that were there before the reading, and
   stays on them (with `doDaylightCycle` on, the sky turns about 4 degrees during a reading: the chain turns with it).
5. **Offhand.** Put the amulet in the offhand and use it: same drawing.
6. **Pocketing it.** Use it, then switch to another item at once: the drawing still plays to the end.
7. **A second reading** (after the 2 s cooldown): the old chain disappears, a new one traces. With an amulet in each
   hand, using one then the other: the newer reading shows, the two never flicker back and forth.
8. **Rain dims it.** Use it, then `/weather rain` while the chain holds: it dims with the stars.
9. **Late night.** `/time set 22000` and use it facing north, east, south, west in turn: a chain every time. The moon
   is low in the west then; a reading toward the west may run its line across the moon (it stays visible, whiter).
10. **Dusk and dawn.** `/time set 13000` and `/time set 23000`: the chain is fainter, like the stars.
11. **Only you.** Two clients (LAN, or a dedicated server): only the player who used the amulet sees the chain.
12. **Dedicated server.** On a dedicated server the reading still draws on the client of the player who used it.
13. **Shader packs** (Iris or Oculus, only if you use them): best effort. A pack that draws its own sky may hide the
    chain or tint it; not a blocker.

## Known, on purpose

- A reading taken while the sky is hidden (Blindness, Darkness, inside powder snow, thick fog) draws nothing: the chain
  starts only while the stars are drawn, within 2 s of the reading. Since the merge, part C refuses such a reading for
  Blindness, Darkness and eyes in water, lava or powder snow (no charge spent); thick fog still spends it.
- The chain pauses with the game (singleplayer pause menu) and resumes.

## If something looks wrong

- Nothing at all: check the log for the `Star reading toward` line. No line means the client never saw a fresh
  reading on the held amulet (the amulet carries it as `bmh_reading` with `azimuth` and `time`; a reading more than
  40 ticks old when the client sees it is skipped on purpose).
- Lines in the wrong place (not on stars): note the loader and version, take a screenshot (F2).
