---
name: horizon-port-area
description: Port one area of Balanced Minecraft Hardcore to Horizon, or review and fix one, as the coordinator of the run (branch horizon-26.3). Use when asked to continue the Horizon port, port homecoming, eternal, totem, ritual, tools, amulet or sky, merge an area agent's worktree, review a ported area, run a fix round, write an area agent's prompt, or decide the order of the areas left.
---

# Port, review and fix one area (coordinator)

You are the coordinator. Area agents write the code in their own worktrees. You commit, merge, gate and review. Read first: `.claude/CLAUDE.md`, `docs/horizon/ARCHITECTURE.md`, the section "Resume here" of `docs/horizon/LOG.md`, and `docs/horizon/briefs/port.md` (the brief an area agent gets). For builds and test scripts load the skill `horizon-build-test`.

## Why the method is this one

- The Rust crate compiles as a whole. In a shared tree a half-written file of one agent breaks the build of every other agent. So one area agent works in its own git worktree (the Agent tool, or a workflow agent, with isolation "worktree").
- Trap: the app cuts that worktree from `origin/<default branch>` (the app's default for a new worktree), not from your HEAD. So you commit first, give the tip sha as BASE in the prompt, and the brief's first commands reset the agent's throwaway branch (its name starts with `worktree-`) to BASE.
- Reviews are not optional. The respawn area compiled and passed 31 unit tests, and two reviewers still found 12 findings, 4 of them major (`docs/horizon/review/respawn-*.json`). The skeleton's review found five holes in the contracts between areas (LOG.md step 2). Two load-bearing conclusions of the first audits were wrong until an independent verifier re-read the code.
- A compile and unit tests prove little. The game runs with the mod since 2026-11-17, so a report says that a feature "works" only with its in-game check (section "In the game").

## Procedure

1. Guards. `git rev-parse --show-toplevel` must print this repo's root. `git branch --show-current` must print `horizon-26.3`. If it prints a `claude/...` branch, run `git switch horizon-26.3` (the desktop app resets the branch when it restarts the session; `horizon/` then shows as untracked build folders and nothing is lost). `git status --short` must be clean.
2. Usage. Read the usage before each fan-out (the memory note names `get_usage`). Stop rules of the owner (LOG.md "Resume here"): stop cleanly when the 5-hour window reaches 60 %; stop when the week passes 40 % (all models) and 50 % (Fable). On 2026-11-12 the owner's weekly guard stopped the run; ask the owner which number stops the run before the next large fan-out. At a stop: merge and commit what is finished, then update "Resume here" in `docs/horizon/LOG.md`.
3. Gates on your tree before you fan out: `sh horizon/tools/test-logic.sh` and `sh horizon/tools/check.sh`. Both must pass. Fix the brief first if it names a path that changed (it names the cache paths and, in its guard, the coordinator's tree path).
4. Commit everything. `BASE=$(git rev-parse HEAD)`. The agent's prompt carries this full sha.
5. Start one agent per area, each in its own worktree (isolation "worktree"). Models: Sonnet for a bounded port (settings was such a port), Opus for new structural code (the respawn search, the journey drawing) and for the host-game review lens. Areas that do not read each other's new code can run at the same time; the order below says which cannot.
6. The area agent's prompt (see the table below for the per-area parts):
   - "Read `docs/horizon/briefs/port.md` in your worktree and follow it."
   - BASE (the full sha).
   - The area name, its module file and logic file, and its log file `docs/horizon/log/<area>.md`.
   - Its GAPS.md rows (`docs/horizon/GAPS.md`, section 2).
   - The audit and verify sections for those rows (`docs/horizon/audit/<n>-*.md` and `verify-<n>.md`). Where they differ, the verification wins.
   - Its Java sources under `common/src/main/java/github/mrornithorynque/bmh/` and `neoforge/src/main/java/github/mrornithorynque/bmh/neoforge/`.
   - The contracts it gives and reads (`docs/horizon/HOOKS.md`, "Contracts").
   - The ported areas' logs as worked examples (`docs/horizon/log/respawn.md`, `settings.md`, `journey.md`) and the open review findings that touch its contracts.
   - The non-functional rules, in the first prompt and not later: both gates must pass; Simplified Technical English; no em-dash or en-dash; no git add, commit, stash or branch change; no sub-agents; no background jobs; the structured result the brief asks for.
   - For any agent that touches files: absolute paths and a first command that checks the tree (`git rev-parse --show-toplevel`; stop if it is the coordinator's tree or a stale path).
7. Merge the agent's work. In the agent's worktree: `git add -A` then `git commit -m wip`. In your tree: `git cherry-pick -n <that sha>`. Then run `sh horizon/tools/test-logic.sh` and `sh horizon/tools/check.sh`. If both pass, commit with a real message (no em-dash). Then `git worktree remove --force <worktree>` and `git branch -D <its worktree-* branch>`.
8. Update `docs/horizon/LOG.md` (a step with the commit, the test count and what is not proven) and `docs/horizon/HOOKS.md` if the agent wired a hook (its report names every change outside its files; read each one).
9. Review (next section), then the fix round, then update LOG.md "Resume here".

## In the game

The game runs since 2026-11-17 (skill `horizon-build-test`). So a port is not done at the merge:

- Give "game rights" to one agent at a time, in its prompt: only that agent runs `check.sh build`, `test-slice.sh` or the game. Two builds at once swap the binary under a running test.
- Each area adds its scenario to `horizon/tools/test-slice.sh` (PASS and FAIL lines, never weakened) and its row to `docs/horizon/TESTING.md`. A check that fails on a defect of the mod stays red until the mod is fixed.
- Evidence or it did not happen. A screenshot that a report cites must be on disk at that path when the agent finishes, kept under `docs/horizon/log/<name>-shots/` and scaled down to 1280 pixels. Open it yourself before you relay the claim: on 2026-11-17 an agent cited two screenshots that were not on disk, and its claim was thrown out.
- The owner's own play folder is `play/` in the cache: no agent touches it.

## Review after the merge

Two read-only reviewers per area, with different lenses, in clean contexts:

- Parity with the Java mod (Sonnet): for each formula, constant, order of conditions, text key, sound and message in the Java source of the area, find the Rust counterpart and compare. Also check that every difference is listed in the area log's "Differences from the Java mod" and allowed by GAPS.md.
- Use of the host game (Opus): for every hook and helper the area uses, open its doc comment and its call site in the Nightly's tree (`~/Library/Caches/bmh-horizon/horizon-mods/native/target/horizon/fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca`). Check the side (window's game, server's game or both), whether a player is swapped in at that point, what the hook may change, and the order of calls.

Brief them as read-only outside the filesystem too: no change to any file, and no command that changes the cache or a game. Give each an adversarial task and the evidence rule: every finding carries its evidence on both sides, as `path:line` and the quoted line. The finding is dropped if the quote cannot be found at that place.

Save the findings as JSON under `docs/horizon/review/<area>-parity.json` and `<area>-semantics.json`. The shape (as in `respawn-*.json`): an object with `area`, `lens`, `reviewed_commit`, `status` ("open: not fixed yet" until fixed), `verdict`, and `findings`: a list of objects with `severity` (`major` or `minor`), `file`, `what`, `evidence` (ours, the host game, the Java source), `fix`.

## Fix round

One fix agent per area, in its own worktree cut from a new BASE. Its prompt names the review files and says: check each finding's evidence first (open the file at the line and read the quote); change nothing for a finding whose evidence you cannot find or that the code refutes; report each finding as fixed, refuted or not fixed with the reason. Both gates must pass. Merge as in step 7. Set `status` in the JSON files and update LOG.md.

State of the review at the time of writing (check LOG.md "Resume here", it is the live truth): the findings of the respawn area are closed (`status` in `docs/horizon/review/respawn-*.json`: fixed, one deferred). Settings and journey are not reviewed yet.

## The areas, the files and what each needs

The area names and files come from `ARCHITECTURE.md` ("Areas, and who owns which file"). Done: respawn, settings, journey (merged, see LOG.md step 3). Left: homecoming, eternal, totem, ritual, tools, amulet, sky.

The GAPS rows are my reading of ARCHITECTURE.md and GAPS.md (not stated in one place): confirm them against both files before you write a prompt. The audit file for a row is named in the Source column of GAPS.md section 2 (`v1` is `verify-1.md`, `a1` is `1-respawn-server.md`).

| Area | Module and logic file | GAPS rows | Java sources (`handlers/` etc. under `bmh/`) | Contracts it gives | Contracts it reads |
|---|---|---|---|---|---|
| respawn (done, verified in the game) | `bmh_respawn.rs`, `respawn.rs` | C1 to C9 | `RespawnPositionHandler`, `RespawnSearch`, `RespawnEffectHandler`, `JourneySignal`, `mixin/F_*` (server) | Journey signals (`wire::JourneySignal`), `Exile` | Eternal Items, rules and config, Held Respawn request |
| settings (ported; fix round 1 merged on 2026-11-18; not reviewed) | `bmh_settings.rs`, `settings.rs` | S1 to S4 | `utilities/BMHGameRules`, `BMHConfigLimits`, `neoforge/NeoForgeConfig`, `client/config/ConfigScreen`, `mixin/CreateWorldGameTabMixin`, `N_*` | `bmh_settings::rule`, `bmh_settings::config`, the `/bmh rule` subcommand | none |
| journey (ported; defects open, not reviewed) | `bmh_journey.rs`, `journey.rs` | C4 (client side), C15, C16, C17 | `client/journey/*`, `mixin/F_GuiTitleMixin`, `F_LocalPlayerRespawnMixin`, `F_MinecraftOverlayMixin` | Held Respawn request (`wire::RespawnRequest`) | Journey signals |
| amulet | `bmh_amulet.rs`, `amulet.rs` (and `bmh_amulet_world.rs`) | A1 to A6 | `items/NavigatorAmuletItem`, `handlers/Amulet*`, `client/amulet/*` | `bmh_amulet::points(game, player) -> Vec<contract::AmuletPoint>`, the reading (no contract fixed yet), the `/bmh point` subcommand | Eternal Items, rules and config |
| sky | `bmh_sky.rs`, `stars.rs` (and `bmh_sky_render.rs`) | V1, V2 | `client/sky/*`, `mixin/D_*`, `K_*` | none | the amulet's reading and points (no contract fixed yet) |
| homecoming | `bmh_homecoming.rs`, `homecoming.rs` | C10 | `handlers/HomecomingHandler`, `mixin/C_ServerPlayerMixin`, `E_ServerPlayerMixin` | none | `Exile` (`bmh_homecoming::on_exile`), amulet points, rules and config |
| eternal | `bmh_eternal.rs`, `eternal.rs` | C11, C13, C14, E10, E11, E12 | `handlers/KeepItemAfterDeath`, `EternalItemHandler`, `SleepHandler`, `DisableDebugInfo`, `items/EternalBundleItem`, `mixin/G_ItemEntityMixin` | Eternal Items (`ids::is_eternal`, integrator) | rules and config |
| totem | `bmh_totem.rs`, `totem.rs` (and `bmh_totem_world.rs`) | T1, T2, T3, T4, T5, T6, T11, T12, T13, T14, E6 | `handlers/TotemOfEternityHandler`, `TotemAwakeningHandler`, `mixin/A_*`, `B_AnvilMenuMixin`, `I_*` | none | Eternal Items |
| ritual | `bmh_ritual.rs`, `moon.rs` | T7, T8, T9, T10, T15 | `handlers/MoonRitualHandler`, `items/DormantTotemOfEternityItem`, `client/Ritual*` | none | none named in HOOKS.md |
| tools | `bmh_tools.rs`, `tools.rs` (and `bmh_tools_lib.rs`) | C12, E2 to E5, E7, E8, E9 (E1 is content) | `handlers/EternalWear`, `EchoStrikeHandler`, `AxeSmashHandler`, `EchoSenseHandler`, `items/Eternal*`, `mixin/E_*`, `L_*` | none | Eternal Items, rules and config |

Audit files by area: respawn, homecoming, eternal (C11, C13, C14) and settings (S1, S3, S4 in audit 1; S2 and S4 in audit 4; S1 and S2 also in audit 5, G4): `1-respawn-server.md`, `verify-1.md`. Totem and ritual: `2-totems.md`, `verify-2.md`. Tools and eternal (E10 to E12): `3-items-tools.md`, `verify-3.md`. Amulet, sky, journey and the config screen: `4-amulet-client.md`, `verify-4.md`. Platform questions: `5-platform.md`, `verify-5.md`. Which hooks fire for the single-player owner, and the traps: `6-single-player-owner.md`.

### Order of the areas left

Amulet first, before sky and homecoming: they read its points and its reading. Sky and homecoming after it. The only order the owner gave is amulet before sky and homecoming. By the Contracts table of HOOKS.md, eternal, totem, ritual and tools share no contract with the amulet except `ids::is_eternal`, which already exists, so they may run beside it (my reading, not checked by a run). Stop at a usage limit (step 2) and record where you stopped.

When the amulet area gives a value that sky or homecoming reads, fix its shape as an integrator change in `logic/src/contract.rs` (and a row in HOOKS.md "Contracts") before sky or homecoming starts, so the two owners can work from the same signature.

## The integrator files and how an area may touch them

Integrator files: the manifest `horizon-mixin.toml`, every `hook_*.rs`, `bmh.rs`, `logic/src/lib.rs`, `logic/src/wire.rs`, `logic/src/ids.rs`, `logic/src/contract.rs`, `logic/Cargo.toml`, HOOKS.md, and the content files that HOOKS.md ("Content files") does not give to an area.

An area agent edits these only in its own worktree, only as small as possible, and names every change in its log ("Changes outside my files") and in its report. Allowed reasons (port.md): wire a hook the area needs that is not wired yet (a new `hook_<hook>.rs`, its `[[injection]]`, its HOOKS.md row); add a binding that the area's function needs to an existing helper (and pass it to that function only); add an id or a `pub mod` line; add a lang key to `en_us.json` and the same English text to the other lang files when Java had no key. Never change what another area's function receives. A mod message goes in `wire.rs` in a section with the area's name, as an integrator change. Content owners: advancements go to the area that awards them; `sounds.json` and `assets/bmh/sounds/` go to journey; the rest goes to the integrator.

When you merge, read each such change yourself: two agents may have changed the same integrator file. Resolve a conflict by keeping both areas' lines. Order matters in a `hook_*.rs` helper where the Java mod used event priorities: HOOKS.md records the order and its reason.

## Closing a session

Update `docs/horizon/LOG.md`: a numbered step per merged area with its commit and test count, and the section "Resume here" with the date, what is done, what is open in order, and why the run stopped. No clock times in the numbered steps (the "Resume here" entries do carry some). Commit. Leave the worktrees removed and the `worktree-*` branches deleted.
