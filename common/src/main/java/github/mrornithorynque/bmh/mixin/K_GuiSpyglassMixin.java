package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import github.mrornithorynque.bmh.client.sky.StarReadingRenderer;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.GuiGraphics;

/**
 * Through a spyglass, the name of the amulet's star line the player aims at shows above the centre of the view
 * (client/sky/StarReadingRenderer.aimedName). Vanilla, Forge and NeoForge all draw the spyglass through this method,
 * from 1.20 to 1.21.1, and only while the player looks through it.
 */
@Mixin(Gui.class)
public abstract class K_GuiSpyglassMixin {

    // Amethyst, the colour of the star lines and of the amulet's starlight.
    private static final int NAME_COLOR = 0xB58CFF;

    @Inject(method = "renderSpyglassOverlay", at = @At("TAIL"))
    private void bmh$pointName(GuiGraphics graphics, float scopeScale, CallbackInfo ci) {

        Minecraft minecraft = Minecraft.getInstance();
        String name = StarReadingRenderer.aimedName(minecraft.getTimer().getGameTimeDeltaPartialTick(true));

        if (name != null) {
            graphics.drawCenteredString(minecraft.font, name, graphics.guiWidth() / 2, graphics.guiHeight() / 2 - 24, NAME_COLOR);
        }
    }
}
