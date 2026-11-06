package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import github.mrornithorynque.bmh.handlers.MoonRitualHandler;
import github.mrornithorynque.bmh.items.DormantTotemOfEternityItem;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.player.LocalPlayer;

/**
 * The bar of the moon ritual: a thin bar above the hotbar that fills while the player holds use with a Dormant
 * Totem. It sits under the action bar text and over the name of the held item, clear of the hearts and the armor.
 * Vanilla, Forge and NeoForge all draw the hotbar through this method, from 1.20 to 1.20.4.
 */
@Mixin(Gui.class)
public abstract class O_GuiRitualBarMixin {

    private static final int WIDTH = 182;
    private static final int HEIGHT = 3;
    // From the bottom of the screen. The action bar text ends at 63 and the held item name starts at 59.
    private static final int FROM_BOTTOM = 62;
    private static final int BACK_COLOR = 0x90000000;
    // Pale silver blue, the colour of moonlight.
    private static final int FILL_COLOR = 0xFFD6E2FF;

    @Inject(method = "renderHotbar", at = @At("TAIL"))
    private void bmh$ritualBar(float partialTick, GuiGraphics graphics, CallbackInfo ci) {

        LocalPlayer player = Minecraft.getInstance().player;

        if (player == null || !player.isUsingItem() || !(player.getUseItem().getItem() instanceof DormantTotemOfEternityItem)) {
            return;
        }

        float done = 1.0F - (player.getUseItemRemainingTicks() - partialTick) / MoonRitualHandler.RITUAL_TICKS;
        done = Math.max(0.0F, Math.min(1.0F, done));

        int left = (graphics.guiWidth() - WIDTH) / 2;
        int top = graphics.guiHeight() - FROM_BOTTOM;

        graphics.fill(left - 1, top - 1, left + WIDTH + 1, top + HEIGHT + 1, BACK_COLOR);
        graphics.fill(left, top, left + Math.round(WIDTH * done), top + HEIGHT, FILL_COLOR);
    }
}
