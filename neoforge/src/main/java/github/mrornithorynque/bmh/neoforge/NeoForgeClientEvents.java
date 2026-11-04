package github.mrornithorynque.bmh.neoforge;

import github.mrornithorynque.bmh.client.ClientItemProperties;

import net.minecraft.client.renderer.item.ItemProperties;

import net.neoforged.bus.api.SubscribeEvent;
import net.neoforged.fml.event.lifecycle.FMLClientSetupEvent;

/** Mod bus: the client setup. */
public class NeoForgeClientEvents {

    @SubscribeEvent
    public static void onClientSetup(FMLClientSetupEvent event) {

        // Item properties go in a map that is not thread safe: wait for the main thread.
        event.enqueueWork(() -> ClientItemProperties.registerAll(ItemProperties::register));
    }
}
