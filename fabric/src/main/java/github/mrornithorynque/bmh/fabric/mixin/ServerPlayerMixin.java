package github.mrornithorynque.bmh.fabric.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import github.mrornithorynque.bmh.handlers.KeepItemAfterDeath;
import github.mrornithorynque.bmh.handlers.RespawnPositionHandler;

import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.damagesource.DamageSource;

/**
 * Where Forge fires LivingDeathEvent for a player: after a totem had its chance, before the
 * inventory drops. Fabric API's ALLOW_DEATH comes before the totem and AFTER_DEATH after the drops.
 */
@Mixin(ServerPlayer.class)
public abstract class ServerPlayerMixin {

    @Inject(method = "die", at = @At("HEAD"))
    private void bmh$onDeath(DamageSource damageSource, CallbackInfo ci) {

        ServerPlayer self = (ServerPlayer) (Object) this;

        // Distance first: it counts the Eternal Items that the next call takes out of the inventory.
        RespawnPositionHandler.onPlayerDeath(self);
        KeepItemAfterDeath.onPlayerDeath(self);
    }
}
