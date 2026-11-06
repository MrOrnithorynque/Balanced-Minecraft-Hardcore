package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import github.mrornithorynque.bmh.handlers.RespawnEffectHandler;

import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.tags.DamageTypeTags;
import net.minecraft.world.damagesource.DamageSource;

/**
 * The damage immunity after a distant respawn. Up to 1.21.1 vanilla had a spawn immunity counter on the player, and BMH
 * set it to 200 ticks. 26.1 removed it: vanilla now protects a player only until the client has loaded, or 60 ticks.
 * This counter does what the old one did: it blocks all damage except the damage that bypasses invulnerability.
 */
@Mixin(ServerPlayer.class)
public abstract class F_ServerPlayerJourneyMixin implements RespawnEffectHandler.JourneyShield {

    @Unique
    private int bmh$journeyTicks;

    @Override
    public void bmh$setJourneyInvulnerableTicks(int ticks) {

        this.bmh$journeyTicks = ticks;
    }

    @Inject(method = "tick", at = @At("HEAD"))
    private void bmh$countDown(CallbackInfo ci) {

        if (this.bmh$journeyTicks > 0) {
            this.bmh$journeyTicks--;
        }
    }

    @Inject(method = "hurtServer", at = @At("HEAD"), cancellable = true)
    private void bmh$blockDamage(ServerLevel level, DamageSource source, float damage, CallbackInfoReturnable<Boolean> cir) {

        if (this.bmh$journeyTicks > 0 && !source.is(DamageTypeTags.BYPASSES_INVULNERABILITY)) {
            cir.setReturnValue(false);
        }
    }
}
