package github.mrornithorynque.bmh.fabric;

import github.mrornithorynque.bmh.client.AmuletCompass;
import github.mrornithorynque.bmh.init.ItemInit;
import github.mrornithorynque.utilities.TextDrawer;

import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.rendering.v1.HudRenderCallback;
import net.fabricmc.fabric.api.object.builder.v1.client.model.FabricModelPredicateProviderRegistry;

public class BalancedMcHardcoreFabricClient implements ClientModInitializer {

    @Override
    public void onInitializeClient() {

        HudRenderCallback.EVENT.register((guiGraphics, tickDelta) -> TextDrawer.getInstance().render(guiGraphics));

        FabricModelPredicateProviderRegistry.register(ItemInit.NAVIGATOR_AMULET.get(), AmuletCompass.ANGLE, AmuletCompass.angle());
    }
}
