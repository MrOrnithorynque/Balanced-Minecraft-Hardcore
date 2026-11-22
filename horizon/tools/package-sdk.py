#!/usr/bin/env python3
"""Build the Horizon native SDK package with cargo-horizon-mod."""

import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import tomllib


HORIZON = Path(__file__).resolve().parents[1]


def stage_project(stage):
    source = HORIZON / "sdk"
    for name in ("Cargo.toml", "horizon-mod.toml"):
        shutil.copyfile(source / name, stage / name)
    shutil.copytree(source / "src", stage / "src")
    shutil.copytree(
        HORIZON / "native/mixins/bmh/logic/src",
        stage / "src/logic",
    )
    for folder in ("assets", "data"):
        shutil.copytree(HORIZON / "content" / folder, stage / folder)
    # The SDK registers the same compass property as the Java mod.
    amulet = Path("assets/bmh/items/navigator_amulet.json")
    shutil.copyfile(HORIZON.parent / "common/src/main/resources" / amulet, stage / amulet)
    shutil.copyfile(HORIZON.parent / "LICENSE", stage / "LICENSE")


def package(output, checker):
    source = HORIZON / "sdk"
    manifest = tomllib.loads((source / "horizon-mod.toml").read_text())
    mod_id = manifest["id"]
    version = manifest["version"]
    game = manifest["native"]["game"]
    archive = f"{mod_id}-{version}-horizon-{game}.hmod"
    output.mkdir(parents=True, exist_ok=True)

    with tempfile.TemporaryDirectory(dir=output) as temp:
        stage = Path(temp)
        stage_project(stage)
        subprocess.run([checker, "horizon-mod", "build"], cwd=stage, check=True)
        built = stage / "target/horizon-mod" / f"{mod_id}.hmod"
        subprocess.run([checker, "horizon-mod", "check", str(built)], check=True)
        built.replace(output / archive)

    print(output / archive)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, default=HORIZON / "build")
    parser.add_argument("--stage", type=Path, help="copy the project into a new folder and exit")
    parser.add_argument(
        "--checker", default=os.getenv("BMH_HORIZON_MOD_BIN", "cargo-horizon-mod"),
        help="path to cargo-horizon-mod",
    )
    args = parser.parse_args()
    if args.stage is not None:
        args.stage.mkdir(parents=True)
        stage_project(args.stage)
        print(args.stage.resolve())
        raise SystemExit
    checker = shutil.which(args.checker)
    if checker is None:
        parser.error("Install cargo-horizon-mod or pass its path with --checker.")
    package(args.out.resolve(), Path(checker).resolve())
