package github.mrornithorynque.bmh.handlers;

import java.util.HashMap;
import java.util.Map;
import java.util.UUID;

import org.slf4j.Logger;

import com.mojang.logging.LogUtils;

import github.mrornithorynque.bmh.items.IEternalItem;
import github.mrornithorynque.bmh.utilities.BMHGameRules;
import github.mrornithorynque.bmh.Config;

import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.core.Holder;
import net.minecraft.core.SectionPos;
import net.minecraft.tags.BiomeTags;
import net.minecraft.tags.BlockTags;

import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;

import net.minecraft.util.RandomSource;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.GameType;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.level.levelgen.Heightmap;

import net.minecraftforge.event.entity.living.LivingDeathEvent;
import net.minecraftforge.event.entity.player.PlayerEvent;

import net.minecraftforge.eventbus.api.EventPriority;
import net.minecraftforge.eventbus.api.SubscribeEvent;

public class RespawnPositionHandler {

    private static final int MAX_EASY_LIFE = 9;
    private static final int MAX_POSITION_ATTEMPTS = 32;

    private static final Logger LOGGER = LogUtils.getLogger();

    // One entry per player, filled on death and consumed on respawn.
    private final Map<UUID, Integer> distancesFromBed = new HashMap<>();

    @SubscribeEvent(priority = EventPriority.HIGHEST)
    public void onPlayerRespawn(PlayerEvent.PlayerRespawnEvent event) {

        setNewRespawnPosition(event);
    }

    @SubscribeEvent(priority = EventPriority.HIGHEST)
    public void onPlayerDeath(LivingDeathEvent event) {

        if (event.getEntity() instanceof ServerPlayer serverPlayer) {
            distancesFromBed.put(serverPlayer.getUUID(), calculateDistanceFromBed(serverPlayer));
        }
    }

    private void setNewRespawnPosition(PlayerEvent.PlayerRespawnEvent event) {

        if (!(event.getEntity() instanceof ServerPlayer serverPlayer)) {
            return;
        }

        // Leaving the End through the exit portal is also a "respawn" in vanilla.
        if (event.isEndConquered()) {

            LOGGER.info("[BalancedMcHardcoreMain] Player is coming back from the End, using default respawn position");
            return;
        }

        // Computed at death; fall back to the current inventory if the server restarted in between.
        Integer storedDistance = distancesFromBed.remove(serverPlayer.getUUID());
        int distanceFromBed = storedDistance != null ? storedDistance : calculateDistanceFromBed(serverPlayer);

        ServerLevel serverLevel = serverPlayer.serverLevel();

        if (!serverLevel.getGameRules().getBoolean(BMHGameRules.RULE_RANDOM_DEATH_SPAWN_POINT)) {

            LOGGER.info("[BalancedMcHardcoreMain] Game rule randomDeathSpawnPoint is false, using default respawn position");
            return;
        }

        if (serverPlayer.gameMode.getGameModeForPlayer() != GameType.SURVIVAL) {

            LOGGER.info("[BalancedMcHardcoreMain] Player is not in survival mode, using default respawn position");
            return;
        }

        BlockPos bedLocation = serverPlayer.getRespawnPosition();

        if (bedLocation == null || serverPlayer.getRespawnDimension() != serverLevel.dimension()) {

            bedLocation = serverLevel.getSharedSpawnPos();
            LOGGER.info("[BalancedMcHardcoreMain] No bed in this dimension, using world spawn as center");
        }

        BlockPos respawnPosition = calculateRandomPosition(serverLevel, bedLocation, distanceFromBed);

        if (respawnPosition == null) {

            LOGGER.warn("[BalancedMcHardcoreMain] No safe respawn position found after " + MAX_POSITION_ATTEMPTS
                    + " attempts, using default respawn position");
            return;
        }

        LOGGER.info("[BalancedMcHardcoreMain] new respawn pos and bed : " + respawnPosition + ", " + bedLocation);

        // Only move the player: the bed (or anchor) spawn point stays exactly as vanilla left it.
        serverPlayer.teleportTo(respawnPosition.getX() + 0.5, respawnPosition.getY(), respawnPosition.getZ() + 0.5);
    }

    private BlockPos calculateRandomPosition(ServerLevel serverLevel, BlockPos bedPosition, int distanceFromBed) {

        for (int attempt = 0; attempt < MAX_POSITION_ATTEMPTS; attempt++) {

            BlockPos randomEdgePoint = findRandomEdgePoint(serverLevel.getRandom(), bedPosition, distanceFromBed);

            if (!serverLevel.getWorldBorder().isWithinBounds(randomEdgePoint)) {
                continue;
            }

            // Cheap check first: biome lookup does not generate the chunk.
            Holder<Biome> biome = serverLevel.getBiome(randomEdgePoint.atY(serverLevel.getSeaLevel()));

            if (biome.is(BiomeTags.IS_OCEAN) || biome.is(BiomeTags.IS_DEEP_OCEAN) || biome.is(BiomeTags.IS_RIVER)) {
                continue;
            }

            BlockPos safePosition = findSafePosition(serverLevel, randomEdgePoint.getX(), randomEdgePoint.getZ());

            if (safePosition != null) {
                return safePosition;
            }
        }

        return null;
    }

    private int calculateDistanceFromBed(ServerPlayer serverPlayer) {

        int minDistance = Config.minDistanceFromBed.get();
        int maxDistance = Config.maxDistanceFromBed.get();

        int easyLife = 1;
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
        easyLife = Math.min(easyLife, MAX_EASY_LIFE);

        int distanceFromBed = minDistance + (maxDistance - minDistance) * easyLife / MAX_EASY_LIFE; // TODO : add randomness

        LOGGER.info("[BalancedMcHardcoreMain] Distance from bed: " + distanceFromBed);

        return distanceFromBed;
    }

    // Returns the feet position on solid, dry ground with two free blocks above, or null.
    private BlockPos findSafePosition(ServerLevel serverLevel, int x, int z) {

        int minY = serverLevel.getMinBuildHeight();
        int y;

        if (serverLevel.dimensionType().hasCeiling()) {
            // The heightmap would land on the bedrock roof: start under it and get out of the solid layer first.
            y = minY + serverLevel.dimensionType().logicalHeight() - 2;

            while (y > minY && !isFreeSpace(serverLevel, new BlockPos(x, y, z))) {
                y--;
            }
        } else {
            // Level.getHeight answers minBuildHeight for a chunk that is not loaded, and these points are
            // thousands of blocks away: load (or generate) the chunk and read its heightmap directly.
            LevelChunk chunk = serverLevel.getChunk(SectionPos.blockToSectionCoord(x), SectionPos.blockToSectionCoord(z));
            y = chunk.getHeight(Heightmap.Types.MOTION_BLOCKING_NO_LEAVES, x & 15, z & 15) + 1;
        }

        // Go down through free space (air, grass, snow layers) to the first real block: that is the ground.
        // Stopping there means water or lava is rejected instead of digging under it into a cave.
        while (y > minY && isFreeSpace(serverLevel, new BlockPos(x, y, z))) {
            y--;
        }

        BlockPos groundPos = new BlockPos(x, y, z);
        BlockPos feet = groundPos.above();

        if (y > minY
                && isSafeGround(serverLevel, groundPos, serverLevel.getBlockState(groundPos))
                && isFreeSpace(serverLevel, feet)
                && isFreeSpace(serverLevel, feet.above())) {
            return feet;
        }

        LOGGER.info("[BalancedMcHardcoreMain] No safe ground found at X=" + x + ", Z=" + z);

        return null;
    }

    private boolean isFreeSpace(ServerLevel serverLevel, BlockPos pos) {

        BlockState state = serverLevel.getBlockState(pos);

        return state.getCollisionShape(serverLevel, pos).isEmpty() && state.getFluidState().isEmpty();
    }

    private boolean isSafeGround(ServerLevel serverLevel, BlockPos pos, BlockState ground) {

        return ground.getFluidState().isEmpty()
                && ground.isFaceSturdy(serverLevel, pos, Direction.UP)
                && !ground.is(BlockTags.LEAVES)
                && !ground.is(Blocks.MAGMA_BLOCK);
    }

    public static BlockPos findRandomEdgePoint(RandomSource random, BlockPos center, int radius) {

        double theta = 2 * Math.PI * random.nextDouble();

        int xEdge = center.getX() + (int) (radius * Math.cos(theta));
        int zEdge = center.getZ() + (int) (radius * Math.sin(theta));

        return new BlockPos(xEdge, 0, zEdge);
    }

}
