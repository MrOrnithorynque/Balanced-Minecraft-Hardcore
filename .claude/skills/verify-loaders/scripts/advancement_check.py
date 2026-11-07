#!/usr/bin/env python3
"""Checks the BMH advancement data of a tree. No Gradle, no Minecraft.
usage: python3 -I advancement_check.py [tree root]   (default: the tree that holds this script)
It checks:
  - the data folder fits the version: advancements/ before 1.21, advancement/ from 1.21 (the wrong one is silently ignored)
  - every JSON file parses, has a display, criteria and requirements, and every requirement names a criterion
  - the icon field fits the version: "item" before 1.20.5, "id" after
  - every parent exists, and one root has the tick trigger
  - every title and description key exists in all ten lang files
  - every path given to BmhAdvancements.award(...) in the Java code is an "impossible" advancement
  - every "impossible" advancement is awarded by the code (the list WIRED_LATER below is the exception, and is reported)
Exit status 0 when all is well, 1 on any problem.
"""
import json, os, re, sys

# Awarded by the follow-up that wires the homecoming feature, not by this feature.
WIRED_LATER = {"journey/home_at_last", "journey/long_way_home"}

tree = os.path.abspath(sys.argv[1]) if len(sys.argv) > 1 else os.path.abspath(
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "../../../.."))
props = dict(l.strip().split("=", 1) for l in open(os.path.join(tree, "gradle.properties")) if "=" in l and not l.startswith("#"))
mcv = props["minecraft_version"].strip()
num = [int(x) for x in re.findall(r"\d+", mcv)]
modern_folder = mcv.startswith("26.") or (num[0] == 1 and num[1] >= 21)
icon_id = mcv.startswith("26.") or (num[0] == 1 and (num[1] > 20 or (num[1] == 20 and len(num) > 2 and num[2] >= 5)))

data = os.path.join(tree, "common/src/main/resources/data/bmh")
right, wrong = ("advancement", "advancements") if modern_folder else ("advancements", "advancement")
problems, notes = [], []
if os.path.isdir(os.path.join(data, wrong)):
    problems.append("folder data/bmh/%s exists, but Minecraft %s reads data/bmh/%s" % (wrong, mcv, right))
base = os.path.join(data, right)
found = {}
for root, _, files in os.walk(base):
    for f in files:
        if not f.endswith(".json"):
            continue
        full = os.path.join(root, f)
        path = os.path.relpath(full, base)[:-5].replace(os.sep, "/")
        try:
            found[path] = json.load(open(full, encoding="utf-8"))
        except Exception as e:
            problems.append("%s does not parse: %s" % (path, e))
if not found:
    problems.append("no advancement under %s" % base)

langs = {}
lang_dir = os.path.join(tree, "common/src/main/resources/assets/bmh/lang")
for f in sorted(os.listdir(lang_dir)):
    if f.endswith(".json"):
        langs[f] = json.load(open(os.path.join(lang_dir, f), encoding="utf-8"))
if len(langs) != 10:
    problems.append("%d lang files, expected 10" % len(langs))

ticks = 0
for path, adv in found.items():
    d = adv.get("display")
    if not d:
        problems.append("%s: no display" % path)
        continue
    crit = adv.get("criteria", {})
    reqs = adv.get("requirements", [])
    if not crit:
        problems.append("%s: no criteria" % path)
    for group in reqs:
        for name in group:
            if name not in crit:
                problems.append("%s: requirement %s is not a criterion" % (path, name))
    for name in crit:
        if not any(name in g for g in reqs):
            problems.append("%s: criterion %s is in no requirement" % (path, name))
    ic = d.get("icon", {})
    want, other = ("id", "item") if icon_id else ("item", "id")
    if want not in ic or other in ic:
        problems.append("%s: icon must use the field \"%s\" on Minecraft %s, got %s" % (path, want, mcv, list(ic)))
    parent = adv.get("parent")
    if parent is None:
        if path != "root":
            problems.append("%s: no parent and not the root" % path)
        if any(c.get("trigger") == "minecraft:tick" for c in crit.values()):
            ticks += 1
    elif not parent.startswith("bmh:") or parent[4:] not in found:
        problems.append("%s: parent %s does not exist" % (path, parent))
    for part in ("title", "description"):
        t = d.get(part, {}).get("translate")
        if not t:
            problems.append("%s: %s is not a translate component" % (path, part))
            continue
        for lang, table in langs.items():
            if not table.get(t):
                problems.append("%s: key %s is missing or empty in %s" % (path, t, lang))
if ticks != 1:
    problems.append("%d root advancements with the tick trigger, expected 1" % ticks)

# the awards in the code
awarded = set()
src = os.path.join(tree, "common/src/main/java")
for root, _, files in os.walk(src):
    for f in files:
        if f.endswith(".java") and f != "BmhAdvancements.java":
            for m in re.finditer(r'BmhAdvancements\.award\([^,]+,\s*"([^"]+)"\)', open(os.path.join(root, f), encoding="utf-8").read()):
                awarded.add(m.group(1))
impossible = {p for p, a in found.items() if any(c.get("trigger") == "minecraft:impossible" for c in a.get("criteria", {}).values())}
for p in sorted(awarded - impossible):
    problems.append("the code awards %s, which is not an impossible advancement" % p)
for p in sorted(impossible - awarded - WIRED_LATER):
    problems.append("%s has the impossible trigger and no code awards it" % p)
for p in sorted((impossible & WIRED_LATER) - awarded):
    notes.append("%s is not awarded yet (wired later)" % p)

print("advancement_check %s: Minecraft %s, folder %s, %d advancements (%d by code, %d by inventory), %d lang files" % (
    tree, mcv, right, len(found), len(impossible), len(found) - len(impossible) - 1, len(langs)))
for n in notes:
    print("  note:", n)
for p in problems:
    print("  PROBLEM:", p)
print("advancement_check:", "FAIL" if problems else "OK")
sys.exit(1 if problems else 0)
