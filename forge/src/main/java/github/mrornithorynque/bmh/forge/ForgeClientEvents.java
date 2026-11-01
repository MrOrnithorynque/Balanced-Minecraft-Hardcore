package github.mrornithorynque.bmh.forge;

import github.mrornithorynque.bmh.BalancedMcHardcore;
import github.mrornithorynque.utilities.TextDrawer;

import net.minecraft.resources.ResourceLocation;

import net.minecraftforge.client.event.AddGuiOverlayLayersEvent;
import net.minecraftforge.eventbus.api.SubscribeEvent;

/** Mod bus: the HUD text is one more layer on top of the vanilla ones. */
public class ForgeClientEvents {

    @SubscribeEvent
    public static void onAddGuiOverlayLayers(AddGuiOverlayLayersEvent event) {

        event.getLayeredDraw().add(
                ResourceLocation.fromNamespaceAndPath(BalancedMcHardcore.MODID, "text"),
                (guiGraphics, deltaTracker) -> TextDrawer.getInstance().render(guiGraphics));
    }
}
