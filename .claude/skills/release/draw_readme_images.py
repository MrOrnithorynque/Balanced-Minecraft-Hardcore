"""Draws two README pictures into resources/showcase/: respawn_distance.png (the respawn distance by difficulty and
Eternal Items carried, as rings on a map under the night sky of the journey) and eternal_items.png (the five Eternal
Items, in the style of a Minecraft tooltip).

    python3 -I .claude/skills/release/draw_readme_images.py <vanilla ascii.png> [<repo root>]

The vanilla font comes out of the Loom client jar:
    unzip -p ~/.gradle/caches/fabric-loom/<minecraft_version>/minecraft-client.jar assets/minecraft/textures/font/ascii.png > <scratch>/ascii.png
The distance defaults and the cap of the formula are read from the code (BMHConfigLimits, RespawnPositionHandler); the
formula itself is copied in distance_grid(). The item list is in eternal_items(). Needs Pillow. The map, the stars and
the dust come from seeded random generators, so each run draws the same picture."""
import math
import os
import random
import re
import sys
from PIL import Image, ImageDraw, ImageFilter

FONT_PATH = sys.argv[1]
ROOT = sys.argv[2] if len(sys.argv) > 2 else "."
JAVA = os.path.join(ROOT, "common/src/main/java/github/mrornithorynque/bmh")
TEXTURES = os.path.join(ROOT, "common/src/main/resources/assets/bmh/textures/item")
OUT = os.path.join(ROOT, "resources/showcase")


def java_int(path, name):
    match = re.search(r"\b%s\s*=\s*(\d+)\s*;" % name, open(os.path.join(JAVA, path), encoding="utf-8").read())
    if not match:
        sys.exit("%s: no constant %s" % (path, name))
    return int(match.group(1))

SCALE = 3
BG = (16, 0, 16, 255)
BORDER_TOP = (80, 0, 255, 255)
BORDER_BOTTOM = (40, 0, 127, 255)
WHITE, GRAY, PURPLE = (255, 255, 255), (170, 170, 170), (255, 85, 255)

font = Image.open(FONT_PATH).convert("RGBA")
glyphs = {}
for code in range(32, 127):
    gx, gy = (code % 16) * 8, (code // 16) * 8
    glyph = font.crop((gx, gy, gx + 8, gy + 8))
    cols = [x for x in range(8) if any(glyph.getpixel((x, y))[3] for y in range(8))]
    width = (max(cols) + 1) if cols else 3
    glyphs[chr(code)] = (glyph, width)


def text_width(text):
    return sum(glyphs[c][1] + 1 for c in text) - 1


def draw_text(img, x, y, text, color, shadow=True):
    for c in text:
        glyph, width = glyphs[c]
        for dx, dy, col in ([(1, 1, tuple(v // 4 for v in color))] if shadow else []) + [(0, 0, color)]:
            mask = glyph.split()[3]
            layer = Image.new("RGBA", glyph.size, col + (255,))
            img.paste(layer, (x + dx, y + dy), mask)
        x += width + 1


def panel(w, h):
    """A vanilla tooltip: dark fill, a one pixel outer frame, and a purple inner border from light to dark."""
    img = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    for y in range(1, h - 1):
        for x in range(w):
            img.putpixel((x, y), BG)
    for x in range(1, w - 1):
        img.putpixel((x, 0), BG)
        img.putpixel((x, h - 1), BG)
    for y in range(1, h - 1):
        t = (y - 1) / max(1, h - 3)
        col = tuple(int(a + (b - a) * t) for a, b in zip(BORDER_TOP, BORDER_BOTTOM))
        img.putpixel((1, y), col)
        img.putpixel((w - 2, y), col)
    for x in range(1, w - 1):
        img.putpixel((x, 1), BORDER_TOP)
        img.putpixel((x, h - 2), BORDER_BOTTOM)
    return img


def save(img, name):
    img.resize((img.width * SCALE, img.height * SCALE), Image.NEAREST).save(OUT + "/" + name, optimize=True)


# The respawn distance picture is drawn at full size, 1280 x 720: the README shows it at width 640.
W, H = 1280, 720
CELL = 4  # One map pixel, in picture pixels.


def hex_rgb(value):
    return ((value >> 16) & 255, (value >> 8) & 255, value & 255)


def mix(a, b, t):
    return tuple(int(round(x + (y - x) * t)) for x, y in zip(a, b))


# The colors of the journey cinematic (client/journey/JourneyOverlay.java).
SKY_TOP, SKY_BOTTOM = hex_rgb(0x0B1433), hex_rgb(0x03040B)
LINE_CORE, LINE_EDGE = hex_rgb(0xF0E4FF), hex_rgb(0xB58CFF)
MESSAGE, DUST = hex_rgb(0xFFF1D0), hex_rgb(0x9AA8D8)
STAR_COLORS = [hex_rgb(c) for c in (0xFFFFFF, 0xCFE0FF, 0xFFF2CF, 0xBFD4FF)]
# Peaceful to Hard, from green to orange.
DIFFICULTY_COLORS = [(110, 230, 120), (200, 235, 90), (255, 214, 80), (255, 140, 60)]
# The colors of a vanilla map item (MapColor). The map multipliers 180, 220 and 255 make the relief.
MAP_COLORS = {"water": (64, 64, 255), "grass": (127, 178, 56), "plant": (0, 124, 0), "sand": (247, 233, 163),
              "snow": (255, 255, 255), "stone": (112, 112, 112), "dirt": (151, 109, 77), "ice": (160, 160, 255),
              "wood": (102, 76, 51)}


def text_layer(text, color):
    img = Image.new("RGBA", (text_width(text) + 1, 9), (0, 0, 0, 0))
    x = 0
    for c in text:
        glyph, width = glyphs[c]
        img.paste(Image.new("RGBA", glyph.size, color + (255,)), (x, 0), glyph.split()[3])
        x += width + 1
    return img


def draw_label(img, x, y, text, color, scale, anchor="c"):
    """Draws text at a whole scale, with a thin dark outline and a shadow, so it reads on the busy map. The anchor
    is the center (c), the left end (l) or the right end (r) of the text."""
    w = text_width(text) * scale
    x = {"c": x - w // 2, "l": x, "r": x - w}[anchor]
    size = ((text_width(text) + 1) * scale, 9 * scale)
    body = text_layer(text, color).resize(size, Image.NEAREST)
    dark = text_layer(text, (8, 6, 20)).resize(size, Image.NEAREST)
    step = max(1, scale // 2)
    for dx, dy in ((-1, 0), (1, 0), (0, -1), (0, 1), (scale // 2 + 1, scale // 2 + 1)):
        img.alpha_composite(dark, (x + dx * step, y + dy * step))
    img.alpha_composite(body, (x, y))


class Noise:
    """Value noise on a lattice, smooth enough for coasts and biomes."""

    def __init__(self, seed):
        r = random.Random(seed)
        self.table = [r.random() for _ in range(256 * 256)]

    def lattice(self, x, y):
        return self.table[(x & 255) * 256 + (y & 255)]

    def value(self, x, y):
        x0, y0 = math.floor(x), math.floor(y)
        tx, ty = x - x0, y - y0
        tx, ty = tx * tx * (3 - 2 * tx), ty * ty * (3 - 2 * ty)
        a = self.lattice(x0, y0) + (self.lattice(x0 + 1, y0) - self.lattice(x0, y0)) * tx
        b = self.lattice(x0, y0 + 1) + (self.lattice(x0 + 1, y0 + 1) - self.lattice(x0, y0 + 1)) * tx
        return a + (b - a) * ty

    def fbm(self, x, y, octaves=5):
        total, amp, norm = 0.0, 1.0, 0.0
        for _ in range(octaves):
            total += self.value(x, y) * amp
            norm += amp
            x, y, amp = x * 2.03 + 17.1, y * 2.03 + 5.3, amp * 0.5
        return total / norm


def world_map(cols, rows, home):
    """A fake world as map pixels: height, heat and moisture noise give the sea, the coasts and the biomes."""
    height, heat, wet = Noise(11), Noise(23), Noise(37)
    hx, hy = home
    # Shift the noise so that Home sits on dry land, with sea a little way off.
    ox = next(k for k in range(0, 4000, 7) if 0.44 < height.fbm((hx + k) / 26.0, hy / 26.0) < 0.47)
    elev = [[0.0] * cols for _ in range(rows)]
    for j in range(rows):
        for i in range(cols):
            d = math.hypot(i - hx, j - hy)
            # The ground rises near Home and sinks a little far away.
            elev[j][i] = (height.fbm((i + ox) / 26.0, j / 26.0) + 0.22 * max(0.0, 1 - d / 16.0)
                          - 0.06 * max(0.0, d / 120.0 - 1))
    cells = [[None] * cols for _ in range(rows)]
    for j in range(rows):
        for i in range(cols):
            e = elev[j][i]
            t = heat.fbm(i / 60.0, j / 60.0, 3)
            m = wet.fbm(i / 18.0, j / 18.0, 3)
            depth = 0
            if e < 0.41:
                kind = "ice" if t < 0.36 and e > 0.40 else "water"
                depth = 0.41 - e
            elif e < 0.428 and t > 0.36:
                kind = "sand"
            elif e > 0.70:
                kind = "snow" if e > 0.75 or t < 0.42 else "stone"
            elif t < 0.38:
                kind = "snow" if m < 0.55 else "plant"
            elif t > 0.62:
                kind = "sand" if m < 0.55 else "dirt"
            else:
                kind = "plant" if m > 0.52 else "grass"
            # Like a map item: deep water is darker, and land is lit by its height above the north neighbor.
            if kind in ("water", "ice"):
                shade = 255 if depth < 0.02 else 220 if depth < 0.06 else 180
                if kind == "water" and (i * 7 + j * 3) % 5 == 0 and depth > 0.01:
                    shade = 220 if shade == 255 else 180
            else:
                north = elev[j - 1][i] if j > 0 else e
                diff = (e - north) * 60 + (((i * 31 + j * 17) % 7) - 3) * 0.08
                shade = 255 if diff > 0.35 else 180 if diff < -0.35 else 220
            base = MAP_COLORS[kind]
            if kind == "plant" and (i * 13 + j * 7) % 11 == 0:
                base = MAP_COLORS["wood"]
            cells[j][i] = tuple(c * shade // 255 for c in base)
    return cells


def moonlight(color, t=0.66):
    """Darker and bluer, so the map belongs to the night sky and the rings stand out. Water gets darker still."""
    r, g, b = color
    if b > r + 100:
        t *= 0.72
    return (int(r * t * 0.9), int(g * t * 0.94), int(min(255, b * t + 12)))


def put(img, x, y, col, a, size=2):
    if 0 <= x < W and 0 <= y < H:
        img.alpha_composite(Image.new("RGBA", (size, size), col + (int(255 * a),)), (x, y))


def star(img, x, y, col, a, arms):
    """A star like the journey's: one pixel, and a plus for the bright ones."""
    put(img, x, y, col, a)
    for arm in range(1, arms + 1):
        k = 0.5 if arm == 1 else 0.22
        for dx, dy in ((-2 * arm, 0), (2 * arm, 0), (0, -2 * arm), (0, 2 * arm)):
            put(img, x + dx, y + dy, col, a * k)


def night_sky():
    """The night gradient of the journey, a faint band of the Milky Way and the stars, on a grid of 2 pixels."""
    rng = random.Random(1709)
    img = Image.new("RGBA", (W, H))
    px = img.load()
    for y in range(H):
        col = mix(SKY_TOP, SKY_BOTTOM, y / H) + (255,)
        for x in range(W):
            px[x, y] = col
    dust = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    d = ImageDraw.Draw(dust)
    for _ in range(260):
        u = rng.random()
        x = int(u * W)
        y = int(H * 0.12 + u * H * 0.25 + rng.gauss(0, 55))
        s = rng.choice((8, 12, 16, 20))
        d.rectangle((x, y, x + s, y + s), fill=DUST + (rng.randint(8, 22),))
    img.alpha_composite(dust)
    for _ in range(900):
        x, y = rng.randrange(W), rng.randrange(H)
        col = rng.choice(STAR_COLORS)
        a = rng.random() ** 2 * 0.9 + 0.1
        star(img, x - x % 2, y - y % 2, col, a, 1 if rng.random() < 0.08 else 0)
    return img


def paint_map(img, center, radius, band):
    """The map as a disc that breaks into blocks at its edge, like the pixel dissolve of the journey."""
    cx, cy = center
    cols, rows = W // CELL, H // CELL
    cells = world_map(cols, rows, (cx // CELL, cy // CELL))
    layer = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    rng = random.Random(5)
    for j in range(rows):
        for i in range(cols):
            x, y = i * CELL, j * CELL
            dist = math.hypot(x + CELL / 2 - cx, y + CELL / 2 - cy)
            if dist > radius + band:
                continue
            col = moonlight(cells[j][i])
            if dist <= radius:
                d.rectangle((x, y, x + CELL - 1, y + CELL - 1), fill=col + (255,))
                continue
            # In the band, the blocks get fewer and fainter on a checkerboard, and the map turns into sky.
            t = (dist - radius) / band
            if (i + j) % 2 == 0 and rng.random() > t * 1.1:
                s = CELL - (1 if t > 0.5 else 0)
                d.rectangle((x, y, x + s - 1, y + s - 1), fill=col + (int(255 * (1 - t * 0.7)),))
    img.alpha_composite(layer)


def ring(center, radius, color, dash=0, width=2, alpha=255):
    """A ring of pixels on the 2 pixel grid, solid or dashed, on a layer of its own."""
    cx, cy = center
    layer = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    steps = int(2 * math.pi * radius / 1.5)
    for k in range(steps):
        if dash and int(k * 1.5 / dash) % 2:
            continue
        ang = 2 * math.pi * k / steps
        x = int(round(cx + radius * math.cos(ang))) // 2 * 2
        y = int(round(cy + radius * math.sin(ang))) // 2 * 2
        d.rectangle((x, y, x + width - 1, y + width - 1), fill=color + (alpha,))
    return layer


def glow(img, layer, blur, strength):
    """Draws a layer over a blurred copy of itself."""
    halo = layer.filter(ImageFilter.GaussianBlur(blur))
    halo.putalpha(halo.split()[3].point(lambda v: min(255, int(v * strength))))
    img.alpha_composite(halo)
    img.alpha_composite(layer)


def bed(img, x, y, scale=4):
    """A top-down red bed of 8 x 16 map pixels, in the colors of the vanilla red bed, with the pillow at the top."""
    art = ["oooooooo", "owwwwwwo", "owWWWWwo", "owWwwWwo", "owwwwwwo", "oggggggo", "oddddddo", "orrhrrro",
           "orhhrrro", "orrrrrro", "orrrrhro", "orrrrrro", "orhrrrro", "orrrrrro", "oRRRRRRo", "oooooooo"]
    pal = {"o": (20, 12, 30), "w": (216, 223, 223), "W": (255, 255, 255), "g": (168, 176, 177),
           "d": (107, 18, 19), "r": (172, 41, 34), "h": (191, 59, 51), "R": (133, 26, 26)}
    ox, oy = x - 4 * scale, y - 8 * scale
    d = ImageDraw.Draw(img)
    for j, row in enumerate(art):
        for i, c in enumerate(row):
            d.rectangle((ox + i * scale, oy + j * scale, ox + i * scale + scale - 1, oy + j * scale + scale - 1),
                        fill=pal[c])


def distance_grid():
    # RespawnPositionHandler.calculateDistanceFromBed with the default config.
    lo = java_int("utilities/BMHConfigLimits.java", "DEFAULT_MIN_DISTANCE_FROM_BED")
    hi = java_int("utilities/BMHConfigLimits.java", "DEFAULT_MAX_DISTANCE_FROM_BED")
    cap = java_int("handlers/RespawnPositionHandler.java", "MAX_EASY_LIFE")
    levels = [("Peaceful", 0), ("Easy", 1), ("Normal", 2), ("Hard", 3)]
    items = ["totem_of_eternity", "eternal_sword", "eternal_axe", "eternal_pickaxe", "eternal_bundle"]

    def distance(diff, carried):
        return lo + (hi - lo) * min(1 + carried + diff, cap) // cap

    rings = [(name, distance(diff, 0)) for name, diff in levels]
    # Hard with 1 to 5 Eternal Items: one step of the formula for each item.
    steps = [distance(3, k) for k in range(1, len(items) + 1)]
    step = (hi - lo) // cap

    # Home sits left of center. The farthest ring is a large arc into the sky on the right.
    center, far = (372, 388), 560
    cx, cy = center
    scale = far / hi
    img = night_sky()
    paint_map(img, center, rings[-1][1] * scale + 34, 96)

    # The steps of the Eternal Items: faint dashed amethyst rings, and a solid one at the farthest distance.
    for k, dist in enumerate(steps):
        last = k == len(steps) - 1
        layer = ring(center, dist * scale, LINE_CORE if last else LINE_EDGE, dash=0 if last else 6,
                     alpha=255 if last else 120)
        glow(img, layer, 5 if last else 3, 1.4 if last else 0.6)
    # The difficulty rings, from the outside in, so the glow of an inner ring stays on top.
    for (name, dist), col in reversed(list(zip(rings, DIFFICULTY_COLORS))):
        glow(img, ring(center, dist * scale, col), 5, 1.2)

    # Home: the bed in a double ring, like the Home star of the journey.
    for radius, alpha in ((22, 200), (30, 90)):
        glow(img, ring(center, radius, LINE_CORE, alpha=alpha), 3, 0.8)
    bed(img, cx, cy)
    draw_label(img, cx, cy + 36, "Home", MESSAGE, 2)

    # The Eternal Items: a chain of stars from the Hard ring out to the farthest ring, like the line of the journey.
    ang = math.radians(-14)
    pts = [(cx + d * scale * math.cos(ang), cy + d * scale * math.sin(ang)) for d in [rings[-1][1]] + steps]
    line = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    ld = ImageDraw.Draw(line)
    for a, b in zip(pts, pts[1:]):
        ld.line((a, b), fill=LINE_EDGE + (255,), width=5)
        ld.line((a, b), fill=LINE_CORE + (255,), width=2)
    glow(img, line, 6, 1.2)
    icon = 48
    for k, (x, y) in enumerate(pts[1:]):
        halo = Image.new("RGBA", (W, H), (0, 0, 0, 0))
        ImageDraw.Draw(halo).ellipse((x - icon * 0.62, y - icon * 0.62, x + icon * 0.62, y + icon * 0.62),
                                     fill=(12, 10, 30, 235), outline=LINE_EDGE + (255,), width=2)
        glow(img, halo, 4, 0.7)
        sprite = Image.open(TEXTURES + "/" + items[k] + ".png").convert("RGBA").crop((0, 0, 16, 16))
        sprite = sprite.resize((icon, icon), Image.NEAREST)
        img.alpha_composite(sprite, (int(x - icon / 2), int(y - icon / 2)))
        draw_label(img, int(x), int(y - icon * 0.62 - 22), "+%d" % (k + 1), LINE_EDGE, 2)
    ex, ey = pts[-1]
    draw_label(img, int(ex + icon * 0.8), int(ey - 14), "{:,}".format(hi), LINE_CORE, 3, anchor="l")
    draw_label(img, int(ex + icon * 0.8), int(ey + 16), "the farthest", DUST, 2, anchor="l")
    lx, ly = int(pts[1][0]) - 14, int(pts[1][1]) + 46
    for k, text in enumerate(("+{:,} blocks for each".format(step), "Eternal Item you carry")):
        draw_label(img, lx, ly + 24 * k, text, LINE_CORE, 2, anchor="l")

    # The difficulty labels sit on their rings, up and to the left of Home, on a dark haze over the busy map.
    lab_ang = math.radians(-108)
    labels = [(cx + dist * scale * math.cos(lab_ang), cy + dist * scale * math.sin(lab_ang), name,
               "{:,}".format(dist), col) for (name, dist), col in zip(rings, DIFFICULTY_COLORS)]
    haze = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    hd = ImageDraw.Draw(haze)
    for x, y, name, dist, col in labels:
        half = max(text_width(name), text_width(dist)) + 8
        hd.rounded_rectangle((x - half, y - 22, x + half, y + 24), 10, fill=(6, 6, 18, 200))
    img.alpha_composite(haze.filter(ImageFilter.GaussianBlur(7)))
    for x, y, name, dist, col in labels:
        draw_label(img, int(x), int(y) - 18, name, col, 2)
        draw_label(img, int(x), int(y) + 3, dist, (235, 235, 245), 2)

    draw_label(img, 1240, 54, "Where you wake up", MESSAGE, 4, anchor="r")
    draw_label(img, 1240, 102, "blocks from your bed,", DUST, 2, anchor="r")
    draw_label(img, 1240, 124, "in a random direction", DUST, 2, anchor="r")
    draw_label(img, 1240, 684, "with the default config", DUST, 2, anchor="r")
    img.convert("RGB").save(OUT + "/respawn_distance.png", optimize=True)


def eternal_items():
    items = [("totem_of_eternity", "Totem of", "Eternal Life"), ("eternal_sword", "Sword of", "Eternity"),
             ("eternal_axe", "Eternal", "Axe"), ("eternal_pickaxe", "Eternal", "Pickaxe"),
             ("eternal_bundle", "Eternal", "Bundle")]
    cell, icon = 64, 32
    w = 4 + cell * len(items) + 4
    h = 6 + icon + 6 + 10 + 10 + 6
    img = panel(w, h)
    for i, (tex, line1, line2) in enumerate(items):
        sprite = Image.open(TEXTURES + "/" + tex + ".png").convert("RGBA").crop((0, 0, 16, 16))
        sprite = sprite.resize((icon, icon), Image.NEAREST)
        x0 = 4 + cell * i
        img.paste(sprite, (x0 + (cell - icon) // 2, 6), sprite)
        for j, line in enumerate((line1, line2)):
            draw_text(img, x0 + (cell - text_width(line)) // 2, 6 + icon + 6 + 10 * j, line, PURPLE)
    save(img, "eternal_items.png")


distance_grid()
eternal_items()
