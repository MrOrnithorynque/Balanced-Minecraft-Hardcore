# Totem of Eternity: three states and the moon ritual

Implementation plan for an agent. Owner: Louis (MrOrnithorynque). Written 2026-11-06.

## 0. Before you start

- Work in the worktree `/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/totem-main`, branch `feat/totem-states` (from `main`, 1.20.4). First command of every shell: `git rev-parse --show-toplevel` must print that path; abort if it prints anything else. Use absolute paths.
- Read `/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/CLAUDE.md` first: branches, build, the use/useOn rule, client/server split, writing style (ASD-STE100 for comments and changelog), no em-dash or en-dash anywhere.
- Keep a running log in `docs/totem-states/LOG.md` from your first step, one numbered entry per step (no clock times you did not read from `date`), detailed enough that a fresh agent can resume from it alone.
- The branches `main` and `mc-*` are checked out in other worktrees and other sessions use them. Never check them out, reset or move them yourself. Work on `feat/totem-states` and on one `feat/totem-states-<version>` branch per version, each in its own new worktree under `.claude/worktrees/`. Ask Louis before you update the `mc-*` branches or push anything.
- A Gradle or Minecraft run shares state across checkouts (the Gradle cache, the Minecraft log). Redirect long outputs to files in your worktree and give every run a timeout.

## 1. The design (agreed with Louis)

The Totem of Eternity gets three states.

| State | Id | Display name (en_us) | Eternal (kept on death) | How you get it |
|---|---|---|---|---|
| 1. Hollow Totem | `bmh:dormant_totem` (old id kept) | Hollow Totem of Eternal Life | no | Crafting table, recipe unchanged |
| 2. Totem of Eternity | `bmh:totem_of_eternity` | The Totem of Eternal Life (unchanged) | yes | State 1 + Echo Heart at the anvil (unchanged, 39 levels), or state 3 + the moon ritual |
| 3. Dormant Totem | `bmh:sleeping_totem` (new) | Dormant Totem of Eternal Life | yes | State 2 after it saves you from death |

- **State 1 keeps the id `dormant_totem`.** Worlds from 1.8.0 and earlier hold that id; a new id would delete those items. Old dormant totems simply become Hollow Totems, with the same use.
- **State 3 gets the id `sleeping_totem`, not `dormant_totem_of_eternity`.** The smoke bot and other checks test inventories with `includes('bmh:dormant_totem')` and `includes('bmh:totem_of_eternity')`: an id that contains either string as a substring makes those checks lie.
- **State 3 is an Eternal Item** (`IEternalItem`): kept on death, counts in the respawn distance, one per player under the carry rule, unlimited lifetime on the ground, cleared from the anvil's left slot on take (already true for every `IEternalItem`). It has no durability, so the 7% death wear does nothing. It cannot save anyone and is not a smithing template.
- **The Echo Heart only wakes state 1.** State 3 is woken only by the moon ritual. The Echo Heart recipe keeps the nether star, and the eternal tool recipes do not change: every totem starts from one Echo Heart, so every eternal tool still costs one nether star.
- **No recipe file changes.** `RecipeSetTest` keeps its list.

### The moon ritual (state 3 to state 2)

The player holds use with the Dormant Totem while looking at the moon at night. A bar fills, the levels drain one by one, and at the end the totem wakes with an animation.

Rules:
1. Conditions to start and to keep going, checked on the server every tick: the Overworld, night, no rain, open sky above the eyes, and the look direction within `AIM_DEGREES` of the moon. `AmuletReading.hiddenReason(ServerPlayer)` already tests the first four for the amulet: reuse its logic, but with moon messages (new lang keys), not the star ones.
2. The ritual lasts `RITUAL_TICKS` and costs `RITUAL_LEVELS` levels, drained evenly: one level every `RITUAL_TICKS / RITUAL_LEVELS` ticks. The drain happens on the server; vanilla syncs the levels, so the XP bar drains on screen too.
3. It must be done in one go. Releasing use, looking away from the moon, a broken condition, taking damage (`hurtTime > 0` in the use tick, no mixin needed) or running out of levels all end it as a failure.
4. **A failure keeps the levels already drained.** The totem stays dormant, a puff of smoke and a fizzle sound play ("pouf"), and the action bar says why. Louis wants this on purpose: if you start with too few levels, every level you have goes and the totem does not wake.
5. A success replaces the stack in the hand with a Totem of Eternity and plays the totem animation and sound.

Defaults (constants in one class, so Louis can tune them in one place):

| Constant | Default | Note |
|---|---|---|
| `RITUAL_LEVELS` | 30 | Louis hesitates between 30 and 39. 30 makes the recharge cheaper than the first awakening (39). |
| `RITUAL_TICKS` | 300 (15 s) | One level every 10 ticks at 30 levels. |
| `AIM_DEGREES` | 10 | The cone around the moon's direction. |
| Full moon required | no | The moon phase is not checked. Requiring it adds up to 8 game days of waiting. |
| Refuse to start below the cost | no | Louis's rule 4 above. Claude argued for refusing at the start (keep the risk in the night fight, not in not knowing your level count); Louis decides. If he switches, it is one check in `use()`. |

## 2. Starting point

`feat/totem-states` has one WIP commit (`cc60821`) on top of `main`: the item and anvil part only. Keep what fits and change the rest:
- `ItemInit`: `DORMANT_TOTEM` renamed to `HOLLOW_TOTEM` (id `dormant_totem` kept) and a new `DORMANT_TOTEM_OF_ETERNITY`. **Change its id from `dormant_totem_of_eternity` to `sleeping_totem`** (section 1) and fix the comment.
- `TotemAwakeningHandler.awaken`: it accepts the dormant totem too. **Revert that: only `HOLLOW_TOTEM`** wakes with an Echo Heart. Fix the comments in `TotemAwakeningHandler` and `B_AnvilMenuMixin` to match.
- `TotemOfEternityHandler.trySave` turns the totem into `DORMANT_TOTEM_OF_ETERNITY`: keep.
- `items/DormantTotemOfEternityItem.java`: keep and extend with the ritual (section 3).
- `CreativeTabInit`, `HintItem` and `I_ClientPacketListenerTotemMixin` comments: keep.

## 3. Implementation on `main` (1.20.4)

Paths under `common/src/main/java/github/mrornithorynque/bmh/` unless stated.

### 3.1 The ritual in `items/DormantTotemOfEternityItem.java`

Put the constants and the server logic in a new `handlers/MoonRitualHandler.java`; the item calls it. Pattern of a hold-use item (bow, spyglass):
- `use()`: check the conditions; if they fail, send the reason to the action bar on the server and return `fail`/`pass` on both sides with the same test. If they pass, `player.startUsingItem(hand)` and return `consume(stack)` on both sides (read the use/useOn paragraph of CLAUDE.md: never `pass` on the client while the server consumes).
- `getUseDuration` returns `RITUAL_TICKS`. `finishUsingItem` is the success. `releaseUsing` is the early release, a failure.
- `getUseAnimation`: `UseAnim.SPEAR` (the arm raises the totem above the head, toward the sky). Check the pose in a screenshot.
- `onUseTick`: on the server, drain a level on schedule, and stop the use (`entity.stopUsingItem()` plus the failure effects) when a rule of section 1 breaks. Out of levels at a drain step is a failure too. On the client, spawn the particles (3.3); both sides tick the use, so no packet is needed.
- The moon's direction: derive it from how `LevelRenderer` rotates the sky (`getTimeOfDay(partialTick)` on 1.20.4, the celestial angle; the moon sits opposite the sun). Do not trust a sign you did not check: prove it in `runClient` with a screenshot where the crosshair is on the moon and the log says "aimed". `AmuletReading.azimuth` shows the mod's own angle conventions.
- Tooltip: "Eternity", a line that says how to wake it (night, look at the moon, hold use, the cost in levels from the constant), then `EternalTooltips.addHeavy`. No foil glint (it sleeps).

### 3.2 The totem animation on success

The server replaces the stack and then sends entity event 35. The inventory sync goes after the event, so the client still holds the Dormant Totem when the animation starts, and `I_ClientPacketListenerTotemMixin.findTotem` finds no `TotemOfEternityItem`: vanilla then shows a Totem of Undying. Extend the mixin: when no Totem of Undying is in a hand and no Totem of Eternity is in the inventory but a hand holds the Dormant Totem, return a new Totem of Eternity stack. Prove it in a `runClient` screenshot.

### 3.3 The animation during the ritual

All client side, no network channel (BMH has none):
- **The bar**: a thin bar above the hotbar that fills with `1 - remainingUseTicks / RITUAL_TICKS`, drawn by a new client mixin on `Gui` (add it to the `client` list; use the `add-mixin` skill). `K_GuiSpyglassMixin` shows how BMH draws on `Gui` on this version. The vanilla XP bar drains at the same time.
- **Particles**: a few soft particles each tick that drift from the moon's direction toward the totem in the hand (for example `END_ROD` or `ENCHANT`; pick one that reads at night and say why in the log).
- **The end**: success is the totem animation (3.2) plus a chime; failure is a smoke puff (`LARGE_SMOKE` or `POOF`) and `FIRE_EXTINGUISH`, played on the server so nearby players see and hear it.
- A moon beam in the sky (through `client/sky/`) is a later step, not this one.
- Louis judges the look and feel. Give him two or three screenshots (mid-ritual with the bar, success, failure).

### 3.4 Lang

`common/src/main/resources/assets/bmh/lang/`. Edit with Python, not `sed` or `perl` (non-ASCII). Only `en_us.json` has tooltip and message keys; the other nine files fall back to English for those.

Item names, all ten files: `item.bmh.dormant_totem` becomes the Hollow Totem, and the old text moves to the new key `item.bmh.sleeping_totem`.

| File | `item.bmh.dormant_totem` (new text) | `item.bmh.sleeping_totem` (old text of `dormant_totem`) |
|---|---|---|
| en_us | Hollow Totem of Eternal Life | Dormant Totem of Eternal Life |
| fr_fr | Totem creux de la Vie Éternelle | Totem endormi de la Vie Éternelle |
| de_de | Hohles Totem des Ewigen Lebens | Schlafendes Totem des Ewigen Lebens |
| es_es | Tótem hueco de la Vida Eterna | Tótem dormido de la Vida Eterna |
| it_it | Totem cavo dell'Eterna Vita | Totem dormiente dell'Eterna Vita |
| nl_nl | Holle Totem van Eeuwig Leven | Slapende Totem van Eeuwig Leven |
| pt_br | Totem Oco da Vida Eterna | Totem Adormecido da Vida Eterna |
| ru_ru | Полый тотем вечной жизни | Спящий тотем вечной жизни |
| ja_jp | 空ろな永遠の命のトーテム | 眠れる永遠の命のトーテム |
| ko_kr | 속이 빈 영원한 생명의 토템 | 잠든 영원한 생명의 토템 |

en_us only:
- `tooltip.bmh.dormant_totem`: "Awaken it at an anvil: Hollow Totem left, Echo Heart right".
- `tooltip.bmh.echo_heart`: "Awakens a Hollow Totem at an anvil: totem left, heart right".
- `tooltip.bmh.sleeping_totem`: "Wake it at night: look at the moon and hold use. It costs %s levels, all at once." (the number from the constant).
- `tooltip.bmh.totem_of_eternity_save`: keep ("...then falls dormant").
- New messages for the ritual, in the style of the amulet's `bmh.amulet.hidden.*` keys: the moon is hidden (wrong dimension, rain, day, roof), you looked away, you were hurt, you ran out of levels, the totem wakes.

### 3.5 Assets

- `assets/bmh/models/item/sleeping_totem.json`: a copy of `dormant_totem.json` that points at `bmh:item/sleeping_totem`.
- **Textures, recommendation to show Louis**: the current `dormant_totem.png` (frame 0 of the Totem of Eternity with the eyes closed and the forehead gem dulled, see `docs/redesign/log-f-sprites.md`) is exactly a totem asleep, so it becomes `sleeping_totem.png`. The Hollow Totem gets a new placeholder `dormant_totem.png`: the same sprite with the forehead gem socket and both eyes empty (near-black holes, no lash colour). Make it with a function in `docs/redesign/sprites/make_sprites.py` next to `dormant_totem()`, so it can be redone. Put both sprites side by side, enlarged, in front of Louis: the art is his call.
- Regenerate `resources/dormant_totem_crafting_grid.png` (it shows the Hollow Totem's sprite) with `.claude/skills/release/make_grids.py`; the Echo Heart grid does not change.

### 3.6 Tests

- `.claude/skills/verify-loaders/scripts/smoke_bot.js`, section 9: the save now gives `bmh:sleeping_totem`. Update the three checks around lines 301 to 329 (`turned into a Dormant Totem` must look for `bmh:sleeping_totem`) and the guard `ITEMS.includes('dormant_totem')`. Add: after the save, the bot dies with only the Dormant Totem in its inventory and respawns with it. Add the ritual: set the time to 18000, clear the weather, give 40 levels, `bot.look` at the moon, hold use for `RITUAL_TICKS` plus a margin, then check `bmh:totem_of_eternity` in the inventory and 30 levels gone. And a failure: release after a third of the time, check the totem still dormant and the drained levels gone.
- The anvil check (line 200) still holds for `bmh:dormant_totem` (now the Hollow Totem). Add one: `bmh:sleeping_totem` plus an Echo Heart gives no result.
- `scripts/hook/SmokeClient.java`: the log line `after the totem: ... slot5=` must now say `bmh:sleeping_totem`; update the `verify-loaders` SKILL.md line that quotes it. If you can, add a ritual pass to the hook (time 18000, aim at the moon, hold use) with a screenshot mid-ritual and one at the end.
- Self tests (`scripts/hook/selftest/`): add the Dormant Totem to `DeathKeepTest` (kept on death) and `OneEternalPerTypeTest` (one per player).
- `jarcheck.py`: confirm it finds the new model and texture in every jar.

### 3.7 Docs

- `CHANGELOG.md`, under `## [Unreleased]`, for players: the Dormant Totem is now the Hollow Totem; the Totem of Eternity falls asleep after a save and stays with you through death; wake it with the moon ritual and its cost; the Echo Heart only wakes a Hollow Totem. Plus 1 to 3 in-game pictures of the ritual for the release (memory note: every release section gets pictures).
- `README.md` and `docs/curseforge-description.md`: the totem section (names, the ritual).
- `.claude/CLAUDE.md`, code map: the Totem of Eternity paragraphs (three states, `MoonRitualHandler`, the new client mixin, the `sleeping_totem` id and why it is not `dormant_totem_of_eternity`).

### 3.8 Check on `main`

`JAVA_HOME=/opt/homebrew/opt/openjdk@17 sh ./gradlew build`, `bash .claude/skills/verify-loaders/scripts/compile_check.sh`, a dedicated server boot per loader, then the `verify-loaders` skill on the branch. The ritual and the animation need a real `runClient` run with screenshots. Commit (message ends with the Co-Authored-By line Louis's setup asks for).

## 4. The other branches

- **1.20.x** (`mc-1.20.3`, `mc-1.20.2`, `mc-1.20.1`, `mc-1.20`): cherry-pick the commit(s) onto a `feat/totem-states-<version>` branch made from each `mc-*`, in its own worktree. Keep each branch's own lines in `forge/build.gradle`, `gradle.properties`, `README.md`, `mods.toml`. Then the empty-diff check of CLAUDE.md against your `feat/totem-states` must be empty. Build each one (`openjdk@17`).
- **`mc-1.21.1`**: port by hand (`openjdk@21`). What differs: `appendHoverText(ItemStack, Item.TooltipContext, List, TooltipFlag)`, `getUseDuration(ItemStack, LivingEntity)`, the item data is in components (the ritual stores nothing on the stack, so this should not bite), Forge runs on the official names. Boot a server on each loader.
- **`mc-26.1`**: port by hand from your 1.21.1 port (`openjdk@25`). `Identifier` for `ResourceLocation`, `InteractionResult` in place of `InteractionResultHolder`, `ItemUseAnimation` in place of `UseAnim`, the tooltip and `Gui` APIs changed (screens draw through `GuiGraphicsExtractor`, see the 26.1 section of CLAUDE.md), item models are `assets/bmh/items/*.json` definitions: add `items/sleeping_totem.json`. Check how 26.1 exposes the time of day and the moon before you reuse the 1.20 math. `compile_check.sh 26.1.2`. Read `docs/port-26/LOG.md` first.
- Run the `verify-loaders` skill on every branch. On 26.x the Fabric bot cells read 0/1 (Fabric API refuses vanilla mineflayer clients): that is known, not your bug.

## 5. Done means

- On all seven versions: the three states work, the ritual works and fails as in section 1, the save gives the Dormant Totem and it survives death, the Echo Heart wakes only the Hollow Totem, every jar passes `jarcheck.py`, `verify-loaders` is green apart from known cells.
- Screenshots of the ritual (mid, success, failure) and the two sprites side by side, in front of Louis.
- Nothing pushed and no `mc-*` branch moved before Louis says so. No release (that is the `release` skill, later).
- The tuning constants of section 1 listed in your final report, with what you saw in game.
