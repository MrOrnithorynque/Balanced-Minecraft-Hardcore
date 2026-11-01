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
import net.minecraft.world.level.Level;
import net.minecraftforge.api.distmarker.Dist;
import net.minecraftforge.event.entity.player.PlayerEvent;
import net.minecraftforge.eventbus.api.EventPriority;
import net.minecraftforge.eventbus.api.SubscribeEvent;
import net.minecraftforge.fml.loading.FMLEnvironment;
import net.minecraftforge.fml.common.Mod.EventBusSubscriber;

@EventBusSubscriber(value = Dist.CLIENT)
public class RespawnEffectHandler {

    private static final Logger LOGGER = LogUtils.getLogger();

    private static int delay = 5000;
    private static Player player;

    @SubscribeEvent(priority = EventPriority.LOWEST)
    public void onPlayerRespawn(PlayerEvent.PlayerRespawnEvent event) {

        // Leaving the End is a vanilla respawn too, but not a new journey.
        if (event.isEndConquered()) {
            return;
        }

        // The event fires on the server side, so the sound is played from there for everyone nearby.
        player = event.getEntity();
        Level level = player.level();

        // Same conditions as the distant respawn itself: no BMH respawn, no BMH effects.
        if (!level.getGameRules().getBoolean(BMHGameRules.RULE_RANDOM_DEATH_SPAWN_POINT)
                || player.isCreative() || player.isSpectator()) {
            return;
        }

        playRespawnSound(player);

        // TextDrawer renders on the client of this JVM, so only show the text when the respawning
        // player is the local one (singleplayer or LAN host). Compared by UUID: the event carries
        // the ServerPlayer, never the client's LocalPlayer instance.
        if (FMLEnvironment.dist.isClient() && isLocalPlayer(player)) {

            if (level.getGameRules().getBoolean(BMHGameRules.RULE_DISPLAY_TEXT_ON_RESPAWN)) {

                TextDrawer.getInstance().drawString(
                        "A new journey begin for you, " + player.getName().getString() + ".",
                        TextDrawer.ScreenPosition.CENTER,
                        HexColor.WHITE.getValue(),
                        delay);
            }
        }
    }

    private static boolean isLocalPlayer(Player player) {

        Player localPlayer = Minecraft.getInstance().player;

        return localPlayer != null && localPlayer.getUUID().equals(player.getUUID());
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
