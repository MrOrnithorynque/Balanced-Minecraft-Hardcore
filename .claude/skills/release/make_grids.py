"""README recipe pictures for BMH, in the style of the existing ones (vanilla GUI at scale 4), drawn from the recipe files.

    python3 -I .claude/skills/release/make_grids.py <out dir> [<repo root>]

Pass `resources` as the out dir to replace the pictures the README shows. Everything it reads is in the repo or in the
Gradle cache, nothing to extract by hand:
  recipes   common/src/main/resources/data/bmh/recipes/ (recipe/ on 1.21.1). Every crafting_shaped file gives
            <result>_crafting_grid.png and every smithing_transform file <result>_smithing.png; other types are skipped.
  stamps    the slot, the result slot and the arrow are cut from resources/dormant_totem_crafting_grid.png (the first
            *_crafting_grid.png of the same size if that one is gone), so every picture keeps its exact look.
  textures  vanilla ones from the Loom client jar ~/.gradle/caches/fabric-loom/1.21.1/minecraft-client.jar (zipfile),
            ours from common/src/main/resources/assets/bmh/textures/item/. Icons are the 16x16 texture at 4x (nearest);
            a vanilla block is drawn as the GUI's isometric cube with one texture on every face.
The anvil step (Dormant Totem + Echo Heart -> Totem of Eternity) is no recipe file: its three items are listed below and
its cost is read from TotemAwakeningHandler.COST.
"""
import glob
import io
import json
import os
import re
import sys
import zipfile

from PIL import Image, ImageDraw

OUT = sys.argv[1]
ROOT = os.path.abspath(sys.argv[2]) if len(sys.argv) > 2 else os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
RES = os.path.join(ROOT, "common", "src", "main", "resources")
CLIENT_JAR = os.path.expanduser("~/.gradle/caches/fabric-loom/1.21.1/minecraft-client.jar")
BG = (198, 198, 198, 255)
SLOT_IN = (139, 139, 139, 255)
ANVIL = ("bmh:dormant_totem", "bmh:echo_heart", "bmh:totem_of_eternity")
ICON = {"minecraft:compass": "item/compass_16"}  # items whose texture is not item/<name>.png (the compass has 32 frames)

if not os.path.exists(CLIENT_JAR):
    sys.exit("no " + CLIENT_JAR + ": build the 1.21.1 branch once with Gradle, fabric-loom downloads it")
jar = zipfile.ZipFile(CLIENT_JAR)


def vanilla(path):
    return Image.open(io.BytesIO(jar.read("assets/minecraft/textures/" + path + ".png"))).convert("RGBA")


def stamps():
    wanted = os.path.join(ROOT, "resources", "dormant_totem_crafting_grid.png")
    others = sorted(glob.glob(os.path.join(ROOT, "resources", "*_crafting_grid.png")))
    for path in [wanted] + others:
        if os.path.exists(path) and Image.open(path).size == (484, 256):
            template = Image.open(path).convert("RGBA")
            break
    else:
        sys.exit("no 484x256 *_crafting_grid.png in resources/ to cut the stamps from")
    slot = template.crop((20, 20, 92, 92))
    ImageDraw.Draw(slot).rectangle((4, 4, 67, 67), fill=SLOT_IN)  # the icon that sat in the first slot
    result = template.crop((368, 80, 464, 176))
    ImageDraw.Draw(result).rectangle((4, 4, 91, 91), fill=SLOT_IN)
    return slot, result, template.crop((272, 102, 336, 154))


slot_stamp, result_stamp, arrow_stamp = stamps()


def item_id(entry):
    """The item id of a recipe ingredient or result: {"item": ...}, {"id": ...}, a bare string, or the first alternative."""
    if isinstance(entry, list):
        entry = entry[0]
    if isinstance(entry, str):
        return entry
    if "item" in entry or "id" in entry:
        return entry.get("item") or entry.get("id")
    sys.exit("an ingredient without a single item (%s): draw it by hand" % json.dumps(entry))


def texture(item):
    """(16x16 image, is it a block) for an item id."""
    namespace, name = item.split(":")
    if namespace == "bmh":
        im = Image.open(os.path.join(RES, "assets", "bmh", "textures", "item", name + ".png")).convert("RGBA")
        block = False
    elif item in ICON:
        im, block = vanilla(ICON[item]), False
    elif "assets/minecraft/textures/item/%s.png" % name in jar.namelist():
        im, block = vanilla("item/" + name), False
    else:
        im, block = vanilla("block/" + name), True
    if im.height > im.width:  # animated strip: first frame
        im = im.crop((0, 0, im.width, im.width))
    return im, block


def block_icon(tex, ss=4):
    """The GUI's isometric cube (rotation 30/225, scale 0.625), drawn texel by texel, supersampled."""
    size = 64 * ss
    w2, th, e = 28.28 * ss, 14.14 * ss, 34.64 * ss
    cx, y0 = 32 * ss, (size - (2 * th + e)) / 2
    T, R, C, L = (cx, y0), (cx + w2, y0 + th), (cx, y0 + 2 * th), (cx - w2, y0 + th)
    B, R2, L2 = (cx, y0 + 2 * th + e), (cx + w2, y0 + th + e), (cx - w2, y0 + th + e)
    im = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    faces = [  # origin, u axis, v axis, light
        (L, (T[0] - L[0], T[1] - L[1]), (C[0] - L[0], C[1] - L[1]), 1.0),   # top
        (L, (C[0] - L[0], C[1] - L[1]), (L2[0] - L[0], L2[1] - L[1]), 0.8),  # left (front)
        (C, (R[0] - C[0], R[1] - C[1]), (B[0] - C[0], B[1] - C[1]), 0.6),   # right
    ]
    px = tex.load()
    for (ox, oy), (ux, uy), (vx, vy), light in faces:
        for j in range(16):
            for i in range(16):
                r, g, b, a = px[i, j]
                if a == 0:
                    continue
                c = (int(r * light), int(g * light), int(b * light), 255)
                pts = []
                for du, dv in ((0, 0), (1, 0), (1, 1), (0, 1)):
                    u, v = (i + du) / 16, (j + dv) / 16
                    pts.append((ox + u * ux + v * vx, oy + u * uy + v * vy))
                d.polygon(pts, fill=c, outline=c)
    return im.resize((64, 64), Image.BOX)


def icon(item):
    tex, block = texture(item)
    return block_icon(tex) if block else tex.resize((64, 64), Image.NEAREST)


def put_slot(canvas, x, y, item=None):
    canvas.alpha_composite(slot_stamp, (x, y))
    if item:
        canvas.alpha_composite(icon(item), (x + 4, y + 4))


def put_result(canvas, x, y, item):
    canvas.alpha_composite(result_stamp, (x, y))
    canvas.alpha_composite(icon(item), (x + 16, y + 16))


def crafting(pattern, key, result, out):
    canvas = Image.new("RGBA", (484, 256), BG)
    for r, row in enumerate(pattern):
        for c, ch in enumerate(row.ljust(3)):
            put_slot(canvas, 20 + 72 * c, 20 + 72 * r, key.get(ch))
    canvas.alpha_composite(arrow_stamp, (272, 102))
    put_result(canvas, 368, 80, result)
    canvas.save(os.path.join(OUT, out))


def glyph_text(canvas, text, x, y, scale, color):
    """Vanilla's ascii.png font (8x8 cells), with its drop shadow (the color at a quarter)."""
    font = vanilla("font/ascii")
    shadow = tuple(c // 4 for c in color[:3]) + (255,)

    def width(ch):
        if ch == " ":
            return 4
        code = ord(ch); cell = font.crop(((code % 16) * 8, (code // 16) * 8, (code % 16) * 8 + 8, (code // 16) * 8 + 8))
        cols = [cx for cx in range(8) if any(cell.getpixel((cx, cy))[3] for cy in range(8))]
        return (max(cols) + 2) if cols else 4

    total = sum(width(ch) for ch in text) - 1
    x0 = x - total * scale // 2
    for pass_color, off in ((shadow, scale), (color, 0)):
        cx = x0
        for ch in text:
            if ch != " ":
                code = ord(ch)
                cell = font.crop(((code % 16) * 8, (code // 16) * 8, (code % 16) * 8 + 8, (code // 16) * 8 + 8))
                mask = cell.split()[3].resize((8 * scale, 8 * scale), Image.NEAREST)
                canvas.paste(Image.new("RGBA", mask.size, pass_color), (cx + off, y + off), mask)
            cx += width(ch) * scale


def smithing(template_item, base, addition, result, out):
    canvas = Image.new("RGBA", (484, 136), BG)
    for i, item in enumerate((template_item, base, addition)):
        put_slot(canvas, 20 + 72 * i, 32, item)
    canvas.alpha_composite(arrow_stamp, (272, 42))
    put_result(canvas, 368, 20, result)
    canvas.save(os.path.join(OUT, out))


def anvil(left, right, result, cost, out):
    canvas = Image.new("RGBA", (484, 160), BG)
    put_slot(canvas, 20, 32, left)
    put_slot(canvas, 164, 32, right)
    d = ImageDraw.Draw(canvas)  # the anvil's plus, in the arrow's grey, centred between the two slots
    d.rectangle((102, 62, 153, 73), fill=SLOT_IN)
    d.rectangle((122, 42, 133, 93), fill=SLOT_IN)
    canvas.alpha_composite(arrow_stamp, (272, 42))
    put_result(canvas, 368, 20, result)
    glyph_text(canvas, "Enchantment Cost: %d" % cost, 242, 124, 3, (128, 255, 32, 255))
    canvas.save(os.path.join(OUT, out))


os.makedirs(OUT, exist_ok=True)
recipes = [d for d in ("recipes", "recipe") if os.path.isdir(os.path.join(RES, "data", "bmh", d))]
if not recipes:
    sys.exit("no data/bmh/recipes or data/bmh/recipe under " + RES)
for path in sorted(glob.glob(os.path.join(RES, "data", "bmh", recipes[0], "*.json"))):
    recipe = json.load(open(path))
    kind, name = recipe["type"].replace("minecraft:", ""), os.path.basename(path)
    if kind == "crafting_shaped":
        result = item_id(recipe["result"])
        crafting(recipe["pattern"], {k: item_id(v) for k, v in recipe["key"].items()}, result,
                 result.split(":")[1] + "_crafting_grid.png")
    elif kind == "smithing_transform":
        result = item_id(recipe["result"])
        smithing(item_id(recipe["template"]), item_id(recipe["base"]), item_id(recipe["addition"]), result,
                 result.split(":")[1] + "_smithing.png")
    else:
        print("skipped", name, "(" + kind + ")")
cost_source = open(os.path.join(ROOT, "common", "src", "main", "java", "github", "mrornithorynque", "bmh", "handlers",
                                "TotemAwakeningHandler.java")).read()
anvil(*ANVIL, int(re.search(r"\bCOST\s*=\s*(\d+)", cost_source).group(1)), ANVIL[2].split(":")[1] + "_anvil.png")
print("written:", sorted(os.listdir(OUT)))
