package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import github.mrornithorynque.bmh.client.JourneySmoke;
import net.minecraft.client.Minecraft;

/** THROWAWAY verification hook (never committed): drives JourneySmoke once per client tick. */
@Mixin(Minecraft.class)
public class Z_JourneySmokeClientMixin {

    @Inject(method = "tick", at = @At("TAIL"))
    private void bmh$journeySmokeTick(CallbackInfo ci) {
        JourneySmoke.tick((Minecraft) (Object) this);
    }
}
