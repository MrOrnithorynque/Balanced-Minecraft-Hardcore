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
  near-black) and the toolkit's echo cyans. A 14x12 heart jewel lit from the top left. A dark diamond socket (`3c1470`,
  every pixel within 3 steps of (7,8)) holds a four-point glint: white core, cyan plus, teal ring, periwinkle tips
  (`5979cd`, the teal faded 55% into violet) reaching out like rays. Four flat facets around it: upper left
  `9056e4` with a lit corner on the lobe's edge (`f6eeff`, `c6a2ff`), upper right `6a2cc4` with a lit top edge,
  lower left `6a2cc4`, lower right `4d1890` with a rim of bounced light (`6a2cc4`) inside the outline. Outline graded
  like Louis's (`2c1250` on the lit top and left, `120818` elsewhere).
  How it got there, one blind read per version (answers in the scratchpad, `blind<N>/answers.md`):
  - draft: a plus-shaped glint read as a medical cross (my own review);
  - v1 (smooth facets, glint at (7,7)), read 1: a glossy dome, a dark tip under the cleft read as a crack, the
    light-top/dark-bottom split along the glint's arm as a belt;
  - v2 (six diagonal facets, no top tip), read 2: a two-tone cabochon, a lopsided "Latin cross" glint, dirty
    dark-teal tips, the spark smeared into the glint;
  - v3 (two-pixel bevel round the edge, glint at (7,8)), read 3: "a cut gem with about four broad faces", but a thin
    dark table band on row 6 read as a seam or hairline crack, the glint melted into the lavender on its upper left,
    and a round highlight competed with it;
  - v4 (dark diamond socket, flat facets, small angular highlight), read 4: no crack, no belt, no competing highlight,
    "a window cut into the heart, with the light inside it", distinct from vanilla at 1x and 2x; still "polished
    enamel or a glossy cabochon" (no facet edge, no second glint), dull tips, a dark lower-right wedge;
  - v5 (shipped): v4 plus lighter tips, the right lobe's lit edge and the lower-right bounced-light rim. Read 5 (full
    set, shipped files): "clearly on", a heart at 1x, 2x and 4x, "no crack, band or belt", distinct from all four
    vanilla items at 1x and 2x; still "smooth and glossy... It does not look cut", a soft dark diagonal on the lower
    right ("a cast shadow rather than a crack"), the bottom tip "a little like a stray blue pixel at 4x".
  **Call: ship v5.** Every reader, at every version, read a precious heart jewel with a light inside it, never an
  echo shard or a nether star, which is the brief's test. "Facets" is the one word of the spec only half met: readers
  see a few flat planes (read 3: "a cut gem with about four broad faces"), not a many-faceted cut. The recipe readers
  name for it, light seam lines between planes, was tried (v13 variants b to d in the scratchpad) and crowded the
  glint; at 16x16 the glint's clarity won. The glint sits half a pixel left of the heart's axis: an even-width heart
  with a one-pixel white core cannot centre it; readers call it invisible at 1x and 2x.
  It is the one vivid item next to the dark eternal items on purpose: it is the precious ingredient, and the readers
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

9. Heart v3 to v5 (see Calls), with blind reads 3 and 4 between them; second pass committed as `9fbb2f0` (main) and
   `0bbb67a` (1.21), pushed. Final heart (v5) written into `make_sprites.py`, which reproduces it pixel for pixel;
   committed `f2baaaa` (main) and `664ca7e` (1.21), pushed. The four PNGs have the same blob hash on both branches.
10. Final builds: main `f2baaaa` `BUILD SUCCESSFUL in 1m 11s`, 1.21.1 `664ca7e` `BUILD SUCCESSFUL in 40s`; all six
    jars (forge, fabric, neoforge on each side) contain the four textures, and their `echo_heart.png` hashes match the
    source file. `origin/redesign/integration` (`432d0a2`) and `origin/redesign/integration-1.21` (`a886d42`) had not
    moved, so there was nothing to merge. `git merge-tree` against every pushed sibling branch on its side: only
    `CHANGELOG.md` (every part adds under `## [Unreleased]`, not avoidable from here) and, with part B,
    `dormant_totem.png` and `echo_heart.png` (B's placeholders; the brief says F's file wins). Clean against `g`.
11. Blind read 5 on the shipped files (see Calls); no change made after it.

## Resume here

Done. Part F is complete on both branches; nothing left but G's merge (take F's `dormant_totem.png` and
`echo_heart.png` over B's placeholders, fold F's one `### Changed` changelog line into B's `### Changed` list).
