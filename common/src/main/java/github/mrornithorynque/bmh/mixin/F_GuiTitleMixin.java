package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import github.mrornithorynque.bmh.client.amulet.PointSignal;
import github.mrornithorynque.bmh.client.journey.JourneyCinematic;

import net.minecraft.client.gui.Gui;
import net.minecraft.network.chat.Component;

/**
 * Takes BMH's signals out of the title packets before vanilla shows them: the journey (handlers/JourneySignal)
 * and the naming screen of an amulet point (handlers/AmuletPointSignal).
 */
@Mixin(Gui.class)
public abstract class F_GuiTitleMixin {

    @Inject(method = "setTitle", at = @At("HEAD"), cancellable = true)
    private void bmh$journeySignal(Component title, CallbackInfo ci) {

        if (JourneyCinematic.onTitle(title) || PointSignal.onTitle(title)) {
            ci.cancel();
        }
    }
}
