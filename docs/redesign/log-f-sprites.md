# Log: part F, sprites

Session F (`f-sprites`) of the BMH redesign run. Spec: `docs/redesign/BRIEF.md`, part `F · Sprites`.
Branches: `redesign/f-sprites` (main side, this worktree), `redesign/f-sprites-1.21` (1.21.1 side, worktree
`<this worktree>-121`). Steps are numbered, not timed.

## What ships

Four 16x16 RGBA textures in `common/src/main/resources/assets/bmh/textures/item/`, the same bytes on both sides,
all static (no `.mcmeta`), opaque or fully transparent pixels only:

| File | What | Pixels changed |
|---|---|---|
| `eternal_bundle.png` | Louis's original + a 4x4 gold ring hanging under the drawstring's knot (x 8-11, y 12-15) | 8 |
| `eternal_bundle_filled.png` | Louis's original + the same ring under the end of the neck cord (x 8-11, y 10-13) | 8 |
| `dormant_totem.png` | frame 0 of `totem_of_eternity.png`, forehead gem dulled, both eyes closed | 12 |
| `echo_heart.png` | new: faceted crying-obsidian violet heart, four-point echo-cyan glint at its centre | all |

`docs/redesign/sprites/make_sprites.py` rebuilds all four and the sheets (`review-sheet.png`, `before-after.png`,
`family.png`) from Louis's originals read from git at `5062a5e` (release 1.4.0), so a re-run never stacks edits.
`python3 docs/redesign/sprites/make_sprites.py --blind <dir>` also writes unlabeled renders for a blind read.

## Calls made (and why)

- **Ring shape**: the 4x4 ring with cut corners and a 2x2 hole from the earlier design page (`bmh-item-redesign.html`,
  its bundle sprites), the smallest ring that still shows a hole at 2x. A 3x3 ring read as a gold blob at 2x (tried).
- **Ring gold**: the toolkit's gold ramp (`f7d95c`, `dba52a`, `a2691a`, `5c3808`). A softer gold closer to Louis's tan
  cord (tried) read as part of the cord, not as "one gold accent".
- **Ring place**: same columns (x 8-11) on both textures, so the ring does not jump when the model switches between
  empty and filled. Empty: right under the knot (9,11); the cord's two loose ends now meet the ring's shoulders (the
  ring covers one tail pixel, (8,13)). Filled: under the last pixel of the cord hanging from the neck (9,9).
- **Dormant totem, eyes closed rather than only dimmed**: "asleep" reads at 2x as two lash lines; dimmed open eyes
  (tried, `D1`) read as "same totem in shadow". Each eye's top row takes the face grey `4d4143` (lid), the bottom row a
  near-black lash line that keeps a trace of the eye's colour (`1f3a28` green, `1c3836` cyan). Gem: `5e5268`, `4f4560`,
  `4f4560`, `3a3148` (dull violet-grey, no white spark).
- **Echo Heart**: crying-obsidian violets sampled from vanilla `crying_obsidian.png` (`2b0178`, `6006ab`, `8308e4`
  over near-black) and the toolkit's echo cyans. 14x12 heart, lit from the top left: one bright crown facet and a
  white spark on the left lobe, a dark pavilion with bevel rims (added so the point does not vanish on dark grounds,
  checked on `1d1d1d`), and a 7x7 four-point glint (diamond core, tapering arms) instead of a plus sign, which read as a
  medical cross (tried). It is the one vivid item of the family on purpose: precious, like vanilla's gems.
- **No vanilla textures committed**: the comparison against vanilla's echo shard, nether star, amethyst shard and heart
  of the sea was rendered in the scratchpad only (Mojang's assets stay out of the repo).

## Steps

1. Tree guard: `git rev-parse --show-toplevel` prints
   `/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/f-sprites-82f4dd`. Branch renamed
   `claude/f-sprites-82f4dd` -> `redesign/f-sprites`. Read the brief and `.claude/CLAUDE.md`. Shared notes folder empty.
   Originals: `eternal_bundle.png` and `eternal_bundle_filled.png` (16x16), `totem_of_eternity.png` (16x128, 8 frames,
   mcmeta); same blobs on `origin/redesign/integration` and `origin/redesign/integration-1.21`.
2. Drafted candidates in the scratchpad (ring placements x gold tones, four dormant variants, heart v1 to v7).
3. Wrote `make_sprites.py`, generated the four textures and the three sheets. Verified with a pixel diff against
   `5062a5e`: only the 8 ring pixels per bundle and the 12 gem and eye pixels of the totem differ.

## Resume here

Step 3 done. Next: blind read by a fresh sub-agent, then commit, port the four PNGs to the 1.21 worktree, changelog
line, checklist, merge integration, push both branches.
