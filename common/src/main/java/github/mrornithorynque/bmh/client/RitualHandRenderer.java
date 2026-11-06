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
 * crosshair, and the light (RitualParticles) goes into it. mixin/P_ItemInHandRitualMixin calls it in place of vanilla's hand.
 */
public final class RitualHandRenderer {

    // Where the totem stops, in the camera's space: x to the right, y up, z toward the camera. Its top is just under
    // the crosshair, so the player still aims at the moon over it.
    private static final float HOLD_Y = -0.15F;
    private static final float HOLD_Z = -0.75F;
    // Where it starts: the normal place of an item in the hand (vanilla's applyItemArmTransform, no equip offset).
    private static final float START_X = 0.56F;
    private static final float START_Y = -0.52F;
    private static final float START_Z = -0.72F;
    private static final float RAISE_TICKS = 10.0F;
    private static final float SCALE = 0.26F;

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
        float hover = Mth.sin((player.tickCount + partialTick) * 0.12F) * 0.012F * raise;

        poseStack.pushPose();
        poseStack.translate(Mth.lerp(raise, side * START_X, 0.0F), Mth.lerp(raise, START_Y, HOLD_Y) + hover,
                Mth.lerp(raise, START_Z, HOLD_Z));
        poseStack.scale(SCALE, SCALE, SCALE);
        // Face the camera: the fixed display of a flat item model shows its back to the camera.
        poseStack.mulPose(Axis.YP.rotationDegrees(180.0F));
        renderer.renderItem(player, stack, ItemDisplayContext.FIXED, !rightArm, poseStack, buffers, light);
        poseStack.popPose();
    }
}
