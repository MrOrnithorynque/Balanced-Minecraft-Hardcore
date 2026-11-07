package github.mrornithorynque.bmh.handlers;

import github.mrornithorynque.bmh.init.SoundInit;
import github.mrornithorynque.bmh.utilities.BMHGameRules;

import net.minecraft.server.level.ServerPlayer;
import net.minecraft.sounds.SoundSource;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.level.Level;

public class RespawnEffectHandler {

    /** Implemented by ServerPlayer through F_ServerPlayerJourneyMixin: damage immunity for a number of ticks. */
    public interface JourneyShield {

        void bmh$setJourneyInvulnerableTicks(int ticks);
    }

    // The client cinematic lasts about 5 s after the respawn, and longer while the terrain loads. Vanilla gives up to 60 ticks.
    private static final int JOURNEY_INVULNERABLE_TICKS = 200;

    // Called on the server after a respawn, once the position and the kept items are settled.
    public static void onPlayerRespawn(Player player, boolean endConquered) {

        // Leaving the End is a vanilla respawn too, but not a new journey.
        if (endConquered || !(player instanceof ServerPlayer serverPlayer)) {
            return;
        }

        // Only a respawn that got a distant position is a new journey. In a hardcore world the player is a spectator now.
        boolean exiled = RespawnPositionHandler.takeExiled(serverPlayer);
        RespawnPositionHandler.Exile exile = RespawnPositionHandler.takeExile(serverPlayer);

        if (!exiled || serverPlayer.level().getServer().isHardcore()) {
            return;
        }

        // The homes are fixed now: a bed or a point made later does not end this exile.
        if (exile != null) {
            HomecomingHandler.onExile(serverPlayer, exile);
        }

        // The player cannot see or move during the cinematic.
        ((JourneyShield) serverPlayer).bmh$setJourneyInvulnerableTicks(JOURNEY_INVULNERABLE_TICKS);

        playRespawnSound(serverPlayer);

        JourneySignal.sendArrival(serverPlayer,
                serverPlayer.level().getGameRules().get(BMHGameRules.RULE_DISPLAY_TEXT_ON_RESPAWN));
    }

    // For the other players nearby. The respawned player hears the sounds from the cinematic on its own client.
    private static void playRespawnSound(ServerPlayer serverPlayer) {

        Level level = serverPlayer.level();

        level.playSeededSound(
                serverPlayer,
                serverPlayer.blockPosition().getX(),
                serverPlayer.blockPosition().getY(),
                serverPlayer.blockPosition().getZ(),
                SoundInit.BELL_SOUND.get(),
                SoundSource.PLAYERS,
                1.0F,
                1.0F,
                0);

        level.playSeededSound(
                serverPlayer,
                serverPlayer.blockPosition().getX(),
                serverPlayer.blockPosition().getY(),
                serverPlayer.blockPosition().getZ(),
                SoundInit.RESPAWN_SOUND.get(),
                SoundSource.PLAYERS,
                0.3F,
                1.0F,
                0);
    }
}
