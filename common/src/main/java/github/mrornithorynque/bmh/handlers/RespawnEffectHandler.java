package github.mrornithorynque.bmh.handlers;

import org.slf4j.Logger;

import com.mojang.logging.LogUtils;

import github.mrornithorynque.bmh.client.LocalPlayerText;
import github.mrornithorynque.bmh.init.SoundInit;
import github.mrornithorynque.bmh.platform.Services;
import github.mrornithorynque.bmh.utilities.BMHGameRules;

import net.minecraft.world.entity.player.Player;
import net.minecraft.sounds.SoundSource;
import net.minecraft.world.level.Level;

public class RespawnEffectHandler {

    private static final Logger LOGGER = LogUtils.getLogger();

    // Called on the server after a respawn, once the position and the kept items are settled.
    public static void onPlayerRespawn(Player player, boolean endConquered) {

        // Leaving the End is a vanilla respawn too, but not a new journey.
        if (endConquered) {
            return;
        }

        // The event fires on the server side, so the sound is played from there for everyone nearby.
        Level level = player.level();

        // Same conditions as the distant respawn itself: no BMH respawn, no BMH effects.
        if (!level.getGameRules().getBoolean(BMHGameRules.RULE_RANDOM_DEATH_SPAWN_POINT)
                || player.isCreative() || player.isSpectator()) {
            return;
        }

        playRespawnSound(player);

        if (Services.PLATFORM.isPhysicalClient()
                && level.getGameRules().getBoolean(BMHGameRules.RULE_DISPLAY_TEXT_ON_RESPAWN)) {

            LocalPlayerText.showToLocalPlayer(player, "A new journey begin for you, " + player.getName().getString() + ".");
        }
    }

    private static void playRespawnSound(Player player) {

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
