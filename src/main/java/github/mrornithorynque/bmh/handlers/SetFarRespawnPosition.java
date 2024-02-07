package github.mrornithorynque.bmh.handlers;

import java.util.Random;

import org.slf4j.Logger;

import com.mojang.logging.LogUtils;

import github.mrornithorynque.bmh.utilities.BMHGameRules;

import net.minecraft.core.BlockPos;

import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;

import net.minecraft.world.level.GameRules;
import net.minecraft.world.level.GameType;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.Biomes;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.levelgen.Heightmap;

import net.minecraftforge.event.entity.player.PlayerEvent;
import net.minecraftforge.eventbus.api.EventPriority;
import net.minecraftforge.eventbus.api.SubscribeEvent;

public class SetFarRespawnPosition {

    private static final int MIN_DISTANCE = 50000;
    private static final int MAX_DISTANCE = 70000;
    private static final int MAX_BUILD_HEIGHT = 320;
    private static final int MIN_BUILD_HEIGHT = -64;

    private static final Logger LOGGER = LogUtils.getLogger();

    @SubscribeEvent(priority = EventPriority.HIGHEST)
    public void onPlayerRespawn(PlayerEvent.PlayerRespawnEvent event) {

        setNewRespawnPosition(event);
    }

    private void setNewRespawnPosition(PlayerEvent.PlayerRespawnEvent event) {

        // move this on respawn handler because huge lag
        if (event.getEntity() instanceof ServerPlayer) {

            ServerPlayer player = (ServerPlayer) event.getEntity();
            ServerLevel serverLevel = player.serverLevel();

            BlockPos bedLocation = player.getRespawnPosition();
            boolean isRespawnForced = player.isRespawnForced();

            GameRules gameRules = serverLevel.getGameRules();

            if (!serverLevel.getGameRules().getBoolean(BMHGameRules.RULE_RANDOM_DEATH_SPAWN_POINT)) {

                LOGGER.info("RULE_RANDOM_DEATH_SPAWN_POINT: "
                        + gameRules.getBoolean(BMHGameRules.RULE_RANDOM_DEATH_SPAWN_POINT));
                LOGGER.info("Game rule randomDeathSpawnPoint is false, using default respawn position");

                return;
            }

            if (player.gameMode.getGameModeForPlayer() != GameType.SURVIVAL) {

                LOGGER.info("Player is not in survival mode, using default respawn position");
                return;
            }

            if (bedLocation == null || !isRespawnForced) {

                bedLocation = serverLevel.getSharedSpawnPos();
                LOGGER.info("Respawn is not forced by a bed");
            }

            BlockPos respawnPosition = calculateRandomPosition(serverLevel, bedLocation);

            player.setRespawnPosition(serverLevel.dimension(), respawnPosition, 0.0f, true, false);
        }
    }

    private BlockPos calculateRandomPosition(ServerLevel serverLevel, BlockPos bedPosition) {

        Random RANDOM = new Random();

        int distanceBoundX = MAX_DISTANCE - MIN_DISTANCE + 1 /* + bedPosition.getX() */;
        int distanceBoundY = MAX_DISTANCE - MIN_DISTANCE + 1 /* + bedPosition.getY() */;

        Biome biome;
        do {

            LOGGER.info("Calculating new respawn position");

            int randomX = RANDOM.nextInt(distanceBoundX) + MIN_DISTANCE;
            int randomZ = RANDOM.nextInt(distanceBoundY) + MIN_DISTANCE;

            int newX = bedPosition.getX() + (RANDOM.nextBoolean() ? randomX : -randomX);
            int newZ = bedPosition.getZ() + (RANDOM.nextBoolean() ? randomZ : -randomZ);

            biome = serverLevel.getBiome(new BlockPos(newX, 0, newZ)).get(); // Assign the biome variable correctly

            if (!biome.equals(Biomes.OCEAN) && !biome.equals(Biomes.RIVER)) {

                LOGGER.info("Found a suitable biome that is not an ocean or a river");
                LOGGER.info("serverLevel.getMinBuildHeight() : " + serverLevel.getMinBuildHeight());
                LOGGER.info("serverLevel.getHeight(Heightmap.Types.MOTION_BLOCKING, newX, newZ); : "
                        + serverLevel.getHeight(Heightmap.Types.MOTION_BLOCKING, newX, newZ));

                int newY = getYRespawnPosition(newX, newZ, serverLevel);

                bedPosition = new BlockPos(newX, newY, newZ);
                break;
            }
        } while (biome.equals(Biomes.OCEAN) || biome.equals(Biomes.RIVER));

        LOGGER.info("New respawn position: X=" + bedPosition.getX() + ", Y=" + bedPosition.getY() + ", Z="
                + bedPosition.getZ());

        return bedPosition;
    }

    private int getYRespawnPosition(int x, int z, ServerLevel serverLevel) {

        LOGGER.info("Checking respawn position for X=" + x + ", Z=" + z);

        for (int y = MAX_BUILD_HEIGHT; y >= MIN_BUILD_HEIGHT; y--) {
            BlockState blockState = serverLevel.getBlockState(new BlockPos(x, y, z));
            Block block = blockState.getBlock();

            if (!block.equals(Blocks.AIR) && !block.equals(Blocks.CAVE_AIR) && !block.equals(Blocks.VOID_AIR)) {
                LOGGER.info("Found non-air block (" + block + ") at Y=" + y);
                return y + 1;
            }
        }

        LOGGER.warn("No non-air blocks found, defaulting to MAX_BUILD_HEIGHT");

        return MAX_BUILD_HEIGHT;
    }
}
