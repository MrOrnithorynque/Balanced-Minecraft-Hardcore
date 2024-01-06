package github.mrornithorynque.bmh.handlers;

import java.util.Random;
import org.slf4j.Logger;

import com.mojang.logging.LogUtils;

import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.Biomes;
import net.minecraft.world.level.levelgen.Heightmap;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.core.BlockPos;

import net.minecraftforge.event.entity.living.LivingDeathEvent;
import net.minecraftforge.eventbus.api.SubscribeEvent;

public class PlayerDeathHandler {

    private static final int MIN_DISTANCE = 50000;
    private static final int MAX_DISTANCE = 70000;

    private static final Logger LOGGER = LogUtils.getLogger();

    @SubscribeEvent
    public void onPlayerDeath(LivingDeathEvent event) {

        LOGGER.info("HELLO FROM PLAYER DEATH");

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

            BlockPos respawnPosition = calculateRandomPosition(serverLevel, bedLocation);

            float respawnAngle = 0.0f;
            player.setRespawnPosition(serverLevel.dimension(), respawnPosition, respawnAngle, true, false);
        }
    }

    private BlockPos calculateRandomPosition(ServerLevel serverLevel, BlockPos bedPosition) {

        Random RANDOM = new Random();
        int distanceBound = MAX_DISTANCE - MIN_DISTANCE + 1;

        if (distanceBound <= 0) {

            LOGGER.error("Invalid distance bounds: MIN_DISTANCE must be less than MAX_DISTANCE");
            return bedPosition;
        }

        Biome biome;
        do {

            LOGGER.info("Calculating new respawn position");

            int randomX = RANDOM.nextInt(distanceBound) + MIN_DISTANCE;
            int randomZ = RANDOM.nextInt(distanceBound) + MIN_DISTANCE;

            int newX = bedPosition.getX() + (RANDOM.nextBoolean() ? randomX : -randomX);
            int newZ = bedPosition.getZ() + (RANDOM.nextBoolean() ? randomZ : -randomZ);

            biome = serverLevel.getBiome(new BlockPos(newX, 0, newZ)).get(); // Assign the biome variable correctly

            if (!biome.equals(Biomes.OCEAN) && !biome.equals(Biomes.RIVER)) {

                int newY = serverLevel.getHeight(Heightmap.Types.WORLD_SURFACE_WG, newX, newZ);
                if (newY <= serverLevel.getMinBuildHeight()) {
                    newY = serverLevel.getSeaLevel();
                }

                LOGGER.info("getHeight WORLD_SURFACE_WG : " + serverLevel.getHeight(Heightmap.Types.WORLD_SURFACE_WG, newX, newZ));
                LOGGER.info("getHeight OCEAN_FLOOR_WG : " + serverLevel.getHeight(Heightmap.Types.OCEAN_FLOOR_WG, newX, newZ));
                LOGGER.info("getHeight OCEAN_FLOOR : " + serverLevel.getHeight(Heightmap.Types.OCEAN_FLOOR, newX, newZ));
                LOGGER.info("getHeight WORLD_SURFACE : " + serverLevel.getHeight(Heightmap.Types.WORLD_SURFACE, newX, newZ));
                LOGGER.info("getHeight MOTION_BLOCKING : " + serverLevel.getHeight(Heightmap.Types.MOTION_BLOCKING, newX, newZ));
                LOGGER.info("getHeight MOTION_BLOCKING_NO_LEAVES : " + serverLevel.getHeight(Heightmap.Types.MOTION_BLOCKING_NO_LEAVES, newX, newZ));

                LOGGER.info("New respawn position: X=" + newX + ", Y=" + newY + ", Z=" + newZ);
                // Set the respawn position here
                bedPosition = new BlockPos(newX, newY, newZ);
                break;
            }

        } while (biome.equals(Biomes.OCEAN) || biome.equals(Biomes.RIVER));

        return bedPosition;
    }
}
