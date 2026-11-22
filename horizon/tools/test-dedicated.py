#!/usr/bin/env python3
"""Test the supplied BMH package on an isolated, loopback-only Horizon dedicated server."""
import argparse
from contextlib import ExitStack, contextmanager
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


def install(package, game, binary):
    if package.suffix == ".hmod":
        # The launcher stages the native library and its content beside the executable.
        provenance = json.loads(binary.with_name("native-mods.json").read_text())
        checked(any(mod["id"] == "bmh" for mod in provenance["mods"]),
                "The Horizon executable has no staged BMH native mod")
        return
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


def process_env(home, temp):
    env = {k: v for k, v in os.environ.items() if not k.startswith("HORIZON_")}
    for key in ("HOME", "USERPROFILE", "HORIZON_HOME", "APPDATA", "LOCALAPPDATA"):
        env[key] = str(home)
    for key in ("TMPDIR", "TEMP", "TMP"):
        env[key] = str(temp)
    for key, directory in (("CACHE", "cache"), ("CONFIG", "config"), ("DATA", "data")):
        env[f"XDG_{key}_HOME"] = str(home / directory)
    env.update(HORIZON_NO_AUDIO="1", HORIZON_NO_DISCORD="1", HORIZON_TELEMETRY_URL="off",
               HORIZON_API_DISCOVERY="http://127.0.0.1:0/", HORIZON_LAUNCHER_NO_SELF_UPDATE="1")
    return env


def stop_process(proc):
    if proc.poll() is None:
        proc.terminate()
        try:
            proc.wait(timeout=4)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()


@contextmanager
def boot(args, data, evidence, number, max_players=1):
    game, home = data / "game", data / "home"
    p = port()
    (game / "server.properties").write_text(
        "server-ip=127.0.0.1\nonline-mode=false\nenforce-secure-profile=false\n"
        "enable-query=false\nenable-rcon=false\nmanagement-server-enabled=false\n"
        "pause-when-empty-seconds=-1\nlevel-seed=20261119\nview-distance=2\n"
        "white-list=true\nop-permission-level=2\n"
        f"simulation-distance=3\nmax-players={max_players}\nlevel-name=smoke\nserver-port={p}\n")
    env = process_env(home, data / "tmp")
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
        yield command, p
        proc.stdin.write(b"stop\n")
        proc.stdin.flush()
        checked(proc.wait(timeout=max(0, deadline - time.monotonic())) == 0, "Server stop failed")
    except Exception:
        with log.open("rb") as stream:
            stream.seek(max(0, log.stat().st_size - 2400))
            print(stream.read(2400).decode(errors="replace"), file=sys.stderr)
        raise
    finally:
        stop_process(proc)
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


def prepare_server(args, data):
    game = data / "game"
    for d in (game, data / "home", data / "tmp", game / "config"):
        d.mkdir()
    install(args.package, game, args.binary)
    (game / "eula.txt").write_text("eula=true\n")
    return game


def run(args, data, evidence, facts):
    game = prepare_server(args, data)
    conf = game / "config/bmh-common.toml"
    conf.write_text("minDistanceFromBed = 12000\nmaxDistanceFromBed = 70000\nmaxAmuletPoints = 99\n")
    world = game / "worlds/smoke"
    rulefile = world / "bmh/rules.txt"
    with boot(args, data, evidence, 1) as (command, _):
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
    with boot(args, data, evidence, 2) as (command, _):
        facts["restart_query"] = query(command, RULE, "false")
        checked(rules(rulefile)[RULE] is False, "Changed world rule did not survive restart")
        shutil.copyfile(rulefile, evidence / "boot-2-rules.txt")
    dat = world / "data/minecraft/game_rules.dat"
    java_fixture(dat)
    shutil.copyfile(dat, evidence / "fixture-game_rules.dat")
    rulefile.unlink()
    with boot(args, data, evidence, 3) as (command, _):
        facts["import_queries"] = [query(command, k, "false") for k in (RANDOM, RULE)]
        migrated = rules(rulefile)
        checked({k for k, v in migrated.items() if not v} == {RANDOM, RULE},
                f"Legacy Java import mismatch: {migrated}")
        facts["import_rules"] = migrated
        shutil.copyfile(rulefile, evidence / "boot-3-imported-rules.txt")


def eventually(read, ok, message, timeout=8):
    deadline = time.monotonic() + timeout
    while True:
        value = read()
        if ok(value):
            return value
        checked(time.monotonic() < deadline, f"{message}. Last value: {repr(value)[:1200]}")
        time.sleep(0.1)


@contextmanager
def connect(args, data, evidence, address, name):
    game = data / name
    game.mkdir()
    install(args.package, game, args.binary)
    with tempfile.TemporaryDirectory(prefix="bmh-client-") as local:
        sock = Path(local) / "ctl.sock"
        ctl = args.binary.with_name("horizon-ctl" + args.binary.suffix)
        with (evidence / f"{name}.log").open("wb") as log:
            proc = subprocess.Popen(
                [args.binary, "--resources", args.resources, "--game-dir", game, "--home", game,
                 "--offscreen", "--render-distance", "2", "--control", "--control-socket", sock,
                 "--server", address, "--name", name],
                cwd=game, env=process_env(game, data / "tmp"), stdin=subprocess.DEVNULL,
                stdout=log, stderr=subprocess.STDOUT,
            )

        def call(*action):
            result = subprocess.run([ctl, "--socket", sock, "--timeout", "8", *map(str, action)],
                                    capture_output=True, text=True, timeout=10)
            checked(result.returncode == 0, f"{name} {action}: {result.stdout or result.stderr}")
            return json.loads(result.stdout)

        try:
            eventually(lambda: sock.is_socket() or proc.poll() is not None, bool, f"{name} control socket", 35)
            checked(proc.poll() is None, f"{name} failed to start. See {evidence / (name + '.log')}")
            eventually(lambda: call("state")["screen"], lambda screen: screen == "playing",
                       f"{name} did not join {address}", 45)
            yield call
        finally:
            if proc.poll() is None and sock.is_socket():
                try:
                    call("quit")
                    proc.wait(timeout=6)
                except (RuntimeError, subprocess.TimeoutExpired):
                    pass
            stop_process(proc)


def permission_checks(command, guard, miner, rulefile, opsfile):
    def denials():
        return sum(msg.get("translate") == "command.unknown.command" and msg.get("color") == "red"
                   for msg in guard("messages", 20)["styled"])

    before = denials()
    for index, action in enumerate((f"bmh rule {RULE} false", f"gamerule bmh:{RULE} false"), start=1):
        guard("chat", "/" + action)
        eventually(denials, lambda count: count >= before + index, f"Guard denial for {action}")
        checked(rules(rulefile)[RULE], f"Guard changed {RULE} via {action}")
    command("op miner", "Made miner a server operator")
    levels = {op["name"]: op["level"] for op in json.loads(opsfile.read_text())}
    checked(levels == {"miner": 2}, f"Unexpected operator levels: {levels}")
    miner("chat", f"/bmh rule {RULE} false")
    eventually(lambda: rules(rulefile)[RULE], lambda value: not value, "Level 2 rule change")
    miner("chat", f"/gamerule bmh:{RULE} true")
    eventually(lambda: rules(rulefile)[RULE], bool, "Level 2 gamerule alias")
    command("deop miner", "Made miner no longer a server operator")
    miner("chat", f"/bmh rule {RULE} false")
    eventually(lambda: miner("messages", 20)["styled"][0].get("translate"),
               lambda key: key == "command.unknown.command", "Revoked rule permission")
    checked(rules(rulefile)[RULE], "De-opped miner changed the rule")
    checked(not json.loads(opsfile.read_text()), "The server still lists an operator")
    return {"level_0_denials": 2, "level_2_set": True, "level_2_alias": True, "deop_denial": True}


def slot(inventory, index):
    return next((item.get("item") for item in inventory["main"] if item["slot"] == index), None)


def health(client):
    return client("state")["player"]["health"]


def equip(command, player, index, item):
    command(f"item replace entity {player} {index} with {item}", r"Replaced 1 slot")


def pocket_totem(command, guard):
    equip(command, "guard", "weapon.mainhand", "minecraft:iron_sword")
    equip(command, "guard", "weapon.offhand", "minecraft:shield")
    equip(command, "guard", "inventory.0", "bmh:totem_of_eternity")
    eventually(lambda: slot(guard("inventory"), 9), lambda item: item == "bmh:totem_of_eternity", "Slot 9 setup")
    command("damage guard 100 minecraft:generic", r"Applied 100")
    inventory = eventually(lambda: guard("inventory"), lambda inv: slot(inv, 9) == "bmh:sleeping_totem",
                           "Pocket totem was not replaced")
    hands = [inventory[k]["item"] for k in ("held", "offhand")]
    hp = health(guard)
    server_hp = float(command("data get entity guard Health", r"entity data: [0-9.]+f").split()[-1][:-1])
    effects = {effect["id"]: effect["amplifier"] for effect in guard("effects")["effects"]["active"]}
    checked(hands == ["minecraft:iron_sword", "minecraft:shield"], f"Changed hands: {hands}")
    checked(0 < hp < 20 and 0 < server_hp < 20, f"Health after the save: client {hp}, server {server_hp}")
    checked(effects == {"regeneration": 1, "absorption": 1, "fire_resistance": 0}, f"Totem effects: {effects}")
    return {"slot": 9, "after": slot(inventory, 9), "hands": hands, "client_health": hp,
            "server_health": server_hp, "effects": effects}


def amulet_checks(command, guard, miner):
    def actionbar(client, expected):
        return eventually(lambda: client("messages", 8).get("actionbar"),
                          lambda actual: actual == expected, expected)

    command("setblock 1 180 1 minecraft:lodestone", "Changed the block")
    miner("chat", "/bmh point save 1 180 1 NoAmulet")
    actionbar(miner, "Hold the Navigator's Amulet to change a point.")
    for name in ("guard", "miner"):
        command(f"give {name} bmh:navigator_amulet", r"Gave 1")
    for client in (guard, miner):
        client("select", "bmh:navigator_amulet")
    miner("chat", "/bmh point save 1 180 1 MinedCamp")
    actionbar(miner, "Point saved: MinedCamp")
    guard("look-at", 1.5, 180.5, 1.5)
    guard("key", "shift", "down")
    try:
        guard("wait", 5)
        guard("use-item")
        eventually(lambda: guard("state")["screen"], lambda screen: screen == "mod", "Amulet naming screen")
    finally:
        guard("key", "shift", "up")
    guard("type", "GuardCamp")
    guard("key", "enter", "tap")
    actionbar(guard, "Point saved: GuardCamp")
    command("weather clear", "weather")
    command("time set 18000", r"18000")
    guard("wait", 25)
    evidence = {}
    for client, own, other in ((guard, "GuardCamp", "MinedCamp"), (miner, "MinedCamp", "GuardCamp")):
        client("look", 0, -20)
        client("use-item")
        reading = eventually(lambda: client("inventory")["held"].get("components", ""),
                             lambda components: own in components, f"Reading with {own}")
        checked(other not in reading, f"{other} leaked into {own}'s reading")
        checked("bmh_starlight:2" in reading, f"A reading must consume one charge: {reading}")
        evidence[own] = reading
    return {"no_amulet_refused": True, "named_through_screen": "GuardCamp", "readings": evidence}


def pvp_checks(command, guard, miner):
    command("clear @a", r"Removed")
    command("effect clear @a", r"effect")
    command("gamerule natural_health_regeneration false", r"false")
    command("bmh rule random_death_spawn_point false", r"false")

    def reset_attacker(item):
        command("tp miner 2.5 180 0.5", r"Teleported miner")
        equip(command, "miner", "weapon.mainhand", item)
        miner("wait", 35)

    def reset_victim():
        command("tp guard 0.5 180 0.5", r"Teleported guard")
        command("effect give guard minecraft:instant_health 1 10 true", r"Instant Health")
        eventually(lambda: health(guard), lambda hp: hp == 20, "Full health before PvP")

    def echo_target_health():
        reply = command("data get entity @e[tag=bmh_echo,limit=1] Health", r"entity data: [0-9.]+f")
        return float(reply.split()[-1][:-1])

    def ground_hit(item):
        reset_victim()
        reset_attacker(item)
        miner("look-at", 0.5, 181.4, 0.5)
        miner("attack")
        return 20 - eventually(lambda: health(guard), lambda hp: hp < 20, "PvP damage")

    command("gamerule pvp true", r"true")
    worn = ground_hit("bmh:eternal_sword[minecraft:damage=32999]")
    checked(worn == 5.5, f"Worn sword damage: {worn}")
    command("difficulty normal", r"Normal")
    command('summon husk 0.5 180 4.5 {NoAI:1b,Silent:1b,Tags:[bmh_echo]}', r"Summoned")
    fresh = ground_hit("bmh:eternal_sword")
    checked(fresh == 11, f"Fresh sword damage: {fresh}")
    miner("wait", 20)
    echo = echo_target_health()
    checked(0 < echo < 20, f"Echo target health: {echo}")
    command("advancement revoke miner only bmh:eternity/echo_strike", r"Revoked the advancement .* from miner")

    command("gamerule pvp false", r"false")
    reset_victim()
    reset_attacker("bmh:eternal_sword")
    miner("look-at", 0.5, 181.4, 0.5)
    miner("attack")
    miner("wait", 20)
    checked(health(guard) == 20, "PvP off did not block player damage")
    echo_after = echo_target_health()
    checked(echo_after == echo, "PvP off did not block Echo Strike")

    command("gamerule pvp true", r"true")
    axe = ground_hit("bmh:eternal_axe")
    checked(axe == 18, f"Grounded axe damage: {axe}")
    reset_victim()
    reset_attacker("bmh:eternal_axe")
    command("tick rate 5", r"5.0")
    try:
        command("tp miner 2.5 188 0.5", r"Teleported miner")
        miner("look-at", 0.5, 181.4, 0.5)
        eventually(lambda: miner("state")["player"],
                   lambda p: not p["on_ground"] and 180.8 < p["position"][1] < 182.8,
                   "Falling attack window")
        miner("look-at", 0.5, 181.4, 0.5)
        miner("attack")
    finally:
        command("tick rate 20", r"20.0")
    after = eventually(lambda: health(guard), lambda hp: hp == 0, "Falling axe hit from full health")
    miner("wait", 10)
    command("advancement revoke miner only bmh:eternity/smash", r"Revoked the advancement .* from miner")
    return {"worn_sword": worn, "fresh_sword": fresh, "echo_target_health": echo, "echo_awarded": True,
            "pvp_off": True, "ground_axe": axe, "falling_axe_victim_health": after, "smash_awarded": True}


def multiplayer(args, data, evidence, facts):
    game = prepare_server(args, data)
    checks = facts["checks"] = {}
    with ExitStack() as stack:
        command, port = stack.enter_context(boot(args, data, evidence, 1, max_players=2))
        for name in ("guard", "miner"):
            command(f"whitelist add {name}", rf"Added {name} to the whitelist")
        address = f"127.0.0.1:{port}"
        guard, miner = [stack.enter_context(connect(args, data, evidence, address, name))
                        for name in ("guard", "miner")]
        facts["server_list"] = command("list", r"There are 2 of a max of 2 players online: [^\n]+")
        command("difficulty peaceful", "Peaceful")
        command("fill -5 179 -5 7 179 7 minecraft:stone", "Successfully filled")
        command("tp guard 0.5 180 0.5", "Teleported guard")
        command("tp miner 2.5 180 0.5", "Teleported miner")
        rulefile = game / "worlds/smoke/bmh/rules.txt"
        for name, test, params in (
            ("permissions", permission_checks, (command, guard, miner, rulefile, game / "ops.json")),
            ("pocket_totem", pocket_totem, (command, guard)),
            ("amulet", amulet_checks, (command, guard, miner)),
            ("pvp", pvp_checks, (command, guard, miner)),
        ):
            try:
                checks[name] = {"result": "pass", **test(*params)}
                print(f"PASS {name}", flush=True)
            except (RuntimeError, subprocess.TimeoutExpired) as err:
                checks[name] = {"result": "fail", "error": str(err)}
                print(f"FAIL {name}: {err}", flush=True)
    failed = [name for name, check in checks.items() if check["result"] == "fail"]
    checked(not failed, f"Failed multiplayer checks: {', '.join(failed)}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for key in ("package", "bin-dir", "resources"):
        parser.add_argument("--" + key, type=Path, required=True)
    parser.add_argument("--multiplayer", action="store_true", help="Run the two-client gameplay checks")
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
            test = multiplayer if args.multiplayer else run
            test(args, data, evidence, facts)
            facts["result"] = "pass"
        except BaseException as err:
            facts.update(result="fail", error=str(err))
            raise
        finally:
            (evidence / "result.json").write_text(json.dumps(facts, indent=2) + "\n")
    if args.multiplayer:
        print("PASS: dedicated multiplayer checks.")
    else:
        print("PASS: dedicated config, packaged items, console rules, restart and Java rule import.")


if __name__ == "__main__":
    main()
