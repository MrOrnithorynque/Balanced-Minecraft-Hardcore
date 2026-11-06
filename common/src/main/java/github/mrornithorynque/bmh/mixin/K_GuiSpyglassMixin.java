package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import github.mrornithorynque.bmh.client.sky.StarReadingRenderer;

import net.minecraft.util.Util;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.GuiGraphicsExtractor;

/**
 * Through a spyglass, the name of the amulet's star line whose last star the player aims at shows under the centre
 * of the view, where the line does not run (client/sky/StarReadingRenderer.aimedName). It fades in over 2 seconds.
 * Since 26.1 vanilla draws the spyglass through Gui.extractSpyglassOverlay, and only while the player looks through it.
 */
@Mixin(Gui.class)
public abstract class K_GuiSpyglassMixin {

    // Amethyst, the colour of the star lines and of the amulet's starlight, without its alpha.
    private static final int NAME_RGB = 0xB58CFF;
    // The name fades in over this time while the view stays on the star.
    private static final long FADE_IN_MILLIS = 2000L;
    // Below this alpha the name is not drawn, as in 1.21.1 (its Font drew an alpha below 4 at full alpha).
    private static final int MIN_ALPHA = 4;

    @Unique
    private static String bmh$aimedName;
    @Unique
    private static long bmh$aimedSince;

    // Gui.extractCameraOverlays calls it in first person while the player scopes, after the vignette.
    @Inject(method = "extractSpyglassOverlay(Lnet/minecraft/client/gui/GuiGraphicsExtractor;F)V", at = @At("TAIL"))
    private void bmh$pointName(GuiGraphicsExtractor graphics, float scopeScale, CallbackInfo ci) {

        Minecraft minecraft = Minecraft.getInstance();
        String name = StarReadingRenderer.aimedName(minecraft.getDeltaTracker().getGameTimeDeltaPartialTick(true));
        long now = Util.getMillis();

        // A new star under the view starts its fade again; leaving the star hides the name at once.
        if (name == null || !name.equals(bmh$aimedName)) {
            bmh$aimedName = name;
            bmh$aimedSince = now;
        }

        if (name == null) {
            return;
        }

        int alpha = (int) (255L * Math.min(now - bmh$aimedSince, FADE_IN_MILLIS) / FADE_IN_MILLIS);

        if (alpha >= MIN_ALPHA) {
            graphics.centeredText(minecraft.font, name, graphics.guiWidth() / 2, graphics.guiHeight() / 2 + 16,
                    alpha << 24 | NAME_RGB);
        }
    }
}
