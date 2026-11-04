"""Contact sheet of screenshots: sheet.py <out.png> <columns> <cell width> <label> <image> [<label> <image> ...]
(a missing image gives a red cell). Needs Pillow. Use it to look at the quick-play client frames of every loader at once."""
import os, sys
from PIL import Image, ImageDraw, ImageFont
out, cols, cw = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
pairs = list(zip(sys.argv[4::2], sys.argv[5::2]))
ch = cw * 9 // 16; lh = 34
try:
    font = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 22)  # macOS
except OSError:
    font = ImageFont.load_default()
rows = (len(pairs) + cols - 1) // cols
sheet = Image.new("RGB", (cols * (cw + 8) + 8, rows * (ch + lh + 8) + 8), (32, 32, 36))
d = ImageDraw.Draw(sheet)
for i, (label, path) in enumerate(pairs):
    x = 8 + (i % cols) * (cw + 8); y = 8 + (i // cols) * (ch + lh + 8)
    d.text((x + 4, y + 5), label, font=font, fill=(235, 235, 235))
    if os.path.exists(path):
        sheet.paste(Image.open(path).convert("RGB").resize((cw, ch), Image.LANCZOS), (x, y + lh))
    else:
        d.rectangle((x, y + lh, x + cw, y + lh + ch), fill=(70, 30, 30)); d.text((x + 10, y + lh + 10), "no frame", font=font, fill=(255, 200, 200))
sheet.save(out); print(sheet.size)
