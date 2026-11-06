# Port to 26.1.2: client agent (sky, journey, amulet screens, config screen, totem animation)

Scope: `client/sky/*`, `client/journey/*`, `client/amulet/*`, `client/config/ConfigScreen.java`, `mixin/D_LevelRendererSkyMixin.java`, `mixin/K_GuiSpyglassMixin.java`, `mixin/F_GuiTitleMixin.java`, `mixin/F_LocalPlayerRespawnMixin.java`, `mixin/F_MinecraftOverlayMixin.java`, `mixin/F_MusicManagerAccessor.java`, `mixin/F_SoundInstanceVolumeAccessor.java`, `mixin/I_ClientPacketListenerTotemMixin.java` (paths under `common/src/main/java/github/mrornithorynque/bmh/`).

`SRC` = `$SCRATCH/src26` (decompiled 26.1.2, Mojang names). `B3D` = `$SCRATCH/agent-client/b3d` (com.mojang.blaze3d of the 26.1.2 client jar, decompiled by this agent with Vineflower 1.10.1: `src26` has only `net/minecraft`). Line numbers below are in those files.

Started 2026-11-06 03:36 CET (`date`). Last `date` read: 03:53 CET.

## Step 1: compile state at the start

`c26.sh` (TAG=client): my files had 30 (StarReadingRenderer) + 11 (JourneyOverlay) + 6 + 6 + 4 + 3 + 1 + 1 errors. Causes: `GuiGraphics` gone, `ShaderInstance`/`BufferUploader`/`GlStateManager` gone, `getTimeOfDay`/`getStarBrightness` gone, `ReceivingLevelScreen` gone, `ChunkPos.x` private, `sendUnsignedCommand` gone, `EditBox.setFilter` gone, `keyPressed(int,int,int)` gone, `updateSourceVolume` gone.

## Step 2: sky (StarField, StarChain, StarReadingRenderer, D_LevelRendererSkyMixin)

- `D_LevelRendererSkyMixin` now targets `SkyRenderer.renderStars(FLcom/mojang/blaze3d/vertex/PoseStack;)V` at TAIL (SRC `client/renderer/SkyRenderer.java` 399-422; descriptor checked with javap on the client jar; also present in Forge 26.1.2-64.1.6 patched sources, line 415). The class keeps its name (the mixin config lists it).
  - Audit: the pose at TAIL is the stars' pose. `renderSunMoonAndStars` (SkyRenderer 315-342) pushes, applies YP(-90) then XP(starAngle), calls `renderStars` only when `starBrightness > 0`, then pops. `renderStars` never changes `poseStack` and balances its model view push/pop (401/421). So at TAIL `poseStack.last().pose()` is the star pose and the model view stack is as vanilla left it. Holds.
- `StarReadingRenderer.render(PoseStack, float starBrightness)` replaces `render(ClientLevel, Matrix4f, float)` (only my mixin called it). The draw follows `SkyRenderer.renderStars`: `modelViewStack.mul(pose)`, `writeTransform(..., (1,1,1,1), ...)`, a `RenderPass` on the main target. Vertices: `BufferBuilder` on a `ByteBufferBuilder`, then `VertexFormat.uploadImmediateVertexBuffer` (as `WeatherEffectRenderer.render`, SRC 133-161), `renderPass.draw(0, vertexCount)` (non indexed, as `renderSkyDisc`).
- Own pipeline `bmh:pipeline/star_reading`: uniforms DynamicTransforms + Projection (the STARS snippet, RenderPipelines 24-27), shaders `core/position_color`, `BlendFunction.OVERLAY`, POSITION_COLOR TRIANGLES, no depth state, cull default (true).
  - Audit: `BlendFunction.OVERLAY` = (SRC_ALPHA, ONE, ONE, ZERO) (B3D `pipeline/BlendFunction.java` line 9) = the 1.21.1 `blendFuncSeparate`. STARS has no depth state and cull true (RenderPipelines 627-635, `RenderPipeline.Builder.build` uses `cull.orElse(true)`). `position_color.fsh` multiplies by ColorModulator and discards alpha 0, as 1.21.1. Pipelines compile lazily on `setPipeline` (`GlRenderPass` 65, `GlDevice.getOrCompilePipeline` 358), so an unregistered pipeline works.
- Star brightness: `skyAlpha = min(1, starBrightness / 0.5)`. Full night is still 0.5 (`Timelines` STAR_BRIGHTNESS keyframes 13228 and 22772). The rain is no longer applied a second time: the weather layer already lerps STAR_BRIGHTNESS to 0 (`WeatherAttributes` 20, 31, 43-62). The reading still dims exactly like vanilla's stars. Under thunder the curve differs slightly from 1.21.1 because vanilla's own star curve differs.
- Time of day: `timeOfDay = STAR_ANGLE / 360`, read through `camera.attributeProbe()` as `SkyRenderer.extractRenderState` does (SRC 278). StarChain keeps its 0-to-1 input. Sun and moon still turn with the stars (SUN_ANGLE track = STAR_ANGLE track, MOON_ANGLE is +180: `Timelines` 53-55), so `behindSunOrMoon` stays valid.
- Partial tick in `render`: `getDeltaTracker().getGameTimeDeltaPartialTick(false)`, the one LevelRenderer uses for the sky (SRC LevelRenderer 471, 580).
- Look vector: `Camera.getLookVector()` became `forwardVector()` (same `forwards` field).
- NBT: `getCompoundOrEmpty`, `getLongOr`, `getListOrEmpty`, `getFloatOr`, `getStringOr`, `getInt(...).orElse(-1)`; `CustomData.contains` is gone, so the copied tag is tested.
- `StarField`: 26.1.2 builds the star quads differently (SkyRenderer 172-201): each corner is centre + rotated offset (`Matrix3f.rotateTowards(-centre, up).rotateZ(-zRot)`), not the 1.21.1 chained `add`. StarField now mirrors that code. Same random stream: `createThreadLocalInstance(10842)` is `SingleThreadedRandomSource`, the same LCG and the same `nextFloat`/`nextDouble` as LegacyRandomSource (SRC `levelgen/SingleThreadedRandomSource.java`, `BitRandomSource.java` 50-59), and the same call order, so star i is still vanilla's star i. Star centres move by up to 0.3 sky units (0.18 degrees) from 1.21.1. Winding: the corner order is vanilla's, and vanilla's star quads are visible with culling on, so the star squares in StarTrace stay front facing.
- `docs/redesign/sky-previews/SkyPreview.java` does not exist on this branch: nothing to change there.

Unverified (needs a human in game): the reading draws at all (pipeline compiles, buffer upload), lines sit on the right stars, colours and additive glow match 1.21.1, it fades with dawn and rain like the stars, no line shows through a back face. Log line `[BalancedMcHardcoreMain] Star reading toward ...` should appear on a reading.

## Step 3: spyglass (K_GuiSpyglassMixin)

- Target `Gui.extractSpyglassOverlay(Lnet/minecraft/client/gui/GuiGraphicsExtractor;F)V` at TAIL (SRC `client/gui/Gui.java` 1014-1027, called from `extractCameraOverlays` 260). NeoForge 26.1.2.126 patches only its blit texture (userdev `Gui.java.patch` hunk at 1028) and calls `extractCameraOverlays` from its layer manager. Forge 64.1.6 makes it public (patched source line 1062); the name and descriptor are the same.
- `drawCenteredString` became `centeredText` (shadow on, as before). `getTimer()` became `getDeltaTracker()`.
- 26.1 text skips alpha 0 and has no "alpha under 4 is opaque" rule. The MIN_ALPHA guard stays, so the fade starts at the same frame as in 1.21.1.

Unverified: the name shows under the cross through a spyglass, fades in over 2 s.

## Step 4: journey (JourneyOverlay, JourneyMusic; the F_ mixins unchanged)

- `render(GuiGraphics, ...)` became `extractRenderState(GuiGraphicsExtractor, ...)` (Renderable, SRC `gui/components/Renderable.java`). GameRenderer still calls the overlay instead of the screen (SRC GameRenderer 487-504).
- During the fade the covered screen draws through `extractRenderStateWithTooltipAndSubtitles`, the call vanilla makes (GameRenderer 504). NeoForge wraps that call in `ClientHooks.drawScreen` (screen render events); the overlay skips those events, as 1.21.1 skipped them with `screen.render`.
- Z order: 26.1 has no z. `graphics.nextStratum()` before the overlay's own drawing puts it above the HUD and the screen, text included (`GuiRenderState.nextStratum` 32-35). This replaces `translate(0, 0, 1000)`.
- Pose: `Matrix3x2fStack` (`pushMatrix`, `translate(x, y)`, `scale(x, y)`). `drawString(..., false)` became `text(..., false)`.
- Performance fix: in 26.1 each `fill` is an element that `GuiRenderState.findAppropriateNode` places by comparing it with the elements already on the layers (`GuiRenderState` 97-148). Thousands of pixel fills make that quadratic. The art now goes into one `ArtLayer` (a `GuiElementRenderState`) that records every fill in order and emits the same quads as `ColoredRectangleRenderState.buildVertices` (same corner swap as `fill`, none for `fillGradient`, pipeline `RenderPipelines.GUI`, no texture, no scissor). It is added once per frame through `minecraft.gameRenderer.getGameRenderState().guiRenderState.addGuiElement(...)`, the same `GuiRenderState` the extractor writes to (GameRenderer 477-478; Forge patched GameRenderer 496-497). GuiRenderer builds all its quads into one mesh (`GuiRenderer.addElementToMesh` 282-298), so blend order is the fill order, as with 1.21.1's `drawManaged`.
- Terrain test: `ReceivingLevelScreen` became `LevelLoadingScreen`. `handleRespawn` always opens it (`ClientPacketListener` 1299, `startWaitingForNewLevel` 1630-1641). `ChunkPos.x/z` became `x()/z()`.
- JourneyMusic: `updateSourceVolume(MUSIC, 1)` became `refreshCategoryVolume(MUSIC)` (SRC `sounds/SoundEngine.java` 118-127), which re-applies `calculateVolume` per playing sound like 1.21.1 did. `updateCategoryVolume` would also store a gain of 1 for MUSIC (153-156), so it is not used. Difference: 1.21.1 stopped a channel at volume 0; 26.1 sets volume 0; `stopPlaying()` then stops the music as before.
- F_ mixins checked, no change needed: `Gui.setTitle(Component)V` (Gui 1230, called by `ClientPacketListener.setTitleText` 1959), `LocalPlayer.respawn()V` (327; called by DeathScreen 48/77 and `Minecraft.setScreen` 1143), `Minecraft.setOverlay(Overlay)V` (1172), `MusicManager.currentMusic` (SoundInstance), `AbstractSoundInstance.volume` (float). All also present in Forge 64.1.6 patched sources.

Unverified (human must watch a distant respawn on each loader): the death screen fades to black with its text covered, the sky, stars, chain, message and pixel dissolve look as in 1.21.1, frame rate during the cinematic, the music fades, the bell rings at the end, the HUD stays hidden under the overlay.

## Step 5: amulet and config screens (PointScreen, PointListScreen, ConfigScreen; PointSignal unchanged)

- `render` split into `extractBackground` (the dim fill, then vanilla's background, the same order as 1.21.1) and `extractRenderState` (widgets, then the text). Screen order: SRC `gui/screens/Screen.java` 104-117, 374-386.
- Text colours now carry alpha 0xFF: 26.1 does not draw a colour with alpha 0 (`GuiGraphicsExtractor.text` 255-259), 1.21.1's font made it opaque. EditBox text colours too (`EditBox.DEFAULT_TEXT_COLOR` = 0xFFE0E0E0).
- `keyPressed(int, int, int)` became `keyPressed(KeyEvent)`; `isConfirmation()` is exactly KEY_RETURN or KEY_NUMPADENTER (`InputWithModifiers` 22-24).
- `sendUnsignedCommand` is gone. `PointScreen.send` (now static, package private; PointListScreen uses it) sends `ServerboundChatCommandPacket(command)` on `connection.send`, the packet 1.21.1's method sent for a command with nothing to sign (the name argument is `greedyString`, not signable).
- ConfigScreen: `EditBox.setFilter` is gone. An anonymous EditBox subclass refuses an `insertText` whose filtered text is not all digits, as the 1.21.1 filter refused it. `setValue` only gets digits here.

Unverified: screens look as in 1.21.1 (dim layer plus blur), Enter saves, digits-only fields, Save/Remove/Delete reach the server.

## Step 6: totem animation (I_ClientPacketListenerTotemMixin)

- Target `findTotem(Lnet/minecraft/world/entity/player/Player;)Lnet/minecraft/world/item/ItemStack;` at RETURN (SRC ClientPacketListener 1190-1199; Forge patched 1192). 26.1.2 vanilla returns the first hand item with `DataComponents.DEATH_PROTECTION`, not the item `TOTEM_OF_UNDYING`. The mixin now leaves vanilla's answer when a hand holds such an item, else shows the first Totem of Eternity in the inventory.
- Unverified: the animation shows the Totem of Eternity when it saves a player. Depends on how the item port gives (or not) the Totem of Eternity the DEATH_PROTECTION component.

## Step 7: checks

- `c26.sh` TAG=client: 0 errors in my files (2 left in the tree, both `init/CreativeTabInit.java`, not mine).
- Extra javac run with `-XDshould-stop.ifError=FLOW -Xlint:all` (`$SCRATCH/agent-client/flow.sh`, log `$SCRATCH/agent-client/flow.log`): no error and no warning in my files.
- Dash check on my added lines: none.
