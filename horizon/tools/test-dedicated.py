#!/usr/bin/env python3
"""Run three isolated, loopback-only Horizon dedicated boots against the supplied BMH package."""
import argparse
from contextlib import contextmanager
import gzip
import json
import os
from pathlib import Path
import re
import shutil
import signal
import socket
import subprocess
import sys
import tempfile
import time
import zipfile

RULE = "display_text_on_respawn"
RANDOM = "random_death_spawn_point"
NAMES = {RULE, RANDOM, "food_and_health_regen_on_wake_up", "only_carry_one_eternal_item_type",
         "damage_eternal_item_when_dead", "remove_coordinates_in_survival"}
LIMIT = 8 * 1024 * 1024


def checked(condition, message):
    if not condition:
        raise RuntimeError(message)


def install(package, game):
    with zipfile.ZipFile(package) as z:
        for name in ("horizon-mods/bmh.hmod", "horizon-native-mods/bmh.hnmod"):
            dest = game / name
            dest.parent.mkdir()
            dest.write_bytes(z.read(name))
    (game / "horizon-mods.txt").write_text("mod.bmh:true\n")


def port():
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


@contextmanager
def boot(args, data, evidence, number):
    game, home = data / "game", data / "home"
    p = port()
    (game / "server.properties").write_text(
        "server-ip=127.0.0.1\nonline-mode=false\nenforce-secure-profile=false\n"
        "enable-query=false\nenable-rcon=false\nmanagement-server-enabled=false\n"
        "pause-when-empty-seconds=-1\nlevel-seed=20261119\nview-distance=2\n"
        f"simulation-distance=3\nmax-players=1\nlevel-name=smoke\nserver-port={p}\n")
    env = {k: v for k, v in os.environ.items() if not k.startswith("HORIZON_")}
    for key in ("HOME", "USERPROFILE", "HORIZON_HOME", "APPDATA", "LOCALAPPDATA"):
        env[key] = str(home)
    for key in ("TMPDIR", "TEMP", "TMP"):
        env[key] = str(data / "tmp")
    for key, directory in (("CACHE", "cache"), ("CONFIG", "config"), ("DATA", "data")):
        env[f"XDG_{key}_HOME"] = str(home / directory)
    env.update(HORIZON_NO_AUDIO="1", HORIZON_NO_DISCORD="1", HORIZON_TELEMETRY_URL="off",
               HORIZON_API_DISCOVERY="http://127.0.0.1:0/", HORIZON_LAUNCHER_NO_SELF_UPDATE="1")
    cmd = [str(args.binary), "--resources", str(args.resources), "--game-dir", str(game),
           "--home", str(home), "--dedicated", "--nogui", "--port", str(p),
           "--world", "smoke", "--universe", str(game / "worlds")]
    log = evidence / f"boot-{number}.log"
    deadline = time.monotonic() + args.boot_timeout
    with log.open("wb") as output:
        proc = subprocess.Popen(cmd, cwd=game, env=env, stdin=subprocess.PIPE,
                                stdout=output, stderr=subprocess.STDOUT)

    def expect(pattern, offset, timeout):
        end = min(deadline, time.monotonic() + timeout)
        with log.open("rb") as stream:
            while time.monotonic() < end:
                checked(log.stat().st_size <= LIMIT, f"Boot {number} exceeded the log limit")
                stream.seek(offset)
                match = re.search(pattern.encode(), stream.read(LIMIT))
                if match:
                    return match.group().decode()
                if proc.poll() is not None:
                    break
                time.sleep(0.1)
        raise RuntimeError(f"Boot {number} missed {pattern!r} (exit {proc.poll()})")

    def command(line, pattern):
        offset = log.stat().st_size
        proc.stdin.write((line + "\n").encode())
        proc.stdin.flush()
        return expect(pattern, offset, 30)

    try:
        expect(rf"Starting Minecraft server on 127\.0\.0\.1:{p}\b", 0, args.boot_timeout)
        expect(r"Done \([\d.]+s\)!", 0, args.boot_timeout)
        # The server prints Done before its first tick registers the native commands.
        clock = r"The game time is [0-9]+ tick\(s\)"
        first = command("time query gametime", clock)
        while command("time query gametime", clock) == first:
            time.sleep(0.1)
        yield command
        proc.stdin.write(b"stop\n")
        proc.stdin.flush()
        checked(proc.wait(timeout=max(0, deadline - time.monotonic())) == 0, "Server stop failed")
    except Exception:
        with log.open("rb") as stream:
            stream.seek(max(0, log.stat().st_size - 2400))
            print(stream.read(2400).decode(errors="replace"), file=sys.stderr)
        raise
    finally:
        if proc.poll() is None:
            proc.terminate()
            try:
                proc.wait(timeout=4)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait()
        proc.stdin.close()


def rules(path):
    pairs = re.findall(r"^([a-z_]+) = (true|false)$", path.read_text(), re.M)
    checked(len(pairs) == 6 and {key for key, value in pairs} == NAMES, f"Expected six rules in {path}")
    return {key: value == "true" for key, value in pairs}


def java_fixture(path):
    # Insert the Java rule tags into the world saved by the second boot.
    raw = gzip.decompress(path.read_bytes())
    marker = b"\x0a\x00\x04data"
    checked(raw.count(marker) == 1, "Missing or ambiguous game_rules.dat data compound")
    pos = raw.index(marker) + len(marker)

    def nbt_string(s):
        data = s.encode("ascii")
        return len(data).to_bytes(2, "big") + data

    entries = (b"\x01" + nbt_string("randomDeathSpawnPoint") + b"\x00"
               + b"\x08" + nbt_string("bmh:display_text_on_respawn") + nbt_string("false"))
    checked(b"randomDeathSpawnPoint" not in raw and b"bmh:display_text_on_respawn" not in raw,
            "Fixture had preexisting BMH keys")
    path.write_bytes(gzip.compress(raw[:pos] + entries + raw[pos:], mtime=0))


def query(command, name, value):
    return command(f"bmh rule {name}", rf"bmh:{name} is currently set to:? {value}\b")


def run(args, data, evidence, facts):
    game = data / "game"
    for d in (game, data / "home", data / "tmp", game / "config"):
        d.mkdir()
    install(args.package, game)
    (game / "eula.txt").write_text("eula=true\n")
    conf = game / "config/bmh-common.toml"
    conf.write_text("minDistanceFromBed = 12000\nmaxDistanceFromBed = 70000\nmaxAmuletPoints = 99\n")
    world = game / "worlds/smoke"
    rulefile = world / "bmh/rules.txt"
    with boot(args, data, evidence, 1) as command:
        facts["initial_query"] = query(command, RULE, "true")
        checked(all(rules(rulefile).values()), "Fresh world did not start with all rules enabled")
        text = conf.read_text()
        values = dict(re.findall(r"^(\w+)\s*=\s*(\d+)\s*$", text, re.M))
        checked(values == {"minDistanceFromBed": "12000", "maxDistanceFromBed": "70000", "maxAmuletPoints": "5"},
                f"First boot did not load and correct dedicated Common config: {values}")
        facts["corrected_config"] = values
        (evidence / "corrected-config.toml").write_text(text)
        tables = list((game / ".horizon-cache/mod-content").glob("*/items.json"))
        checked(len(tables) == 1, f"Expected one freshly assembled mod content root, found {len(tables)}")
        items = json.loads(tables[0].read_bytes())
        totem = items["bmh:totem_of_eternity"]
        checked("minecraft:death_protection" not in totem, "The BMH totem would protect mobs that hold it")
        facts["loaded_item"] = "bmh:totem_of_eternity"
        facts["set"] = command(f"bmh rule {RULE} false", rf"bmh:{RULE} (?:is now set to|has been updated to):? false\b")
        checked(not rules(rulefile)[RULE], "Dedicated console command did not write the world rule")
        shutil.copyfile(rulefile, evidence / "boot-1-rules.txt")
    with boot(args, data, evidence, 2) as command:
        facts["restart_query"] = query(command, RULE, "false")
        checked(rules(rulefile)[RULE] is False, "Changed world rule did not survive restart")
        shutil.copyfile(rulefile, evidence / "boot-2-rules.txt")
    dat = world / "data/minecraft/game_rules.dat"
    java_fixture(dat)
    shutil.copyfile(dat, evidence / "fixture-game_rules.dat")
    rulefile.unlink()
    with boot(args, data, evidence, 3) as command:
        facts["import_queries"] = [query(command, k, "false") for k in (RANDOM, RULE)]
        migrated = rules(rulefile)
        checked({k for k, v in migrated.items() if not v} == {RANDOM, RULE},
                f"Legacy Java import mismatch: {migrated}")
        facts["import_rules"] = migrated
        shutil.copyfile(rulefile, evidence / "boot-3-imported-rules.txt")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for key in ("package", "bin-dir", "resources"):
        parser.add_argument("--" + key, type=Path, required=True)
    parser.add_argument("--out", type=Path, help="Keep logs and results in a new directory under OUT")
    parser.add_argument("--boot-timeout", type=int, default=240, help="Maximum seconds per boot")
    args = parser.parse_args()
    args.binary = (args.bin_dir / ("horizon-client.exe" if os.name == "nt" else "horizon-client")).resolve()
    args.resources = args.resources.resolve()
    checked(args.binary.is_file(), f"Missing Horizon executable: {args.binary}")
    checked((args.resources / "horizon-resources.json").is_file(), "Missing prepared Horizon resources")
    checked(args.package.is_file(), f"Missing BMH package: {args.package}")
    signal.signal(signal.SIGTERM, lambda *_: sys.exit(143))
    with tempfile.TemporaryDirectory(prefix="bmh-dedicated-") as tmp:
        data = Path(tmp)
        evidence = data
        if args.out:
            args.out.mkdir(parents=True, exist_ok=True)
            evidence = Path(tempfile.mkdtemp(prefix="dedicated-", dir=args.out.resolve()))
            print(f"Evidence: {evidence}", flush=True)
        facts = {}
        try:
            run(args, data, evidence, facts)
            facts["result"] = "pass"
        except BaseException as err:
            facts.update(result="fail", error=str(err))
            raise
        finally:
            (evidence / "result.json").write_text(json.dumps(facts, indent=2) + "\n")
    print("PASS: dedicated config, packaged items, console rules, restart and Java rule import.")


if __name__ == "__main__":
    main()
