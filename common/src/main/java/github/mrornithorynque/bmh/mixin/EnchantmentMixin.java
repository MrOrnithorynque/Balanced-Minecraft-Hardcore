package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import github.mrornithorynque.bmh.items.NavigatorAmuletItem;

import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.enchantment.Enchantment;

/**
 * Enchantment.canEnchant is what the anvil asks (on Fabric through Fabric API's canBeEnchantedWith).
 * Since 1.21 it only reads the enchantment's item tags, which the Navigator Amulet is in none of:
 * without this, the amulet would take no book at all, on both loaders.
 */
@Mixin(Enchantment.class)
public abstract class EnchantmentMixin {

    @Inject(method = "canEnchant", at = @At("HEAD"), cancellable = true)
    private void bmh$amuletEnchantments(ItemStack stack, CallbackInfoReturnable<Boolean> cir) {

        if (stack.getItem() instanceof NavigatorAmuletItem amulet) {
            cir.setReturnValue(amulet.canApplyAtEnchantingTable(stack, (Enchantment) (Object) this));
        }
    }
}
