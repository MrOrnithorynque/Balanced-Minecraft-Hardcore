package github.mrornithorynque.bmh.handlers;

import org.slf4j.Logger;

import com.mojang.logging.LogUtils;

import github.mrornithorynque.utilities.TextDrawer;
import github.mrornithorynque.bmh.init.SoundInit;
import github.mrornithorynque.bmh.utilities.BMHGameRules;
import github.mrornithorynque.utilities.HexColor;

import net.minecraft.world.entity.player.Player;
import net.minecraft.client.Minecraft;
import net.minecraft.sounds.SoundSource;
import net.minecraft.world.level.GameType;
import net.minecraft.world.level.Level;
import net.minecraftforge.api.distmarker.Dist;
import net.minecraftforge.event.entity.player.PlayerEvent;
import net.minecraftforge.eventbus.api.EventPriority;
import net.minecraftforge.eventbus.api.SubscribeEvent;
import net.minecraftforge.fml.common.Mod.EventBusSubscriber;

@EventBusSubscriber(value = Dist.CLIENT)
public class RespawnEffectHandler {

    private static final Logger LOGGER = LogUtils.getLogger();

    private static int delay = 5000;
    private static Player player;

    @SubscribeEvent(priority = EventPriority.LOWEST)
    public void onPlayerRespawn(PlayerEvent.PlayerRespawnEvent event) {

        if (event.getEntity().equals(Minecraft.getInstance().player)) {

            player = (Player) event.getEntity();
            Level level = player.level();

            // if (serverPlayer.gameMode.getGameModeForPlayer() == GameType.SURVIVAL) {
                playRespawnSound(player);
            // }

            if (level.getGameRules().getBoolean(BMHGameRules.RULE_DISPLAY_TEXT_ON_RESPAWN)) {

                TextDrawer.getInstance().drawString(
                        "A new journey begin for you, " + player.getName().getString() + ".",
                        TextDrawer.ScreenPosition.CENTER,
                        HexColor.WHITE.getValue(),
                        delay);
            }
        }
    }

    private void playRespawnSound(Player player) {

        Level level = player.level();

        level.playSeededSound(
                null,
                player.blockPosition().getX(),
                player.blockPosition().getY(),
                player.blockPosition().getZ(),
                SoundInit.BELL_SOUND.get(),
                SoundSource.PLAYERS,
                1.0F,
                1.0F,
                0);

        level.playSeededSound(
                null,
                player.blockPosition().getX(),
                player.blockPosition().getY(),
                player.blockPosition().getZ(),
                SoundInit.RESPAWN_SOUND.get(),
                SoundSource.PLAYERS,
                0.3F,
                1.0F,
                0);
    }
}
