package github.mrornithorynque.bmh.forge;

import github.mrornithorynque.utilities.TextDrawer;

import net.minecraftforge.client.event.RenderGuiOverlayEvent;
import net.minecraftforge.eventbus.api.SubscribeEvent;

public class ForgeClientEvents {

    @SubscribeEvent
    public static void onRenderGuiOverlay(RenderGuiOverlayEvent.Post event) {

        TextDrawer.getInstance().render(event.getGuiGraphics());
    }
}
