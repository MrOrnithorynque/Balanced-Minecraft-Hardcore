package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import github.mrornithorynque.bmh.client.journey.JourneyCinematic;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.screens.Overlay;

/** Sends a held respawn request when another overlay (a resource reload) takes the place of the journey. */
@Mixin(Minecraft.class)
public abstract class F_MinecraftOverlayMixin {

    @Inject(method = "setOverlay", at = @At("HEAD"))
    private void bmh$journeyReplaced(Overlay overlay, CallbackInfo ci) {

        JourneyCinematic.onOverlayChange(overlay);
    }
}
