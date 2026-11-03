package github.mrornithorynque.bmh.fabric;

import github.mrornithorynque.bmh.client.ClientItemProperties;
import github.mrornithorynque.utilities.TextDrawer;

import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.rendering.v1.HudRenderCallback;
import net.fabricmc.fabric.api.object.builder.v1.client.model.FabricModelPredicateProviderRegistry;

public class BalancedMcHardcoreFabricClient implements ClientModInitializer {

    @Override
    public void onInitializeClient() {

        FabricAmuletClient.register();

        HudRenderCallback.EVENT.register((guiGraphics, tickDelta) -> TextDrawer.getInstance().render(guiGraphics));

        // The main entrypoint has registered the items by now.
        ClientItemProperties.registerAll(FabricModelPredicateProviderRegistry::register);
    }
}
