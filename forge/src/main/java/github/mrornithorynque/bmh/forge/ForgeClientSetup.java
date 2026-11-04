package github.mrornithorynque.bmh.forge;

import github.mrornithorynque.bmh.client.ClientItemProperties;

import net.minecraft.client.renderer.item.ItemProperties;

import net.minecraftforge.fml.event.lifecycle.FMLClientSetupEvent;

/**
 * Mod bus, client only. FMLClientSetupEvent never fires on the game bus, so the constructor of the @Mod
 * class hooks this one up with addListener, behind the client check.
 */
public class ForgeClientSetup {

    public static void onClientSetup(FMLClientSetupEvent event) {

        // Item properties go in a map that is not thread safe: wait for the main thread.
        event.enqueueWork(() -> ClientItemProperties.registerAll(ItemProperties::register));
    }
}
