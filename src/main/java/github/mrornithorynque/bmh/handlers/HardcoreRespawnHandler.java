package github.mrornithorynque.bmh.handlers;

import java.util.Random;
import org.slf4j.Logger;

import net.minecraftforge.event.entity.player.PlayerEvent.PlayerRespawnEvent;
import net.minecraftforge.eventbus.api.SubscribeEvent;

import com.mojang.logging.LogUtils;

import net.minecraft.core.BlockPos;

import net.minecraft.world.level.levelgen.Heightmap;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;


public class HardcoreRespawnHandler {

    // Define the minimum distance from the original death point
    private static final int MIN_DISTANCE = 100000;
    private static final int MAX_DISTANCE = 40000;

    private static final Logger LOGGER = LogUtils.getLogger();

    @SubscribeEvent
    public void onPlayerRespawn(PlayerRespawnEvent event) {

        LOGGER.info("HELLO FROM PLAYER RESPAWN");

        // Ensure we're dealing with a server player (as respawn locations are server-side logic)
        if (event.getEntity() instanceof ServerPlayer) {

            LOGGER.info("Player is a server player");

            ServerPlayer player = (ServerPlayer) event.getEntity();
            ServerLevel serverLevel = player.serverLevel();

            BlockPos bedLocation = player.getRespawnPosition();
            boolean isRespawnForced = player.isRespawnForced();

            // If the bedLocation is null or the respawn is not forced by a bed, use the world spawn
            if (bedLocation == null || !isRespawnForced) {
                bedLocation = serverLevel.getSharedSpawnPos();
                LOGGER.info("Respawn is not forced by a bed");
            }

            // Calculate a new random position based on the bed's location or world spawn
            BlockPos respawnPosition = calculateRandomPosition(serverLevel, bedLocation);

            float respawnAngle = 0.0f;
            player.setRespawnPosition(serverLevel.dimension(), respawnPosition, respawnAngle, true, false);
        }
    }

    private BlockPos calculateRandomPosition(ServerLevel serverLevel, BlockPos bedPosition) {

        Random RANDOM = new Random();

        int randomX = RANDOM.nextInt(MAX_DISTANCE - MIN_DISTANCE + 1) + MIN_DISTANCE;
        int randomZ = RANDOM.nextInt(MAX_DISTANCE - MIN_DISTANCE + 1) + MIN_DISTANCE;

        int newX = bedPosition.getX() + (RANDOM.nextBoolean() ? randomX : -randomX);
        int newZ = bedPosition.getZ() + (RANDOM.nextBoolean() ? randomZ : -randomZ);
        int newY = serverLevel.getHeight(Heightmap.Types.MOTION_BLOCKING, newX, newZ);

        LOGGER.info("New respawn position: " + newX + ", " + newY + ", " + newZ);

        return new BlockPos(newX, newY, newZ);
    }
}
