package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyVariable;

import github.mrornithorynque.bmh.handlers.EternalWear;

import net.minecraft.world.item.ItemStack;

/**
 * An Eternal tool never breaks: its durability stops at the last point (EternalWear). On 1.20.x every
 * durability loss goes through {@code ItemStack.hurt}, which returns true when the item must break.
 * Forge's {@code hurtAndBreak} calls the item's {@code damageItem} first, then this method, so one
 * common mixin covers the three loaders.
 */
@Mixin(ItemStack.class)
public abstract class L_ItemStackWearMixin {

    @ModifyVariable(method = "hurt(ILnet/minecraft/util/RandomSource;Lnet/minecraft/server/level/ServerPlayer;)Z", at = @At("HEAD"), argsOnly = true)
    private int bmh$keepLastPoint(int amount) {

        return EternalWear.capDamage((ItemStack) (Object) this, amount);
    }
}
