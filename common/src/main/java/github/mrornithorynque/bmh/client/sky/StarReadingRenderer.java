package github.mrornithorynque.bmh.client.sky;

import java.lang.ref.WeakReference;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.OptionalDouble;
import java.util.OptionalInt;

import org.jspecify.annotations.Nullable;

import org.joml.Matrix4f;
import org.joml.Matrix4fStack;
import org.joml.Vector3f;
import org.joml.Vector3fc;
import org.joml.Vector4f;
import org.slf4j.Logger;

import com.mojang.blaze3d.buffers.GpuBuffer;
import com.mojang.blaze3d.buffers.GpuBufferSlice;
import com.mojang.blaze3d.pipeline.BlendFunction;
import com.mojang.blaze3d.pipeline.ColorTargetState;
import com.mojang.blaze3d.pipeline.RenderPipeline;
import com.mojang.blaze3d.shaders.UniformType;
import com.mojang.blaze3d.systems.RenderPass;
import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.textures.GpuTextureView;
import com.mojang.blaze3d.vertex.BufferBuilder;
import com.mojang.blaze3d.vertex.ByteBufferBuilder;
import com.mojang.blaze3d.vertex.DefaultVertexFormat;
import com.mojang.blaze3d.vertex.MeshData;
import com.mojang.blaze3d.vertex.PoseStack;
import com.mojang.blaze3d.vertex.VertexFormat;
import com.mojang.logging.LogUtils;

import github.mrornithorynque.bmh.items.NavigatorAmuletItem;
import net.minecraft.client.Minecraft;
import net.minecraft.client.multiplayer.ClientLevel;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.core.component.DataComponents;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.ListTag;
import net.minecraft.resources.Identifier;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.attribute.EnvironmentAttributes;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.component.CustomData;

/**
 * Draws a Navigator's Amulet star reading in the sky, for this client's player only. D_LevelRendererSkyMixin calls it
 * right after vanilla draws the stars (SkyRenderer.renderStars), with the stars' own pose, so the chain stays pinned to
 * them as the sky turns. Render thread only.
 */
public final class StarReadingRenderer {

    private static final Logger LOGGER = LogUtils.getLogger();

    // The amulet's reading (written server side by the star reading, in the stack's custom data): one azimuth per
    // point, in degrees clockwise from north, from the player to the point, and the level's game time of the reading.
    private static final String READING = "bmh_reading";
    private static final String AZIMUTHS = "azimuths";
    private static final String NAMES = "names";
    // The index of the world spawn in both lists: its line is drawn lighter. Missing in a reading from 1.7.0 or older.
    private static final String SPAWN = "spawn";
    /**
     * Through a spyglass, the last star of a line within this angle of the view's centre shows the line's name. The
     * spyglass shows about 7 degrees of sky: 0.8 keeps the star close to the cross.
     */
    private static final double AIM_DEGREES = 0.8;
    private static final String TIME = "time";
    /** An amulet whose reading is this old when the client first sees it (picked up, relogged) shows nothing. */
    private static final long FRESH_TICKS = 40;
    /** Vanilla's star brightness at full night (the overworld day timeline): a reading there is at full strength. */
    private static final float FULL_NIGHT_STARS = 0.5F;

    /**
     * Vanilla's star pipeline (RenderPipelines.STARS, 26.1.2) with a colour per vertex: the same uniforms, the same
     * blend (SRC_ALPHA, ONE: light adds up), no depth test, back faces culled. The shader is vanilla's position_color.
     */
    private static final RenderPipeline PIPELINE = RenderPipeline.builder()
            .withLocation(Identifier.fromNamespaceAndPath("bmh", "pipeline/star_reading"))
            .withUniform("DynamicTransforms", UniformType.UNIFORM_BUFFER)
            .withUniform("Projection", UniformType.UNIFORM_BUFFER)
            .withVertexShader("core/position_color")
            .withFragmentShader("core/position_color")
            .withColorTargetState(new ColorTargetState(BlendFunction.OVERLAY))
            .withVertexFormat(DefaultVertexFormat.POSITION_COLOR, VertexFormat.Mode.TRIANGLES)
            .build();

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
    // spawnChain is the index in chains of the line toward the world spawn, or -1.
    private record Shown(List<int[]> chains, List<String> names, int spawnChain, double startTick) {
    }

    private StarReadingRenderer() {
    }

    /**
     * D_LevelRendererSkyMixin calls it at the end of SkyRenderer.renderStars. {@code poseStack} holds the stars' pose
     * and {@code starBrightness} is vanilla's star brightness for this frame, rain included.
     */
    public static void render(PoseStack poseStack, float starBrightness) {
        Minecraft minecraft = Minecraft.getInstance();
        LocalPlayer player = minecraft.player;
        ClientLevel level = minecraft.level;
        if (player == null || level == null) {
            return;
        }
        // The partial tick of the level render (LevelRenderer.renderLevel reads the same one for the sky).
        float partialTick = minecraft.getDeltaTracker().getGameTimeDeltaPartialTick(false);
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
        // Since 26.1 the star brightness attribute already goes to 0 with the rain, so the rain is not applied again.
        float skyAlpha = Math.min(1.0F, starBrightness / FULL_NIGHT_STARS);
        List<float[]> parts = new ArrayList<>();
        int length = 0;
        for (int c = 0; c < reading.chains().size(); c++) {
            int[] chain = reading.chains().get(c);
            if (seconds <= StarTrace.duration(chain.length)) {
                float[] part = StarTrace.triangles(StarField.stars(), chain, seconds, skyAlpha, c == reading.spawnChain());
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
            draw(poseStack.last().pose(), triangles);
        }
    }

    // A reading starts the moment the player holds an amulet carrying one newer than the last shown (with an amulet in
    // each hand, the newest wins instead of the two taking turns). Once started it plays to the end, amulet in hand or
    // not.
    private static void startFreshReading(ClientLevel level, LocalPlayer player, double now, float partialTick) {
        for (InteractionHand hand : InteractionHand.values()) {
            ItemStack stack = player.getItemInHand(hand);
            CustomData data = stack.get(DataComponents.CUSTOM_DATA);
            if (!(stack.getItem() instanceof NavigatorAmuletItem) || data == null) {
                continue;
            }
            CompoundTag tag = data.copyTag();
            if (!tag.contains(READING)) {
                continue;
            }
            CompoundTag reading = tag.getCompoundOrEmpty(READING);
            long time = reading.getLongOr(TIME, 0L);
            if (time <= lastShownTime || Math.abs(level.getGameTime() - time) >= FRESH_TICKS) {
                continue;
            }
            lastShownTime = time;
            ListTag azimuths = reading.getListOrEmpty(AZIMUTHS);
            ListTag names = reading.getListOrEmpty(NAMES);
            List<int[]> chains = new ArrayList<>();
            List<String> chainNames = new ArrayList<>();
            // A reading from 1.7.0 or older has no world spawn line.
            int spawn = reading.getInt(SPAWN).orElse(-1);
            int spawnChain = -1;
            for (int i = 0; i < azimuths.size(); i++) {
                int[] chain = pick(azimuths.getFloatOr(i, 0.0F), partialTick);
                if (chain.length > 0) {
                    if (i == spawn) {
                        spawnChain = chains.size();
                    }
                    chains.add(chain);
                    chainNames.add(i < names.size() ? names.getStringOr(i, "") : "");
                }
            }
            shown = chains.isEmpty() ? null : new Shown(chains, chainNames, spawnChain, now);
            return;
        }
    }

    /**
     * The name of the line the view's centre aims at, or null: the last star of a named line still in the sky (the lowest
     * one, toward the point) within AIM_DEGREES of the camera's direction, the nearest first. mixin/K_GuiSpyglassMixin
     * shows it through a spyglass.
     */
    @Nullable
    public static String aimedName(float partialTick) {
        Minecraft minecraft = Minecraft.getInstance();
        Shown reading = shown;
        if (reading == null || minecraft.level == null) {
            return null;
        }
        double seconds = (minecraft.gui.getGuiTicks() + partialTick - reading.startTick()) / 20.0;
        double timeOfDay = timeOfDay(partialTick);
        Vector3fc look = minecraft.gameRenderer.getMainCamera().forwardVector();
        double best = Math.cos(Math.toRadians(AIM_DEGREES));
        String name = null;
        for (int c = 0; c < reading.chains().size(); c++) {
            int[] chain = reading.chains().get(c);
            if (reading.names().get(c).isEmpty() || seconds > StarTrace.duration(chain.length)) {
                continue;
            }
            StarField.Star star = StarField.stars().get(chain[chain.length - 1]);
            double[] sky = StarChain.toWorld(star.x, star.y, star.z, timeOfDay);
            double dot = sky[0] * look.x() + sky[1] * look.y() + sky[2] * look.z();
            if (dot > best) {
                best = dot;
                name = reading.names().get(c);
            }
        }
        return name;
    }

    /** The sky direction (a unit vector, y up) of the last star of a shown line, or null. */
    @Nullable
    public static double[] lastStar(int line, float partialTick) {
        Minecraft minecraft = Minecraft.getInstance();
        Shown reading = shown;
        if (reading == null || minecraft.level == null || line >= reading.chains().size()) {
            return null;
        }
        int[] chain = reading.chains().get(line);
        StarField.Star star = StarField.stars().get(chain[chain.length - 1]);
        return StarChain.toWorld(star.x, star.y, star.z, timeOfDay(partialTick));
    }

    /**
     * The stars' turn as a part of a full turn (0 to 1), the time of day that StarChain takes. SkyRenderer turns the
     * stars by the star angle attribute (degrees), read through the camera, the way SkyRenderer.extractRenderState
     * reads it. In the overworld it is the old timeOfDay times 360.
     */
    private static double timeOfDay(float partialTick) {
        float degrees = Minecraft.getInstance().gameRenderer.getMainCamera().attributeProbe()
                .getValue(EnvironmentAttributes.STAR_ANGLE, partialTick);
        return degrees / 360.0;
    }

    private static int[] pick(float azimuth, float partialTick) {
        // The sky as it stands now; within the 40 fresh ticks it turns less than a degree.
        double timeOfDay = timeOfDay(partialTick);
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

    // Same path as vanilla's stars (SkyRenderer.renderStars, 26.1.2): the pose goes on the model view stack, the
    // vertices go to the immediate vertex buffer of their format, and one render pass draws them on the main target.
    private static void draw(Matrix4f pose, float[] v) {
        int vertices = v.length / StarTrace.STRIDE;
        GpuBuffer vertexBuffer;
        int vertexCount;
        try (ByteBufferBuilder bytes = ByteBufferBuilder.exactlySized(vertices * DefaultVertexFormat.POSITION_COLOR.getVertexSize())) {
            BufferBuilder buffer = new BufferBuilder(bytes, VertexFormat.Mode.TRIANGLES, DefaultVertexFormat.POSITION_COLOR);
            for (int i = 0; i < v.length; i += StarTrace.STRIDE) {
                buffer.addVertex(v[i], v[i + 1], v[i + 2]).setColor(v[i + 3], v[i + 4], v[i + 5], v[i + 6]);
            }
            try (MeshData mesh = buffer.buildOrThrow()) {
                vertexBuffer = PIPELINE.getVertexFormat().uploadImmediateVertexBuffer(mesh.vertexBuffer());
                vertexCount = mesh.drawState().vertexCount();
            }
        }

        Matrix4fStack modelViewStack = RenderSystem.getModelViewStack();
        modelViewStack.pushMatrix();
        modelViewStack.mul(pose);
        GpuBufferSlice dynamicTransforms = RenderSystem.getDynamicUniforms()
                .writeTransform(modelViewStack, new Vector4f(1.0F, 1.0F, 1.0F, 1.0F), new Vector3f(), new Matrix4f());
        GpuTextureView colorTexture = Minecraft.getInstance().getMainRenderTarget().getColorTextureView();
        GpuTextureView depthTexture = Minecraft.getInstance().getMainRenderTarget().getDepthTextureView();

        try (RenderPass renderPass = RenderSystem.getDevice()
                .createCommandEncoder()
                .createRenderPass(() -> "BMH star reading", colorTexture, OptionalInt.empty(), depthTexture, OptionalDouble.empty())) {
            renderPass.setPipeline(PIPELINE);
            RenderSystem.bindDefaultUniforms(renderPass);
            renderPass.setUniform("DynamicTransforms", dynamicTransforms);
            renderPass.setVertexBuffer(0, vertexBuffer);
            renderPass.draw(0, vertexCount);
        }

        modelViewStack.popMatrix();
    }
}
