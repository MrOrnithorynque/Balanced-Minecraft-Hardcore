package github.mrornithorynque.bmh.handlers;

import java.util.Random;

import org.joml.Math;
import org.slf4j.Logger;

import com.mojang.logging.LogUtils;

import github.mrornithorynque.bmh.items.IEternalItem;
import github.mrornithorynque.bmh.utilities.BMHGameRules;
import github.mrornithorynque.bmh.Config;

import net.minecraft.core.BlockPos;

import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;

import net.minecraft.world.Difficulty;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.GameRules;
import net.minecraft.world.level.GameType;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.Biomes;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.levelgen.Heightmap;

import net.minecraftforge.event.entity.living.LivingDeathEvent;
import net.minecraftforge.event.entity.player.PlayerEvent;

import net.minecraftforge.eventbus.api.EventPriority;
import net.minecraftforge.eventbus.api.SubscribeEvent;

public class RespawnPositionHandler {

    private static final int MAX_BUILD_HEIGHT = 320;
    private static final int MIN_BUILD_HEIGHT = -64;

    private static final Logger LOGGER = LogUtils.getLogger();

    private int distanceFromBed = Config.maxDistanceFromBed.get();;

    @SubscribeEvent(priority = EventPriority.HIGHEST)
    public void onPlayerRespawn(PlayerEvent.PlayerRespawnEvent event) {

        setNewRespawnPosition(event);
    }

    @SubscribeEvent(priority = EventPriority.HIGHEST)
    public void onPlayerDeath(LivingDeathEvent event) {

        if (event.getEntity() instanceof ServerPlayer) {
            distanceFromBed = calculateDistanceFromBed((ServerPlayer) event.getEntity());
        }
    }

    private void setNewRespawnPosition(PlayerEvent.PlayerRespawnEvent event) {

        if (event.getEntity() instanceof ServerPlayer) {

            ServerPlayer serverPlayer = (ServerPlayer) event.getEntity();
            ServerLevel serverLevel = serverPlayer.serverLevel();
            BlockPos bedLocation = serverPlayer.getRespawnPosition();

            LOGGER.info("[BalancedMcHardcoreMain] bedLocation : " + bedLocation);

            GameRules gameRules = serverLevel.getGameRules();

            if (!serverLevel.getGameRules().getBoolean(BMHGameRules.RULE_RANDOM_DEATH_SPAWN_POINT)) {

                LOGGER.info("[BalancedMcHardcoreMain] RULE_RANDOM_DEATH_SPAWN_POINT: "
                        + gameRules.getBoolean(BMHGameRules.RULE_RANDOM_DEATH_SPAWN_POINT));
                LOGGER.info("[BalancedMcHardcoreMain] Game rule randomDeathSpawnPoint is false, using default respawn position");

                return;
            }

            if (serverPlayer.gameMode.getGameModeForPlayer() != GameType.SURVIVAL) {

                LOGGER.info("[BalancedMcHardcoreMain] Player is not in survival mode, using default respawn position");
                return;
            }

            if (bedLocation == null) {

                bedLocation = serverLevel.getSharedSpawnPos();
                LOGGER.info("[BalancedMcHardcoreMain] Respawn is not forced by a bed");
            }

            BlockPos respawnPosition = calculateRandomPosition(serverPlayer, serverLevel, bedLocation);

            LOGGER.info("[BalancedMcHardcoreMain] new respawn pos and last : " + respawnPosition + ", " + bedLocation);

            serverPlayer.setRespawnPosition(serverLevel.dimension(), respawnPosition, 0.0f, true, false);
            serverPlayer.teleportTo(respawnPosition.getX(), respawnPosition.getY(), respawnPosition.getZ());
            serverPlayer.setRespawnPosition(serverLevel.dimension(), bedLocation, 0.0f, true, false);
        }
    }

    private BlockPos calculateRandomPosition(ServerPlayer serverPlayer, ServerLevel serverLevel, BlockPos bedPosition) {

        Biome biome;
        do {

            LOGGER.info("[BalancedMcHardcoreMain] Calculating new respawn position");

            BlockPos randomEdgePoint = findRandomEdgePoint(bedPosition, this.distanceFromBed);

            int newX = randomEdgePoint.getX();
            int newZ = randomEdgePoint.getZ();

            biome = serverLevel.getBiome(new BlockPos(newX, 0, newZ)).get();

            if (!biome.equals(Biomes.OCEAN) && !biome.equals(Biomes.RIVER)) {

                LOGGER.info("[BalancedMcHardcoreMain] Found a suitable biome that is not an ocean or a river");
                LOGGER.info("[BalancedMcHardcoreMain] serverLevel.getMinBuildHeight() : " + serverLevel.getMinBuildHeight());
                LOGGER.info("[BalancedMcHardcoreMain] serverLevel.getHeight(Heightmap.Types.MOTION_BLOCKING, newX, newZ); : "
                        + serverLevel.getHeight(Heightmap.Types.MOTION_BLOCKING, newX, newZ));

                int newY = getYRespawnPosition(newX, newZ, serverLevel);

                bedPosition = new BlockPos(newX, newY, newZ);

                break;
            }
        } while (biome.equals(Biomes.OCEAN) || biome.equals(Biomes.RIVER));

        return bedPosition;
    }

    private int calculateDistanceFromBed(ServerPlayer serverPlayer) {

        int minDistance = Config.minDistanceFromBed.get();
        int maxDistance = Config.maxDistanceFromBed.get();

        final int maxEasyLife = 9;

        int easyLife = 1;
        int distanceFromBed = maxDistance;
        int eternalItemCount = 0;

        if (minDistance > maxDistance) {
            LOGGER.warn("[BalancedMcHardcoreMain] minDistanceFromBed is greater than maxDistanceFromBed, using default values");
            maxDistance = minDistance;
        }

        for (int i = 0; i < serverPlayer.getInventory().getContainerSize(); i++) {
            ItemStack stack = serverPlayer.getInventory().getItem(i);
            if (stack.getItem() instanceof IEternalItem) {
                eternalItemCount++;
            }
        }

        easyLife += eternalItemCount;
        easyLife += serverPlayer.level().getDifficulty().getId();

        float x = (maxDistance - minDistance);
        float y = (maxEasyLife / easyLife);
        float z = x / y;

        distanceFromBed = (int)z + minDistance; // TODO : add randomness

        LOGGER.info("[BalancedMcHardcoreMain] Distance from bed: " + distanceFromBed);

        return distanceFromBed;
    }

    private int getYRespawnPosition(int x, int z, ServerLevel serverLevel) {

        LOGGER.info("[BalancedMcHardcoreMain] Checking respawn position for X=" + x + ", Z=" + z);

        for (int y = MAX_BUILD_HEIGHT; y >= MIN_BUILD_HEIGHT; y--) {
            BlockState blockState = serverLevel.getBlockState(new BlockPos(x, y, z));
            Block block = blockState.getBlock();

            if (!block.equals(Blocks.AIR) && !block.equals(Blocks.CAVE_AIR) && !block.equals(Blocks.VOID_AIR)) {
                LOGGER.info("[BalancedMcHardcoreMain] Found non-air block (" + block + ") at Y=" + y);
                return y + 1;
            }
        }

        LOGGER.warn("[BalancedMcHardcoreMain] No non-air blocks found, defaulting to MAX_BUILD_HEIGHT");

        return MAX_BUILD_HEIGHT;
    }

    public static BlockPos findRandomEdgePoint(BlockPos center, int radius) {

        Random random = new Random();
        double theta = 2 * Math.PI * random.nextDouble();

        int xEdge = center.getX() + (int) (radius * Math.cos(theta));
        int zEdge = center.getZ() + (int) (radius * Math.sin(theta));

        return new BlockPos(xEdge, 0, zEdge);
    }

}
