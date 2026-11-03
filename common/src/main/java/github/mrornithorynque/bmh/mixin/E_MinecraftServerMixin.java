package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import github.mrornithorynque.bmh.handlers.EchoSenseHandler;

import net.minecraft.server.MinecraftServer;

/** Once the levels exist the data packs are loaded: the place to check that Echo Sense's tag is there. */
@Mixin(MinecraftServer.class)
public abstract class E_MinecraftServerMixin {

    @Inject(method = "loadLevel", at = @At("TAIL"))
    private void bmh$checkEchoSenseTag(CallbackInfo ci) {

        EchoSenseHandler.logTagSize();
    }
}
