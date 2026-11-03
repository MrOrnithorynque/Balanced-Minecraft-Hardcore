"""Part F of the redesign: builds the four item textures and their review sheets.

    python3 docs/redesign/sprites/make_sprites.py            # textures + sheets
    python3 docs/redesign/sprites/make_sprites.py --blind D  # also unlabeled renders for a blind read, into folder D

Louis's originals are read from git at the 1.4.0 release (5062a5e), so a re-run never stacks edits on edits.
Every texture is 16x16 RGBA, opaque or fully transparent pixels only, no .mcmeta (all four are static).
"""
import io
import subprocess
import sys
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'docs/redesign/assets/px'))
import px  # noqa: E402

ORIGINALS = '5062a5e'
TEX = 'common/src/main/resources/assets/bmh/textures/item/'
SHEETS = ROOT / 'docs/redesign/sprites'
h = px.hexrgba


def original(name):
    data = subprocess.run(['git', '-C', str(ROOT), 'show', f'{ORIGINALS}:{TEX}{name}'],
                          check=True, capture_output=True).stdout
    return Image.open(io.BytesIO(data)).convert('RGBA')


def paint(img, pixels):
    im = img.copy()
    p = im.load()
    for (x, y), c in pixels.items():
        p[x, y] = c
    return im


# Eternal Bundle: Louis's two textures plus the eternity ring, nothing else.
# A 4x4 gold ring, corners cut, lit from the top left; '.' keeps Louis's pixel (the hole shows his leather).
RING = ['.YG.',
        'Y..g',
        'G..g',
        '.ge.']
RING_GOLD = {'Y': h('f7d95c'), 'G': h('dba52a'), 'g': h('a2691a'), 'e': h('5c3808')}


def with_ring(img, ox, oy):
    return paint(img, {(ox + x, oy + y): RING_GOLD[ch]
                       for y, row in enumerate(RING) for x, ch in enumerate(row) if ch != '.'})


def eternal_bundle():
    # Under the knot of the drawstring (the light pixel at 9,11); the cord's two loose ends meet the ring's shoulders.
    return with_ring(original('eternal_bundle.png'), 8, 12)


def eternal_bundle_filled():
    # Under the end of the cord hanging from the neck (9,9), same columns as the empty one so the ring stays put.
    return with_ring(original('eternal_bundle_filled.png'), 8, 10)


# Dormant Totem: frame 0 of Louis's Totem of Eternity, asleep. Only the forehead gem and the eyes change:
# the gem loses its light (dull violet-grey, no white spark), each eye closes (lid in the face's own grey over a
# dark lash line that keeps a trace of the eye's green or cyan). Everything else is his pixels.
LID = h('4d4143')  # the face colour around the eyes in his texture


def dormant_totem():
    f0 = original('totem_of_eternity.png').crop((0, 0, 16, 16))
    return paint(f0, {
        (7, 2): h('5e5268'), (8, 2): h('4f4560'), (7, 3): h('4f4560'), (8, 3): h('3a3148'),
        (5, 5): LID, (6, 5): LID, (5, 6): h('1f3a28'), (6, 6): h('1f3a28'),
        (9, 5): LID, (10, 5): LID, (9, 6): h('1c3836'), (10, 6): h('1c3836'),
    })


# Echo Heart: a cut crystal heart in crying-obsidian violets (vanilla crying obsidian runs #2b0178, #6006ab, #8308e4
# over near-black), lit from the top left: one bright crown facet and a white spark on the left lobe, a dark
# pavilion below with bevel rims so the point survives dark grounds, and a four-point echo-cyan glint at its heart.
HEART_PAL = {
    '.': (0, 0, 0, 0),
    'o': h('120818'),
    '1': h('2b0e55'), '2': h('3c1470'), '3': h('4d1890'), '4': h('6a2cc4'),
    '5': h('9056e4'), '6': h('c6a2ff'), 'S': h('f6eeff'),
    't': h('145a66'), 'c': h('16a3b0'), 'C': h('5fe8ee'), 'W': h('e4ffff'),
}
HEART = [
    '................',
    '................',
    '...oooo..oooo...',
    '..o5S66oo5544o..',
    '.o5S666t554444o.',
    '.o55566c544444o.',
    '.o5555cCc44443o.',
    '.o33tcCWCct222o.',
    '..o433cCc2223o..',
    '...o422c1113o...',
    '....o42t113o....',
    '.....o4213o.....',
    '......o23o......',
    '.......oo.......',
    '................',
    '................',
]


def echo_heart():
    assert len(HEART) == 16 and all(len(r) == 16 for r in HEART)
    return px.to_image(HEART, HEART_PAL)


SPRITES = {
    'eternal_bundle.png': eternal_bundle,
    'eternal_bundle_filled.png': eternal_bundle_filled,
    'dormant_totem.png': dormant_totem,
    'echo_heart.png': echo_heart,
}


# Sheets ---------------------------------------------------------------------------------------------------------

def review_sheet(items, path, scale=16):
    """One row per sprite: gridded close-up, then 1x, 2x, 4x on an inventory slot, then 4x on dark and light."""
    f = px.font(15)
    cell_h = 16 * scale + 34
    w = 16 * scale + 20 + 30 + 48 + 84 + 76 * 2 + 20
    sheet = Image.new('RGBA', (w, cell_h * len(items) + 10), h('2b2b2b'))
    d = ImageDraw.Draw(sheet)
    for i, (name, img) in enumerate(items):
        y = 28 + i * cell_h
        d.text((10, y - 22), name, fill=(230, 230, 230, 255), font=f)
        sheet.alpha_composite(px.big_with_grid(img, scale), (10, y))
        x = 10 + 16 * scale + 20
        for s in (1, 2, 4):
            t = px.on_slot(img, s)
            sheet.alpha_composite(t, (x, y))
            x += t.size[0] + 12
        for col in ('1d1d1d', 'e8e8e8'):
            sheet.alpha_composite(px.on_color(img, 4, col), (x, y))
            x += 64 + 12
    sheet.save(path)


def before_after(pairs, path, scale=10):
    """Original and new side by side, close up and at 2x on a slot."""
    f = px.font(15)
    n = 16 * scale
    cell_h = n + 40
    sheet = Image.new('RGBA', (2 * n + 30 + 2 * 44 + 40, cell_h * len(pairs) + 10), h('2b2b2b'))
    d = ImageDraw.Draw(sheet)
    for i, (name, a, b) in enumerate(pairs):
        y = 30 + i * cell_h
        d.text((10, y - 24), name, fill=(230, 230, 230, 255), font=f)
        sheet.alpha_composite(px.big_with_grid(a, scale), (10, y))
        sheet.alpha_composite(px.big_with_grid(b, scale), (20 + n, y))
        sheet.alpha_composite(px.on_slot(a, 2), (30 + 2 * n, y))
        sheet.alpha_composite(px.on_slot(b, 2), (30 + 2 * n + 44, y))
    sheet.save(path)


def lineup(imgs, scale, bg, gap=8):
    im = Image.new('RGBA', (len(imgs) * (16 * scale + gap) + gap, 16 * scale + 2 * gap), h(bg))
    for i, s in enumerate(imgs):
        im.alpha_composite(s.resize((16 * scale, 16 * scale), Image.NEAREST), (gap + i * (16 * scale + gap), gap))
    return im


def slot_row(imgs, scale):
    tiles = [px.on_slot(s, scale) for s in imgs]
    im = Image.new('RGBA', (sum(t.size[0] for t in tiles) + 8 * (len(tiles) + 1), tiles[0].size[1] + 16), h('c6c6c6'))
    x = 8
    for t in tiles:
        im.alpha_composite(t, (x, 8))
        x += t.size[0] + 8
    return im


def family_sheet(imgs, path):
    """The new sprites next to the awakened totem, at the scales a GUI uses, on slots and on dark and light grounds."""
    rows = [slot_row(imgs, s) for s in (1, 2, 3, 4)] + [lineup(imgs, 4, '1d1d1d'), lineup(imgs, 4, 'e8e8e8')]
    sheet = Image.new('RGBA', (max(r.size[0] for r in rows) + 20, sum(r.size[1] + 10 for r in rows) + 10), h('2b2b2b'))
    y = 10
    for r in rows:
        sheet.alpha_composite(r, (10, y))
        y += r.size[1] + 10
    sheet.save(path)


def main(argv):
    made = {name: build() for name, build in SPRITES.items()}
    for name, img in made.items():
        assert img.size == (16, 16)
        assert set(img.getchannel('A').tobytes()) <= {0, 255}, name
        img.info = {}  # drop the ICC profile the totem strip carries: plain sRGB pixels like the other textures
        img.save(ROOT / TEX / name)
    totem = original('totem_of_eternity.png').crop((0, 0, 16, 16))
    review_sheet([('eternal_bundle.png', made['eternal_bundle.png']),
                  ('eternal_bundle_filled.png', made['eternal_bundle_filled.png']),
                  ('dormant_totem.png', made['dormant_totem.png']),
                  ('echo_heart.png', made['echo_heart.png'])], SHEETS / 'review-sheet.png')
    before_after([('eternal_bundle: original | new', original('eternal_bundle.png'), made['eternal_bundle.png']),
                  ('eternal_bundle_filled: original | new', original('eternal_bundle_filled.png'),
                   made['eternal_bundle_filled.png']),
                  ('totem_of_eternity frame 0 | dormant_totem', totem, made['dormant_totem.png'])],
                 SHEETS / 'before-after.png')
    family_sheet([made['dormant_totem.png'], made['echo_heart.png'], totem, made['eternal_bundle.png'],
                  made['eternal_bundle_filled.png']], SHEETS / 'family.png')
    if '--blind' in argv:
        out = Path(argv[argv.index('--blind') + 1])
        out.mkdir(parents=True, exist_ok=True)
        for i, name in enumerate(['echo_heart.png', 'dormant_totem.png', 'eternal_bundle.png',
                                  'eternal_bundle_filled.png'], 1):
            img = made[name]
            tiles = [px.on_slot(img, s) for s in (1, 2, 4)] + [px.on_color(img, 4, c) for c in ('1d1d1d', 'e8e8e8')]
            sheet = Image.new('RGBA', (sum(t.size[0] for t in tiles) + 12 * 6, 90), h('2b2b2b'))
            x = 12
            for t in tiles:
                sheet.alpha_composite(t, (x, 12))
                x += t.size[0] + 12
            sheet.save(out / f'sprite-{i}.png')
        lineup([totem, made['dormant_totem.png']], 6, '8b8b8b').save(out / 'pair.png')
    print('wrote', ', '.join(made))


if __name__ == '__main__':
    main(sys.argv[1:])
