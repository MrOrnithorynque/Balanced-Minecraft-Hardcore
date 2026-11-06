package github.mrornithorynque.bmh.neoforge;

import github.mrornithorynque.bmh.client.config.ConfigScreen;

import net.neoforged.fml.ModContainer;
import net.neoforged.neoforge.client.gui.IConfigScreenFactory;

/** The client side of the NeoForge mod. Since 26.1 the item models need no client setup: their JSON files switch on their own. */
public class NeoForgeClientEvents {

    // The Config button of BMH in the mods list. The constructor of the @Mod class calls it, behind the client check.
    public static void registerConfigScreen(ModContainer container) {

        container.registerExtensionPoint(IConfigScreenFactory.class, (modContainer, parent) -> new ConfigScreen(parent));
    }
}
