package github.mrornithorynque.bmh.client.journey;

import java.util.Arrays;
import java.util.Random;

import org.joml.Matrix3x2f;
import org.joml.Matrix3x2fc;

import com.mojang.blaze3d.pipeline.RenderPipeline;
import com.mojang.blaze3d.vertex.VertexConsumer;

import github.mrornithorynque.bmh.init.SoundInit;

import net.minecraft.util.Util;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.navigation.ScreenRectangle;
import net.minecraft.client.gui.render.TextureSetup;
import net.minecraft.client.gui.screens.Overlay;
import net.minecraft.client.gui.screens.LevelLoadingScreen;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.client.renderer.RenderPipelines;
import net.minecraft.client.renderer.state.gui.GuiElementRenderState;
import net.minecraft.network.chat.Component;
import net.minecraft.sounds.SoundEvent;
import net.minecraft.sounds.SoundEvents;
import net.minecraft.sounds.SoundSource;
import net.minecraft.util.Mth;
import net.minecraft.world.level.ChunkPos;

/**
 * The journey cinematic of a distant respawn. The death screen fades to black and a night sky comes in. One star
 * lights up for Home. Then a line goes from star to star, like a star reading of the Navigator's Amulet, and the view
 * follows it far away. The message shows. The sky breaks into pixel blocks from the last star over the new place, and
 * the bell rings.
 *
 * Minecraft draws an Overlay above the HUD and in place of the current screen. Vanilla screens (death screen, terrain
 * loading) still open, close and tick under it, but they stay out of sight. Since 26.1 a frame of the GUI is a list of
 * layers (strata) that the overlay fills in extractRenderState. Render thread only.
 */
public final class JourneyOverlay extends Overlay {

    // The timeline, in milliseconds. FADE_OUT counts from the start, the others from the moment the screen is black.
    static final long FADE_OUT = 600;
    private static final long BLACK = 250;
    private static final long SKY_IN = 1200;
    private static final long HOME_AT = 700;
    private static final long TRACE_AT = 1500;
    private static final long SEGMENT = 210;
    private static final long MESSAGE_AT = 1700;
    private static final long MESSAGE_IN = 400;
    private static final long MESSAGE_HOLD = 1600;
    private static final long MESSAGE_OUT = 400;
    // The last star shines alone for a moment before the sky breaks up.
    private static final long LAST_STAR_BEAT = 400;
    private static final long STAR_LIGHT_UP = 150;
    private static final long ARRIVAL_WAIT = 1000;
    private static final long TERRAIN_GRACE = 500;
    private static final long DISSOLVE = 1000;
    // Vanilla's terrain screen also stops waiting after 30 s. The count starts at the respawn.
    private static final long GIVE_UP = 30000;
    // The server holds the respawn while it generates the landing chunk, and gives up after 120 s itself.
    private static final long SERVER_WAIT = 150000;

    // The sky, the stars and the blocks draw on a grid twice as fine as the GUI, in whole screen pixels.
    private static final double ART_RESOLUTION = 2.0;
    // The blocks of the dissolve across the screen height.
    private static final int BLOCKS_HIGH = 72;

    private static final int SKY_TOP = 0x0B1433;
    private static final int SKY_BOTTOM = 0x03040B;
    private static final int MESSAGE_COLOR = 0xFFF1D0;
    private static final int DUST_COLOR = 0x9AA8D8;
    private static final int[] STAR_COLORS = { 0xFFFFFF, 0xCFE0FF, 0xFFF2CF, 0xBFD4FF };
    // The colors of the amulet's star reading (client/sky/StarTrace): amethyst white with a violet edge.
    private static final int LINE_CORE = 0xF0E4FF;
    private static final int LINE_EDGE = 0xB58CFF;

    // A 4x4 ordered dither, tiled over the blocks: the edge of the reveal is a pixel pattern, not a line.
    private static final int[] BAYER = { 0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5 };

    // The sky, in units of the screen height. The stars repeat every TILE across, so the view can travel far.
    private static final double TILE = 3.0;
    private static final int STAR_COUNT = 1100;
    private static final int BAND_STAR_COUNT = 360;
    private static final int DUST_COUNT = 420;
    // How fast each layer moves with the view: the slower ones look farther away. The line moves at 1.
    private static final double[] LAYER_SPEED = { 0.3, 0.6 };
    private static final double BAND_SPEED = 0.3;
    private static final int SEGMENTS = 11;
    // Where Home is at the start, and where the view keeps the head of the line, in screen widths.
    private static final double HOME_SCREEN_X = 0.3;
    private static final double HEAD_SCREEN_X = 0.55;
    // The line stops this many pixels from each star, so the stars stand out as the knots of the chain.
    private static final int STAR_GAP = 8;

    private final Minecraft minecraft;
    // The dead player that waits for the fade, or null when the respawn happened before the overlay.
    private final LocalPlayer deadPlayer;
    private final long start;
    private final long darkStart;
    private final Star[] stars;
    private final Star[] bandStars;
    private final Dust[] dust;
    // The chain from Home (index 0) to the new place, in units of the screen height, with the length up to each star.
    private final double[] chainU = new double[SEGMENTS + 1];
    private final double[] chainV = new double[SEGMENTS + 1];
    private final double[] chainLength = new double[SEGMENTS + 1];

    private final JourneyMusic music = new JourneyMusic();
    private boolean respawnSent;
    private long respawnedAt = -1;
    private boolean arrived;
    private Component message;
    private long messageStart = -1;
    private long chunkSeenAt = -1;
    private long dissolveStart = -1;
    private int dissolveX;
    private int dissolveY;
    private boolean ambientPlayed;
    // The next star of the chain to chime, from Home (0) to the last one (SEGMENTS).
    private int nextChime;

    JourneyOverlay(Minecraft minecraft, LocalPlayer deadPlayer) {

        this.minecraft = minecraft;
        this.deadPlayer = deadPlayer;
        this.start = Util.getMillis();
        this.darkStart = deadPlayer != null ? this.start + FADE_OUT : this.start;
        this.respawnSent = deadPlayer == null;

        Random random = new Random();
        this.stars = new Star[STAR_COUNT];
        this.bandStars = new Star[BAND_STAR_COUNT];
        this.dust = new Dust[DUST_COUNT];

        for (int i = 0; i < STAR_COUNT; i++) {
            this.stars[i] = new Star(random, random.nextDouble() * TILE, random.nextDouble(), i % 3 == 0 ? 1 : 0);
        }

        for (int i = 0; i < BAND_STAR_COUNT; i++) {
            double along = -0.1 + random.nextDouble() * (TILE + 0.1);
            this.bandStars[i] = new Star(random, along, bandV(along) + random.nextGaussian() * 0.06, -1);
        }

        for (int i = 0; i < DUST_COUNT; i++) {
            this.dust[i] = new Dust(random);
        }

        // The direction is random on purpose: the way back is the amulet's to tell. The line zigzags: each segment
        // turns up or down by 18 to 62 degrees, and it usually turns the other way than the segment before.
        this.chainV[0] = 0.4 + random.nextDouble() * 0.15;
        int turn = random.nextBoolean() ? 1 : -1;

        for (int i = 1; i <= SEGMENTS; i++) {
            if (random.nextFloat() < 0.7F) {
                turn = -turn;
            }

            double length = 0.14 + random.nextDouble() * 0.16;
            double angle = Math.toRadians(turn * (18 + random.nextDouble() * 44));
            double v = this.chainV[i - 1] + length * Math.sin(angle);

            // At the top or the bottom limit, the segment bounces back and the zigzag goes on.
            if (v < 0.22 || v > 0.66) {
                v = v < 0.22 ? 0.44 - v : 1.32 - v;
                turn = v < 0.44 ? 1 : -1;
            }

            this.chainU[i] = this.chainU[i - 1] + length * Math.cos(angle);
            this.chainV[i] = v;
            this.chainLength[i] = this.chainLength[i - 1]
                    + Math.hypot(this.chainU[i] - this.chainU[i - 1], this.chainV[i] - this.chainV[i - 1]);
        }
    }

    // The server signal after the respawn. The message is null when the game rule hides it.
    void arrive(Component message) {

        this.arrived = true;

        // Too late for a message once the sky breaks up.
        if (this.dissolveStart < 0) {
            this.message = message;
        }
    }

    @Override
    public boolean isPauseScreen() {

        // A pause would also stop the singleplayer server, and with it the terrain the overlay waits for.
        return false;
    }

    @Override
    public void extractRenderState(GuiGraphicsExtractor guiGraphics, int mouseX, int mouseY, float partialTick) {

        long now = Util.getMillis();
        // The art grid: one art pixel is a whole number of screen pixels, about half a GUI pixel.
        double guiScale = this.minecraft.getWindow().getGuiScale();
        int pixel = Math.max(1, (int) Math.round(guiScale / ART_RESOLUTION));
        float artScale = (float) (pixel / guiScale);
        int width = (this.minecraft.getWindow().getWidth() + pixel - 1) / pixel;
        int height = (this.minecraft.getWindow().getHeight() + pixel - 1) / pixel;

        if (!update(now, width, height)) {
            return;
        }

        if (now < this.darkStart && this.minecraft.screen != null) {
            // The fade to black: the death screen stays under it. This is the call vanilla makes for a screen.
            this.minecraft.screen.extractRenderStateWithTooltipAndSubtitles(guiGraphics, mouseX, mouseY, partialTick);
        }

        // A new layer: the overlay draws in front of the HUD and of the screen it covers, their text included.
        // 1.21.1 moved the overlay to z 1000 for this.
        guiGraphics.nextStratum();
        guiGraphics.pose().pushMatrix();

        if (now < this.darkStart) {
            guiGraphics.fill(0, 0, guiGraphics.guiWidth(), guiGraphics.guiHeight(),
                    argb(ease((float) (now - this.start) / FADE_OUT), 0x000000));
        } else {
            long time = now - this.darkStart;
            float sky = Mth.clamp((float) (time - BLACK) / SKY_IN, 0.0F, 1.0F);
            float reveal = this.dissolveStart < 0 ? 0.0F : ease((float) (now - this.dissolveStart) / DISSOLVE);
            double traced = tracedLength(time);
            double camera = camera(traced, width, height);

            guiGraphics.pose().pushMatrix();
            guiGraphics.pose().scale(artScale, artScale);
            Frame frame = new Frame(new ArtLayer(guiGraphics.pose(), width, height), width, height, reveal);
            drawSky(frame, sky);
            drawDust(frame, sky, camera);
            drawStars(frame, now, time, sky, camera);
            drawChain(frame, now, time, sky, traced, camera);
            // The whole art goes to the GUI as one element, as 1.21.1's drawManaged drew it as one batch.
            this.minecraft.gameRenderer.getGameRenderState().guiRenderState.addGuiElement(frame.art);
            guiGraphics.pose().popMatrix();

            // The message stays at the size of the GUI text.
            drawMessage(guiGraphics, guiGraphics.guiWidth(), guiGraphics.guiHeight(), now);
        }

        guiGraphics.pose().popMatrix();
    }

    // Moves the timeline on. Returns false when the overlay is done and draws nothing more.
    private boolean update(long now, int width, int height) {

        if (this.minecraft.getOverlay() != this) {
            return false;
        }

        // The player left the world (disconnect, kick): nothing to show any more.
        if (this.minecraft.level == null || this.minecraft.player == null) {
            this.minecraft.setOverlay(null);
            return false;
        }

        // The game music fades out with the screen. The bell and the cinematic sounds are other sounds and stay.
        if (this.deadPlayer != null) {
            this.music.fade(1.0F - ease((float) (now - this.start) / FADE_OUT));
        }

        if (!this.respawnSent && now >= this.darkStart) {
            this.respawnSent = true;
            JourneyCinematic.sendRespawn();
        }

        boolean respawned = this.respawnSent && this.minecraft.player != this.deadPlayer;

        if (respawned) {
            if (this.respawnedAt < 0) {
                this.respawnedAt = now;
            }

            holdInput();

            if (!this.ambientPlayed && now >= this.darkStart + BLACK) {
                this.ambientPlayed = true;
                playSound(SoundInit.RESPAWN_SOUND.get(), 0.3F, 1.0F);
            }
        }

        if (this.message != null && this.messageStart < 0 && now >= this.darkStart + MESSAGE_AT) {
            this.messageStart = now;
        }

        chimeStars(now - this.darkStart);

        if (this.dissolveStart < 0 && (readyToReveal(now, respawned) || gaveUp(now))) {
            // The pixels break up from the last star of the chain.
            int[] last = chainPoint(SEGMENTS, camera(this.chainLength[SEGMENTS], width, height), width, height);
            this.dissolveX = last[0];
            this.dissolveY = last[1];
            this.dissolveStart = now;
            releaseInput();
            // The sky shatters.
            playSound(SoundEvents.AMETHYST_CLUSTER_BREAK, 0.6F, 0.8F);
        }

        if (this.dissolveStart >= 0 && now - this.dissolveStart >= DISSOLVE) {
            this.minecraft.setOverlay(null);
            playSound(SoundInit.BELL_SOUND.get(), 1.0F, 1.0F);
            return false;
        }

        return true;
    }

    // The server may hold the respawn for a long time: the wait for the terrain counts from the respawn, not from the start.
    private boolean gaveUp(long now) {

        return this.respawnedAt >= 0 ? now - this.respawnedAt > GIVE_UP : now - this.start > SERVER_WAIT;
    }

    private boolean readyToReveal(long now, boolean respawned) {

        if (!respawned) {
            return false;
        }

        // First, so that the grace for the terrain runs while the line is traced.
        boolean terrainReady = terrainReady(now);

        if (now < this.darkStart + TRACE_AT + SEGMENTS * SEGMENT + LAST_STAR_BEAT) {
            return false;
        }

        // The arrival signal comes right after the respawn. Without it (no distant position), go on without a message.
        if (!this.arrived && now - this.respawnedAt < ARRIVAL_WAIT) {
            return false;
        }

        if (this.messageStart >= 0 && now < this.messageStart + MESSAGE_IN + MESSAGE_HOLD + MESSAGE_OUT) {
            return false;
        }

        if (this.message != null && this.messageStart < 0) {
            return false;
        }

        return terrainReady;
    }

    // The same test as vanilla's terrain screen where it exists (1.20.2 and later), plus the player's chunk: 1.20 and
    // 1.20.1 show no terrain screen on a respawn in the same dimension. Since 26.1 the terrain screen of a respawn is
    // LevelLoadingScreen (ClientPacketListener.handleRespawn opens it), in place of ReceivingLevelScreen.
    private boolean terrainReady(long now) {

        if (this.minecraft.screen instanceof LevelLoadingScreen) {
            return false;
        }

        ChunkPos chunk = this.minecraft.player.chunkPosition();

        if (!this.minecraft.level.getChunkSource().hasChunk(chunk.x(), chunk.z())) {
            this.chunkSeenAt = -1;
            return false;
        }

        if (this.chunkSeenAt < 0) {
            this.chunkSeenAt = now;
        }

        // The chunk arrives before its sections are built.
        return now - this.chunkSeenAt >= TERRAIN_GRACE;
    }

    // With no screen open, Minecraft gives the keys and the mouse to the player, who would walk in the dark.
    private void holdInput() {

        if (this.dissolveStart < 0 && this.minecraft.screen == null) {
            this.minecraft.setScreen(new HoldScreen());
        }
    }

    private void releaseInput() {

        if (this.minecraft.screen instanceof HoldScreen) {
            this.minecraft.setScreen(null);
        }
    }

    // Amethyst, like the color of the line: a chime for Home, a higher one for each star the line reaches, and a long
    // resonance for the last star.
    private void chimeStars(long time) {

        if (this.nextChime > SEGMENTS || time < HOME_AT) {
            return;
        }

        boolean reached = this.nextChime == 0 || (this.nextChime == SEGMENTS
                ? time >= TRACE_AT + SEGMENTS * SEGMENT
                : tracedLength(time) >= this.chainLength[this.nextChime]);

        if (!reached) {
            return;
        }

        if (this.nextChime == SEGMENTS) {
            playSound(SoundEvents.AMETHYST_BLOCK_RESONATE, 1.0F, 1.0F);
        } else {
            playSound(SoundEvents.AMETHYST_BLOCK_CHIME, this.nextChime == 0 ? 1.0F : 0.7F,
                    0.7F + 0.8F * this.nextChime / SEGMENTS);
        }

        this.nextChime++;
    }

    private void playSound(SoundEvent sound, float volume, float pitch) {

        LocalPlayer player = this.minecraft.player;
        this.minecraft.level.playLocalSound(player.getX(), player.getY(), player.getZ(), sound, SoundSource.PLAYERS,
                volume, pitch, false);
    }

    // How much of the chain the line has traced, in units of the screen height: one segment after the other, slow at
    // the start and at the end.
    private double tracedLength(long time) {

        double progress = Mth.clamp((double) (time - TRACE_AT) / (SEGMENTS * SEGMENT), 0.0, 1.0);
        return this.chainLength[SEGMENTS] * ease((float) progress);
    }

    // How far the view has travelled, in pixels: it starts to follow the head of the line once the head passes it.
    private double camera(double traced, int width, int height) {

        return Math.max(0.0, HOME_SCREEN_X * width + uAt(traced) * height - HEAD_SCREEN_X * width);
    }

    // The horizontal place of a point at this length along the chain.
    private double uAt(double length) {

        for (int i = 0; i < SEGMENTS; i++) {
            if (length <= this.chainLength[i + 1]) {
                double part = (length - this.chainLength[i]) / (this.chainLength[i + 1] - this.chainLength[i]);
                return Mth.lerp(part, this.chainU[i], this.chainU[i + 1]);
            }
        }

        return this.chainU[SEGMENTS];
    }

    private int[] chainPoint(int i, double camera, int width, int height) {

        return new int[] { (int) Math.round(HOME_SCREEN_X * width + this.chainU[i] * height - camera),
                (int) Math.round(this.chainV[i] * height) };
    }

    // The black, then the night gradient. During the dissolve, only the blocks that are not revealed yet.
    private void drawSky(Frame frame, float sky) {

        if (frame.reveal <= 0.0F) {
            frame.art.fill(0, 0, frame.width, frame.height, 0xFF000000);
            frame.art.fillGradient(0, 0, frame.width, frame.height, argb(sky, SKY_TOP), argb(sky, SKY_BOTTOM));
            return;
        }

        for (int y = 0; y < frame.height; y += frame.cell) {
            int color = 0xFF000000 | lerpColor(0x000000, lerpColor(SKY_TOP, SKY_BOTTOM, (float) y / frame.height), sky);

            for (int x = 0; x < frame.width; x += frame.cell) {
                if (!revealedBlock(frame, x, y)) {
                    frame.art.fill(x, y, Math.min(x + frame.cell, frame.width), Math.min(y + frame.cell, frame.height), color);
                }
            }
        }
    }

    // The faint glow of a band of the Milky Way: wide, dim pixel blocks on the farthest layer.
    private void drawDust(Frame frame, float sky, double camera) {

        // The glow fades out as the sky breaks up: it has no blocks of its own to hide in.
        float alpha = sky * (1.0F - Mth.clamp(frame.reveal * 5.0F, 0.0F, 1.0F));

        if (alpha <= 0.0F) {
            return;
        }

        int unit = Math.max(1, frame.height / 240);

        for (Dust blob : this.dust) {
            int x = (int) Math.round(blob.u * frame.height - BAND_SPEED * camera);
            int y = (int) Math.round(blob.v * frame.height);
            int size = blob.size * unit;

            if (x + size < 0 || x - size > frame.width) {
                continue;
            }

            frame.art.fill(x - size / 2, y - size / 2, x + size / 2, y + size / 2, argb(alpha * blob.alpha, DUST_COLOR));
        }
    }

    private void drawStars(Frame frame, long now, long time, float sky, double camera) {

        if (sky <= 0.0F) {
            return;
        }

        long tile = Math.round(TILE * frame.height);

        for (Star star : this.stars) {
            int x = (int) Math.floorMod(Math.round(star.u * frame.height - LAYER_SPEED[star.layer] * camera), tile);
            drawStar(frame, star, x, now, time);
        }

        for (Star star : this.bandStars) {
            drawStar(frame, star, (int) Math.round(star.u * frame.height - BAND_SPEED * camera), now, time);
        }
    }

    private void drawStar(Frame frame, Star star, int x, long now, long time) {

        int y = (int) Math.round(star.v * frame.height);
        // Each star lights up on its own, while the sky comes in.
        float appear = Mth.clamp((time - BLACK - star.delay) / 300.0F, 0.0F, 1.0F);

        if (appear <= 0.0F || x < 2 || y < 2 || x >= frame.width - 2 || y >= frame.height - 2 || !frame.covered(x, y)) {
            return;
        }

        float twinkle = 1.0F - star.twinkle * (0.5F + 0.5F * (float) Math.sin(star.phase + now * star.speed));
        float alpha = star.brightness * twinkle * appear;
        frame.pixel(x, y, argb(alpha, star.color));

        // A plus for the bright stars, longer arms for the brightest.
        for (int arm = 1; arm <= star.arms; arm++) {
            int armColor = argb(alpha * (arm == 1 ? 0.5F : 0.22F), star.color);
            frame.pixel(x - arm, y, armColor);
            frame.pixel(x + arm, y, armColor);
            frame.pixel(x, y - arm, armColor);
            frame.pixel(x, y + arm, armColor);
        }
    }

    // Home lights up first. Then the line goes from star to star, and each star brightens as the line reaches it.
    private void drawChain(Frame frame, long now, long time, float sky, double traced, double camera) {

        if (time < HOME_AT) {
            return;
        }

        // Not dimmed by the sky: the chain must stand out while the sky is still coming in.
        float appear = Mth.clamp((float) (time - HOME_AT) / STAR_LIGHT_UP, 0.0F, 1.0F);

        for (int i = 0; i < SEGMENTS; i++) {
            if (traced <= this.chainLength[i]) {
                break;
            }

            int[] from = chainPoint(i, camera, frame.width, frame.height);
            int[] to = chainPoint(i + 1, camera, frame.width, frame.height);
            double part = Math.min(1.0, (traced - this.chainLength[i]) / (this.chainLength[i + 1] - this.chainLength[i]));
            drawSegment(frame, from, to, part, appear, part < 1.0);
        }

        for (int i = 0; i <= SEGMENTS; i++) {
            int[] at = chainPoint(i, camera, frame.width, frame.height);
            // 0 until the line reaches the star, then 1 within STAR_LIGHT_UP. Home is lit from the start. The line
            // stops on the last star and never goes past it, so the last star counts from the end of the trace.
            double reached = i == SEGMENTS ? time - TRACE_AT - SEGMENTS * SEGMENT
                    : (traced - this.chainLength[i]) / this.chainLength[SEGMENTS] * SEGMENTS * SEGMENT;
            float lit = i == 0 ? appear : Mth.clamp((float) (reached / STAR_LIGHT_UP), 0.0F, 1.0F) * appear;

            if (i == 0) {
                // Home pulses: a bright star in a double ring.
                float pulse = 0.5F + 0.5F * (float) Math.sin(now * 0.008);
                drawHalo(frame, at[0], at[1], 10 + Math.round(pulse * 2), lit * (0.75F + 0.25F * pulse), LINE_EDGE);
                drawHalo(frame, at[0], at[1], 16 + Math.round(pulse * 3), lit * 0.3F, LINE_EDGE);
                drawKnot(frame, at[0], at[1], lit, 3);
            } else if (lit > 0.0F) {
                boolean last = i == SEGMENTS;
                drawHalo(frame, at[0], at[1], last ? 12 : 7, lit, LINE_EDGE);
                drawHalo(frame, at[0], at[1], last ? 18 : 4, lit * (last ? 0.35F : 0.5F), last ? LINE_EDGE : LINE_CORE);
                drawKnot(frame, at[0], at[1], lit, last ? 3 : 2);
            } else {
                // A star of the chain that the line has not reached yet: already a little brighter than the others.
                drawKnot(frame, at[0], at[1], 0.5F * appear * sky, 1);
            }
        }
    }

    // A star of the chain: a bright core with arms, and faint corners for the bigger ones.
    private static void drawKnot(Frame frame, int x, int y, float alpha, int arms) {

        frame.pixel(x, y, argb(alpha, 0xFFFFFF));

        for (int arm = 1; arm <= arms; arm++) {
            int color = argb(alpha * (arm == 1 ? 0.8F : 0.4F), LINE_CORE);
            frame.pixel(x - arm, y, color);
            frame.pixel(x + arm, y, color);
            frame.pixel(x, y - arm, color);
            frame.pixel(x, y + arm, color);
        }

        if (arms > 1) {
            int corner = argb(alpha * 0.35F, LINE_CORE);
            frame.pixel(x - 1, y - 1, corner);
            frame.pixel(x + 1, y - 1, corner);
            frame.pixel(x - 1, y + 1, corner);
            frame.pixel(x + 1, y + 1, corner);
        }
    }

    // A pixel line with a violet edge and a faint glow, stopped short of both stars, and a spark at its moving head.
    private void drawSegment(Frame frame, int[] from, int[] to, double part, float alpha, boolean head) {

        int dx = to[0] - from[0];
        int dy = to[1] - from[1];
        int steps = Math.max(Math.abs(dx), Math.abs(dy));
        boolean flat = Math.abs(dx) >= Math.abs(dy);
        int last = (int) Math.round(steps * part);
        int[] spark = null;

        for (int k = STAR_GAP; k <= Math.min(last, steps - STAR_GAP); k++) {
            int x = from[0] + Math.round((float) dx * k / steps);
            int y = from[1] + Math.round((float) dy * k / steps);
            spark = new int[] { x, y };

            // The core, the violet edge in two steps, then a faint glow.
            for (int side = -3; side <= 3; side++) {
                int px = flat ? x : x + side;
                int py = flat ? y + side : y;
                int abs = Math.abs(side);
                frame.pixel(px, py, abs == 0 ? argb(alpha, LINE_CORE) : abs == 1 ? argb(alpha * 0.6F, LINE_EDGE)
                        : abs == 2 ? argb(alpha * 0.3F, LINE_EDGE) : argb(alpha * 0.12F, LINE_EDGE));
            }
        }

        if (head && spark != null) {
            for (int sx = -1; sx <= 1; sx++) {
                for (int sy = -1; sy <= 1; sy++) {
                    frame.pixel(spark[0] + sx, spark[1] + sy, argb(alpha * (sx == 0 || sy == 0 ? 1.0F : 0.5F), 0xFFFFFF));
                }
            }
            drawHalo(frame, spark[0], spark[1], 4, alpha * 0.8F, LINE_EDGE);
        }
    }

    // A ring of pixels around a star. Enough points that the ring has no holes at this radius.
    private static void drawHalo(Frame frame, int x, int y, int radius, float alpha, int color) {

        int points = Math.max(8, (int) Math.ceil(radius * 2 * Math.PI));

        for (int k = 0; k < points; k++) {
            double angle = Math.PI * 2 * k / points;
            frame.pixel(x + (int) Math.round(Math.cos(angle) * radius), y + (int) Math.round(Math.sin(angle) * radius),
                    argb(alpha * 0.55F, color));
        }
    }

    private void drawMessage(GuiGraphicsExtractor guiGraphics, int width, int height, long now) {

        if (this.message == null || this.messageStart < 0) {
            return;
        }

        long time = now - this.messageStart;
        float alpha;

        if (time < MESSAGE_IN) {
            alpha = (float) time / MESSAGE_IN;
        } else if (time < MESSAGE_IN + MESSAGE_HOLD) {
            alpha = 1.0F;
        } else {
            alpha = 1.0F - (float) (time - MESSAGE_IN - MESSAGE_HOLD) / MESSAGE_OUT;
        }

        // 1.21.1's font drew an alpha under 4 as opaque: the message skips those frames, as it did then.
        int alphaByte = Math.round(Mth.clamp(alpha, 0.0F, 1.0F) * 255);

        if (alphaByte < 4) {
            return;
        }

        Font font = this.minecraft.font;
        int textWidth = font.width(this.message);
        float scale = textWidth * 2 <= width - 32 ? 2.0F : 1.0F;

        // Under the chain, which crosses the middle of the screen.
        guiGraphics.pose().pushMatrix();
        guiGraphics.pose().translate(width / 2.0F, height * 0.8F);
        guiGraphics.pose().scale(scale, scale);
        guiGraphics.text(font, this.message, -textWidth / 2, -font.lineHeight / 2, (alphaByte << 24) | MESSAGE_COLOR, false);
        guiGraphics.pose().popMatrix();
    }

    // The block at (x, y) is revealed when the dissolve passes its threshold: near the last star first, with a dither
    // on the edge.
    private boolean revealedBlock(Frame frame, int x, int y) {

        int cell = frame.cell;
        double dx = x + cell / 2.0 - this.dissolveX;
        double dy = y + cell / 2.0 - this.dissolveY;
        double farX = Math.max(this.dissolveX, frame.width - this.dissolveX);
        double farY = Math.max(this.dissolveY, frame.height - this.dissolveY);
        double radial = Math.min(1.0, Math.sqrt(dx * dx + dy * dy) / Math.sqrt(farX * farX + farY * farY));
        double dither = BAYER[((y / cell) & 3) * 4 + ((x / cell) & 3)] / 16.0;

        return frame.reveal > 0.6 * radial + 0.4 * dither;
    }

    // The band of the Milky Way crosses the sky from high on the left to low on the right.
    private static double bandV(double u) {

        return 0.1 + 0.24 * u / TILE;
    }

    private static float ease(float t) {

        t = Mth.clamp(t, 0.0F, 1.0F);
        return t * t * (3.0F - 2.0F * t);
    }

    private static int argb(float alpha, int rgb) {

        return (Math.round(Mth.clamp(alpha, 0.0F, 1.0F) * 255) << 24) | (rgb & 0xFFFFFF);
    }

    private static int lerpColor(int from, int to, float t) {

        int r = Math.round(Mth.lerp(t, (from >> 16) & 0xFF, (to >> 16) & 0xFF));
        int g = Math.round(Mth.lerp(t, (from >> 8) & 0xFF, (to >> 8) & 0xFF));
        int b = Math.round(Mth.lerp(t, from & 0xFF, to & 0xFF));
        return (r << 16) | (g << 8) | b;
    }

    /** What a frame draws on, with the dissolve state for the blocks. */
    private final class Frame {

        final ArtLayer art;
        final int width;
        final int height;
        final float reveal;
        final int cell;

        Frame(ArtLayer art, int width, int height, float reveal) {

            this.art = art;
            this.width = width;
            this.height = height;
            this.reveal = reveal;
            this.cell = Math.max(4, height / BLOCKS_HIGH);
        }

        // True while the block of this pixel still shows the sky.
        boolean covered(int x, int y) {

            return this.reveal <= 0.0F || !revealedBlock(this, x - Math.floorMod(x, this.cell), y - Math.floorMod(y, this.cell));
        }

        void pixel(int x, int y, int color) {

            if (x >= 0 && y >= 0 && x < this.width && y < this.height && covered(x, y)) {
                this.art.fill(x, y, x + 1, y + 1, color);
            }
        }
    }

    /**
     * Every fill of one frame of the art, in order, as one GUI element. Since 26.1 each GuiGraphicsExtractor.fill is
     * an element that GuiRenderState places on a layer: it compares the element with the elements already there. A
     * frame has thousands of pixel fills, and that search grows with the square of their count. One element is placed
     * once. The rectangles are the ones fill and fillGradient make (ColoredRectangleRenderState, 26.1.2), drawn with the
     * same pipeline in the same order, so the blend is the same.
     */
    private static final class ArtLayer implements GuiElementRenderState {

        // Per rectangle: x0, y0, x1, y1, the colour at y0, the colour at y1.
        private static final int STRIDE = 6;

        private final Matrix3x2f pose;
        private final ScreenRectangle bounds;
        private int[] rects = new int[STRIDE * 4096];
        private int size;

        ArtLayer(Matrix3x2fc pose, int width, int height) {

            this.pose = new Matrix3x2f(pose);
            this.bounds = new ScreenRectangle(0, 0, width, height).transformMaxBounds(this.pose);
        }

        // As GuiGraphicsExtractor.fill: the corners are swapped the same way, so the quad winds the same way.
        void fill(int x0, int y0, int x1, int y1, int color) {

            if (x0 < x1) {
                int swap = x0;
                x0 = x1;
                x1 = swap;
            }

            if (y0 < y1) {
                int swap = y0;
                y0 = y1;
                y1 = swap;
            }

            add(x0, y0, x1, y1, color, color);
        }

        // As GuiGraphicsExtractor.fillGradient: the top colour at y0, the bottom colour at y1.
        void fillGradient(int x0, int y0, int x1, int y1, int top, int bottom) {

            add(x0, y0, x1, y1, top, bottom);
        }

        private void add(int x0, int y0, int x1, int y1, int color0, int color1) {

            if (this.size + STRIDE > this.rects.length) {
                this.rects = Arrays.copyOf(this.rects, this.rects.length * 2);
            }

            this.rects[this.size++] = x0;
            this.rects[this.size++] = y0;
            this.rects[this.size++] = x1;
            this.rects[this.size++] = y1;
            this.rects[this.size++] = color0;
            this.rects[this.size++] = color1;
        }

        @Override
        public void buildVertices(VertexConsumer consumer) {

            for (int i = 0; i < this.size; i += STRIDE) {
                int x0 = this.rects[i];
                int y0 = this.rects[i + 1];
                int x1 = this.rects[i + 2];
                int y1 = this.rects[i + 3];
                int color0 = this.rects[i + 4];
                int color1 = this.rects[i + 5];
                consumer.addVertexWith2DPose(this.pose, x0, y0).setColor(color0);
                consumer.addVertexWith2DPose(this.pose, x0, y1).setColor(color1);
                consumer.addVertexWith2DPose(this.pose, x1, y1).setColor(color1);
                consumer.addVertexWith2DPose(this.pose, x1, y0).setColor(color0);
            }
        }

        @Override
        public RenderPipeline pipeline() {

            return RenderPipelines.GUI;
        }

        @Override
        public TextureSetup textureSetup() {

            return TextureSetup.noTexture();
        }

        @Override
        public ScreenRectangle scissorArea() {

            return null;
        }

        @Override
        public ScreenRectangle bounds() {

            return this.bounds;
        }
    }

    /** One star of the night sky, at its place before the view moves. */
    private static final class Star {

        // In units of the screen height.
        final double u;
        final double v;
        // 0 for the far layer, 1 for the near one, -1 for a star of the Milky Way band.
        final int layer;
        final float brightness;
        // 0 for a dot, 1 for a plus, 2 for a plus with long arms.
        final int arms;
        final float twinkle;
        final int color;
        final double phase;
        final double speed;
        final long delay;

        Star(Random random, double u, double v, int layer) {

            this.u = u;
            this.v = v;
            this.layer = Math.max(0, layer);
            float kind = random.nextFloat();
            // The near layer has the bright stars, the band has the faint ones.
            this.arms = layer == 1 ? (kind < 0.12F ? 2 : kind < 0.45F ? 1 : 0) : layer == 0 && kind < 0.06F ? 1 : 0;
            this.brightness = this.arms > 0 ? 0.8F + random.nextFloat() * 0.2F
                    : (layer < 0 ? 0.15F : 0.2F) + random.nextFloat() * 0.5F;
            this.twinkle = this.arms > 0 ? 0.45F : 0.25F;
            this.color = STAR_COLORS[random.nextInt(STAR_COLORS.length)];
            this.phase = random.nextDouble() * Math.PI * 2;
            this.speed = 0.0015 + random.nextDouble() * 0.0025;
            this.delay = (long) (random.nextDouble() * SKY_IN * 0.8);
        }
    }

    /** One blob of the Milky Way glow. */
    private static final class Dust {

        final double u;
        final double v;
        // In units of a 240th of the screen height.
        final int size;
        final float alpha;

        Dust(Random random) {

            this.u = -0.1 + random.nextDouble() * (TILE + 0.1);
            this.v = bandV(this.u) + random.nextGaussian() * 0.045;
            this.size = 2 + random.nextInt(5);
            this.alpha = 0.02F + random.nextFloat() * 0.035F;
        }
    }

    /** Takes the keys and the mouse while the screen is dark. The overlay draws in its place. */
    static final class HoldScreen extends Screen {

        HoldScreen() {

            super(Component.empty());
        }

        @Override
        public boolean isPauseScreen() {

            return false;
        }

        @Override
        public boolean shouldCloseOnEsc() {

            return false;
        }

        // Another overlay (a resource reload) can take the place of the journey: then nothing must stay held.
        @Override
        public void tick() {

            if (!(this.minecraft.getOverlay() instanceof JourneyOverlay)) {
                this.minecraft.setScreen(null);
            }
        }
    }
}
