package github.mrornithorynque.bmh.forge;

import github.mrornithorynque.bmh.client.ClientItemProperties;
import github.mrornithorynque.bmh.client.config.ConfigScreen;

import net.minecraft.client.renderer.item.ItemProperties;

import net.minecraftforge.client.ConfigScreenHandler;
import net.minecraftforge.eventbus.api.SubscribeEvent;
import net.minecraftforge.fml.event.lifecycle.FMLClientSetupEvent;
import net.minecraftforge.fml.javafmlmod.FMLJavaModLoadingContext;

/** Mod bus: the client setup. */
public class ForgeClientEvents {

    @SubscribeEvent
    public static void onClientSetup(FMLClientSetupEvent event) {

        // Item properties go in a map that is not thread safe: wait for the main thread.
        event.enqueueWork(() -> ClientItemProperties.registerAll(ItemProperties::register));
    }

    // The Config button of BMH in the mods list. The constructor of the @Mod class calls it, behind the client check.
    public static void registerConfigScreen(FMLJavaModLoadingContext context) {

        context.registerExtensionPoint(ConfigScreenHandler.ConfigScreenFactory.class,
                () -> new ConfigScreenHandler.ConfigScreenFactory((minecraft, parent) -> new ConfigScreen(parent)));
    }
}
