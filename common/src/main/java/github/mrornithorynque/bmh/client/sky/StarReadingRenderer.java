package github.mrornithorynque.bmh.client.sky;

import java.lang.ref.WeakReference;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;

import javax.annotation.Nullable;

import org.joml.Matrix4f;
import org.joml.Vector3f;
import org.slf4j.Logger;

import com.mojang.blaze3d.platform.GlStateManager;
import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.vertex.BufferBuilder;
import com.mojang.blaze3d.vertex.BufferUploader;
import com.mojang.blaze3d.vertex.DefaultVertexFormat;
import com.mojang.blaze3d.vertex.MeshData;
import com.mojang.blaze3d.vertex.Tesselator;
import com.mojang.blaze3d.vertex.VertexFormat;
import com.mojang.logging.LogUtils;

import github.mrornithorynque.bmh.items.NavigatorAmuletItem;
import net.minecraft.client.Minecraft;
import net.minecraft.client.multiplayer.ClientLevel;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.client.renderer.GameRenderer;
import net.minecraft.client.renderer.ShaderInstance;
import net.minecraft.core.component.DataComponents;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.ListTag;
import net.minecraft.nbt.Tag;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.component.CustomData;

/**
 * Draws a Navigator's Amulet star reading in the sky, for this client's player only. D_LevelRendererSkyMixin calls it
 * right after vanilla draws the stars, with the stars' own pose, so the chain stays pinned to them as the sky turns.
 * Render thread only.
 */
public final class StarReadingRenderer {

    private static final Logger LOGGER = LogUtils.getLogger();

    // The amulet's reading (written server side by the star reading, in the stack's custom data): one azimuth per
    // point, in degrees clockwise from north, from the player to the point, and the level's game time of the reading.
    private static final String READING = "bmh_reading";
    private static final String AZIMUTHS = "azimuths";
    private static final String NAMES = "names";
    /** Through a spyglass, a star of a line within this angle of the view's centre shows the line's name. */
    private static final double AIM_DEGREES = 2.0;
    private static final String TIME = "time";
    /** An amulet whose reading is this old when the client first sees it (picked up, relogged) shows nothing. */
    private static final long FRESH_TICKS = 40;
    /** Vanilla's star brightness at full night: a reading there is at full strength. */
    private static final float FULL_NIGHT_STARS = 0.5F;

    private static long lastShownTime = Long.MIN_VALUE;
    private static WeakReference<ClientLevel> lastLevel = new WeakReference<>(null);
    private static Shown shown;

    static {
        // The azimuth convention shared with the amulet (brief, part C): Home at (0, -100) seen from (0, 0) is 0,
        // Home at (100, 0) is 90. This is the direction the picker actually aims at.
        double[] north = StarChain.toward(0.0), east = StarChain.toward(90.0);
        LOGGER.info(String.format(Locale.ROOT,
                "[BalancedMcHardcoreMain] Sky azimuth self-check: from (0, 0), azimuth 0 aims at (%d, %d), azimuth 90 at (%d, %d); expected (0, -100) and (100, 0)",
                Math.round(north[0] * 100), Math.round(north[1] * 100), Math.round(east[0] * 100), Math.round(east[1] * 100)));
    }

    // One chain of stars per point of the reading, all started at the same tick, and the point's name for each.
    private record Shown(List<int[]> chains, List<String> names, double startTick) {
    }

    private StarReadingRenderer() {
    }

    public static void render(ClientLevel level, Matrix4f starPose, float partialTick) {
        Minecraft minecraft = Minecraft.getInstance();
        LocalPlayer player = minecraft.player;
        if (player == null) {
            return;
        }
        if (lastLevel.get() != level) {
            // Another world or dimension: another game time, and a chain belongs to the sky it was read in. Weak, so
            // a world left behind is not kept in memory.
            lastLevel = new WeakReference<>(level);
            lastShownTime = Long.MIN_VALUE;
            shown = null;
        }
        // The animation runs on client ticks: the client's game time snaps back at each server time sync when the
        // server lags, which would retract the lines.
        double now = minecraft.gui.getGuiTicks() + partialTick;
        startFreshReading(level, player, now, partialTick);

        Shown reading = shown;
        if (reading == null) {
            return;
        }
        double seconds = (now - reading.startTick()) / 20.0;
        float skyAlpha = Math.min(1.0F,
                level.getStarBrightness(partialTick) * (1.0F - level.getRainLevel(partialTick)) / FULL_NIGHT_STARS);
        List<float[]> parts = new ArrayList<>();
        int length = 0;
        for (int[] chain : reading.chains()) {
            if (seconds <= StarTrace.duration(chain.length)) {
                float[] part = StarTrace.triangles(StarField.stars(), chain, seconds, skyAlpha);
                parts.add(part);
                length += part.length;
            }
        }
        if (parts.isEmpty()) {
            shown = null;
            return;
        }
        // One draw call for every chain.
        float[] triangles = new float[length];
        int at = 0;
        for (float[] part : parts) {
            System.arraycopy(part, 0, triangles, at, part.length);
            at += part.length;
        }
        if (triangles.length > 0) {
            draw(starPose, triangles);
        }
    }

    // A reading starts the moment the player holds an amulet carrying one newer than the last shown (with an amulet in
    // each hand, the newest wins instead of the two taking turns). Once started it plays to the end, amulet in hand or
    // not.
    private static void startFreshReading(ClientLevel level, LocalPlayer player, double now, float partialTick) {
        for (InteractionHand hand : InteractionHand.values()) {
            ItemStack stack = player.getItemInHand(hand);
            CustomData data = stack.get(DataComponents.CUSTOM_DATA);
            if (!(stack.getItem() instanceof NavigatorAmuletItem) || data == null || !data.contains(READING)) {
                continue;
            }
            CompoundTag reading = data.copyTag().getCompound(READING);
            long time = reading.getLong(TIME);
            if (time <= lastShownTime || Math.abs(level.getGameTime() - time) >= FRESH_TICKS) {
                continue;
            }
            lastShownTime = time;
            ListTag azimuths = reading.getList(AZIMUTHS, Tag.TAG_FLOAT);
            ListTag names = reading.getList(NAMES, Tag.TAG_STRING);
            List<int[]> chains = new ArrayList<>();
            List<String> chainNames = new ArrayList<>();
            for (int i = 0; i < azimuths.size(); i++) {
                int[] chain = pick(level, azimuths.getFloat(i), partialTick);
                if (chain.length > 0) {
                    chains.add(chain);
                    chainNames.add(i < names.size() ? names.getString(i) : "");
                }
            }
            shown = chains.isEmpty() ? null : new Shown(chains, chainNames, now);
            return;
        }
    }

    /**
     * The name of the line the view's centre aims at, or null: a star of a named line still in the sky within AIM_DEGREES
     * of the camera's direction (the nearest wins where lines meet). mixin/K_GuiSpyglassMixin shows it through a spyglass.
     */
    @Nullable
    public static String aimedName(float partialTick) {
        Minecraft minecraft = Minecraft.getInstance();
        Shown reading = shown;
        if (reading == null || minecraft.level == null) {
            return null;
        }
        double seconds = (minecraft.gui.getGuiTicks() + partialTick - reading.startTick()) / 20.0;
        float timeOfDay = minecraft.level.getTimeOfDay(partialTick);
        Vector3f look = minecraft.gameRenderer.getMainCamera().getLookVector();
        double best = Math.cos(Math.toRadians(AIM_DEGREES));
        String name = null;
        for (int c = 0; c < reading.chains().size(); c++) {
            int[] chain = reading.chains().get(c);
            if (reading.names().get(c).isEmpty() || seconds > StarTrace.duration(chain.length)) {
                continue;
            }
            for (int index : chain) {
                StarField.Star star = StarField.stars().get(index);
                double[] sky = StarChain.toWorld(star.x, star.y, star.z, timeOfDay);
                double dot = sky[0] * look.x() + sky[1] * look.y() + sky[2] * look.z();
                if (dot > best) {
                    best = dot;
                    name = reading.names().get(c);
                }
            }
        }
        return name;
    }

    private static int[] pick(ClientLevel level, float azimuth, float partialTick) {
        // The sky as it stands now; within the 40 fresh ticks it turns less than a degree.
        float timeOfDay = level.getTimeOfDay(partialTick);
        int[] chain = StarChain.pick(StarField.stars(), azimuth, timeOfDay);
        if (chain.length > 0) {
            StarField.Star star = StarField.stars().get(chain[chain.length - 1]);
            double[] last = StarChain.toWorld(star.x, star.y, star.z, timeOfDay);
            double lastAzimuth = (Math.toDegrees(Math.atan2(last[0], -last[2])) + 360.0) % 360.0;
            LOGGER.info(String.format(Locale.ROOT,
                    "[BalancedMcHardcoreMain] Star reading toward %.1f: %d stars, the last at azimuth %.1f, altitude %.1f",
                    azimuth, chain.length, lastAzimuth, Math.toDegrees(Math.asin(last[1]))));
        }
        return chain;
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
        BufferBuilder buffer = Tesselator.getInstance().begin(VertexFormat.Mode.TRIANGLES, DefaultVertexFormat.POSITION_COLOR);
        for (int i = 0; i < v.length; i += StarTrace.STRIDE) {
            buffer.addVertex(pose, v[i], v[i + 1], v[i + 2]).setColor(v[i + 3], v[i + 4], v[i + 5], v[i + 6]);
        }
        MeshData mesh = buffer.build();
        if (mesh != null) {
            BufferUploader.drawWithShader(mesh);
        }

        RenderSystem.setShaderColor(previousColour[0], previousColour[1], previousColour[2], previousColour[3]);
        RenderSystem.setShader(() -> previousShader);
    }
}
