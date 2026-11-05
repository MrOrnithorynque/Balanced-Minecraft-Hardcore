package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import net.minecraft.world.damagesource.DamageSource;
import net.minecraft.world.entity.item.ItemEntity;

import github.mrornithorynque.bmh.items.IEternalItem;

/**
 * An Eternal Item on the ground lasts: explosions, cactus and other damage do not destroy it, and it never despawns.
 * The void still removes it, and so does /kill.
 *
 * No loader has an event for item entity damage, and Fabric has none for the despawn: one mixin covers the three
 * loaders. The vanilla names are the same from 1.20 to 1.21.1.
 */
@Mixin(ItemEntity.class)
public abstract class G_ItemEntityMixin {

    @Inject(method = "hurt", at = @At("HEAD"), cancellable = true)
    private void bmh$eternalItemIgnoresDamage(DamageSource source, float amount, CallbackInfoReturnable<Boolean> cir) {

        if (((ItemEntity) (Object) this).getItem().getItem() instanceof IEternalItem) {
            cir.setReturnValue(false);
        }
    }

    // Vanilla does not count the age of an item with an unlimited lifetime. The check runs every tick, because an
    // item entity can change its stack, and an item loaded from a save keeps its old age.
    @Inject(method = "tick", at = @At("HEAD"))
    private void bmh$eternalItemNeverDespawns(CallbackInfo ci) {

        ItemEntity itemEntity = (ItemEntity) (Object) this;

        if (itemEntity.getItem().getItem() instanceof IEternalItem) {
            itemEntity.setUnlimitedLifetime();
        }
    }
}
