"""Check of the release jars of one BMH branch, without a game (branch.sh runs it after the build).

usage: python3 -I jarcheck.py <abs worktree> <minecraft version> <abs JAVA_HOME>
Checks every loader jar the branch built: mixin configs and classes, the refmap a production install needs (Forge 1.20.x
SRG names, Fabric intermediary names), the shadow fields renamed in the shipped classes, assets and data in the right
version folders, and that no throwaway test class shipped. Prints PASS/FAIL lines, then "ITEMS ...", "RECIPES ..." and
"JARS <passed>/<total>". Exit status 0 when every check passed, 1 otherwise (a failed check, or no jar at all): the release
skill's publish_local.py stops on it.

A mixin with by-name targets or a @Shadow field needs its runtime names below (REFMAP_IDS, SHADOW_FIELDS): find them
with the add-mixin skill's step 4 (unzip the refmap, javap -p the shipped class) and add them here.
"""
import glob
import json
import os
import re
import subprocess
import sys
import tempfile
import zipfile

W, MCV, JH = sys.argv[1], sys.argv[2], sys.argv[3]
V121 = MCV.startswith("1.21")
results = []

# Members the hand-overs of parts A, B and E name for a production install (stable ids across 1.20 to 1.21.1).
REFMAP_IDS = {
    "forge": {"A_LivingEntityTotemMixin": ["m_21262_"], "B_AnvilMenuMixin": ["m_6640_"],
              "E_PlayerAttackMixin": ["m_5706_", "m_6469_", "m_36399_"],
              "F_PlayerListRespawnMixin": ["m_11236_"], "F_ServerPlayerAccessor": ["f_8921_"],
              "F_GuiTitleMixin": ["m_168714_"], "F_LocalPlayerRespawnMixin": ["m_7583_"],
              "F_MinecraftOverlayMixin": ["m_91150_"], "G_ItemEntityMixin": ["m_6469_", "m_8119_"], "H_ItemRendererSwordMixin": ["m_115143_"], "I_SlotTotemMixin": ["m_5857_"], "I_ClientPacketListenerTotemMixin": ["m_104927_"],
              "K_GuiSpyglassMixin": ["m_280278_"]},
    "fabric": {"A_LivingEntityTotemMixin": ["method_6095"], "B_AnvilMenuMixin": ["method_24928"],
               "E_PlayerAttackMixin": ["method_7324", "method_5643", "method_7322"],
               "F_PlayerListRespawnMixin": ["method_14556"], "F_ServerPlayerAccessor": ["field_13998"],
               "F_GuiTitleMixin": ["method_34004"], "F_LocalPlayerRespawnMixin": ["method_7331"],
               "F_MinecraftOverlayMixin": ["method_18502"], "G_ItemEntityMixin": ["method_5643", "method_5773"], "H_ItemRendererSwordMixin": ["method_23179"], "I_SlotTotemMixin": ["method_7680"], "I_ClientPacketListenerTotemMixin": ["method_19691"],
               "J_CommandsMixin": ["class_2170;<init>"], "K_GuiSpyglassMixin": ["method_32598"]},
}
# A mixin whose only target is a constructor of a vanilla class has nothing to remap on Forge 1.20.x: Forge keeps the
# official class names there (only methods and fields get SRG names), so its refmap has no entry for it. Fabric renames
# the classes, so its refmap has one (REFMAP_IDS above).
CONSTRUCTOR_ONLY = {"J_CommandsMixin"}
# The distant respawn hook's other target differs per codebase: noCollision on 1.20.x, findRespawnPositionAndUseSpawnBlock
# on 1.21.1 (Fabric only there: Forge 1.21.1 ships no refmap).
if V121:
    REFMAP_IDS["fabric"]["F_PlayerListRespawnMixin"].append("method_60590")
else:
    REFMAP_IDS["forge"]["F_PlayerListRespawnMixin"].append("m_45786_")
    REFMAP_IDS["fabric"]["F_PlayerListRespawnMixin"].append("method_17892")
# The last point of an Eternal tool: ItemStack.hurt on 1.20.x, the player and slot hurtAndBreak methods on 1.21.1
# (Fabric only there: Forge 1.21.1 ships no refmap).
if V121:
    REFMAP_IDS["fabric"]["L_ItemStackWearMixin"] = ["method_7956", "method_7970"]
else:
    REFMAP_IDS["forge"]["L_ItemStackWearMixin"] = ["m_220157_"]
    REFMAP_IDS["fabric"]["L_ItemStackWearMixin"] = ["method_7970"]
SHADOW_FIELDS = {
    "forge": {"B_AnvilMenuMixin": ["f_39000_", "f_39002_"], "E_ServerPlayerGameModeMixin": ["f_9245_"]},
    "fabric": {"B_AnvilMenuMixin": ["field_7776", "field_7770"], "E_ServerPlayerGameModeMixin": ["field_14008"]},
}
# A mixin that only adds a method overriding a vanilla one has no refmap entry: the build renames the method
# inside the shipped class (I_ContainerTotemMixin adds Container.canPlaceItem). Its runtime name must show in javap.
ADDED_METHODS = {
    "forge": {"I_ContainerTotemMixin": ["m_7013_"]},
    "fabric": {"I_ContainerTotemMixin": ["method_5437"]},
}
# A throwaway hook class (selftest.sh, branch.sh, death_client.sh): any selftest folder, any Z_ mixin, the client hooks
THROWAWAY = re.compile(r"(selftest|devskyshots|smokeclient|journeysmoke|/mixin/Z_)", re.IGNORECASE)


def check(name, ok, detail=""):
    results.append(ok)
    print(("PASS " if ok else "FAIL ") + name + (("  [" + str(detail)[:400] + "]") if detail else ""), flush=True)


def release_jar(loader):
    jars = [j for j in glob.glob(os.path.join(W, loader, "build", "libs", "*.jar"))
            if not re.search(r"-(sources|dev|dev-shadow|javadoc|slim)\.jar$", j)]
    return jars


def mixin_names(z, config):
    try:
        data = json.loads(z.read(config))
    except KeyError:
        return None, []
    names = data.get("mixins", []) + data.get("client", []) + data.get("server", [])
    return data.get("package", ""), names


def javap_fields(z, entry):
    with tempfile.TemporaryDirectory() as tmp:
        path = os.path.join(tmp, "M.class")
        with open(path, "wb") as f:
            f.write(z.read(entry))
        out = subprocess.run([os.path.join(JH, "bin", "javap"), "-p", path], capture_output=True, text=True).stdout
    return out


def check_jar(loader, jar):
    tag = loader + " " + os.path.basename(jar)
    z = zipfile.ZipFile(jar)
    names = set(z.namelist())

    # Mixin configs and their classes
    configs = ["bmh.mixins.json"] + (["bmh.fabric.mixins.json"] if loader == "fabric" else [])
    all_mixins = []
    for config in configs:
        package, mixins = mixin_names(z, config)
        check(tag + ": " + config + " in the jar", package is not None)
        for m in mixins:
            entry = package.replace(".", "/") + "/" + m.replace(".", "/") + ".class"
            check(tag + ": mixin class " + m, entry in names)
            all_mixins.append((m, entry))
    leftovers = sorted(n for n in names if THROWAWAY.search(n))
    check(tag + ": no throwaway test class shipped", not leftovers, leftovers)

    # How the loader finds the mixins
    if loader == "forge":
        manifest = z.read("META-INF/MANIFEST.MF").decode("utf-8", "replace")
        check(tag + ": manifest MixinConfigs", "bmh.mixins.json" in manifest)
    elif loader == "fabric":
        fmj = json.loads(z.read("fabric.mod.json"))
        listed = json.dumps(fmj.get("mixins", []))
        check(tag + ": fabric.mod.json lists both mixin configs",
              "bmh.mixins.json" in listed and "bmh.fabric.mixins.json" in listed, listed)
    else:
        toml = "META-INF/neoforge.mods.toml" if V121 else "META-INF/mods.toml"
        text = z.read(toml).decode("utf-8", "replace") if toml in names else ""
        check(tag + ": " + toml + " lists bmh.mixins.json", "bmh.mixins.json" in text)

    # The refmap a production install needs: Forge before 1.20.5 (SRG) and Fabric (intermediary)
    needs_refmap = loader == "fabric" or (loader == "forge" and not V121)
    if needs_refmap:
        has = "bmh.refmap.json" in names
        check(tag + ": bmh.refmap.json present", has)
        mappings = json.loads(z.read("bmh.refmap.json")).get("mappings", {}) if has else {}
        member = re.compile(r"m_\d+_|f_\d+_") if loader == "forge" else re.compile(r"method_\d+|field_\d+")
        for m, entry in all_mixins:
            key = entry[:-len(".class")]
            entries = mappings.get(key, {})
            if m not in ADDED_METHODS[loader] and not (loader == "forge" and m in CONSTRUCTOR_ONLY):
                check(tag + ": refmap entry for " + m, bool(entries), "absent")
            plain = [k + " -> " + v for k, v in entries.items()
                     if ("(" in v or ":" in v) and not member.search(v) and "<init>" not in v]
            check(tag + ": refmap members of " + m + " are runtime names", not plain, plain)
        for m, ids in REFMAP_IDS[loader].items():
            key = "github/mrornithorynque/bmh/mixin/" + m
            if any(n == m for n, _ in all_mixins):
                text = json.dumps(mappings.get(key, {}))
                missing = [i for i in ids if i not in text]
                check(tag + ": " + m + " targets " + " ".join(ids), not missing, "missing " + " ".join(missing))
        for m, fields in SHADOW_FIELDS[loader].items():
            entry = "github/mrornithorynque/bmh/mixin/" + m + ".class"
            if entry in names:
                out = javap_fields(z, entry)
                missing = [f for f in fields if f not in out]
                check(tag + ": " + m + " shadow fields renamed (" + " ".join(fields) + ")", not missing, "missing " + " ".join(missing))
        for m, methods in ADDED_METHODS[loader].items():
            entry = "github/mrornithorynque/bmh/mixin/" + m + ".class"
            if entry in names:
                out = javap_fields(z, entry)
                missing = [f for f in methods if f not in out]
                check(tag + ": " + m + " added methods renamed (" + " ".join(methods) + ")", not missing, "missing " + " ".join(missing))

    # Assets: every lang item has a model, every model texture and override exists
    lang = json.loads(z.read("assets/bmh/lang/en_us.json"))
    items = sorted(k[len("item.bmh."):] for k in lang if k.startswith("item.bmh."))
    for item in items:
        model_path = "assets/bmh/models/item/" + item + ".json"
        if model_path not in names:
            check(tag + ": model of " + item, False, "absent")
            continue
        model = json.loads(z.read(model_path))
        refs = [t for t in model.get("textures", {}).values() if t.startswith("bmh:")]
        overrides = [o["model"] for o in model.get("overrides", []) if o.get("model", "").startswith("bmh:")]
        missing = [t for t in refs if "assets/bmh/textures/" + t[4:] + ".png" not in names]
        for o in overrides:
            sub = "assets/bmh/models/" + o[4:] + ".json"
            if sub not in names:
                missing.append(o)
                continue
            for t in json.loads(z.read(sub)).get("textures", {}).values():
                if t.startswith("bmh:") and "assets/bmh/textures/" + t[4:] + ".png" not in names:
                    missing.append(t)
        check(tag + ": " + item + " model, " + str(len(overrides)) + " overrides, textures all present", not missing, missing)

    # Data in the right version folders
    recipe_dir, other_dir = ("data/bmh/recipe/", "data/bmh/recipes/") if V121 else ("data/bmh/recipes/", "data/bmh/recipe/")
    recipes = sorted(n[len(recipe_dir):-5] for n in names if n.startswith(recipe_dir) and n.endswith(".json"))
    check(tag + ": recipes in " + recipe_dir + " (" + str(len(recipes)) + ")", recipes and not any(n.startswith(other_dir) for n in names))
    tag_dir = "data/bmh/tags/block/" if V121 else "data/bmh/tags/blocks/"
    check(tag + ": echo_sense_ores tag in " + tag_dir, tag_dir + "echo_sense_ores.json" in names)
    if V121:
        for t in ("swords", "axes", "pickaxes"):
            path = "data/minecraft/tags/item/" + t + ".json"
            check(tag + ": tools in #minecraft:" + t, path in names and "bmh:eternal_" in z.read(path).decode())
    return items, recipes


def main():
    loaders = [l for l in ("forge", "fabric", "neoforge") if release_jar(l)]
    items, recipes = [], []
    for loader in loaders:
        jars = release_jar(loader)
        check(loader + ": one release jar", len(jars) == 1, jars)
        for jar in jars:
            items, recipes = check_jar(loader, jar)
    print("ITEMS " + " ".join(items))
    print("RECIPES " + " ".join(recipes))
    print("JARS %d/%d" % (sum(results), len(results)))
    sys.exit(0 if results and all(results) else 1)


main()
