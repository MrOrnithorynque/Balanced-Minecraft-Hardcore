package github.mrornithorynque.bmh.handlers;

import java.util.HashMap;
import java.util.HashSet;
import java.util.Map;
import java.util.Set;
import java.util.UUID;

import org.slf4j.Logger;

import com.mojang.logging.LogUtils;

import github.mrornithorynque.bmh.items.IEternalItem;
import github.mrornithorynque.bmh.utilities.BMHGameRules;
import github.mrornithorynque.bmh.platform.Services;

import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.core.Holder;
import net.minecraft.core.SectionPos;
import net.minecraft.tags.BiomeTags;
import net.minecraft.tags.BlockTags;

import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;

import net.minecraft.util.Mth;
import net.minecraft.util.RandomSource;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.GameType;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.border.WorldBorder;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.level.levelgen.Heightmap;

public class RespawnPositionHandler {

    private static final int MAX_EASY_LIFE = 9;
    private static final int MAX_POSITION_ATTEMPTS = 32;

    private static final Logger LOGGER = LogUtils.getLogger();

    // One entry per player, filled on death and consumed on respawn.
    private static final Map<UUID, Integer> distancesFromBed = new HashMap<>();

    // The players that got a distant respawn position, until the respawn effects read it.
    private static final Set<UUID> exiledPlayers = new HashSet<>();

    // The center and the distance of each of those exiles, for the homecoming.
    private static final Map<UUID, Exile> exiles = new HashMap<>();

    /** Where an exile was measured from, and how far it went in Overworld blocks. */
    public record Exile(BlockPos center, int distance) {
    }

    // Called first on death, while the Eternal Items are still in the inventory.
    public static void onPlayerDeath(ServerPlayer serverPlayer) {

        distancesFromBed.put(serverPlayer.getUUID(), calculateDistanceFromBed(serverPlayer));
    }

    // The conditions of a distant respawn. The game rules are the same in every level.
    public static boolean isDistantRespawn(ServerPlayer serverPlayer) {

        return serverPlayer.level().getGameRules().get(BMHGameRules.RULE_RANDOM_DEATH_SPAWN_POINT)
                && serverPlayer.gameMode.getGameModeForPlayer() == GameType.SURVIVAL;
    }

    // True once for a player that got a distant respawn position.
    public static boolean takeExiled(ServerPlayer serverPlayer) {

        return exiledPlayers.remove(serverPlayer.getUUID());
    }

    // The exile of a player that got a distant respawn position, once. Call it with takeExiled.
    public static Exile takeExile(ServerPlayer serverPlayer) {

        return exiles.remove(serverPlayer.getUUID());
    }

    /**
     * Called by F_PlayerListRespawnMixin while vanilla respawns a player, before vanilla sends a position to the
     * client. On 1.21.1 and 26.1 the player is the old player, because the new one does not exist yet. On 1.20.x it is the new
     * player. Returns the feet position of the distant respawn in respawnLevel, or null to keep the vanilla position.
     */
    public static BlockPos findRespawnPosition(ServerPlayer serverPlayer, ServerLevel respawnLevel,
            boolean missingRespawnBlock, boolean endConquered) {

        // Leaving the End through the exit portal is also a "respawn" in vanilla.
        if (endConquered) {

            LOGGER.info("[BalancedMcHardcoreMain] Player is coming back from the End, using default respawn position");
            return null;
        }

        // Computed at death; fall back to the current inventory if the server restarted in between.
        Integer storedDistance = distancesFromBed.remove(serverPlayer.getUUID());
        int distanceFromBed = storedDistance != null ? storedDistance : calculateDistanceFromBed(serverPlayer);

        if (!respawnLevel.getGameRules().get(BMHGameRules.RULE_RANDOM_DEATH_SPAWN_POINT)) {

            LOGGER.info("[BalancedMcHardcoreMain] Game rule random_death_spawn_point is false, using default respawn position");
            return null;
        }

        if (serverPlayer.gameMode.getGameModeForPlayer() != GameType.SURVIVAL) {

            LOGGER.info("[BalancedMcHardcoreMain] Player is not in survival mode, using default respawn position");
            return null;
        }

        // Vanilla forgets a bed or anchor that is gone, so the exile is then around the world spawn.
        ServerPlayer.RespawnConfig respawnConfig = missingRespawnBlock ? null : serverPlayer.getRespawnConfig();
        BlockPos bedLocation = respawnConfig != null ? respawnConfig.respawnData().pos() : null;

        if (bedLocation == null || respawnConfig.respawnData().dimension() != respawnLevel.dimension()) {

            bedLocation = respawnLevel.getRespawnData().pos();
            LOGGER.info("[BalancedMcHardcoreMain] No bed in this dimension, using world spawn as center");
        }

        int exileDistance = toLevelBlocks(respawnLevel, distanceFromBed);
        BlockPos respawnPosition = calculateRandomPosition(respawnLevel, bedLocation, exileDistance);

        if (respawnPosition == null) {

            LOGGER.warn("[BalancedMcHardcoreMain] No safe respawn position found after " + MAX_POSITION_ATTEMPTS
                    + " attempts, using default respawn position");
            return null;
        }

        LOGGER.info("[BalancedMcHardcoreMain] new respawn pos and bed : " + respawnPosition + ", " + bedLocation);

        exiledPlayers.add(serverPlayer.getUUID());
        exiles.put(serverPlayer.getUUID(), new Exile(bedLocation.immutable(), distanceFromBed));

        // Only the position changes: the bed (or anchor) spawn point stays exactly as vanilla left it.
        return respawnPosition;
    }

    // The distance is measured in Overworld blocks, but a Nether block covers 8 of them. Dividing by the
    // respawn level's coordinate scale makes an anchor exile as far in world terms as an Overworld one.
    private static int toLevelBlocks(ServerLevel serverLevel, int worldDistance) {

        double coordinateScale = serverLevel.dimensionType().coordinateScale();
        // Capped at the farthest a world border can reach: a data pack with a tiny scale would overflow an int.
        int levelDistance = Mth.floor(Mth.clamp(worldDistance / coordinateScale + 0.5D, 0.0D, WorldBorder.MAX_CENTER_COORDINATE));

        LOGGER.info("[BalancedMcHardcoreMain] Exile distance: " + worldDistance + " world blocks, " + levelDistance
                + " blocks in " + serverLevel.dimension().identifier() + " (coordinate scale " + coordinateScale + ")");

        return levelDistance;
    }

    private static BlockPos calculateRandomPosition(ServerLevel serverLevel, BlockPos bedPosition, int distanceFromBed) {

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

    private static int calculateDistanceFromBed(ServerPlayer serverPlayer) {

        int minDistance = Services.PLATFORM.minDistanceFromBed();
        int maxDistance = Services.PLATFORM.maxDistanceFromBed();

        int easyLife = 1;
        int eternalItemCount = 0;

        if (minDistance > maxDistance) {
            LOGGER.warn("[BalancedMcHardcoreMain] minDistanceFromBed is greater than maxDistanceFromBed, using minDistanceFromBed for both");
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
    private static BlockPos findSafePosition(ServerLevel serverLevel, int x, int z) {

        int minY = serverLevel.getMinY();
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

    private static boolean isFreeSpace(ServerLevel serverLevel, BlockPos pos) {

        BlockState state = serverLevel.getBlockState(pos);

        return state.getCollisionShape(serverLevel, pos).isEmpty() && state.getFluidState().isEmpty();
    }

    private static boolean isSafeGround(ServerLevel serverLevel, BlockPos pos, BlockState ground) {

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
