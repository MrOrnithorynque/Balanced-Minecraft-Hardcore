"""Replays vanilla's compass angle for a non-local holder looking in every direction, resolves the amulet model overrides
the way ItemOverrides does (highest threshold <= angle wins), and checks the frame's needle (index i = 360*i/32 degrees
clockwise from screen-up) points at north on screen."""
import json, math, sys
model = json.load(open(sys.argv[1]))
overrides = [(o["predicate"]["angle"], int(o["model"].rsplit("_", 1)[1])) for o in model["overrides"]]
def frame(angle):
    best = None
    for threshold, idx in overrides:          # file order is ascending: the last match wins
        if angle >= threshold: best = idx
    return best
worst = 0.0
for tenth in range(0, 3600):
    yaw = tenth / 10.0                         # Minecraft yaw: 0 south, 90 west, 180 north, 270 east
    d1 = (yaw / 360.0) % 1.0                   # getWrappedVisualRotationY
    d0 = math.atan2(-1e6, 0.0) / (2 * math.pi) # getAngleFromEntityToPos, target due north: dx = 0, dz = -1e6
    angle = (0.5 - (d1 - 0.25 - d0)) % 1.0     # branch for entities that are not the local player (no wobble)
    f = frame(angle)
    needle = 360.0 * f / 32.0
    bearing = (yaw + 180.0) % 360.0            # facing, clockwise from north
    expected = (-bearing) % 360.0              # where north is on screen, clockwise from up
    err = abs((needle - expected + 180.0) % 360.0 - 180.0)
    worst = max(worst, err)
print("worst needle error over 3600 yaw values: %.3f deg (a frame is 11.25 deg wide, so 5.625 is the limit)" % worst)
for yaw, name in [(180, "north"), (270, "east"), (0, "south"), (90, "west")]:
    d1 = (yaw / 360.0) % 1.0; angle = (0.5 - (d1 - 0.25 + 0.25)) % 1.0
    print("facing %-5s yaw %3d -> angle %.4f -> frame %02d -> needle %.1f deg clockwise from up" % (name, yaw, angle, frame(angle), 360.0 * frame(angle) / 32))
sys.exit(0 if worst <= 5.625 + 1e-9 else 1)
