# Homecoming log

Plan: `docs/homecoming/PLAN.md`. Numbered steps, no clock times. A fresh agent can resume from here.

Branches (all local, nothing pushed): `feat/homecoming` (1.20.4, from `origin/main` 89b3e7e), then `feat/homecoming-1.20`, `-1.20.1`, `-1.20.2`, `-1.20.3` (cherry-picks, each from `origin/mc-<version>`), and hand ports `feat/homecoming-1.21.1` (from `origin/mc-1.21.1`), `feat/homecoming-26.1` (from `origin/mc-26.1`).

Verify runs use `BMH_PORT`, `BMH_SELFTEST_PORT`, `BMH_LOCK_DIR`, `BMH_SLOT_PREFIX=/private/tmp/hc-gradle`, `BMH_SESSION=hc` and `BMH_VERIFY_OUT=/private/tmp/hc-verify`, so they do not meet the other session's locks.

## Steps

1. Read the plan and `.claude/CLAUDE.md`. `git fetch origin`. The session worktree was on `claude/homecoming-4ed075` at `origin/main`: renamed to `feat/homecoming` with `git switch -c`.
2. Wrote `handlers/HomecomingHandler.java`: the record (`Record`, `Home`), `onExile`, `tick`, the moment (title, subtitle, bell, particle ring), NBT `write` and `read` (tag `bmh_homecoming`).
3. `RespawnPositionHandler`: new `Exile(center, distance)` record and map, filled in `findRespawnPosition` next to the exile flag, read once with `takeExile`. `takeExiled` is unchanged.
4. `RespawnEffectHandler.onPlayerRespawn` calls `HomecomingHandler.onExile` after the hardcore check.
5. `C_ServerPlayerMixin` also implements `HomecomingHandler.Holder` (save, load, copy on `restoreFrom`). `E_ServerPlayerMixin` has a second `tick` HEAD inject that calls `HomecomingHandler.tick` when `tickCount % 20 == 0`.
6. Lang keys `bmh.homecoming.*` in all ten files, changelog line under `### Added`.
7. Selftest cell `hook/selftest/HomecomingTest.java` (added to `Tests.java`), with `subtitleText`, `saveData` and `loadData` helpers in the MockPlayers copies.

## Judgment calls

- The amulet points are Overworld points. They are homes only when the exile is in the Overworld. In an exile around a Nether anchor, the center is the only home.
- The nearest home within 24 blocks wins, the center wins a tie.
- Title of a named point: the point name as a literal text. The center: `bmh.homecoming.home`.
- Spectators and Creative players that fly home also trigger it. The record exists only after a Survival exile.
