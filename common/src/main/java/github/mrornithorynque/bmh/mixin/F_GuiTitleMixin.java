package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import github.mrornithorynque.bmh.client.journey.JourneyCinematic;

import net.minecraft.client.gui.Gui;
import net.minecraft.network.chat.Component;

/** Takes the journey signals (handlers/JourneySignal) out of the title packets before vanilla shows them. */
@Mixin(Gui.class)
public abstract class F_GuiTitleMixin {

    @Inject(method = "setTitle", at = @At("HEAD"), cancellable = true)
    private void bmh$journeySignal(Component title, CallbackInfo ci) {

        if (JourneyCinematic.onTitle(title)) {
            ci.cancel();
        }
    }
}
