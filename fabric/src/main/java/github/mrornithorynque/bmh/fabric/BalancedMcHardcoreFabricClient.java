package github.mrornithorynque.bmh.fabric;

import github.mrornithorynque.utilities.TextDrawer;

import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.rendering.v1.HudRenderCallback;

public class BalancedMcHardcoreFabricClient implements ClientModInitializer {

    @Override
    public void onInitializeClient() {

        FabricAmuletClient.register();

        HudRenderCallback.EVENT.register((guiGraphics, tickDelta) -> TextDrawer.getInstance().render(guiGraphics));
    }
}
