# Sky previews (part D)

Offline renders of what a Navigator's Amulet star reading draws in the sky, made before any in-game look by
`SkyPreview.java` in this folder. It runs the mod's own star code (`StarField`, `StarChain`, `StarTrace` from
`common/src/main/java/github/mrornithorynque/bmh/client/sky/`) and rasterizes vanilla's stars plus the very triangles
the game draws, with the game's additive blend.

Each picture: on the left the player's view (facing the reading's azimuth, looking 32 degrees up, vertical FOV 70 like
the default), with compass ticks on the horizon and a red marker on the true direction; on the right an all-sky map
(zenith in the centre, rings every 30 degrees, north up, east right, the dashed line is the true direction). The grey
square is the moon's bright disc. The header gives the chain's length and where its last star sits.

In-game check: `ingame-az090-midnight.png` and `ingame-az270-late-night.png` are real Fabric 1.20.4 frames (a
non-interactive client run with a development trigger, since removed), next to the matching preview. The player was a
spectator, and flying widens the FOV by 10 percent, so the in-game chain looks a little smaller than in the preview;
the stars and the chain are the same. `ingame-detail-az090-midnight.png` is a crop of a frame at full resolution (a
Retina window): the line, its glow and the lit stars as the game draws them.

- `az000-midnight.png`, `az090-midnight.png`, `az180-midnight.png`, `az270-midnight.png`: readings toward north, east,
  south, west at dayTime 18000 (midnight), 2.5 s after the reading (trace done, holding).
- `az000-late-night.png` ... `az270-late-night.png`: the same at dayTime 22000 (late night: the sky has turned 70
  degrees, the moon is low in the west).
- `trace-az135-midnight-*.png`: one reading toward the south-east, at 0.4 s, 0.9 s and 1.6 s (tracing), 9.0 s
  (holding) and 10.2 s (half faded).
- `stats.txt`: every azimuth (1-degree steps) over the whole night (dayTime 12500 to 23500, steps of 250): how many
  stars, how far the last star sits from the true direction, how sharp the chain bends.

Regenerate from the repository root (JDK 17 or later):

```bash
S=common/src/main/java/github/mrornithorynque/bmh/client/sky
javac -d /tmp/sky docs/redesign/sky-previews/SkyPreview.java $S/StarField.java $S/StarChain.java $S/StarTrace.java
java -Djava.awt.headless=true -cp /tmp/sky SkyPreview docs/redesign/sky-previews
```
