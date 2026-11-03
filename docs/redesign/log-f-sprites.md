# Log: part F, sprites

Session F (`f-sprites`) of the BMH redesign run. Spec: `docs/redesign/BRIEF.md`, part `F · Sprites`.
Branches: `redesign/f-sprites` (main side, this worktree), `redesign/f-sprites-1.21` (1.21.1 side, worktree
`/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/f-sprites-82f4dd-121`).
Steps are numbered, not timed.

## What ships

Four 16x16 RGBA textures in `common/src/main/resources/assets/bmh/textures/item/`, the same bytes on both sides,
all static (no `.mcmeta`), opaque or fully transparent pixels only, no PNG metadata:

| File | What | Pixels changed |
|---|---|---|
| `eternal_bundle.png` | Louis's original + a 4x4 gold ring hanging under the drawstring's knot (x 8-11, y 12-15) | 8 |
| `eternal_bundle_filled.png` | Louis's original + the same ring under the end of the neck cord (x 8-11, y 10-13) | 8 |
| `dormant_totem.png` | frame 0 of `totem_of_eternity.png`, forehead gem dulled, both eyes closed | 12 |
| `echo_heart.png` | new: faceted crying-obsidian violet heart, four-point echo-cyan glint at its centre | all |

`docs/redesign/sprites/make_sprites.py` rebuilds all four and the sheets (`review-sheet.png`, `before-after.png`,
`family.png`) from Louis's originals read from git at `5062a5e` (release 1.4.0), so a re-run never stacks edits.
`python3 docs/redesign/sprites/make_sprites.py --blind <dir>` also writes unlabeled renders for a blind read.
The 1.21.1 side carries only the four PNGs (byte-identical) and the changelog line; the sheets, script, log and
checklist live on the main side.

## Calls made (and why)

- **Ring shape**: the 4x4 ring with cut corners and a 2x2 hole from the earlier design page (`bmh-item-redesign.html`,
  its bundle sprites), the smallest ring that still shows a hole at 2x. A 3x3 ring read as a gold blob (tried).
- **Ring gold**: the toolkit's gold (`f7d95c` on top and the upper left, `dba52a` elsewhere), bright all round. A softer
  gold closer to Louis's tan cord read as part of the cord (tried). A darker lower right (`a2691a`, `5c3808`, first
  version) made the hole read as "an amber stone in a gold setting" in blind read 1, so the hoop has no dark pixel.
- **Ring place**: same columns (x 8-11) on both textures, so the ring does not jump when the model switches between
  empty and filled. Empty: right under the knot (9,11), hanging; it covers one loose-end pixel (8,13) and touches the
  bag's bottom row (blind read 1 found it a little cramped there; moving it up onto the knot would replace Louis's knot
  and make it threaded rather than hanging, so it stays). Filled: under the last pixel of the neck cord (9,9).
- **Dormant totem, eyes closed rather than only dimmed**: dimmed open eyes (tried) read as "the same totem in shadow";
  closed eyes read as asleep. Each eye: top row a lid `5f5759` (a shade lighter than his face grey `4d4143`, which left
  the eyes invisible on their own in blind read 1), bottom row a near-black lash line with a trace of the eye's colour
  (`101c14` green, `0f1c1b` cyan). Gem `776a88`, `5b4e72`, `5b4e72`, `3a2e4c`: dull, no white spark, visible.
- **Echo Heart**: crying-obsidian violets (vanilla `crying_obsidian.png` runs `2b0178`, `6006ab`, `8308e4` over
  near-black) and the toolkit's echo cyans. A 14x12 heart jewel lit from the top left: a two-pixel crown bevel round
  the edge, its facets toned by the way they face (lavender and a white spark on the left lobe, mid violet on the
  right lobe, a lighter rim on the lower right so the point holds on dark grounds), around a darker table holding a
  four-point glint (diamond core, four equal arms, centre (7,8)). Outline graded like Louis's (`2c1250` on the lit top
  and left, `120818` elsewhere). How it got there: a plus-shaped glint read as a medical cross (draft); version 1
  (smooth facets, glint at (7,7)) read in blind read 1 as a glossy dome with a crack under the cleft (dark tip) and a
  belt (light top over dark bottom split along the glint's arm); version 2 (six diagonal facets, no top tip) read in
  blind read 2 as a two-tone cabochon with a lopsided "Latin cross" glint, dirty dark-teal tips and the spark smeared
  into the glint; version 3 (bevel, equal arms, tips faded 55% into the violet under them: `5979cd`, `34579e`,
  `2b548d`, mid violet between spark and glint) went to blind read 3.
  It is the one vivid item next to the dark eternal items on purpose: it is the precious ingredient, and both readers
  matched its colours to the lit totem's violet gem and cyan eye.
- **No vanilla textures committed**: the comparison with vanilla's echo shard, nether star, amethyst shard and heart of
  the sea was rendered in the scratchpad only (Mojang's assets stay out of the repo). Blind read 1: distinct from all
  four at 1x and 2x; closest by colour the amethyst shard, by silhouette the heart of the sea.

## Steps

1. Tree guard: `git rev-parse --show-toplevel` prints
   `/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/f-sprites-82f4dd`. Branch renamed
   `claude/f-sprites-82f4dd` -> `redesign/f-sprites`. Read the brief and `.claude/CLAUDE.md`. Shared notes folder empty.
   Originals: `eternal_bundle.png` and `eternal_bundle_filled.png` (16x16), `totem_of_eternity.png` (16x128, 8 frames,
   mcmeta); same blobs on `origin/redesign/integration` and `origin/redesign/integration-1.21`.
2. Drafted candidates in the scratchpad (ring placements x gold tones, four dormant variants, heart v1 to v7).
3. Wrote `make_sprites.py`, generated the four textures and the three sheets. Pixel diff against `5062a5e`: only the
   8 ring pixels per bundle and the 12 gem and eye pixels of the totem differ. Committed `4de56a8`, pushed.
4. Changelog line (`### Changed`, the bundle's ring; the two new items' lines are part B's) and
   `docs/redesign/checklist-f-sprites.md`, committed `8255b8a`. 1.21.1 side: worktree from
   `origin/redesign/integration-1.21`, the four PNGs copied byte for byte and the same changelog line
   (`CHANGELOG.md` was identical on both integration branches), committed `4d6bf53`. Both pushed. The bundle models
   on both sides already point at `bmh:item/eternal_bundle` and `bmh:item/eternal_bundle_filled`.
5. Builds: main side `BUILD SUCCESSFUL in 3m 11s` (JDK 17, `--max-workers=2`, Gradle slot 1), 1.21.1 side
   `BUILD SUCCESSFUL in 5m` (JDK 21, slot 2); the forge, fabric and neoforge jars of the main side each contain the
   four textures (`unzip -l`). Logs in each worktree's `build/f-build-*.log`. No server boots: part F ships no code.
6. Blind read 1 (fresh agent, unlabeled renders; answers in the scratchpad, `blind/answers.md`). Its pixel citations
   checked against the files. Findings acted on: ring read as a set amber stone, heart read as a glossy dome with a
   crack and a belt, dormant face invisible on its own. Second pass (ring A, lit lids and black lashes, diagonal
   facets) regenerated by the script; pixel diff unchanged (8, 8, 12).
7. Blind read 2 (another fresh agent, `blind2/answers.md`): the ring now reads "clearly a gold ring: a 4x4 loop with a
   2x2 hole showing the bag through it" at 4x (a ring or a coin at 2x); the dormant face reads "closed eyes, asleep
   more than dead". Remaining critiques of the bags and the totem body are Louis's own pixels (the open bag's
   lopsided cord, the totem's belt row and flat shading, both dark on black), left alone as the brief asks. The
   filled bag's ring sits 1.5 px right of the knot; kept at x 8-11 so it matches the empty bag's ring. The heart got
   a third version (see Calls), heart-only blind read 3 launched.
8. Sibling check (`git merge-tree`): only the expected conflicts, CHANGELOG.md with every part (each adds under
   `## [Unreleased]`) and the two placeholder PNGs with part B (brief: F's file wins). B's models for the two new
   items point at `bmh:item/dormant_totem` and `bmh:item/echo_heart` on both sides.

## Resume here

Step 8 done, blind read 3 pending. Next: act on it, commit the final pass on both sides, merge
`origin/redesign/integration` and `origin/redesign/integration-1.21`, rebuild both, merge-tree, push, report.
