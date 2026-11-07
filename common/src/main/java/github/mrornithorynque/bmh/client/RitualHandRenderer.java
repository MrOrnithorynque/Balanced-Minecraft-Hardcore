package github.mrornithorynque.bmh.client;

import com.mojang.blaze3d.vertex.PoseStack;
import com.mojang.math.Axis;

import github.mrornithorynque.bmh.handlers.MoonRitualHandler;

import net.minecraft.client.renderer.ItemInHandRenderer;
import net.minecraft.client.renderer.MultiBufferSource;
import net.minecraft.util.Mth;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.item.ItemDisplayContext;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.phys.Vec3;

/**
 * The first person view of the moon ritual: the player raises the Dormant Totem in front of the eyes, just under the
 * crosshair, and the light (RitualParticles) goes into it. The totem trembles more and more as the charge grows. mixin/P_ItemInHandRitualMixin calls it in place of vanilla's hand.
 */
public final class RitualHandRenderer {

    // Where the totem stops, in the camera's space: x to the right, y up, z toward the camera. Its top is just under
    // the crosshair, so the player still aims at the moon over it.
    private static final float HOLD_Y = -0.1F;
    private static final float HOLD_Z = -0.75F;
    // Where it starts: the normal place of an item in the hand (vanilla's applyItemArmTransform, no equip offset).
    private static final float START_X = 0.56F;
    private static final float START_Y = -0.52F;
    private static final float START_Z = -0.72F;
    private static final float RAISE_TICKS = 10.0F;
    private static final float SCALE = 0.16F;
    // The night light leaves the totem almost black: it catches the moonlight, so it is drawn at full brightness.
    private static final int FULL_BRIGHT = 0xF000F0;
    // The tremble at full charge. It grows with the square of the charge: faint at first, strong at the end.
    private static final float MAX_SHAKE = 0.012F;
    private static final float MAX_SHAKE_DEGREES = 7.0F;

    private RitualHandRenderer() {
    }

    /** Where the raised totem is in the world, for the particles: the same place as the drawing, without view bobbing. */
    public static Vec3 totemPosition(Player player) {

        Vec3 look = player.getViewVector(1.0F);
        Vec3 right = look.cross(new Vec3(0.0, 1.0, 0.0));
        // Straight up or down, the cross product is zero: any right vector does.
        right = right.lengthSqr() < 1.0E-6 ? new Vec3(1.0, 0.0, 0.0) : right.normalize();
        Vec3 up = right.cross(look);

        return player.getEyePosition().add(look.scale(-HOLD_Z)).add(up.scale(HOLD_Y));
    }

    public static void render(ItemInHandRenderer renderer, Player player, ItemStack stack, boolean rightArm, float partialTick,
            PoseStack poseStack, MultiBufferSource buffers, int light) {

        float elapsed = MoonRitualHandler.RITUAL_TICKS - player.getUseItemRemainingTicks() + partialTick;
        float raise = Mth.clamp(elapsed / RAISE_TICKS, 0.0F, 1.0F);
        // Smooth start and smooth stop.
        raise = raise * raise * (3.0F - 2.0F * raise);
        float side = rightArm ? 1.0F : -1.0F;
        float time = player.tickCount + partialTick;
        float hover = Mth.sin(time * 0.12F) * 0.012F * raise;
        float charge = Mth.clamp(elapsed / MoonRitualHandler.RITUAL_TICKS, 0.0F, 1.0F);
        float shake = charge * charge;
        // Two fast waves of different speeds on each axis: an uneven tremble, not a regular swing.
        float shakeX = (Mth.sin(time * 2.9F) + Mth.sin(time * 4.3F + 1.0F)) * 0.5F * shake;
        float shakeY = (Mth.sin(time * 3.7F + 2.0F) + Mth.sin(time * 5.1F)) * 0.5F * shake;

        poseStack.pushPose();
        poseStack.translate(Mth.lerp(raise, side * START_X, 0.0F) + shakeX * MAX_SHAKE,
                Mth.lerp(raise, START_Y, HOLD_Y) + hover + shakeY * MAX_SHAKE, Mth.lerp(raise, START_Z, HOLD_Z));
        poseStack.mulPose(Axis.ZP.rotationDegrees(shakeX * MAX_SHAKE_DEGREES));
        poseStack.scale(SCALE, SCALE, SCALE);
        // Face the camera: the fixed display of a flat item model shows its back to the camera.
        poseStack.mulPose(Axis.YP.rotationDegrees(180.0F));
        renderer.renderItem(player, stack, ItemDisplayContext.FIXED, !rightArm, poseStack, buffers, FULL_BRIGHT);
        poseStack.popPose();
    }
}
