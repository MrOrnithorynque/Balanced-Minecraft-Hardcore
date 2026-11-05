package github.mrornithorynque.bmh.fabric.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import github.mrornithorynque.bmh.handlers.EternalItemHandler;

import net.minecraft.world.entity.item.ItemEntity;
import net.minecraft.world.entity.player.Player;

/** Forge's cancellable EntityItemPickupEvent: a player never picks up a second copy of an Eternal Item. */
@Mixin(ItemEntity.class)
public abstract class ItemEntityMixin {

    @Inject(method = "playerTouch", at = @At("HEAD"), cancellable = true)
    private void bmh$blockSecondEternalItem(Player player, CallbackInfo ci) {

        if (EternalItemHandler.shouldBlockPickup(player, ((ItemEntity) (Object) this).getItem())) {
            ci.cancel();
        }
    }
}
