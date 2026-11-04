package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import github.mrornithorynque.bmh.client.journey.JourneyCinematic;

import net.minecraft.client.player.LocalPlayer;

/**
 * Holds the respawn request of a journey until the screen is black. The death screen button and an immediate respawn
 * both call LocalPlayer.respawn.
 */
@Mixin(LocalPlayer.class)
public abstract class F_LocalPlayerRespawnMixin {

    @Inject(method = "respawn", at = @At("HEAD"), cancellable = true)
    private void bmh$waitForBlack(CallbackInfo ci) {

        if (JourneyCinematic.onRespawn((LocalPlayer) (Object) this)) {
            ci.cancel();
        }
    }
}
