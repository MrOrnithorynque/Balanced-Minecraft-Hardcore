package github.mrornithorynque.bmh.mixin;

import java.util.function.BooleanSupplier;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import github.mrornithorynque.bmh.selftest.SelfTest;
import net.minecraft.server.MinecraftServer;

/**
 * THROWAWAY verification hook (its copy in common/src is never committed, see selftest.sh): drives {@link SelfTest}
 * once per server tick. It does nothing unless BMH_SELFTEST=1.
 */
@Mixin(MinecraftServer.class)
public class Z_SelfTestMixin {

    @Inject(method = "tickServer", at = @At("HEAD"))
    private void bmh$selfTestTick(BooleanSupplier hasTimeLeft, CallbackInfo ci) {
        SelfTest.tick((MinecraftServer) (Object) this);
    }
}
