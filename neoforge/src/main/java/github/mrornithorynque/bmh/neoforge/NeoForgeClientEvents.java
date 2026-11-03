package github.mrornithorynque.bmh.neoforge;

import github.mrornithorynque.bmh.BalancedMcHardcore;
import github.mrornithorynque.bmh.client.ClientItemProperties;
import github.mrornithorynque.utilities.TextDrawer;

import net.minecraft.client.renderer.item.ItemProperties;
import net.minecraft.resources.ResourceLocation;

import net.neoforged.bus.api.SubscribeEvent;
import net.neoforged.fml.event.lifecycle.FMLClientSetupEvent;
import net.neoforged.neoforge.client.event.RegisterGuiLayersEvent;

/**
 * Mod bus, client only (the @Mod class registers it behind the client check): the HUD text is one more
 * layer on top of the vanilla ones, and the item properties wait for the client setup.
 */
public class NeoForgeClientEvents {

    @SubscribeEvent
    public static void onRegisterGuiLayers(RegisterGuiLayersEvent event) {

        event.registerAboveAll(
                ResourceLocation.fromNamespaceAndPath(BalancedMcHardcore.MODID, "text"),
                (guiGraphics, deltaTracker) -> TextDrawer.getInstance().render(guiGraphics));
    }

    @SubscribeEvent
    public static void onClientSetup(FMLClientSetupEvent event) {

        // Item properties go in a map that is not thread safe: wait for the main thread.
        event.enqueueWork(() -> ClientItemProperties.registerAll(ItemProperties::register));
    }
}
