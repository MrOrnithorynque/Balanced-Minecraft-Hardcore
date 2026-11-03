package github.mrornithorynque.bmh.fabric;

import github.mrornithorynque.bmh.client.AmuletCompass;
import github.mrornithorynque.bmh.init.ItemInit;

import net.fabricmc.fabric.api.object.builder.v1.client.model.FabricModelPredicateProviderRegistry;

/** Called from the client entrypoint only: the amulet's needle, an item property the model switches on. */
public class FabricAmuletClient {

    public static void register() {

        // The main entrypoint has registered the items by now.
        FabricModelPredicateProviderRegistry.register(ItemInit.NAVIGATOR_AMULET.get(), AmuletCompass.ANGLE, AmuletCompass.angle());
    }
}
