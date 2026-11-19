#!/usr/bin/env python3
"""Check the four Camp glyphs at the spyglass label's GUI position."""

import sys
from pathlib import Path

from PIL import Image


def glyphs(resources, name):
    path = resources / "assets/minecraft/textures/font/ascii.png"
    with Image.open(path) as image:
        atlas = image.convert("RGBA").getchannel("A")
    cells = []
    advance = 0
    for char in name:
        x = ord(char) % 16 * 8
        y = ord(char) // 16 * 8
        cell = atlas.crop((x, y, x + 8, y + 8))
        width = cell.getbbox()[2]
        cells.append((advance, cell.crop((0, 0, width + 1, 8))))
        advance += width + 1
    return cells, advance


def match(image, gui_size, cells, width):
    gui_w, gui_h = gui_size
    sx, sy = image.width / gui_w, image.height / gui_h

    def amethyst(x, y):
        r, g, b = image.getpixel((int((x + 0.5) * sx), int((y + 0.5) * sy)))
        return b > 135 and b - g >= 40 and r - g >= 12 and b - r >= 20

    def score(left, top):
        scores = []
        for advance, cell in cells:
            foreground = correct = background = clear = 0
            for y in range(cell.height):
                for x in range(cell.width):
                    ink = amethyst(left + advance + x, top + y)
                    if cell.getpixel((x, y)):
                        foreground += 1
                        correct += ink
                    else:
                        background += 1
                        clear += not ink
            scores.append(min(correct / foreground, clear / background))
        return min(scores)

    left = gui_w // 2 - width // 2
    top = gui_h // 2 + 16
    # The GUI and the screenshot can round their origins to different pixels.
    return max(score(left + dx, top + dy) for dx in range(-2, 3) for dy in range(-2, 3))


def main():
    screenshot, resources, size = sys.argv[1:]
    with Image.open(screenshot) as img:
        rgb = img.convert("RGB")
    cells, width = glyphs(Path(resources), "Camp")
    confidence = match(rgb, tuple(map(int, size.split())), cells, width)
    print(f"Camp glyph match (weakest letter): {confidence:.0%}")
    return 0 if confidence >= 0.80 else 1


if __name__ == "__main__":
    sys.exit(main())
