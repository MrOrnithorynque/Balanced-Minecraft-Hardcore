package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import github.mrornithorynque.bmh.handlers.TotemOfEternityHandler;

import net.minecraft.world.inventory.ShulkerBoxSlot;
import net.minecraft.world.inventory.Slot;
import net.minecraft.world.item.ItemStack;

/**
 * One Totem of Eternity per container, in every menu: chests, barrels, shulker boxes, the ender chest and the player
 * inventory. A click, a shift click, a drag and a number key all ask mayPlace. ShulkerBoxSlot does not call the mayPlace
 * of Slot, so the mixin targets it too.
 */
@Mixin({ Slot.class, ShulkerBoxSlot.class })
public abstract class I_SlotTotemMixin {

    @Inject(method = "mayPlace", at = @At("RETURN"), cancellable = true)
    private void bmh$oneTotemPerContainer(ItemStack stack, CallbackInfoReturnable<Boolean> cir) {

        Slot slot = (Slot) (Object) this;

        if (cir.getReturnValueZ() && TotemOfEternityHandler.holdsAnotherTotem(slot.container, slot.getContainerSlot(), stack)) {
            cir.setReturnValue(false);
        }
    }
}
