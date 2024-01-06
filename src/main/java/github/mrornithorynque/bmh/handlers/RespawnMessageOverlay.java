package github.mrornithorynque.bmh.handlers;

import org.slf4j.Logger;

import com.ibm.icu.impl.RuleCharacterIterator.Position;
import com.mojang.logging.LogUtils;

import github.mrornithorynque.bmh.utilities.TextDrawer;

import net.minecraft.world.entity.player.Player;
import net.minecraft.client.Minecraft;
import net.minecraftforge.api.distmarker.Dist;
import net.minecraftforge.event.entity.player.PlayerEvent;
import net.minecraftforge.eventbus.api.SubscribeEvent;
import net.minecraftforge.fml.common.Mod.EventBusSubscriber;

@EventBusSubscriber(value = Dist.CLIENT)
public class RespawnMessageOverlay {

    private static final Logger LOGGER = LogUtils.getLogger();

    private static int delay = 5000;
    private static Player player;

    @SubscribeEvent
    public void onPlayerRespawn(PlayerEvent.PlayerRespawnEvent event) {

        if (event.getEntity().equals(Minecraft.getInstance().player)) {

            player = (Player) event.getEntity();

            TextDrawer.getInstance().drawStringFadeInOut(
                "A new journey begin for you, " + player.getName().getString() + ".",
                TextDrawer.ScreenPosition.CENTER,
                0xFFFFFF,
                500,
                500,
                2000
            );
        }
    }
}
