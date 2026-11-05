"""Writes docs/curseforge-description.md from README.md. The README is the source of truth: the CurseForge page shows
the same text. CurseForge does not resolve relative paths and drops most HTML, so the GitHub header (banner, badges)
becomes the banner alone, and every <img> becomes a markdown image with an absolute URL.

    python3 -I .claude/skills/release/readme_to_curseforge.py <repo root>

Paste the result into the project description on CurseForge (Description, then the markdown editor) after each change.
"""
import os
import re
import sys

RAW = "https://raw.githubusercontent.com/MrOrnithorynque/Balanced-Minecraft-Hardcore/main/"

root = sys.argv[1] if len(sys.argv) > 1 else "."
text = open(os.path.join(root, "README.md"), encoding="utf-8").read()

# The GitHub header ends at the second </h1>: the banner image, the title and the badges. The first </h1> is a stray tag
# at the end of the banner line; if someone removes it, this stops with the message below instead of cutting too much.
end = text.find("</h1>", text.find("</h1>") + 1)
if end < 0:
    sys.exit("README.md: no header that ends with two </h1>")
text = "![Balanced Minecraft Hardcore](%sresources/bmh_banner.png)\n" % RAW + text[end + len("</h1>"):]


def image(match):
    tag = match.group(0)
    src = re.search(r'src="([^"]+)"', tag).group(1)
    alt = re.search(r'alt="([^"]*)"', tag)
    url = src if src.startswith("http") else RAW + src
    return "![%s](%s)" % (alt.group(1) if alt else "", url)


text = re.sub(r"<img [^>]*>", image, text)
# Each <p> and </p> line becomes a blank line, so each <p> block stays a paragraph of its own (its images side by side).
text = re.sub(r"^[ \t]*</?p[^>]*>[ \t]*$", "", text, flags=re.M)
text = re.sub(r"^[ \t]+(!\[)", r"\1", text, flags=re.M)
text = re.sub(r"\n{3,}", "\n\n", text).strip() + "\n"

out = os.path.join(root, "docs", "curseforge-description.md")
open(out, "w", encoding="utf-8").write(text)
print("wrote", out, len(text), "characters")
