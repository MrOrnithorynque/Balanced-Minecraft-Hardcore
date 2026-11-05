"""Publishes one built release worktree from this Mac: CurseForge, the GitHub release, Discord.

It does what .github/disabled-workflows/publish.yml did on GitHub Actions, for one Minecraft version:
one CurseForge file per loader (Forge, Fabric with Fabric API as a required dependency, NeoForge where
the branch builds it; the 1.20.1 Forge jar is also listed for NeoForge, through the documented upload API:
https://support.curseforge.com/en/support/solutions/articles/9000197321-curseforge-upload-api), one GitHub
release that holds every jar, and, for the 1.21.1 tag only, the changelog on Discord. Modrinth is not done
here: its token answered 401 since 2026-11-04.

usage: python3 -I publish_local.py <abs worktree> [--dry-run] [--only curseforge,github,discord]
                                   [--loader all|forge|fabric|neoforge]

The worktree must be built (gradlew build) and sit on the commit of its tag mc-<mc>-v<mod>, read from its
gradle.properties. A real run checks, before any upload, that the tag is on GitHub at that commit, that gh
is logged in, and that every secret it needs is there. The changelog is the "## [<mod>]" section of
CHANGELOG.md, as the workflow used it. "--only discord" needs no jar: it reads only the changelog.

Secrets come from the macOS Keychain, never from the command line or a file:
  security add-generic-password -a "$USER" -s bmh-curseforge-token -w    (CurseForge author API token)
  security add-generic-password -a "$USER" -s bmh-discord-webhook -w     (the Discord webhook URL)
Each command asks for the value; it never shows in the shell history. The GitHub release uses gh's login.

--dry-run uploads and posts nothing: it prints the files, names and versions it would send, and, when the
token is there, the CurseForge game version id of each name (read only requests). A Minecraft version name
exists in several CurseForge game version types, so the script sends ids, never bare names, and stops before
any upload when a name has no match or more than one in its type.

Every line it prints starts with PUBLISHED, SKIPPED, DRY, WARNING or ERROR. ERROR stops the script: what
was PUBLISHED before it is public, nothing after it was sent.
"""
import glob
import json
import os
import re
import subprocess
import sys
import urllib.error
import urllib.request
import uuid

CURSEFORGE_PROJECT = 982343
CURSEFORGE_API = "https://minecraft.curseforge.com/api"
CURSEFORGE_PAGE = "https://www.curseforge.com/minecraft/mc-mods/balanced-minecraft-hardcore"
REPO = "MrOrnithorynque/Balanced-Minecraft-Hardcore"
DISCORD_MAX = 3900
# Discord's Cloudflare refuses Python's default agent (403, error code 1010).
USER_AGENT = "BMH-release (https://github.com/MrOrnithorynque/Balanced-Minecraft-Hardcore, 1.0)"
TARGETS = {"curseforge", "github", "discord"}
LOADERS = {"all", "forge", "fabric", "neoforge"}


def fail(message):
    print("ERROR " + message, flush=True)
    sys.exit(1)


def keychain(service):
    out = subprocess.run(["security", "find-generic-password", "-s", service, "-w"], capture_output=True, text=True)
    return out.stdout.strip() if out.returncode == 0 else ""


def git(tree, *args):
    return subprocess.run(["git", "-C", tree] + list(args), capture_output=True, text=True, check=True).stdout.strip()


def props(tree):
    values = {}
    for line in open(os.path.join(tree, "gradle.properties"), encoding="utf-8"):
        if "=" in line and not line.lstrip().startswith("#"):
            key, value = line.split("=", 1)
            values[key.strip()] = value.strip()
    java = re.search(r"JavaLanguageVersion\.of\((\d+)\)", open(os.path.join(tree, "build.gradle"), encoding="utf-8").read())
    values["java"] = java.group(1) if java else "17"
    return values


def changelog(tree, mod):
    lines, found = [], False
    for line in open(os.path.join(tree, "CHANGELOG.md"), encoding="utf-8"):
        if line.startswith("## [" + mod + "]"):
            found = True
            continue
        if found and line.startswith("## ["):
            break
        if found:
            lines.append(line)
    text = "".join(lines).strip()
    if not text:
        fail("CHANGELOG.md has no non-empty '## [%s]' section" % mod)
    return text


def jar(tree, loader):
    jars = [j for j in glob.glob(os.path.join(tree, loader, "build", "libs", "*.jar"))
            if not re.search(r"-(dev|sources|javadoc)\.jar$", j)]
    if len(jars) != 1:
        fail("%s: expected one release jar in %s/build/libs, found %s" % (loader, loader, jars))
    return jars[0]


def request(url, data=None, headers=None, method=None, shown=None):
    """An HTTP request. On an HTTP error it stops with the server's answer; `shown` replaces a URL that holds a secret."""
    req = urllib.request.Request(url, data=data, headers={"User-Agent": USER_AGENT, **(headers or {})}, method=method)
    try:
        with urllib.request.urlopen(req, timeout=120) as response:
            body = response.read().decode("utf-8")
    except urllib.error.HTTPError as e:
        fail("%s %s: HTTP %d %s" % (method or "GET", shown or url, e.code, e.read().decode("utf-8", errors="replace")[:500]))
    except urllib.error.URLError as e:
        fail("%s %s: %s" % (method or "GET", shown or url, e.reason))
    return json.loads(body) if body else None


def gh(*args):
    out = subprocess.run(["gh"] + list(args), capture_output=True, text=True)
    if out.returncode != 0:
        fail("gh %s: %s" % (" ".join(args[:2]), (out.stderr or out.stdout).strip()[:500]))
    return out.stdout


def curseforge_ids(token, names):
    """The CurseForge game version id of each name, in the right game version type. A Minecraft version name such as
    1.21.1 exists 3 times (Minecraft 1.21, Addons, and type 1): only the one in a minecraft-* type is right. Loaders are
    in the modloader type, Java versions in the java type, Client and Server in the environment type (required since
    2026-11: an upload without one gets error 1021). The version types list is not in CurseForge's upload API page, but
    mc-publish used it for every release until 1.6.2. Stops on 0 or more than 1 match."""
    headers = {"X-Api-Token": token}
    types = {t["id"]: t["slug"] for t in request(CURSEFORGE_API + "/game/version-types", headers=headers)}
    versions = request(CURSEFORGE_API + "/game/versions", headers=headers)
    ids = {}
    for name in names:
        if name in ("Forge", "Fabric", "NeoForge"):
            kind = lambda slug: slug == "modloader"
        elif name.startswith("Java "):
            kind = lambda slug: slug == "java"
        elif name in ("Client", "Server"):
            kind = lambda slug: slug == "environment"
        else:
            kind = lambda slug: slug.startswith("minecraft-")
        match = [v["id"] for v in versions if v["name"] == name and kind(types.get(v["gameVersionTypeID"], ""))]
        if len(match) != 1:
            fail("CurseForge game version %r: %d matches in its type" % (name, len(match)))
        ids[name] = match[0]
    return ids


def curseforge_upload(token, path, metadata):
    boundary = uuid.uuid4().hex
    body = b"".join([
        ("--%s\r\nContent-Disposition: form-data; name=\"metadata\"\r\n\r\n" % boundary).encode(),
        json.dumps(metadata).encode(), b"\r\n",
        ("--%s\r\nContent-Disposition: form-data; name=\"file\"; filename=\"%s\"\r\nContent-Type: application/java-archive\r\n\r\n"
         % (boundary, os.path.basename(path))).encode(),
        open(path, "rb").read(), b"\r\n",
        ("--%s--\r\n" % boundary).encode(),
    ])
    headers = {"X-Api-Token": token, "Content-Type": "multipart/form-data; boundary=" + boundary}
    return request("%s/projects/%d/upload-file" % (CURSEFORGE_API, CURSEFORGE_PROJECT), body, headers, "POST")


def discord_payload(mod, log):
    """The embed publish.yml posted: image lines out of the text, the first image as the embed's picture."""
    image = re.search(r"^!\[[^\]]*\]\(([^)]*)\)", log, re.M)
    text = re.sub(r"^!\[.*\n?", "", log, flags=re.M)
    text = re.sub(r"^#+[ \t]+(.*)$", r"**\1**", text, flags=re.M)
    text = re.sub(r"\n{3,}", "\n\n", text).strip()
    releases = "https://github.com/%s/releases" % REPO
    if len(text) > DISCORD_MAX:
        text = text[:DISCORD_MAX] + "...\n\n[Full changelog](%s)" % releases
    embed = {"title": "Balanced Minecraft Hardcore " + mod, "url": CURSEFORGE_PAGE, "color": 9330905, "description": text,
             "fields": [{"name": "Download", "value": "[CurseForge](%s/files) | [GitHub](%s)" % (CURSEFORGE_PAGE, releases)}]}
    if image:
        embed["image"] = {"url": image.group(1)}
    return {"allowed_mentions": {"parse": []}, "embeds": [embed]}


def option(args, name, default):
    if name not in args:
        return default
    i = args.index(name)
    if i + 1 >= len(args):
        fail("%s needs a value" % name)
    return args[i + 1]


def main():
    args = sys.argv[1:]
    if not args or not os.path.isabs(args[0]):
        fail("usage: publish_local.py <abs worktree> [--dry-run] [--only curseforge,github,discord] [--loader ...]")
    tree = args[0]
    dry = "--dry-run" in args
    only = option(args, "--only", "curseforge,github,discord").split(",")
    loader_arg = option(args, "--loader", "all")
    if set(only) - TARGETS:
        fail("--only takes %s, not %s" % (sorted(TARGETS), sorted(set(only) - TARGETS)))
    if loader_arg not in LOADERS:
        fail("--loader takes %s, not %s" % (sorted(LOADERS), loader_arg))

    p = props(tree)
    mc, mod, java = p["minecraft_version"], p["mod_version"], p["java"]
    tag = "mc-%s-v%s" % (mc, mod)
    head = git(tree, "rev-parse", "HEAD")
    try:
        tagged = git(tree, "rev-parse", tag + "^{commit}")
    except subprocess.CalledProcessError:
        tagged = None
    if tagged is None and dry:
        print("WARNING no tag %s yet: a real run needs it on this commit, pushed to GitHub" % tag, flush=True)
    elif tagged is None:
        fail("no tag %s here: create it on this commit and push it first" % tag)
    elif tagged != head:
        fail("%s points to %s, the worktree is on %s" % (tag, tagged[:7], head[:7]))
    log = changelog(tree, mod)
    posts_discord = "discord" in only and mc == "1.21.1" and loader_arg == "all"

    # Everything a real run needs, checked before the first upload: nothing goes out half way for a missing piece.
    token = keychain("bmh-curseforge-token") if "curseforge" in only else ""
    hook = keychain("bmh-discord-webhook") if posts_discord else ""
    if not dry:
        remote = subprocess.run(["git", "-C", tree, "ls-remote", "origin", "refs/tags/%s^{}" % tag, "refs/tags/%s" % tag],
                                capture_output=True, text=True).stdout.split()
        if head not in remote:
            fail("tag %s is not on GitHub at %s: push it first (git push origin %s)" % (tag, head[:7], tag))
        if "github" in only:
            gh("auth", "status")
        if "curseforge" in only and not token:
            fail("no CurseForge token in the Keychain (service bmh-curseforge-token), see the header of this script")
        if posts_discord and not hook:
            fail("no Discord webhook in the Keychain (service bmh-discord-webhook), see the header of this script")

    uploads = []
    if only != ["discord"]:
        loaders = ["forge", "fabric"] + (["neoforge"] if "neoforge_version" in p else [])
        if loader_arg != "all":
            loaders = [loader_arg]
        for loader in loaders:
            label = {"forge": "Forge", "fabric": "Fabric", "neoforge": "NeoForge"}[loader]
            listed = [label] + (["NeoForge"] if loader == "forge" and mc == "1.20.1" else [])
            uploads.append({
                "loader": loader,
                "file": jar(tree, loader),
                "displayName": "Balanced Minecraft Hardcore %s for %s %s" % (mod, label, mc),
                "versionNames": [mc, "Java " + java, "Client", "Server"] + listed,
                "relations": {"projects": [{"slug": "fabric-api", "type": "requiredDependency"}]} if loader == "fabric" else None,
            })

        jarcheck = os.path.join(tree, ".claude", "skills", "verify-loaders", "scripts", "jarcheck.py")
        java_home = "/opt/homebrew/opt/openjdk@" + java
        check = subprocess.run([sys.executable, "-I", jarcheck, tree, mc, java_home], capture_output=True, text=True)
        jars_line = [l for l in check.stdout.splitlines() if l.startswith("JARS ")]
        print("%s: jarcheck %s" % (tag, jars_line[-1] if jars_line else "?"), flush=True)
        if check.returncode != 0:
            for line in check.stdout.splitlines():
                if line.startswith("FAIL"):
                    print("  " + line, flush=True)
            fail("jarcheck failed: fix the jars before publishing")

    if "curseforge" in only and uploads:
        # Every id resolved before the first upload: an unknown or ambiguous name stops the run with nothing sent.
        names = sorted({n for u in uploads for n in u["versionNames"]})
        ids = curseforge_ids(token, names) if token else {n: "unresolved, no token" for n in names}
        for u in uploads:
            metadata = {"changelog": log, "changelogType": "markdown", "displayName": u["displayName"],
                        "gameVersions": [ids[n] for n in u["versionNames"]], "releaseType": "release"}
            if u["relations"]:
                metadata["relations"] = u["relations"]
            if dry:
                print("DRY curseforge %s: %s | %s | versions %s" % (u["loader"], os.path.basename(u["file"]),
                      u["displayName"], {n: ids[n] for n in u["versionNames"]}), flush=True)
            else:
                answer = curseforge_upload(token, u["file"], metadata)
                print("PUBLISHED curseforge %s: file id %s" % (u["loader"], (answer or {}).get("id")), flush=True)

    if "github" in only and uploads:
        title = "Balanced Minecraft Hardcore %s for %s" % (mod, mc)
        files = [u["file"] for u in uploads]
        if dry:
            print("DRY github release %s: %s | %s" % (tag, title, [os.path.basename(f) for f in files]), flush=True)
        else:
            notes = os.path.join(tree, "build", "release-changelog.md")
            os.makedirs(os.path.dirname(notes), exist_ok=True)
            open(notes, "w", encoding="utf-8").write(log + "\n")
            exists = subprocess.run(["gh", "release", "view", tag, "--repo", REPO], capture_output=True).returncode == 0
            if exists:
                gh("release", "upload", tag, "--repo", REPO, "--clobber", *files)
            else:
                gh("release", "create", tag, "--repo", REPO, "--verify-tag", "--title", title, "--notes-file", notes, *files)
            print("PUBLISHED github release %s (%d jars)" % (tag, len(files)), flush=True)

    if posts_discord:
        payload = discord_payload(mod, log)
        embed = payload["embeds"][0]
        if dry:
            print("DRY discord: %d characters, image %s" % (len(embed["description"]), embed.get("image", {}).get("url")), flush=True)
            print(embed["description"], flush=True)
        else:
            request(hook, json.dumps(payload).encode(), {"Content-Type": "application/json"}, "POST", shown="the Discord webhook")
            print("PUBLISHED discord: the %s changelog" % mod, flush=True)
    elif "discord" in only and mc == "1.21.1" and loader_arg != "all":
        print("SKIPPED discord: only with --loader all", flush=True)


main()
