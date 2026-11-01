package github.mrornithorynque.bmh.fabric.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import github.mrornithorynque.bmh.items.NavigatorAmuletItem;

import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.enchantment.Enchantment;

/**
 * Forge routes Enchantment.canEnchant (what the anvil asks) to the item's canApplyAtEnchantingTable.
 * Without this, a Mending book would go on the Navigator Amulet on Fabric only.
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
