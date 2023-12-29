package github.mrornithorynque.bmh.handlers;

import github.mrornithorynque.bmh.BalancedMcHardcoreMain;

import net.minecraftforge.api.distmarker.Dist;
import net.minecraftforge.eventbus.api.SubscribeEvent;
import net.minecraftforge.fml.common.Mod;
import net.minecraft.client.Minecraft;
import net.minecraftforge.event.entity.player.PlayerEvent;

@Mod(BalancedMcHardcoreMain.MODID)
public class RespawnMessageOverlay {

    private boolean showRespawnMessage = false;

    @SubscribeEvent
    public void onPlayerRespawn(PlayerEvent.PlayerRespawnEvent event) {

        // When the player respawns, set the flag to show the message
        if (event.getEntity().equals(Minecraft.getInstance().player)) {
            showRespawnMessage = true;
        }
    }

    // @SubscribeEvent
    // public void onRenderGameOverlay(RenderGameOverlayEvent.Post event) {

    //     // Check if we're rendering the type of overlay where we want our message to appear
    //     if (event.getType() == RenderGameOverlayEvent.ElementType.ALL) {

    //         Minecraft mc = Minecraft.getInstance();

    //         // Check if we should show the respawn message
    //         if (showRespawnMessage) {

    //             // Reset the flag so the message doesn't show continuously
    //             showRespawnMessage = false;

    //             // Set the message and its position
    //             String message = "You have respawned!";
    //             int x = mc.getWindow().getGuiScaledWidth() / 2;
    //             int y = mc.getWindow().getGuiScaledHeight() / 2;

    //             // Draw the message on the screen
    //             mc.font.draw(event.getMatrixStack(), message, x, y, 0xFFFFFF);
    //         }
    //     }
    // }
}
