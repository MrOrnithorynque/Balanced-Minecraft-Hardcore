package github.mrornithorynque.bmh.forge;

import github.mrornithorynque.bmh.client.config.ConfigScreen;

import net.minecraftforge.client.ConfigScreenHandler;
import net.minecraftforge.fml.javafmlmod.FMLJavaModLoadingContext;

/** The client side of the Forge mod. Since 26.1 the item models need no client setup: their JSON files switch on their own. */
public class ForgeClientEvents {

    // The Config button of BMH in the mods list. The constructor of the @Mod class calls it, behind the client check.
    public static void registerConfigScreen(FMLJavaModLoadingContext context) {

        context.registerExtensionPoint(ConfigScreenHandler.ConfigScreenFactory.class,
                () -> new ConfigScreenHandler.ConfigScreenFactory((minecraft, parent) -> new ConfigScreen(parent)));
    }
}
