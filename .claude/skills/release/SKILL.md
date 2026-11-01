---
name: release
description: Release a new version of Balanced Minecraft Hardcore to Modrinth and CurseForge, for every Minecraft version at once. Use when the user wants to release, publish, ship, tag or put out a new version of the mod, bump mod_version, or asks why a release did not show up on Modrinth or CurseForge.
---

# Releasing BMH

A release is never "main only". The mod ships for every Minecraft version the repo supports, one branch per version (see the table in `.claude/CLAUDE.md`), and every one of those branches gets the same version number, the same changelog and its own tag.

Publishing is automatic: pushing a tag named `mc-<minecraft_version>-v<mod_version>` runs `.github/workflows/publish.yml` on the tagged commit, which builds that commit and uploads both jars to **Modrinth and CurseForge** (and one GitHub release) in one go: a Forge version (`<mc>-<mod>`) and a Fabric version (`<mc>-<mod>-fabric`). One tag per version branch, so one release per Minecraft version and loader on both sites. Pushing a tag is the point of no return: the upload is public and cannot be cleanly taken back.

Run every multi-line snippet below as a `bash` script, with absolute paths: shell variables do not survive between tool calls, and zsh reads `$b:gradle.properties` as `$b` plus the `:g`/`:r` modifiers (`dev-46.0adle.properties`). Always write `"${b}:path"`.

## 0. Preconditions (stop and tell the user if one fails)

```bash
cd /Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore
git fetch origin --tags
git status --short          # clean tree
for b in main $(git for-each-ref --format='%(refname:short)' 'refs/heads/dev-*'); do
  echo "== $b behind/ahead: $(git rev-list --left-right --count "origin/${b}...${b}")"
  git diff --stat main "$b" -- common fabric forge/src CHANGELOG.md .github ':!forge/src/main/resources/META-INF/mods.toml' | tail -1   # must print nothing
  git cat-file -e "${b}:.github/workflows/publish.yml" 2>/dev/null || echo "   NO publish workflow on $b"
  git cat-file -e "${b}:CHANGELOG.md" 2>/dev/null || echo "   NO CHANGELOG.md on $b"
done
gh secret list    # needs MODRINTH_TOKEN and CURSEFORGE_TOKEN
```

- Every branch is level with `origin` (`0	0`). Anything ahead means unpushed work; anything behind means pull first.
- `common`, `fabric`, `forge/src`, `CHANGELOG.md` and `.github` are identical everywhere (`mods.toml` excluded: `dev-46.0.14` lacks a commented-out template block, nothing else). If a branch differs, a change was not ported: port it (cherry-pick from `main`) before releasing.
- The publish workflow exists **on every branch**. A tag only publishes if the workflow file is in the tagged commit. A branch without it releases nothing, silently.
- Both secrets are set. Do not try to set them yourself: the user does.

## 1. Pick the version

Read the current `mod_version` from `gradle.properties` and the `## [Unreleased]` section of `CHANGELOG.md`. Propose the next number (semver: fixes only = patch, new feature = minor, breaking for existing worlds = major) and confirm it with the user. It is the same number on every branch.

Check it is free: `git tag -l "mc-*-v<version>"` must be empty.

## 2. Changelog and version bump, on main first

In one commit on `main`:

- `gradle.properties`: `mod_version=<version>`.
- `CHANGELOG.md`: rename `## [Unreleased]` to `## [<version>] - <YYYY-MM-DD>` and add a fresh empty `## [Unreleased]` above it. If the section is empty or thin, write it from `git log <last tag>..main` (the last `mc-1.20.4-v*` tag): one line per change a player would notice, in player words.

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

`mod_version` sits far from the per-branch lines of `gradle.properties` and `CHANGELOG.md` is identical everywhere, so this should apply cleanly. If a conflict appears (`forge/build.gradle`, `gradle.properties` and `README.md` all differ per branch), keep the branch's own version of every line the release commit did not touch and apply only that commit's hunks. Never resolve by taking `main`'s whole file: `dev-46.0.14`'s `forge/build.gradle` has a different layout.

## 4. Build every branch

```bash
#!/bin/bash
W=<absolute scratchpad path>/wt
export JAVA_HOME=/opt/homebrew/opt/openjdk@17
for b in dev-49.0.31 dev-49.0.2 dev-48.1.0 dev-47.2.0 dev-46.0.14; do   # the real list from step 0
  if ( cd "$W/$b" && sh ./gradlew build --console=plain ) > "$W/../build-$b.log" 2>&1; then
    forge=$(find "$W/$b/forge/build/libs" -name '*.jar' | head -1)
    fabric=$(find "$W/$b/fabric/build/libs" -name '*.jar' ! -name '*-sources.jar' | head -1)
    unzip -l "$forge" | grep -q bmh.refmap.json && unzip -p "$forge" META-INF/MANIFEST.MF | grep -q MixinConfigs \
      && unzip -l "$fabric" | grep -q bmh.refmap.json \
      && echo "$b PASS" || echo "$b FAIL (a jar has no refmap, or the Forge jar no MixinConfigs)"
  else
    echo "$b FAIL (see build-$b.log)"
  fi
done
```

Run it in the background (the first build of an old Minecraft version downloads its setup). A branch that fails is not released: report it and stop, do not tag the others "for now" without asking.

## 5. Push the branches

```bash
git push origin main dev-49.0.31 dev-49.0.2 dev-48.1.0 dev-47.2.0 dev-46.0.14   # the real list from step 0
```

No force push. If a push is rejected, someone pushed meanwhile: stop and look.

## 6. Tag every version branch (Modrinth + CurseForge)

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
for t in mc-1.20-v<mod> mc-1.20.1-v<mod> mc-1.20.2-v<mod> mc-1.20.3-v<mod> mc-1.20.4-v<mod>; do
  git push origin "$t"
done
```

## 7. Check the publish runs

```bash
gh run list --workflow publish.yml --limit 10
gh run view <run-id> --log-failed     # for any failure
```

- One green run per tag means that Minecraft version is on both Modrinth and CurseForge, for Forge and Fabric. If the Forge step passed and the Fabric step failed, the Forge version is already public: fix and re-run the failed job only (re-running all would try to publish the Forge version twice).
- A red "Tag ... does not match gradle.properties" run means the tag was made on the wrong branch or before the bump.
- A red "CHANGELOG.md has no non-empty '## [<version>]' section" run means step 2 did not reach that branch: fix the changelog on every branch, push, then move the tag (delete and re-create it, after asking).
- No run at all for a tag means it went out in a push with more than three tags: delete it on the remote (`git push origin :refs/tags/<tag>`, after asking) and push it again alone.

Report each Minecraft version's result to the user, then clean up the scratch worktrees (`git worktree remove`).
