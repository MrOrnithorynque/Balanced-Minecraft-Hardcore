"""Tiny pixel-art toolkit: sprites are 16 strings of 16 palette keys, '.' is transparent."""
from PIL import Image, ImageDraw, ImageFont

def hexrgba(h, a=255):
    h = h.lstrip('#')
    return (int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16), a)

# Shared palette. Eternal family: netherite black, gold, echo cyan. Mortal: copper, amethyst.
PAL = {
    '.': (0, 0, 0, 0),
    # outlines
    'o': hexrgba('140e14'),   # cold near-black (eternal)
    'O': hexrgba('2a1608'),   # warm brown-black (copper, leather)
    # netherite black
    'k': hexrgba('251d24'),
    'K': hexrgba('3a2f38'),
    'h': hexrgba('5a4c56'),
    'H': hexrgba('7d6c78'),
    # gold
    'e': hexrgba('5c3808'),
    'g': hexrgba('a2691a'),
    'G': hexrgba('dba52a'),
    'Y': hexrgba('f7d95c'),
    'y': hexrgba('fff4b8'),
    # echo cyan
    'd': hexrgba('0b4a52'),
    'c': hexrgba('16a3b0'),
    'C': hexrgba('5fe8ee'),
    'W': hexrgba('e4ffff'),
    # grip leather
    'r': hexrgba('2a1a16'),
    'R': hexrgba('4c3029'),
    's': hexrgba('6f4a3b'),
    # bundle leather
    'l': hexrgba('2c1d19'),
    'L': hexrgba('412b23'),
    'm': hexrgba('5b3d30'),
    'M': hexrgba('7a5444'),
    # copper
    'n': hexrgba('6b3018'),
    'p': hexrgba('a8582c'),
    'P': hexrgba('d6844a'),
    'Q': hexrgba('f3b47c'),
    # amethyst
    'a': hexrgba('3a1c64'),
    'A': hexrgba('7a49c8'),
    'V': hexrgba('b58cff'),
    'v': hexrgba('f0e4ff'),
    # needle
    'x': hexrgba('8c1c1a'),
    'X': hexrgba('d8392f'),
    'z': hexrgba('ece6da'),
    # cord
    'u': hexrgba('3b2a20'),
    'U': hexrgba('6a5545'),
}

def sprite(rows, name='?'):
    assert len(rows) == 16, f'{name}: {len(rows)} rows'
    for i, r in enumerate(rows):
        assert len(r) == 16, f'{name}: row {i} has {len(r)} chars: {r!r}'
        for ch in r:
            assert ch in PAL, f'{name}: unknown key {ch!r} in row {i}'
    return rows

def to_image(rows, pal=PAL):
    im = Image.new('RGBA', (16, 16))
    px = im.load()
    for y, r in enumerate(rows):
        for x, ch in enumerate(r):
            px[x, y] = pal[ch]
    return im

def grid_from(rows):
    return [list(r) for r in rows]

def rows_from(grid):
    return [''.join(r) for r in grid]

def slot_bg(scale):
    """Inventory-slot backdrop: mid grey with a bevel, like a container slot."""
    n = 16 * scale
    pad = scale
    im = Image.new('RGBA', (n + 2 * pad, n + 2 * pad), hexrgba('8b8b8b'))
    d = ImageDraw.Draw(im)
    d.rectangle([0, 0, n + 2 * pad - 1, pad - 1], fill=hexrgba('373737'))
    d.rectangle([0, 0, pad - 1, n + 2 * pad - 1], fill=hexrgba('373737'))
    d.rectangle([0, n + pad, n + 2 * pad - 1, n + 2 * pad - 1], fill=hexrgba('ffffff'))
    d.rectangle([n + pad, 0, n + 2 * pad - 1, n + 2 * pad - 1], fill=hexrgba('ffffff'))
    return im

def on_slot(img, scale):
    bg = slot_bg(scale)
    bg.alpha_composite(img.resize((16 * scale, 16 * scale), Image.NEAREST), (scale, scale))
    return bg

def on_color(img, scale, color):
    bg = Image.new('RGBA', (16 * scale, 16 * scale), hexrgba(color))
    bg.alpha_composite(img.resize((16 * scale, 16 * scale), Image.NEAREST))
    return bg

def big_with_grid(img, scale=24):
    """Upscaled sprite over a checkerboard with a faint pixel grid, for close critique."""
    n = 16 * scale
    bg = Image.new('RGBA', (n, n))
    d = ImageDraw.Draw(bg)
    for y in range(16):
        for x in range(16):
            c = hexrgba('b9b9b9') if (x + y) % 2 == 0 else hexrgba('a9a9a9')
            d.rectangle([x * scale, y * scale, x * scale + scale - 1, y * scale + scale - 1], fill=c)
    bg.alpha_composite(img.resize((n, n), Image.NEAREST))
    d = ImageDraw.Draw(bg)
    for i in range(17):
        d.line([(i * scale, 0), (i * scale, n)], fill=(0, 0, 0, 40))
        d.line([(0, i * scale), (n, i * scale)], fill=(0, 0, 0, 40))
    return bg

def font(size=14):
    for f in ['/System/Library/Fonts/Menlo.ttc', '/System/Library/Fonts/SFNSMono.ttf']:
        try:
            return ImageFont.truetype(f, size)
        except Exception:
            pass
    return ImageFont.load_default()

def review_sheet(items, path, scale=20):
    """One row per sprite: big gridded view, then 1x, 2x, 3x on slots, then on dark and light grounds."""
    f = font(16)
    rowh = 16 * scale + 40
    w = 16 * scale + 30 + (16 + 2) + 12 + (32 + 4) + 12 + (48 + 6) + 12 + 64 + 12 + 64 + 40
    w = max(w, 900)
    sheet = Image.new('RGBA', (w, rowh * len(items) + 10), hexrgba('2b2b2b'))
    d = ImageDraw.Draw(sheet)
    for i, (name, img) in enumerate(items):
        y = i * rowh + 30
        d.text((10, y - 24), name, fill=(230, 230, 230, 255), font=f)
        sheet.alpha_composite(big_with_grid(img, scale), (10, y))
        x = 16 * scale + 30
        for s in (1, 2, 3):
            t = on_slot(img, s)
            sheet.alpha_composite(t, (x, y))
            x += t.size[0] + 12
        for col in ('1d1d1d', 'e8e8e8'):
            sheet.alpha_composite(on_color(img, 4, col), (x, y))
            x += 64 + 12
    sheet.save(path)
    return sheet.size

def strip(frames):
    im = Image.new('RGBA', (16, 16 * len(frames)))
    for i, fr in enumerate(frames):
        im.alpha_composite(fr, (0, 16 * i))
    return im

def gif(frames, path, scale=8, durations=None, bg='8b8b8b'):
    out = [on_color(fr, scale, bg).convert('RGB') for fr in frames]
    durations = durations or [100] * len(frames)
    out[0].save(path, save_all=True, append_images=out[1:], duration=durations, loop=0, disposal=2)
