package github.mrornithorynque.bmh.neoforge;

import github.mrornithorynque.bmh.BalancedMcHardcore;
import github.mrornithorynque.utilities.TextDrawer;

import net.minecraft.resources.ResourceLocation;

import net.neoforged.bus.api.SubscribeEvent;
import net.neoforged.neoforge.client.event.RegisterGuiLayersEvent;

/** Mod bus: the HUD text is one more layer on top of the vanilla ones. */
public class NeoForgeClientEvents {

    @SubscribeEvent
    public static void onRegisterGuiLayers(RegisterGuiLayersEvent event) {

        event.registerAboveAll(
                ResourceLocation.fromNamespaceAndPath(BalancedMcHardcore.MODID, "text"),
                (guiGraphics, deltaTracker) -> TextDrawer.getInstance().render(guiGraphics));
    }
}
