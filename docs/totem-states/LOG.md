# Totem states: implementation log

Running log of the implementation of `docs/totem-states/PLAN.md`. One numbered entry per step. Clock times only from `date`.

Worktree of `main` (1.20.4): `/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/totem-main`, branch `feat/totem-states`.

## 1. Start (date: Fri Nov 6 19:55 CET 2026)

- Read PLAN.md and `.claude/CLAUDE.md`. Starting point: `cc60821` (WIP item and anvil part) and `ae17634` (the plan).
- Decompiled the 1.20.4 classes I need with Vineflower into the scratchpad (`LevelRenderer`, `LivingEntity`, `LocalPlayer`, `Gui`, `EndRodParticle`, `DimensionType`, `ClientPacketListener`). Facts:
  - Sky: `renderSky` does `mulPose(YP(-90))` then `mulPose(XP(getTimeOfDay(pt) * 360))`, sun quad at y = +100, moon quad at y = -100. So with t = timeOfDay * 2 pi, the sun is at (-sin t, cos t, 0) and the moon at (sin t, -cos t, 0). Noon (dayTime 6000) gives timeOfDay 0 (sun up); midnight (18000) gives 0.5 (moon straight up). Sunrise (dayTime 0) puts the sun at +X (east). To prove in runClient.
  - `LivingEntity.updateUsingItem`: `onUseTick(level, this, remaining)` then `--useItemRemaining == 0` completes on the server only. First call has remaining = duration, last call remaining = 1.
  - `ServerPlayer.completeUsingItem` sends entity event 9 first, then `finishUsingItem` runs (so an event 35 sent from `finishUsingItem` arrives after 9). The client runs `finishUsingItem` on event 9 too.
  - `LocalPlayer.onSyncedDataUpdated` stops the client's use when the server clears the "using" flag: a server-side `stopUsingItem()` reaches the client, no packet of ours needed.
  - `Gui.renderHotbar(float, GuiGraphics)` is private in vanilla, and Forge 49.0.31 (`VanillaGuiOverlay.HOTBAR`) and NeoForge 20.4.253 (`VanillaGuiOverlay`) both call it. So a `TAIL` injection there draws on all three loaders, in creative too.
  - `EndRodParticle`: gravity 0.0125, lifetime 60 to 71 ticks, friction 0.91 (SimpleAnimatedParticle), full bright.

## 2. Louis's instruction: one version first

Louis (chat): implement on one Minecraft version, he validates, then the change goes to the others. So this run does `main` (1.20.4) only and stops before any port.

## 3. Code on `feat/totem-states` (1.20.4)

- `ItemInit`: the Dormant Totem's id is now `sleeping_totem` (field name `DORMANT_TOTEM_OF_ETERNITY` kept), comment says why.
- `TotemAwakeningHandler.awaken`: only `HOLLOW_TOTEM` + Echo Heart (WIP had accepted the dormant one too). Comments fixed there and in `B_AnvilMenuMixin`.
- `AmuletReading.skyHiddenReason(Player, prefix)`: the old `hiddenReason(ServerPlayer)` logic, now for either side and any key prefix. The amulet calls it with `bmh.amulet.hidden`, the ritual with `bmh.ritual.hidden`.
- New `handlers/MoonRitualHandler.java`: the constants (`RITUAL_LEVELS` 30, `RITUAL_TICKS` 300, `AIM_DEGREES` 10, plus `FAIL_COOLDOWN_TICKS` 20), `moonDirection`, `blockedReason`, `tick`, `fail`, `succeed`, `release`.
- `DormantTotemOfEternityItem`: `use()` runs `blockedReason` on both sides and returns `consume` on both sides in every case (the server sends the reason). Deviation from the plan's "fail/pass on failure": CLAUDE.md says a pass or fail on the client also uses the off-hand item in the same click; consume avoids that and both sides still agree. `getUseDuration` 300, `UseAnim.SPEAR`, `onUseTick` (server: `MoonRitualHandler.tick`, `stopUsingItem` on failure; client: one END_ROD particle per tick), `finishUsingItem` (server: success; client: also returns the new totem, so the hand changes at event 9), `releaseUsing` (server: failure "released").
- Particles: END_ROD, because it is full bright and white like moonlight (reads at night), and its 0.91 friction slows it as it reaches the totem. Start 4 blocks toward the moon, speed (hand - start) / 11.
- Choices the plan left open: creative players are not drained (like the amulet's free readings in creative); a failure puts a 20 tick cooldown on the item, else a held use key restarts the ritual every 4 ticks. Known gap: a hotbar switch during the ritual stops the use silently (vanilla `stopUsingItem`, no `releaseUsing`): no smoke, no message, the drained levels are gone as with any failure.
- `I_ClientPacketListenerTotemMixin`: when no Totem of Undying is in a hand and no Totem of Eternity is in the inventory but a hand holds the Dormant Totem, the animation shows a new Totem of Eternity stack.
- New client mixin `O_GuiRitualBarMixin` (letter O: M and N are taken on `mc-26.1`): `Gui.renderHotbar` TAIL, a 182 x 3 bar at 62 px from the bottom (between the action bar text, which ends at 63, and the held item name at 59), pale moonlight fill on a dark back.
- Lang: the ten item names of the plan's table (`dormant_totem` = Hollow, new `sleeping_totem` = the old text), en_us tooltips and the `bmh.ritual.*` messages.
- Assets: `models/item/sleeping_totem.json`; `make_sprites.py`: `dormant_totem()` renamed `sleeping_totem()` (writes `sleeping_totem.png`, byte-identical to the old `dormant_totem.png`), new `hollow_totem()` writes `dormant_totem.png` (gem socket and eyes as near-black holes, top row a shade lighter). The other textures came out byte-identical. The review sheets in `docs/redesign/sprites/` now show both totems.
- `make_grids.py resources`: rewrote `dormant_totem_crafting_grid.png` and `totem_of_eternity_anvil.png` (both show the Hollow sprite). It also rewrote `eternal_sword_smithing.png`, unrelated to this change: reverted, left for another time.
- Tests: `smoke_bot.js` (anvil 6b: Dormant + Echo Heart gives nothing; section 9 checks `sleeping_totem`; kept on death; section 10: ritual success and early release), `SmokeClient.java` (ritual at dayTime 15000 with the moon low in the east, mid, back, front, woken and failed screenshots; own moon math so it still compiles on older tags), `DeathKeepTest` and `OneEternalPerTypeTest` (the Dormant Totem), `jarcheck.py` (`O_GuiRitualBarMixin`: `m_280518_`, `method_1759`), SKILL.md line.
- Docs: CHANGELOG (Unreleased), README and curseforge description (totem section), CLAUDE.md code map. Changelog pictures wait for Louis's validation of the screenshots.
- `sh ./gradlew build` (openjdk@17): BUILD SUCCESSFUL. `jarcheck.py` on the three jars: 287/287.

## 4. First verify run (`bda1618`, `branch.sh` with BMH_LOCAL=1, label totem-1204, outputs in the session scratchpad `verify/totem-1204`)

- build ok, jars 287/287. Fabric, Forge, NeoForge: amulet 40/40, smoke 53/53, 0 BMH errors on server and client, 9 screenshots each.
- Client log on each loader: `after the totem: slot5=bmh:sleeping_totem`, `after the ritual: slot5=bmh:totem_of_eternity levels=10`, failure keeps the totem dormant with 36 or 37 levels (3 or 4 drained, kept gone).
- Moon direction proven: at dayTime 15000 the hook aims at (0.80, 0.60, 0.00) (east, 37 degrees up), the server logs "aimed 0.6 degrees from the moon", and in `bmh-smoke-5-ritual.png` the moon is under the crosshair.
- Look problems seen in the screenshots: END_ROD particles ended 0.6 blocks from the eyes and covered the first person view with big white shapes; the failure POOF did the same; the SPEAR pose hid the totem in first person; the "front" shot was first person (a camera change and a queued screenshot in the same tick).

## 5. Louis's look feedback (chat, while run 1 went on)

- First person: the player puts the totem in front and the particles go gently into the totem. Third person: both arms raised (toward the moon), particles into the totem. Then: "not need from the moon if feel weird, but particle go in the totem yes". Then: "the totem is weirdly placed, i dont see it, and their is too much particle" (about run 1's look).
- Changes (`0f87741`): `UseAnim.BOW` (vanilla's two-handed pose: both arms along the look, so toward the moon); new client mixin `P_ItemInHandRitualMixin` (`ItemInHandRenderer.renderArmWithItem` HEAD, cancels for the Dormant Totem in use) and `client/RitualHandRenderer` (the totem raised over 10 ticks from the hand to the centre, 0.2 below the eye line, 0.75 ahead, FIXED display, scale 0.32, a slow hover); `client/RitualParticles` uses ENCHANT (flies to an exact point and goes out there), one per tick, starting 1.2 to 2.4 blocks around the totem on the far side from the camera; the failure puff is 8 POOF, 1.8 blocks in front of the eyes for the player and at the raised totem for others; hook camera switches get their own ticks. jarcheck: `P_ItemInHandRitualMixin` = `m_109371_`, `method_3228`.

## 6. Look iterations on Fabric (labels totem-1204-look2 to look4)

- look2 (`e3f67bb`): totem centred under the crosshair but too big (scale 0.26) and almost black (night light); runes read well; puff heavy. Louis (chat) asked for the totem almost right under the crosshair: done.
- Louis (chat): "il faudrait qu'il puisse glow" while it charges; he likes the enchanting runes. Added `client/RitualGlow` (the item's `isFoil` is true while a player uses that very stack, on the physical client only).
- look4 (`eb49fdd`): scale 0.16, 0.1 below the eye line at 0.75 ahead (its top just under the crosshair), full bright (0xF000F0), glint while charging (first person, raised hands, hotbar slot), puff 6 POOF at 2.5 blocks for the player. Fabric: jars 296/296, amulet 40/40, smoke 53/53, client 9 shots, 0 errors; ritual 40 to 10 levels; failure keeps 4 drained levels.
- Pictures for Louis in `docs/totem-states/pictures/` (1280x720 from look4, plus the sprite sheet), sent to him.

## 7. Louis validated 1.20.4 ("Validated, port it"); the ports

- The two tuning decisions (30 or 39 levels; refuse to start below the cost) are still open with Louis. The code keeps 30 and starts anyway.
- `feat/totem-states` rebased on `origin/main` (one release script commit). 1.20.x: `feat/totem-states-<v>` from `origin/mc-<v>`, the eight feature commits cherry-picked with no conflict, the CLAUDE.md empty-diff check against `feat/totem-states` is empty for all four. Worktrees `.claude/worktrees/totem-<v>`.
- 1.21.1 (`feat/totem-states-1.21.1`, `09e79df`): the feature diff applied with `git apply --3way --binary`, no conflict; hand fixes: `appendHoverText(ItemStack, Item.TooltipContext, ...)`, `getUseDuration(ItemStack, LivingEntity)`, the bar on `Gui.renderItemHotbar(GuiGraphics, DeltaTracker)` (vanilla's `renderHotbarAndDecorations`, Forge 52's patch and NeoForge 21.1's hotbar layer all call it; its intermediary is still `method_1759`). Sky rotation, `renderArmWithItem` and the rune particle (`FlyTowardsPositionParticle`) are unchanged.
- 26.1 (`feat/totem-states-26.1`, `2aff39e`): the 1.21.1 diff applied with 5 conflicts (26.1's `register(name, properties -> ...)`, `isDarkOutside`, the creative tab list, the death protection totem mixin, the mixin list), resolved by hand. API: the moon angle is the `MOON_ANGLE` environment attribute (`level.environmentAttributes().getDimensionValue`), the moon quad sits at +Y so the direction is (-sin a, cos a, 0); `use` returns `InteractionResult.CONSUME`; `releaseUsing` returns a boolean; `ItemUseAnimation.BOW`; the tooltip consumer; `sendOverlayMessage`; cooldowns by stack; `sendParticles` with two booleans; `Gui.extractItemHotbar(GuiGraphicsExtractor, DeltaTracker)` (NeoForge 26.1's hotbar layer calls it too); `ItemInHandRenderer.renderItem` without the left-hand flag and with a `SubmitNodeCollector`; `items/sleeping_totem.json`.
- Harness (`1aaf027` on 26.1, cherry-picked to all six other branches): the hook's moon direction comes from `ClientCompat.moonDirection` (26.1 has no `getTimeOfDay`).
- First verify round (three runs overlapping): 1.20.3 green (252/252, 40/40, 53/53 on Forge and Fabric, clients 9 shots). 1.20.2 green but the Fabric client hit the outage below. 1.20.1, 1.20 and all three 1.21.1 servers: ForgeGradle "Failed to validate certificate for host https://maven.minecraftforge.net/" (transient: curl gives 200 and a valid certificate a little later). 26.1: Forge and Fabric green (178/178, 40/40, 53/53, 9 shots; the Fabric bots got in this time); NeoForge smoke 49/53 (Echo Strike and the three ritual checks; no "Moon ritual" line in the server log) and its client killed at the 600 s timeout. Suspect: load from three overlapping runs. Reruns one at a time (scratchpad `rerun.sh`).

## 8. NeoForge 26.1 bug, batch reruns, the app quit

- 26.1 NeoForge failed again alone on a quiet machine (smoke 50/53, client stopped after 4 shots): a real bug. Client log: "IllegalStateException: Position must always be provided for positional attribute minecraft:visual/moon_angle". `EnvironmentAttributeSystem.getDimensionValue` throws for a positional attribute when `SharedConstants.IS_RUNNING_IN_IDE` (NeoForge's dev runs set it, Forge's and Fabric's do not). So the server's `use()` threw (no ritual) and the hook crashed the client. Fix `7b0113a` (26.1 only): `moonDirection(Player)` reads `getValue(MOON_ANGLE, eye position, null)`. Hook fix `6653376`, cherry-picked to all seven branches.
- Louis (chat) asked for batches instead of one run at a time: done. The shared server lock still serializes servers and clients; only builds overlap.
- Batch results before the app quit: 1.20.1 green (252/252; Forge and Fabric 40/40, 53/53, clients 9 shots). 1.21.1 green on the three servers (229/229, 40/40, 53/53) and on the Forge and Fabric clients; the NeoForge client gave 0 screenshots (cause not read before the logs were lost). 1.20 and the 1.20.2 Fabric client waited 37 minutes on a Gradle slot that the 1.21.1 run I killed never gave back (stale `/private/tmp/bmh-gradle.1`, owner pid gone). 26.1 with the fix: Forge server 40/40, 53/53, then the app quit.
- The restart wiped `/private/tmp` (the verify outputs). Next outputs go to `~/.cache/bmh-verify-totem`.

## 9. The 26.1 client race, server-only start, final batch

- After the app quit, batch (outputs `~/.cache/bmh-verify-totem`): 26.1 green on the three loaders' servers (40/40, 53/53, NeoForge included: the fix works), 1.20 green (Forge and Fabric), 1.21.1 NeoForge green (client 9 shots).
- But the 26.1 clients' first ritual never drained a level, on every loader. Diagnostic (hook logs the server's `blockedReason` by reflection, commit `bd0a5e2`): at tick 301 the server still had the old look (yaw -180, pitch -30, "bmh.ritual.aim"), at tick 320 the right one (0.55 degrees from the server's moon). The hook turned the camera and clicked in the same tick, and the use packet arrived before the rotation. Server and client agree on the moon. The cost for a player: the client had started the ritual on its own test, so a refused start (a quick flick to the moon, lag) showed a filling bar while nothing happened.
- Fix `166ac73` (1.20.4, cherry-picked to 1.20.x and 1.21.1; by hand `5d2aa7d` on 26.1): only the server starts the ritual; the client consumes the click and follows the server's using flag (`LocalPlayer.onSyncedDataUpdated`); a click during a ritual does not start it again.
- Louis (via the coordinator session): "Keep 30" for `RITUAL_LEVELS`. The other decision (refuse to start below the cost) stays open; the code keeps starting.
- Final batch on the new commits: 1.21.1 Fabric, 26.1 NeoForge and 1.20.2 Fabric green; each client's first ritual drains 40 to 10 and wakes the totem, the failure keeps 4 drained levels. 1.20.4's first try failed at configuration (ForgeGradle read the Mojang version manifest while a parallel build rewrote it: "MalformedJsonException ... versions[209].url"; the cached manifests are valid afterwards); rerun alone as `f2-1.20.4`.
- Another project's verify run (Frayed) shares `/private/tmp/bmh-runserver.lock`: a wait on that lock can be theirs.
- `f2-1.20.4` (`bd0a5e2`): jars 296/296; Forge, Fabric, NeoForge: amulet 40/40, smoke 53/53, clients 9 shots, 0 errors; ritual 40 to 10 and the failure keeps 4 levels on each.

## 10. State before the push (waiting for Louis's go)

All seven branches verified on the final code, every loader green (1.20.x and 1.21.1 cells from batches where the later commits changed only the ritual start, rechecked on 1.20.4, 1.20.2, 1.21.1 and 26.1). Branches, each a fast-forward of its `origin` branch: `feat/totem-states` (main), `feat/totem-states-1.20.3`, `-1.20.2`, `-1.20.1`, `-1.20`, `-1.21.1`, `-26.1`. Nothing pushed, no release.
