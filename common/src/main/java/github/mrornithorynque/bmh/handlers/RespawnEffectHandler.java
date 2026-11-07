package github.mrornithorynque.bmh.handlers;

import github.mrornithorynque.bmh.init.SoundInit;
import github.mrornithorynque.bmh.utilities.BMHGameRules;

import net.minecraft.server.level.ServerPlayer;
import net.minecraft.stats.Stats;
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

    // The exile distance, in Overworld blocks, that earns the advancement "Far, far away".
    private static final int FAR_AWAY_BLOCKS = 30000;

    // The number of deaths (the vanilla statistic) that earns the hidden advancement "Shit, here we go again".
    private static final int AGAIN_DEATHS = 10;

    // Called on the server after a respawn, once the position and the kept items are settled.
    public static void onPlayerRespawn(Player player, boolean endConquered) {

        // Leaving the End is a vanilla respawn too, but not a new journey.
        if (endConquered || !(player instanceof ServerPlayer serverPlayer)) {
            return;
        }

        // Only a respawn that got a distant position is a new journey. In a hardcore world the player is a spectator now.
        boolean exiled = RespawnPositionHandler.takeExiled(serverPlayer);
        int exileDistance = RespawnPositionHandler.takeExileDistance(serverPlayer);

        if (!exiled || serverPlayer.level().getServer().isHardcore()) {
            return;
        }

        // The three journey advancements need BMH on in this world: only a distant respawn gets here.
        BmhAdvancements.award(serverPlayer, "journey/exile");

        if (serverPlayer.getStats().getValue(Stats.CUSTOM.get(Stats.DEATHS)) >= AGAIN_DEATHS) {
            BmhAdvancements.award(serverPlayer, "journey/here_we_go_again");
        }

        if (exileDistance >= FAR_AWAY_BLOCKS) {
            BmhAdvancements.award(serverPlayer, "journey/far_away");
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
