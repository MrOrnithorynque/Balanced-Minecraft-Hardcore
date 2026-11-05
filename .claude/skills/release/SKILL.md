---
name: release
description: Release a new version of Balanced Minecraft Hardcore to CurseForge and a GitHub release (Modrinth is paused), for every Minecraft version at once, from this Mac. Use when the user wants to release, publish, ship, tag or put out a new version of the mod, bump mod_version, or asks why a release did not show up on Modrinth or CurseForge, or wants a release's changelog posted to Discord.
---

# Releasing BMH

A release is never "main only". The mod ships for every Minecraft version the repo supports, one branch per version (see the table in `.claude/CLAUDE.md`), and every one of those branches gets the same version number, the same changelog and its own tag.

Publishing runs from this Mac: `.claude/skills/release/publish_local.py`, once per version branch, on that branch's built release worktree. It uploads every jar to **CurseForge** (a Forge file, a Fabric file with Fabric API as a required dependency, and on 1.20.4 and 1.21.1 a NeoForge file; the 1.20.1 Forge file is also listed for NeoForge), creates one **GitHub release** per tag with every jar, and, for the 1.21.1 tag, posts the changelog on **Discord**. Modrinth is not published: its token answered 401 since 2026-11-04. GitHub Actions did this until 2026-11-05 (`publish.yml`); GitHub then locked the account's Actions over billing, so both workflows sit in `.github/disabled-workflows/` (its README says how to turn them back on). A tag no longer publishes anything by itself. An upload is public and cannot be cleanly taken back.

Run every multi-line snippet below as a `bash` script, with absolute paths: write it to a file in the scratchpad and run `bash <file>`. A `#!/bin/bash` first line pasted into a Bash tool call does nothing, zsh runs the code and leaves a list such as `$B` unsplit (2026-11-04: the atomic push of step 5 died that way, nothing pushed). Also: shell variables do not survive between tool calls, and zsh reads `$b:gradle.properties` as `$b` plus the `:g`/`:r` modifiers (`dev-46.0adle.properties`). Always write `"${b}:path"`.

## 0. Preconditions (stop and tell the user if one fails)

```bash
cd /Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore
git fetch origin --tags
git status --short          # clean tree
for b in main $(git for-each-ref --format='%(refname:short)' 'refs/heads/dev-*'); do
  echo "== $b behind/ahead: $(git rev-list --left-right --count "origin/${b}...${b}")"
  # dev-52.1.20 (1.21.1, and any later non-1.20.x branch) has its own sources: skip it here.
  [ "$b" = dev-52.1.20 ] || git diff --stat main "$b" -- . ':!forge/build.gradle' ':!gradle.properties' ':!README.md' ':!forge/src/main/resources/META-INF/mods.toml' ':!.vscode' | tail -1   # must print nothing
  git cat-file -e "${b}:.github/workflows/publish.yml" 2>/dev/null && echo "   publish.yml still active on $b: port the move to .github/disabled-workflows first"
  git cat-file -e "${b}:CHANGELOG.md" 2>/dev/null || echo "   NO CHANGELOG.md on $b"
done
security find-generic-password -s bmh-curseforge-token >/dev/null 2>&1 && echo "CurseForge token: in the Keychain" || echo "CurseForge token: MISSING"; security find-generic-password -s bmh-discord-webhook >/dev/null 2>&1 && echo "Discord webhook: in the Keychain" || echo "Discord webhook: MISSING"; gh auth status
```

- Every branch is level with `origin` (`0	0`). Anything ahead means unpushed work; anything behind means pull first. A branch never pushed yet (`dev-52.1.20` before its first push) has no `origin/` ref and prints a fatal here. Steps 3 and 5 start from `origin/<b>`, so push a new branch once before step 3: `git -C /Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore push -u origin <b>`.
- Everything but `forge/build.gradle`, `gradle.properties`, `README.md` and `.vscode` is identical on every 1.20.x branch (`mods.toml` excluded: `dev-46.0.14` lacks a commented-out template block, nothing else). If a branch differs, a change was not ported: port it (cherry-pick from `main`) before releasing. `dev-52.1.20` (1.21.1) has its own sources and is skipped by that check: compare `git log` of `main` and of `dev-52.1.20` since the last `mc-1.20.4-v*` tag instead, and make sure every change since then reached it (ported by hand where the 1.21.1 code differs, see `.claude/CLAUDE.md`).
- Step 5 needs `matrix.sh`, which needs mineflayer. The variable `BMH_BOT_MODULES` is not saved anywhere: find an install (`ls <dir>/node_modules/mineflayer`, see the `verify-loaders` skill, Setup). If there is none, ask the user now whether to install it from npm, not after the builds.
- Other sessions push to these branches too. Base the release on `origin/<b>` as fetched now, and fetch again right before step 5: a branch that moved means a rebase first.
- The CurseForge author token is in the macOS Keychain (service `bmh-curseforge-token`), and the Discord webhook URL (service `bmh-discord-webhook`) if the user wants the changelog on Discord. Never handle a secret yourself: the user stores it once in their own terminal, `security add-generic-password -a "$USER" -s bmh-curseforge-token -w` (the command asks for the value; it never shows in the shell history). The token comes from https://authors.curseforge.com/#/settings/api-tokens. The webhook URL comes from the Discord server: the #changelog channel's settings, Integrations, Webhooks, Copy URL (a GitHub secret cannot be read back). `gh auth status` shows a login that can create releases on the repository.
- Modrinth gets nothing from this release path: tell the user before step 6.
- `.github/workflows/publish.yml` is in no branch (the loop above prints a line otherwise). A branch that still has it would publish twice once GitHub Actions works again: the tag push and the local run.

## 1. Pick the version

Read the current `mod_version` from `gradle.properties` and the `## [Unreleased]` section of `CHANGELOG.md`. Propose the next number (semver: fixes only = patch, new feature = minor, breaking for existing worlds = major) and confirm it with the user. It is the same number on every branch.

Check it is free: `git tag -l "mc-*-v<version>"` must be empty.

## 2. Changelog and version bump, on main first

Work in detached scratch worktrees, never in the main checkout. That checkout sits on one of the release branches, and `git worktree add <dir> <branch>` refuses a branch that is checked out anywhere (`fatal: '<branch>' is already used by worktree`).

```bash
R=/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore
W=<absolute scratchpad path>/wt
git -C "$R" fetch origin --tags
git -C "$R" worktree add -q --detach "$W/main" origin/main
```

In one commit in `$W/main`:

- `gradle.properties`: `mod_version=<version>`.
- `CHANGELOG.md`: rename `## [Unreleased]` to `## [<version>] - <YYYY-MM-DD>` and add a fresh empty `## [Unreleased]` above it. If the section is empty or thin, write it from `git log <last tag>..HEAD` (the last `mc-1.20.4-v*` tag): one line per change a player would notice, in player words.
- If a recipe changed since the last release, regenerate its README picture from the recipe JSON, in `$W/main`: `python3 -I .claude/skills/release/make_grids.py resources` (`git diff --stat resources` shows which pictures moved).

`publish_local.py` publishes that `## [<version>]` section, and only that section, as the changelog on CurseForge, the GitHub release and, for 1.21.1, Discord. Players read it there, so it must stand on its own. If the heading is missing or the section is empty, the script stops before it checks or uploads anything.

Commit message: `release: <version>`.

## 3. Port to every version branch

Cherry-pick that commit onto each `dev-*` branch, each in its own detached scratch worktree:

```bash
#!/bin/bash
R=/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore
W=<absolute scratchpad path>/wt
C=$(git -C "$W/main" rev-parse HEAD)   # the release commit of step 2
for b in dev-52.1.20 dev-49.0.31 dev-49.0.2 dev-48.1.0 dev-47.2.0 dev-46.0.14; do   # the real list from step 0
  git -C "$R" worktree add -q --detach "$W/$b" "origin/$b" || { echo "$b: no worktree, stop and look"; continue; }
  git -C "$W/$b" cherry-pick "$C" || echo "CONFLICT on $b"
done
```

To ship work that sits on a coordinated run's branches (one integration branch per codebase and one port branch per `dev-*`, as in 1.5.0), start each worktree from the run's branch instead of `origin/<b>`, `$W/main` included. The push in step 5 stays a fast-forward as long as the run's branch contains `origin/<b>`.

`mod_version` sits far from the per-branch lines of `gradle.properties` and `CHANGELOG.md` is identical everywhere, so this should apply cleanly. If a conflict appears (`forge/build.gradle`, `gradle.properties` and `README.md` all differ per branch), keep the branch's own version of every line the release commit did not touch and apply only that commit's hunks. Never resolve by taking `main`'s whole file: `dev-46.0.14`'s and `dev-52.1.20`'s `forge/build.gradle` have a different layout.

## 4. Build every branch

```bash
#!/bin/bash
W=<absolute scratchpad path>/wt
for b in dev-52.1.20 dev-49.0.31 dev-49.0.2 dev-48.1.0 dev-47.2.0 dev-46.0.14; do   # the real list from step 0
  # JDK 17 for 1.20.x, 21 for 1.21.1: the branch's build.gradle says which.
  java=$(grep -o 'JavaLanguageVersion.of([0-9]*)' "$W/$b/build.gradle" | grep -o '[0-9]\+')
  export JAVA_HOME=/opt/homebrew/opt/openjdk@$java
  if ( cd "$W/$b" && sh ./gradlew build --console=plain ) > "$W/../build-$b.log" 2>&1; then
    forge=$(find "$W/$b/forge/build/libs" -name '*.jar' | head -1)
    fabric=$(find "$W/$b/fabric/build/libs" -name '*.jar' ! -name '*-sources.jar' | head -1)
    # A branch with neoforge_version must also have built the NeoForge jar.
    neo=ok; grep -q '^neoforge_version=' "$W/$b/gradle.properties" && ! ls "$W/$b"/neoforge/build/libs/*.jar >/dev/null 2>&1 && neo=missing
    # Forge 1.21.1 runs on the official names: its jar has no refmap, on purpose.
    { [ "$java" = 21 ] || unzip -l "$forge" | grep -q bmh.refmap.json; } && unzip -p "$forge" META-INF/MANIFEST.MF | grep -q MixinConfigs \
      && unzip -l "$fabric" | grep -q bmh.refmap.json && [ "$neo" = ok ] \
      && echo "$b PASS" || echo "$b FAIL (a jar has no refmap, the Forge jar no MixinConfigs, or the NeoForge jar is missing)"
  else
    echo "$b FAIL (see build-$b.log)"
  fi
done
```

Run it in the background (the first build of an old Minecraft version downloads its setup). A branch that fails is not released: report it and stop, do not tag the others "for now" without asking.

## 5. Push the branches

Check that each worktree's commit is a fast-forward of its branch, push them all in one atomic push, then move the local branches:

```bash
#!/bin/bash
R=/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore
W=<absolute scratchpad path>/wt
B="main dev-52.1.20 dev-49.0.31 dev-49.0.2 dev-48.1.0 dev-47.2.0 dev-46.0.14"   # the real list from step 0
git -C "$R" fetch origin
specs=()
for b in $B; do
  sha=$(git -C "$W/$b" rev-parse HEAD)
  git -C "$R" merge-base --is-ancestor "origin/$b" "$sha" || { echo "$b: not a fast-forward of origin/$b, stop and look"; exit 1; }
  specs+=("$sha:refs/heads/$b")
done
git -C "$R" push --atomic origin "${specs[@]}" || exit 1
for b in $B; do
  sha=$(git -C "$W/$b" rev-parse HEAD)
  git -C "$R" merge-base --is-ancestor "refs/heads/$b" "$sha" && git -C "$R" branch -f "$b" "$sha" \
    || echo "$b: local branch not moved (checked out somewhere, or it has commits the release lacks)"
done
```

No force push. If the push is rejected, someone pushed meanwhile: stop and look. Once pushed, tell the user to pull in their checkout: the branch it has checked out keeps its old commit.

Run `bash .claude/skills/verify-loaders/scripts/matrix.sh` (as the `verify-loaders` skill says); every cell green before tagging.

## 6. Tag every version branch

One annotated tag per `dev-*` branch, named from **that branch's** `gradle.properties` (`publish_local.py` refuses a tag that does not match it or that is not on the worktree's commit):

```bash
#!/bin/bash
cd /Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore
W=<absolute scratchpad path>/wt
git fetch origin
for b in dev-52.1.20 dev-49.0.31 dev-49.0.2 dev-48.1.0 dev-47.2.0 dev-46.0.14; do   # the real list from step 0
  [ "$(git rev-parse "origin/${b}")" = "$(git -C "$W/$b" rev-parse HEAD)" ] || { echo "$b: origin/$b is not the commit step 5 pushed, stop and look"; continue; }
  mc=$(git show "origin/${b}:gradle.properties" | grep '^minecraft_version=' | cut -d= -f2)
  mod=$(git show "origin/${b}:gradle.properties" | grep '^mod_version=' | cut -d= -f2)
  git tag -a "mc-${mc}-v${mod}" "origin/${b}" -m "Balanced Minecraft Hardcore ${mod} for Minecraft ${mc}"
done
```

- Tag `origin/<b>` after a fetch, and only where it is the commit step 5 pushed. Never tag the local branch: the one checked out in the main checkout still points at its old commit.
- Tag the `dev-*` branches, not `main`: `main` is the same Minecraft version as `dev-49.0.31`, and the same tag name cannot sit on two commits.
- Format is exactly `mc-<mc>-v<mod>`.

**Show the user the list of tags and the commit each points to, and get a clear yes**: a tag is public and slow to move. Then push them (`git push origin <tag> ...`): the GitHub release of step 7 needs its tag on GitHub, and the script checks it before any upload. With the workflows disabled, a tag push starts nothing, so they can go in one push.

## 7. Publish from this Mac

First a dry run on every worktree. It uploads and posts nothing; it runs `jarcheck.py`, then prints each file, its CurseForge name and game versions (with the token, the CurseForge id of each one: a Minecraft version name such as 1.21.1 exists in three game version types, Minecraft, Addons and type 1, so the script sends the id from the minecraft-* type, and stops before any upload on a name with no match or several), the GitHub release and the Discord embed:

```bash
W=<absolute scratchpad path>/wt
# Each worktree's own copy: it carries the release commit (the main checkout may not have the script yet).
for b in dev-52.1.20 dev-49.0.31 dev-49.0.2 dev-48.1.0 dev-47.2.0 dev-46.0.14; do python3 -I "$W/$b/.claude/skills/release/publish_local.py" "$W/$b" --dry-run; done
```

**Show the user the dry run and get a clear yes**, then run the same loop without `--dry-run`. Each line `PUBLISHED curseforge <loader>: file id <n>`, `PUBLISHED github release <tag>` and `PUBLISHED discord` says what went out. On an `ERROR` line (a refused upload shows the server's answer) nothing more is sent for that worktree: fix the cause, then publish only what is missing with step 8 (a second full run uploads the CurseForge files again).

Then check: the CurseForge files page (https://www.curseforge.com/minecraft/mc-mods/balanced-minecraft-hardcore/files) lists each new file once (a new file waits for CurseForge's review before it shows to players), and `gh release view <tag> --repo MrOrnithorynque/Balanced-Minecraft-Hardcore --json assets` lists the jars. Report each Minecraft version and loader. Keep the scratch worktrees until nothing is left to publish (steps 8 and 9 run on them), then remove them (`git worktree remove`).

## 8. Publishing one platform or one jar late

`--only curseforge`, `--only github` or `--only discord` (comma separated for several), and `--loader forge`, `fabric` or `neoforge` for one jar. The GitHub step uploads to the release when it exists already (`--clobber` replaces a jar of the same name), so it is safe to repeat; a CurseForge upload is not: each run adds a file. Confirm the list with the user first: it publishes.

## 9. Discord

Only the 1.21.1 worktree posts, and only with `--loader all`: `python3 -I "$W/dev-52.1.20/.claude/skills/release/publish_local.py" "$W/dev-52.1.20" --only discord`. Without the webhook in the Keychain it prints `SKIPPED discord` and posts nothing. Image lines leave the text; the first image becomes the embed's picture (Discord animates a GIF there). Confirm with the user first: the post is public, and a second run posts a second message.

## Turning GitHub Actions back on

When the account's Actions work again, move the two files back on every branch (`git mv .github/disabled-workflows/build.yml .github/disabled-workflows/publish.yml .github/workflows/`, then delete `.github/disabled-workflows`), and the tag push publishes again; push the tags one per `git push` then (GitHub starts no run for more than three tags at once), Modrinth included once its token works. The old steps 6 to 9 for the workflow are in the git history of this file (before 2026-11-05).
