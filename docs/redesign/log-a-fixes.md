# Part A log: fixes and rule tweaks (session `a-fixes`)

Brief: `docs/redesign/BRIEF.md`, part A. Two sides, each with its own branch and worktree:

| Side | Branch | Worktree | Cut from |
|---|---|---|---|
| main (Minecraft 1.20.4, JDK 17) | `redesign/a-fixes` | `/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/a-fixes-7acb8d` | `origin/redesign/integration` (432d0a2) |
| 1.21.1 (JDK 21) | `redesign/a-fixes-1.21` | `/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/a-fixes-7acb8d-121` | `origin/redesign/integration-1.21` (a886d42) |

Server port 25611, RCON port 25711, RCON password `bmh`. The step numbers below are not clock times: the clock was read once
(`date`: 2026-11-03 21:30 CET at the start).

## Resume here

Step 1 done (worktrees and branches). Nothing implemented yet. Next: A4 (totem nerf), A3 (exile scale), A2 (bundle `filled`
property plus the mod-bus client setup listener), then the 1.21.1 side (A1 tags and the ports).

## Plan (brief part A)

1. A1, 1.21.1 only: `bmh:eternal_sword|axe|pickaxe` into the vanilla item tags `swords`, `axes`, `pickaxes`, so enchantments
   apply to them again. Proof: RCON `enchant` of a zombie holding the eternal sword, on every loader.
2. A2, all sides: register the `filled` item property for `bmh:eternal_bundle` on each loader's client, and create the mod-bus
   client setup listener on both sides (part C adds its line to it).
3. A3, all sides: divide the exile distance by the respawn level's `coordinateScale()`; log the scaled distance.
4. A4, all sides: when a Totem of Undying saves a player, keep Regeneration and Absorption, remove Fire Resistance.
5. Changelog lines, `docs/redesign/checklist-a-fixes.md`, merge the integration branches, rebuild, push, report.

## Steps and calls

1. Checked the tree guard, renamed the branch (`claude/a-fixes-7acb8d` to `redesign/a-fixes`), read the brief and the repo
   CLAUDE.md, created the 1.21.1 worktree with `git worktree add ... -b redesign/a-fixes-1.21 origin/redesign/integration-1.21`.
   Call: that new branch inherited `origin/redesign/integration-1.21` as its upstream, so it is pushed only with an explicit
   `git push -u origin HEAD:refs/heads/redesign/a-fixes-1.21` after `git branch --unset-upstream`.

## Verification commands

(filled in as they run)
