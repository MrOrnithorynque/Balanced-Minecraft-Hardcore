package github.mrornithorynque.bmh.forge;

import github.mrornithorynque.bmh.client.ClientItemProperties;

import net.minecraft.client.renderer.item.ItemProperties;

import net.minecraftforge.eventbus.api.SubscribeEvent;
import net.minecraftforge.fml.event.lifecycle.FMLClientSetupEvent;

/** Mod bus: the client setup. */
public class ForgeClientEvents {

    @SubscribeEvent
    public static void onClientSetup(FMLClientSetupEvent event) {

        // Item properties go in a map that is not thread safe: wait for the main thread.
        event.enqueueWork(() -> ClientItemProperties.registerAll(ItemProperties::register));
    }
}
