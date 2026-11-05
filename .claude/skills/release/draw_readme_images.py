"""Draws two README pictures in the style of a Minecraft tooltip, into resources/showcase/: respawn_distance.png (the
respawn distance by difficulty and Eternal Items carried) and eternal_items.png (the five Eternal Items).

    python3 -I .claude/skills/release/draw_readme_images.py <vanilla ascii.png> [<repo root>]

The vanilla font comes out of the Loom client jar:
    unzip -p ~/.gradle/caches/fabric-loom/<minecraft_version>/minecraft-client.jar assets/minecraft/textures/font/ascii.png > <scratch>/ascii.png
The distance defaults and the cap of the formula are read from the code (BMHConfigLimits, RespawnPositionHandler); the
formula itself is copied in distance_grid(). The item list is in eternal_items(). Needs Pillow."""
import os
import re
import sys
from PIL import Image

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


def distance_grid():
    # RespawnPositionHandler.calculateDistanceFromBed with the default config.
    lo = java_int("utilities/BMHConfigLimits.java", "DEFAULT_MIN_DISTANCE_FROM_BED")
    hi = java_int("utilities/BMHConfigLimits.java", "DEFAULT_MAX_DISTANCE_FROM_BED")
    cap = java_int("handlers/RespawnPositionHandler.java", "MAX_EASY_LIFE")
    levels = [("Peaceful", 0), ("Easy", 1), ("Normal", 2), ("Hard", 3)]
    colors = [(85, 255, 85), (170, 255, 85), (255, 255, 85), (255, 170, 0), (255, 110, 40), (255, 85, 85)]
    label_w, col_w, row_h = 46, 40, 12
    w = 6 + label_w + col_w * 6 + 4
    h = 6 + 12 + 10 + 12 + row_h * 4 + 4 + 10 + 4
    img = panel(w, h)
    draw_text(img, 6, 6, "How far you respawn from home (blocks)", PURPLE)
    y = 6 + 12
    head = "Eternal Items you carry"
    draw_text(img, 6 + label_w + (col_w * 6 - text_width(head)) // 2, y + 2, head, GRAY)
    y += 10
    for k in range(6):
        s = str(k)
        draw_text(img, 6 + label_w + col_w * k + (col_w - text_width(s)) // 2, y + 2, s, GRAY)
    y += 12
    for name, diff in levels:
        draw_text(img, 6, y + 2, name, WHITE)
        for k in range(6):
            easy = min(1 + k + diff, cap)
            d = lo + (hi - lo) * easy // cap
            s = "{:,}".format(d)
            draw_text(img, 6 + label_w + col_w * k + (col_w - text_width(s)) // 2, y + 2, s, colors[min(5, (easy - 1) * 6 // 9)])
        y += row_h
    y += 4
    draw_text(img, 6, y, "With the default config of the mod.", GRAY)
    save(img, "respawn_distance.png")


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
