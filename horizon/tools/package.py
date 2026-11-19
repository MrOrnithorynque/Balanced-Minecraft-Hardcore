#!/usr/bin/env python3
"""Build both Horizon packages and put them in the launcher's instance folders."""

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import tomllib
import zipfile


def package(output):
    horizon = Path(__file__).resolve().parents[1]
    cache = Path(os.environ.get("BMH_HORIZON_CACHE", Path.home() / "Library/Caches/bmh-horizon"))
    native_tool = cache / "horizon-mods/native/tools/native.py"
    checker = os.environ.get("BMH_HORIZON_MOD_BIN") or shutil.which("cargo-horizon-mod")
    checker = checker or str(cache / "target-tools/debug/cargo-horizon-mod")
    if not native_tool.is_file():
        sys.exit(f"Missing {native_tool}. Run sh horizon/tools/check.sh first.")
    executable = shutil.which(checker)
    if executable is None:
        sys.exit("Set BMH_HORIZON_MOD_BIN to the cargo-horizon-mod executable.")

    checker = Path(executable).resolve()
    content = horizon / "content"
    manifest = tomllib.loads((content / "horizon-mod.toml").read_text())
    mod_id, version = manifest["id"], manifest["version"]
    game = json.loads((horizon / "native/compatibility.json").read_text())["game"]
    filename = f"{mod_id}-{version}-horizon-{game}.zip"
    output.mkdir(parents=True, exist_ok=True)

    with tempfile.TemporaryDirectory(dir=output) as tmp:
        stage = Path(tmp).resolve()
        # Both tools write under their input tree. Each run gets its own tree.
        tool = stage / "native/tools/native.py"
        tool.parent.mkdir(parents=True)
        shutil.copyfile(native_tool, tool)
        bundle = stage / "native" / mod_id
        bundle.mkdir()
        shutil.copyfile(horizon / "native/compatibility.json", bundle / "compatibility.json")
        shutil.copytree(horizon / "native/mixins", bundle / "mixins")
        shutil.copyfile(horizon.parent / "LICENSE", bundle / "LICENSE")
        subprocess.run([sys.executable, tool, "package", mod_id], check=True)

        data = stage / "content"
        data.mkdir()
        shutil.copyfile(content / "horizon-mod.toml", data / "horizon-mod.toml")
        for folder in ("assets", "data"):
            shutil.copytree(content / folder, data / folder)
        subprocess.run([checker, "horizon-mod", "build"], cwd=data, check=True)

        files = {
            f"horizon-mods/{mod_id}.hmod": data / f"target/horizon-mod/{mod_id}.hmod",
            f"horizon-native-mods/{mod_id}.hnmod": stage / f"native/target/{mod_id}-{version}.hnmod",
        }
        for archive in files.values():
            report = subprocess.check_output([checker, "horizon-mod", "check", archive, "--json"])
            facts = json.loads(report)["report"]
            if (facts["id"], facts["version"]) != (mod_id, version):
                sys.exit(f"{archive.name} does not match the content id and version.")

        with zipfile.ZipFile(stage / filename, "w") as distribution:
            for name, archive in files.items():
                entry = zipfile.ZipInfo(name)
                entry.create_system = 3
                entry.external_attr = 0o100644 << 16
                distribution.writestr(entry, archive.read_bytes())
        (stage / filename).replace(output / filename)
    print(output / filename)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, default=Path(__file__).resolve().parents[1] / "build",
                        help="output directory (default: horizon/build)")
    package(parser.parse_args().out.resolve())
