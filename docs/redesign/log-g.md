# Log: part G (integration and ports)

Session G of the BMH redesign run, started 2026-11-03. Spec: `docs/redesign/BRIEF.md`, part G plus rules 1 to 13.
Steps are numbered, not timed (this session reads no clock unless it runs `date`).

## Layout

| Side | Worktree | Branch | Publishes to |
|---|---|---|---|
| main (1.20.4, source of the 1.20.x ports) | `/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/g-integration-90410b` | `redesign/g` | `git push origin HEAD:redesign/integration` |
| 1.21.1 | `/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/g-integration-90410b-121` | `redesign/g-1.21` | `git push origin HEAD:redesign/integration-1.21` |

- Pushes to the integration branches are fast-forward only: fetch and merge `origin/redesign/integration*` first when the coordinator moved them.
- Neither branch has an upstream, on purpose: a bare `git push` can never aim at an integration branch.
- Never check out `redesign/integration*` (the coordinator's worktrees hold them). Never touch Louis's checkout or another session's worktree.
- `redesign/g-1.21` exists only locally until its first merge. To rebuild the second worktree:
  `git -C <main-side worktree> worktree add <main-side worktree>-121 -b redesign/g-1.21 origin/redesign/integration-1.21`.
- Gradle and server output go to `<worktree>/run/logs-g/` (`run` is gitignored). A Gradle run takes a slot in `/private/tmp/bmh-gradle.N`, a server boot takes `/private/tmp/bmh-runserver.lock` (rule 3). My server port is 25617, RCON 25717, password `bmh`.
- JDK: `JAVA_HOME=/opt/homebrew/opt/openjdk@17` on the main side and on every 1.20.x port, `/opt/homebrew/opt/openjdk@21` on 1.21.1.

## Part branches

| Part | Main-side branch | 1.21 branch | Log |
|---|---|---|---|
| A fixes | `redesign/a-fixes` | `redesign/a-fixes-1.21` | `log-a-fixes.md` |
| B totem | `redesign/b-totem` | `redesign/b-totem-1.21` | `log-b-totem.md` |
| C amulet | `redesign/c-amulet` | `redesign/c-amulet-1.21` | `log-c-amulet.md` |
| D sky | `redesign/d-sky` | `redesign/d-sky-1.21` | `log-d-sky.md` |
| E abilities | `redesign/e-abilities` | `redesign/e-abilities-1.21` | `log-e-abilities.md` |
| F sprites | `redesign/f-sprites` | `redesign/f-sprites-1.21` | `log-f-sprites.md` |

## State at READY (step 4)

- `origin/main` 5062a5e, `origin/redesign/integration` 432d0a2, `origin/redesign/integration-1.21` a886d42.
- `origin/dev-52.1.20` de633bf, `dev-49.0.31` 6c47565, `dev-49.0.2` f2e76ed, `dev-48.1.0` 1cf512f, `dev-47.2.0` 356288b, `dev-46.0.14` 4c9201c.
- Each integration branch is its base plus the same two docs commits (the brief and the shared assets). `docs/redesign` is identical on both sides.
- Baseline of the repo CLAUDE.md empty-diff check, each 1.20.x dev branch against `origin/main` (pathspec without `forge/build.gradle`, `gradle.properties`, `README.md`, `mods.toml`, `.vscode`): 0 lines for all five. A later non-empty diff comes from the port, not from drift.
- No Gradle slot or server lock was held at READY; this session took none.

## Recipes

### On `merge <part>`

1. Tree guard (`git rev-parse --show-toplevel` equals the worktree). `git fetch origin --prune`. Diff the brief
   (`git show origin/redesign/integration:docs/redesign/BRIEF.md`) against the worktree copy and re-read what changed. `ls` the shared notes folder.
2. Main side: merge `origin/redesign/integration` (catch-up), then `git merge --no-ff origin/redesign/<part branch>`. 1.21 side: the same with `origin/redesign/integration-1.21` and the `-1.21` branch.
3. Resolve the shared files (rule 8) by keeping the lines of both sides; where two parts add the same texture path, F's file wins.
4. Checks before building: `git diff --check`; every changed `*.json` parses and `en_us.json` has no duplicate key; no em-dash or en-dash in the lines the merge brought in.
5. Build each side, one Gradle run at a time (`build --max-workers=2`). Jar facts of the repo CLAUDE.md: Forge and Fabric jars hold `bmh.refmap.json`, the Forge jar a `MixinConfigs` manifest entry; the NeoForge jar never has a refmap; Forge on 1.21.1 has none either.
6. Push both integration branches, call `get_usage` (rule 9), report `MERGED <part> <sha> | <sha-1.21>`, wait.

### On `port`

1. Branch `redesign/port-<dev branch>` from `origin/<dev branch>` in its own worktree next to mine, then
   `git cherry-pick -m 1` over `git rev-list --reverse --first-parent origin/main..origin/redesign/integration` (kept in a shell array, zsh does not split a plain variable).
2. Conflicts in `forge/build.gradle`, `gradle.properties`, `README.md`, `mods.toml`: keep the branch's own version of every line the change did not touch, apply only the change's hunks. Never take `main`'s whole file.
3. Empty-diff check against `origin/redesign/integration`: `git diff --stat origin/redesign/integration <branch> -- . ':!forge/build.gradle' ':!gradle.properties' ':!README.md' ':!forge/src/main/resources/META-INF/mods.toml' ':!.vscode'` must print nothing.
4. A source fix needed for an older Minecraft goes into `redesign/g` first (so every 1.20.x branch stays identical), then the ports are redone.
5. Build all seven with the right JDK, boot a dedicated server on every loader each branch has. 1.20.1: NeoForge runs the Forge jar. 1.20.2 and 1.20.3, 1.20: no NeoForge.
6. Merge the parts' `checklist-*.md` into `docs/redesign/CHECKLIST.md`, ordered for one `runClient` session per loader. Push only `redesign/*` branches.

## Calls made

1. Dropped the upstream of `redesign/g-1.21` (it tracked `origin/redesign/integration-1.21`). Reason: publishing is always explicit here.
2. This log is committed on `redesign/g`, so G's first-parent history starts with a docs commit. The `-m 1` port walks the first-parent chain, so that commit and the two docs commits travel to the ports with the rest, as the brief's command says. Revisit at port time if the docs make a port conflict.
3. At merge time only the builds are required (part G, item 1). Server boots of the integrated mod on every loader of both sides happen once the last part is in, before the port starts (the port step needs boots on all seven branches anyway).

## Merge ledger

| Part | Main-side sha merged | 1.21 sha merged | Integration after (main / 1.21) | Builds | Conflicts resolved |
|---|---|---|---|---|---|
| none yet | | | | | |

## Resume here

READY sent after setup. Waiting for the coordinator's `merge <part>` messages, then `port`. On every wake: tree guard, `git fetch origin --prune`, re-read the brief from `origin/redesign/integration`, then follow the recipe above.
