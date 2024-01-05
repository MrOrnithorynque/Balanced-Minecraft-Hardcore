package github.mrornithorynque.bmh.handlers;

import org.slf4j.Logger;

import com.mojang.logging.LogUtils;

import net.minecraft.world.entity.player.Player;
import net.minecraft.client.Minecraft;
import net.minecraftforge.api.distmarker.Dist;
import net.minecraftforge.client.event.RenderGuiOverlayEvent;
import net.minecraftforge.event.entity.player.PlayerEvent;
import net.minecraftforge.eventbus.api.SubscribeEvent;
import net.minecraftforge.fml.common.Mod.EventBusSubscriber;

@EventBusSubscriber(value = Dist.CLIENT)
public class RespawnMessageOverlay {

    private static boolean showRespawnMessage = false;

    private static final Logger LOGGER = LogUtils.getLogger();

    private static long messageEndTime;
    private static int delay = 5000;
    private static Player player;

    @SubscribeEvent
    public void onPlayerRespawn(PlayerEvent.PlayerRespawnEvent event) {

        if (event.getEntity().equals(Minecraft.getInstance().player)) {

            showRespawnMessage = true;
            messageEndTime = System.currentTimeMillis() + delay;
            player = (Player) event.getEntity();
        }
    }

    @SubscribeEvent
    public static void onRenderGuiOverlay(RenderGuiOverlayEvent.Post event) {

        if (showRespawnMessage && System.currentTimeMillis() < messageEndTime) {

            Minecraft mc   = Minecraft.getInstance();
            String message = "A new journey begin for you, " + player.getName().getString() + ".";

            int width  = event.getWindow().getGuiScaledWidth();
            int height = event.getWindow().getGuiScaledHeight();

            // Calculate position to render (e.g., centered on screen)
            int x     = (width - mc.font.width(message)) / 2;
            int y     = height / 2;
            int color = 0xFFFFFF; // White color

            LOGGER.info("x: " + x + ", y: " + y);
            LOGGER.info("width: " + width + ", height: " + height);
            LOGGER.info("message: " + message);

            // Use GuiGraphics for rendering
            event.getGuiGraphics().drawString(mc.font, message, x, y, color);
        } else {

            showRespawnMessage = false;
        }
    }
}
