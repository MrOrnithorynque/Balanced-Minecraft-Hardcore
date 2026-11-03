package github.mrornithorynque.bmh.forge;

import github.mrornithorynque.bmh.BalancedMcHardcore;
import github.mrornithorynque.bmh.client.ClientItemProperties;
import github.mrornithorynque.utilities.TextDrawer;

import net.minecraft.client.renderer.item.ItemProperties;
import net.minecraft.resources.ResourceLocation;

import net.minecraftforge.client.event.AddGuiOverlayLayersEvent;
import net.minecraftforge.eventbus.api.SubscribeEvent;
import net.minecraftforge.fml.event.lifecycle.FMLClientSetupEvent;

/** Mod bus: the HUD text is one more layer on top of the vanilla ones. */
public class ForgeClientEvents {

    @SubscribeEvent
    public static void onAddGuiOverlayLayers(AddGuiOverlayLayersEvent event) {

        event.getLayeredDraw().add(
                ResourceLocation.fromNamespaceAndPath(BalancedMcHardcore.MODID, "text"),
                (guiGraphics, deltaTracker) -> TextDrawer.getInstance().render(guiGraphics));
    }

    @SubscribeEvent
    public static void onClientSetup(FMLClientSetupEvent event) {

        // Item properties go in a map that is not thread safe: wait for the main thread.
        event.enqueueWork(() -> ClientItemProperties.registerAll(ItemProperties::register));
    }
}
