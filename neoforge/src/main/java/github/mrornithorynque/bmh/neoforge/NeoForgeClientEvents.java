package github.mrornithorynque.bmh.neoforge;

import github.mrornithorynque.bmh.client.ClientItemProperties;
import github.mrornithorynque.bmh.client.config.ConfigScreen;

import net.minecraft.client.renderer.item.ItemProperties;

import net.neoforged.bus.api.SubscribeEvent;
import net.neoforged.fml.ModLoadingContext;
import net.neoforged.fml.event.lifecycle.FMLClientSetupEvent;
import net.neoforged.neoforge.client.ConfigScreenHandler;

/** Mod bus: the client setup. */
public class NeoForgeClientEvents {

    @SubscribeEvent
    public static void onClientSetup(FMLClientSetupEvent event) {

        // Item properties go in a map that is not thread safe: wait for the main thread.
        event.enqueueWork(() -> ClientItemProperties.registerAll(ItemProperties::register));
    }

    // The Config button of BMH in the mods list. The constructor of the @Mod class calls it, behind the client check.
    public static void registerConfigScreen() {

        ModLoadingContext.get().registerExtensionPoint(ConfigScreenHandler.ConfigScreenFactory.class,
                () -> new ConfigScreenHandler.ConfigScreenFactory((minecraft, parent) -> new ConfigScreen(parent)));
    }
}
