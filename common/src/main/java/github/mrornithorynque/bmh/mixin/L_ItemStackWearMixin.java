package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyVariable;

import github.mrornithorynque.bmh.handlers.EternalWear;

import net.minecraft.world.item.ItemStack;

/**
 * An Eternal tool never breaks: its durability stops at the last point (EternalWear). A hit, a mined
 * block or a stripped log goes through the slot method, which calls the player method on vanilla,
 * Fabric and Forge. NeoForge sends the slot method to its own LivingEntity method, so the slot method
 * must cap the damage too. Both methods exist on the three loaders. A second cap changes nothing.
 */
@Mixin(ItemStack.class)
public abstract class L_ItemStackWearMixin {

    @ModifyVariable(method = {
            "hurtAndBreak(ILnet/minecraft/server/level/ServerLevel;Lnet/minecraft/server/level/ServerPlayer;Ljava/util/function/Consumer;)V",
            "hurtAndBreak(ILnet/minecraft/world/entity/LivingEntity;Lnet/minecraft/world/entity/EquipmentSlot;)V"
    }, at = @At("HEAD"), argsOnly = true)
    private int bmh$keepLastPoint(int amount) {

        return EternalWear.capDamage((ItemStack) (Object) this, amount);
    }
}
