package github.mrornithorynque.bmh.neoforge;

import github.mrornithorynque.bmh.BalancedMcHardcore;
import github.mrornithorynque.bmh.client.ClientItemProperties;
import github.mrornithorynque.utilities.TextDrawer;

import net.minecraft.client.renderer.item.ItemProperties;
import net.minecraft.resources.ResourceLocation;

import net.neoforged.bus.api.SubscribeEvent;
import net.neoforged.fml.event.lifecycle.FMLClientSetupEvent;
import net.neoforged.neoforge.client.event.RegisterGuiOverlaysEvent;

/** Mod bus: the HUD text is one more overlay on top of the vanilla ones. */
public class NeoForgeClientEvents {

    @SubscribeEvent
    public static void onRegisterGuiOverlays(RegisterGuiOverlaysEvent event) {

        event.registerAboveAll(
                new ResourceLocation(BalancedMcHardcore.MODID, "text"),
                (gui, guiGraphics, partialTick, screenWidth, screenHeight) -> TextDrawer.getInstance().render(guiGraphics));
    }

    @SubscribeEvent
    public static void onClientSetup(FMLClientSetupEvent event) {

        // Item properties go in a map that is not thread safe: wait for the main thread.
        event.enqueueWork(() -> ClientItemProperties.registerAll(ItemProperties::register));
    }
}
