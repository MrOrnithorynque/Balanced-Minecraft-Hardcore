package github.mrornithorynque.bmh.client.sky;

import java.util.Locale;

import org.joml.Matrix4f;
import org.slf4j.Logger;

import com.mojang.blaze3d.platform.GlStateManager;
import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.vertex.BufferBuilder;
import com.mojang.blaze3d.vertex.BufferUploader;
import com.mojang.blaze3d.vertex.DefaultVertexFormat;
import com.mojang.blaze3d.vertex.Tesselator;
import com.mojang.blaze3d.vertex.VertexFormat;
import com.mojang.logging.LogUtils;

import github.mrornithorynque.bmh.items.NavigatorAmuletItem;
import net.minecraft.client.Minecraft;
import net.minecraft.client.multiplayer.ClientLevel;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.client.renderer.GameRenderer;
import net.minecraft.client.renderer.ShaderInstance;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.Tag;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.item.ItemStack;

/**
 * Draws a Navigator's Amulet star reading in the sky, for this client's player only. D_LevelRendererSkyMixin calls it
 * right after vanilla draws the stars, with the stars' own pose, so the chain stays pinned to them as the sky turns.
 * Render thread only.
 */
public final class StarReadingRenderer {

    private static final Logger LOGGER = LogUtils.getLogger();

    // The amulet's reading (written server side by the star reading): azimuth in degrees clockwise from north, from
    // the player to Home, and the level's game time of the reading.
    private static final String READING = "bmh_reading";
    private static final String AZIMUTH = "azimuth";
    private static final String TIME = "time";
    /** An amulet whose reading is this old when the client first sees it (picked up, relogged) shows nothing. */
    private static final long FRESH_TICKS = 40;
    /** Vanilla's star brightness at full night: a reading there is at full strength. */
    private static final float FULL_NIGHT_STARS = 0.5F;

    private static long lastShownTime = Long.MIN_VALUE;
    private static Shown shown;

    static {
        // The azimuth convention shared with the amulet (brief, part C): Home at (0, -100) seen from (0, 0) is 0,
        // Home at (100, 0) is 90. This is the direction the picker actually aims at.
        double[] north = StarChain.toward(0.0), east = StarChain.toward(90.0);
        LOGGER.info(String.format(Locale.ROOT,
                "[BalancedMcHardcoreMain] Sky azimuth self-check: from (0, 0), azimuth 0 aims at (%d, %d), azimuth 90 at (%d, %d); expected (0, -100) and (100, 0)",
                Math.round(north[0] * 100), Math.round(north[1] * 100), Math.round(east[0] * 100), Math.round(east[1] * 100)));
    }

    private record Shown(ClientLevel level, int[] chain, double startTick) {
    }

    private StarReadingRenderer() {
    }

    public static void render(ClientLevel level, Matrix4f starPose, float partialTick) {
        LocalPlayer player = Minecraft.getInstance().player;
        if (player == null) {
            return;
        }
        double now = level.getGameTime() + partialTick;
        startFreshReading(level, player, now, partialTick);
        if (DevSkyShots.ENABLED) {
            DevSkyShots.frame(level, player, now, partialTick);
        }

        Shown reading = shown;
        if (reading == null) {
            return;
        }
        double seconds = (now - reading.startTick()) / 20.0;
        if (reading.level() != level || seconds > StarTrace.duration(reading.chain().length)) {
            shown = null;
            return;
        }

        float skyAlpha = Math.min(1.0F,
                level.getStarBrightness(partialTick) * (1.0F - level.getRainLevel(partialTick)) / FULL_NIGHT_STARS);
        float[] triangles = StarTrace.triangles(StarField.stars(), reading.chain(), seconds, skyAlpha);
        if (triangles.length > 0) {
            draw(starPose, triangles);
        }
    }

    // A reading starts the moment the player holds an amulet carrying one this client has not shown yet. Once started
    // it plays to the end, amulet in hand or not.
    private static void startFreshReading(ClientLevel level, LocalPlayer player, double now, float partialTick) {
        for (InteractionHand hand : InteractionHand.values()) {
            ItemStack stack = player.getItemInHand(hand);
            CompoundTag tag = stack.getTag();
            if (!(stack.getItem() instanceof NavigatorAmuletItem) || tag == null
                    || !tag.contains(READING, Tag.TAG_COMPOUND)) {
                continue;
            }
            CompoundTag reading = tag.getCompound(READING);
            long time = reading.getLong(TIME);
            if (time == lastShownTime || Math.abs(level.getGameTime() - time) >= FRESH_TICKS) {
                continue;
            }
            lastShownTime = time;
            show(level, reading.getFloat(AZIMUTH), now, partialTick);
            return;
        }
    }

    static void show(ClientLevel level, float azimuth, double now, float partialTick) {
        // The sky as it stands now; within the 40 fresh ticks it turns less than a degree.
        float timeOfDay = level.getTimeOfDay(partialTick);
        int[] chain = StarChain.pick(StarField.stars(), azimuth, timeOfDay);
        shown = chain.length > 0 ? new Shown(level, chain, now) : null;
        if (chain.length > 0) {
            StarField.Star star = StarField.stars().get(chain[chain.length - 1]);
            double[] last = StarChain.toWorld(star.x, star.y, star.z, timeOfDay);
            double lastAzimuth = (Math.toDegrees(Math.atan2(last[0], -last[2])) + 360.0) % 360.0;
            LOGGER.info(String.format(Locale.ROOT,
                    "[BalancedMcHardcoreMain] Star reading toward %.1f: %d stars, the last at azimuth %.1f, altitude %.1f",
                    azimuth, chain.length, lastAzimuth, Math.toDegrees(Math.asin(last[1]))));
        }
    }

    // Same path as vanilla's sun and moon: vertices through the pose on the CPU, then BufferUploader. Blending stays
    // vanilla's star blend (SRC_ALPHA, ONE: light adds up); the shader and its colour go back as they were.
    private static void draw(Matrix4f pose, float[] v) {
        ShaderInstance previousShader = RenderSystem.getShader();
        float[] previousColour = RenderSystem.getShaderColor().clone();

        RenderSystem.setShader(GameRenderer::getPositionColorShader);
        RenderSystem.setShaderColor(1.0F, 1.0F, 1.0F, 1.0F);
        RenderSystem.enableBlend();
        RenderSystem.blendFuncSeparate(GlStateManager.SourceFactor.SRC_ALPHA, GlStateManager.DestFactor.ONE,
                GlStateManager.SourceFactor.ONE, GlStateManager.DestFactor.ZERO);
        BufferBuilder buffer = Tesselator.getInstance().getBuilder();
        buffer.begin(VertexFormat.Mode.TRIANGLES, DefaultVertexFormat.POSITION_COLOR);
        for (int i = 0; i < v.length; i += StarTrace.STRIDE) {
            buffer.vertex(pose, v[i], v[i + 1], v[i + 2]).color(v[i + 3], v[i + 4], v[i + 5], v[i + 6]).endVertex();
        }
        BufferUploader.drawWithShader(buffer.end());

        RenderSystem.setShaderColor(previousColour[0], previousColour[1], previousColour[2], previousColour[3]);
        RenderSystem.setShader(() -> previousShader);
    }
}
