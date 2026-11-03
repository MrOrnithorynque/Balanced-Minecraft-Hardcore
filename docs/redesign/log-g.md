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
| A fixes | `4353418` | `ebf5363` | `54299cc` / `29f506c` | main 50 s, 1.21 49 s, both green | none (clean merge on both sides) |

## Orders of the coordinator (wake of 2026-11-04, one turn, no report between steps)

All six parts reported DONE at: A `4353418` / `ebf5363`, B `20c3e07` / `242a55c`, C `09fee5e` / `660976a`, D `7d42e91` / `b41955c`,
E `8bef4bb` / `3079dc9`, F `5fd953a` / `664ca7e` (main / 1.21). Hand-over notes of the parts are in the shared notes folder.

1. Merge in the order A, B, F, C, D, E, each with `--no-ff` on both sides, both sides built, both integration branches pushed, a ledger line here.
   - `CHANGELOG.md`: one `## [Unreleased]` with a single Added, Changed and Fixed, every line kept.
   - `bmh.mixins.json`: union of every part's names (A, B, C, E in `mixins`, D in `client`); on 1.21.1 drop `EnchantmentMixin` (C deleted the class).
   - `dormant_totem.png`, `echo_heart.png`: F's files win.
   - Forge `BalancedMcHardcoreMain` and `BalancedMcHardcoreFabricClient`: A and C both add hooks, keep both until call 2a.
   - Once C and D are in, grep both sides for `bmh_reading`: every key C writes must be a key D reads, same name and type.
2. Five integration calls, one commit each on both sides, then build both sides:
   - 2a: C's needle registration moves into A's `ClientItemProperties.registerAll` (`minecraft:`-namespaced `angle`); delete `ForgeAmuletClient`, `NeoForgeAmuletClient`, `FabricAmuletClient` and their hooks.
   - 2b: a star reading is refused, no charge spent, when the player cannot see the sky (Blindness, Darkness, eyes in water or lava, powder snow); reuse C's no-open-sky message.
   - 2c: one DARK_AQUA ability line per Eternal tool in `EternalTooltips` style, before B's gold line (`tooltip.bmh.ability.eternal_sword|axe|pickaxe`); B's tooltip changelog line gains "and each Eternal tool names its ability".
   - 2d: keep "The Totem of Eternal Life"; the new item becomes "Dormant Totem of Eternal Life" (lang, B's checklist, changelog).
   - 2e: the creative tab title shows a raw key (the tab asks for `itemGroup.BMH_tab`): add that lang key and a `### Fixed` changelog line.
3. Re-run B's 73 checks and E's 44 checks on the merged integration, both sides, Forge, Fabric, NeoForge, one suite after the other, never committed. A check that asserts the old dormant name or a changed tooltip line count: fix the test, not the code. Another failure: fix on `redesign/g(-1.21)` if the cause is clear, else `BLOCKED: <failing line>`.
4. Port (brief G.2) and build the five ports. No server boots and no refmap/javap jar checks (the coordinator runs them after DONE). A port that does not compile: fix the cause on the integration side and port again.
5. One `docs/redesign/CHECKLIST.md` (brief G.4) with two added sections: "Coordinator's calls" (2b to 2e with reasons) and "Open before release" (README crafting-grid pictures still show the old recipes; NeoForge 1.20.1 runs the Forge jar without its own boot). Then report DONE with every branch and sha, build results, self-test counts per loader and side, conflicts resolved.

Tools (scratchpad of this session, patched copies of the parts' tools): `gradle-run.sh` (slot rule, log to a file), B's `boot.py`/`chain.sh`/`selftest.sh`, `fold_changelog.py`, `merge_mixins.py`. E's self-test is in the repo (`docs/redesign/e-selftest/` once E is merged).
Builds take about 50 s per side on a warm machine.

## Resume here

Step 1 of the orders in progress: A merged and built on both sides (ledger above). Next: push A, then merge B, F, C, D, E in that order. If this log is all you have: `git fetch origin --prune`, read the ledger to see which parts are in `origin/redesign/integration*`, continue from the next part.
