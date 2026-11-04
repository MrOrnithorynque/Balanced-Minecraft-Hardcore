package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import github.mrornithorynque.bmh.client.SmokeClient;
import net.minecraft.client.Minecraft;

/** THROWAWAY verification hook, never committed: drives {@link SmokeClient} once per client tick. */
@Mixin(Minecraft.class)
public class Z_SmokeClientMixin {

    @Inject(method = "tick", at = @At("TAIL"))
    private void bmh$smokeTick(CallbackInfo ci) {
        SmokeClient.tick((Minecraft) (Object) this);
    }
}
