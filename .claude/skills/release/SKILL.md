---
name: release
description: Release a new version of Balanced Minecraft Hardcore to Modrinth and CurseForge, for every Minecraft version at once. Use when the user wants to release, publish, ship, tag or put out a new version of the mod, bump mod_version, or asks why a release did not show up on Modrinth or CurseForge.
---

# Releasing BMH

A release is never "main only". The mod ships for every Minecraft version the repo supports, one branch per version (see the table in `.claude/CLAUDE.md`), and every one of those branches gets the same version number, the same changelog and its own tag.

Publishing is automatic: pushing a tag named `mc-<minecraft_version>-v<mod_version>` runs `.github/workflows/publish.yml` on the tagged commit, which builds that commit and uploads every jar to **CurseForge, plus Modrinth when `MODRINTH_TOKEN` is set** (and one GitHub release) in one go: a Forge version (`<mc>-<mod>`), a Fabric version (`<mc>-<mod>-fabric`) and, on 1.20.4 and 1.21.1, a NeoForge version (`<mc>-<mod>-neoforge`). The 1.20.1 Forge version is also listed for NeoForge. One tag per version branch, so one release per Minecraft version and loader on each platform reached. Pushing a tag is the point of no return: the upload is public and cannot be cleanly taken back.

Run every multi-line snippet below as a `bash` script, with absolute paths: shell variables do not survive between tool calls, and zsh reads `$b:gradle.properties` as `$b` plus the `:g`/`:r` modifiers (`dev-46.0adle.properties`). Always write `"${b}:path"`.

## 0. Preconditions (stop and tell the user if one fails)

```bash
cd /Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore
git fetch origin --tags
git status --short          # clean tree
for b in main $(git for-each-ref --format='%(refname:short)' 'refs/heads/dev-*'); do
  echo "== $b behind/ahead: $(git rev-list --left-right --count "origin/${b}...${b}")"
  # dev-52.1.20 (1.21.1, and any later non-1.20.x branch) has its own sources: skip it here.
  [ "$b" = dev-52.1.20 ] || git diff --stat main "$b" -- . ':!forge/build.gradle' ':!gradle.properties' ':!README.md' ':!forge/src/main/resources/META-INF/mods.toml' ':!.vscode' | tail -1   # must print nothing
  git cat-file -e "${b}:.github/workflows/publish.yml" 2>/dev/null || echo "   NO publish workflow on $b"
  git cat-file -e "${b}:CHANGELOG.md" 2>/dev/null || echo "   NO CHANGELOG.md on $b"
done
gh secret list    # CURSEFORGE_TOKEN required, MODRINTH_TOKEN optional (see below)
```

- Every branch is level with `origin` (`0	0`). Anything ahead means unpushed work; anything behind means pull first. A branch never pushed yet (`dev-52.1.20` before its first push) has no `origin/` ref and prints a fatal here: that is expected, step 5 pushes it.
- Everything but `forge/build.gradle`, `gradle.properties`, `README.md` and `.vscode` is identical on every 1.20.x branch (`mods.toml` excluded: `dev-46.0.14` lacks a commented-out template block, nothing else). If a branch differs, a change was not ported: port it (cherry-pick from `main`) before releasing. `dev-52.1.20` (1.21.1) has its own sources and is skipped by that check: compare `git log` of `main` and of `dev-52.1.20` since the last `mc-1.20.4-v*` tag instead, and make sure every change since then reached it (ported by hand where the 1.21.1 code differs, see `.claude/CLAUDE.md`).
- The publish workflow exists **on every branch**. A tag only publishes if the workflow file is in the tagged commit. A branch without it releases nothing, silently.
- `CURSEFORGE_TOKEN` is set. If `MODRINTH_TOKEN` is missing, `mc-publish` **silently skips Modrinth** (it skips every platform whose token is empty) and the run still goes green: tell the user and get their OK before tagging. Modrinth can get the version later with the manual trigger in step 8. Never set secrets yourself: the user does.

## 1. Pick the version

Read the current `mod_version` from `gradle.properties` and the `## [Unreleased]` section of `CHANGELOG.md`. Propose the next number (semver: fixes only = patch, new feature = minor, breaking for existing worlds = major) and confirm it with the user. It is the same number on every branch.

Check it is free: `git tag -l "mc-*-v<version>"` must be empty.

## 2. Changelog and version bump, on main first

In one commit on `main`:

- `gradle.properties`: `mod_version=<version>`.
- `CHANGELOG.md`: rename `## [Unreleased]` to `## [<version>] - <YYYY-MM-DD>` and add a fresh empty `## [Unreleased]` above it. If the section is empty or thin, write it from `git log <last tag>..main` (the last `mc-1.20.4-v*` tag): one line per change a player would notice, in player words.
- If a recipe changed since the last release, regenerate its README picture from the recipe JSON: `python3 -I .claude/skills/release/make_grids.py resources` (`git diff --stat resources` shows which pictures moved).

The workflow publishes that `## [<version>]` section, and only that section, as the changelog on Modrinth, CurseForge and the GitHub release. Players read it there, so it must stand on its own. If the heading is missing or the section is empty, the run fails before building.

Commit message: `release: <version>`.

## 3. Port to every version branch

Cherry-pick that commit onto each `dev-*` branch, in scratch worktrees so the main checkout is not switched around:

```bash
cd /Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore
W=<absolute scratchpad path>/wt
for b in $(git for-each-ref --format='%(refname:short)' 'refs/heads/dev-*'); do
  git worktree add -q "$W/$b" "$b" && git -C "$W/$b" cherry-pick <release-commit> || echo "CONFLICT on $b"
done
```

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

```bash
git push origin main dev-52.1.20 dev-49.0.31 dev-49.0.2 dev-48.1.0 dev-47.2.0 dev-46.0.14   # the real list from step 0
```

No force push. If a push is rejected, someone pushed meanwhile: stop and look.

## 6. Tag every version branch (CurseForge, plus Modrinth with a token)

One annotated tag per `dev-*` branch, named from **that branch's** `gradle.properties` (the workflow fails a tag that does not match it):

```bash
for b in $(git for-each-ref --format='%(refname:short)' 'refs/heads/dev-*'); do
  mc=$(git show "${b}:gradle.properties" | grep '^minecraft_version=' | cut -d= -f2)
  mod=$(git show "${b}:gradle.properties" | grep '^mod_version=' | cut -d= -f2)
  git tag -a "mc-${mc}-v${mod}" "$b" -m "Balanced Minecraft Hardcore ${mod} for Minecraft ${mc}"
done
```

- Tag the `dev-*` branches, not `main`: `main` is the same Minecraft version as `dev-49.0.31`, and the same tag name cannot sit on two commits.
- Format is exactly `mc-<mc>-v<mod>`. Any `mc-*` tag starts the workflow, but only a name matching the commit's `gradle.properties` publishes. The old `mc-1.20.2/v1.0.0` (with a slash) was a mistake and never triggers it.

**Before pushing, show the user the list of tags and the commit each points to, and get a clear yes.** Then push them **one tag per `git push`**. GitHub creates no event at all when more than three tags are pushed at once: no run starts, nothing is published, and nothing turns red.

```bash
for t in mc-1.20-v<mod> mc-1.20.1-v<mod> mc-1.20.2-v<mod> mc-1.20.3-v<mod> mc-1.20.4-v<mod> mc-1.21.1-v<mod>; do
  git push origin "$t"
done
```

## 7. Check the publish runs

Each run takes about 2 to 2.5 minutes.

```bash
gh run list --workflow publish.yml --limit 10
gh run view <run-id> --log | grep "Successfully published the assets to"   # which platforms were reached
gh run view <run-id> --log-failed     # for any failure
```

- Green only means nothing failed. The `Successfully published the assets to ...` lines (one per loader step) say where each jar actually went (e.g. `CurseForge, GitHub` when Modrinth was skipped for lack of a token). Report that list per Minecraft version and loader, not "published everywhere".
- The loader steps run independently (Fabric and NeoForge run even when Forge failed). If one failed, the others' versions are already public: publish only the failed loader with step 8 (`-f loader=fabric`, `forge` or `neoforge`), ticking only the platforms its `Successfully published` line does not list. Never re-run the whole run.
- A red "Tag ... does not match gradle.properties" run means the tag was made on the wrong branch or before the bump.
- A red "CHANGELOG.md has no non-empty '## [<version>]' section" run means step 2 did not reach that branch: fix the changelog on every branch, push, then move the tag (delete and re-create it, after asking).
- No run at all for a tag means it went out in a push with more than three tags: delete it on the remote (`git push origin :refs/tags/<tag>`, after asking) and push it again alone.

Report each Minecraft version's result to the user, then clean up the scratch worktrees (`git worktree remove`).

## 8. Publishing one platform late

When a platform was skipped (Modrinth without a token) or failed, **do not re-run the tag's run**: it would upload again to every platform that already has the file (duplicate CurseForge files). Use the manual trigger instead, one run per tag, choosing only the missing platform:

```bash
for t in mc-1.20-v<mod> mc-1.20.1-v<mod> mc-1.20.2-v<mod> mc-1.20.3-v<mod> mc-1.20.4-v<mod> mc-1.21.1-v<mod>; do
  gh workflow run publish.yml --ref main -f tag="$t" -f modrinth=true -f curseforge=false -f github=false
done
```

- Add `-f loader=forge`, `fabric` or `neoforge` to publish one jar only (default `all`; it was `both` before the NeoForge build, so a manual run with `--ref main` takes the new names). A tag from before the Fabric port only has the Forge jar, in `build/libs`; the workflow detects that layout, but `-f loader=fabric` on such a tag publishes nothing and still goes green: use `forge` or `all` there.
- The manual trigger exists only once this workflow is on `main` (it arrived with the Fabric port); before that, `gh workflow run` fails.
- `--ref main` on purpose: the workflow definition comes from `main` (older tags predate the trigger), and the job checks out and builds the tagged commit itself.
- Confirm the tag list and the platform with the user first: it publishes.
- Check each run with step 7. A platform whose token is still missing is skipped, so `No valid platform tokens found` in the log means nothing was published.
