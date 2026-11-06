package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import com.mojang.blaze3d.vertex.PoseStack;

import github.mrornithorynque.bmh.client.RitualHandRenderer;
import github.mrornithorynque.bmh.items.DormantTotemOfEternityItem;

import net.minecraft.client.player.AbstractClientPlayer;
import net.minecraft.client.renderer.ItemInHandRenderer;
import net.minecraft.client.renderer.SubmitNodeCollector;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.entity.HumanoidArm;
import net.minecraft.world.item.ItemStack;

/**
 * In first person, the moon ritual draws the Dormant Totem raised in front of the eyes (client/RitualHandRenderer) in
 * place of vanilla's hand pose for its use animation. Vanilla, Forge and NeoForge all draw each hand through this method.
 */
@Mixin(ItemInHandRenderer.class)
public abstract class P_ItemInHandRitualMixin {

    @Inject(method = "renderArmWithItem", at = @At("HEAD"), cancellable = true)
    private void bmh$ritualHand(AbstractClientPlayer player, float partialTick, float pitch, InteractionHand hand, float swing,
            ItemStack stack, float equip, PoseStack poseStack, SubmitNodeCollector buffers, int light, CallbackInfo ci) {

        if (!player.isUsingItem() || player.getUsedItemHand() != hand || !(stack.getItem() instanceof DormantTotemOfEternityItem)) {
            return;
        }

        HumanoidArm arm = hand == InteractionHand.MAIN_HAND ? player.getMainArm() : player.getMainArm().getOpposite();

        RitualHandRenderer.render((ItemInHandRenderer) (Object) this, player, stack, arm == HumanoidArm.RIGHT, partialTick,
                poseStack, buffers, light);
        ci.cancel();
    }
}
