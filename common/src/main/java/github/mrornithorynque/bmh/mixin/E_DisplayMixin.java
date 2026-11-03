package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import github.mrornithorynque.bmh.handlers.EchoSenseHandler;

import net.minecraft.world.entity.Display;

/** Echo Sense's glowing displays end themselves here, wherever they were loaded from. */
@Mixin(Display.class)
public abstract class E_DisplayMixin {

    @Inject(method = "tick", at = @At("HEAD"))
    private void bmh$discardExpiredEcho(CallbackInfo ci) {

        EchoSenseHandler.discardIfExpired((Display) (Object) this);
    }
}
