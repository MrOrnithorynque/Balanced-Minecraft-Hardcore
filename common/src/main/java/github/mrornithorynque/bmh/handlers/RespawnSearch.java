package github.mrornithorynque.bmh.handlers;

import java.util.HashMap;
import java.util.Map;
import java.util.UUID;

import org.slf4j.Logger;

import com.mojang.logging.LogUtils;

import net.minecraft.core.BlockPos;
import net.minecraft.network.protocol.game.ServerboundClientCommandPacket;
import net.minecraft.resources.ResourceKey;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.server.level.TicketType;
import net.minecraft.util.Util;
import net.minecraft.tags.BlockTags;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;

/**
 * Looks for the landing spot of a distant respawn in the background, from the moment of death. A far chunk can take
 * tens of seconds to generate in a modpack. Loading it inside the respawn froze the whole server for that time.
 * Here the server only asks for the chunk with a ticket and checks it each tick. The respawn request waits
 * (F_ServerGamePacketListenerMixin) while the player sees the journey cinematic.
 * All methods run on the server thread.
 */
public final class RespawnSearch {

    // A chunk that takes longer than this is a broken world generator: the player then gets the vanilla spawn.
    private static final long TIMEOUT_MS = 120_000L;
    // Kept for 5 minutes (6000 ticks), so that the landing chunk is still loaded when the player arrives.
    private static final TicketType TICKET = new TicketType(6000L, TicketType.FLAG_LOADING);

    private static final Logger LOGGER = LogUtils.getLogger();

    private static final Map<UUID, RespawnSearch> searches = new HashMap<>();

    private final ResourceKey<Level> dimension;
    private final BlockPos center;
    private final int distance;
    private final long startedAt = Util.getMillis();
    private int attempts;
    // The point whose chunk is being generated, or null.
    private BlockPos candidate;
    private BlockPos result;
    private boolean failed;
    // The respawn request that arrived before the end of the search.
    private boolean respawnWaiting;

    private RespawnSearch(ResourceKey<Level> dimension, BlockPos center, int distance) {

        this.dimension = dimension;
        this.center = center;
        this.distance = distance;
    }

    /**
     * Called at death, once no other mod can cancel it. It guesses the level and the center that the respawn will
     * use, as vanilla does. If the guess is wrong (a bed that is gone), the respawn searches again.
     */
    static void start(ServerPlayer serverPlayer) {

        Integer worldDistance = RespawnPositionHandler.peekDistance(serverPlayer);

        if (worldDistance == null) {
            return;
        }

        MinecraftServer server = serverPlayer.level().getServer();
        ServerPlayer.RespawnConfig config = serverPlayer.getRespawnConfig();
        ServerLevel level = config != null ? server.getLevel(config.respawnData().dimension()) : null;
        BlockPos bed = config != null ? config.respawnData().pos() : null;

        if (level == null || bed == null || !(config.forced() || isRespawnBlock(level.getBlockState(bed)))) {

            level = server.overworld();
            bed = level.getRespawnData().pos();
        }

        cancel(serverPlayer);

        RespawnSearch search = new RespawnSearch(level.dimension(), bed, RespawnPositionHandler.toLevelBlocks(level, worldDistance));
        searches.put(serverPlayer.getUUID(), search);

        LOGGER.info("[BalancedMcHardcoreMain] Looking for a landing spot in the background");
        search.advance(serverPlayer);
    }

    /** Each tick of the player's connection, which also ticks while the player is dead. */
    public static void tick(ServerPlayer serverPlayer) {

        RespawnSearch search = searches.get(serverPlayer.getUUID());

        if (search == null) {
            return;
        }

        if (!search.isFinished()) {

            if (Util.getMillis() - search.startedAt > TIMEOUT_MS) {

                LOGGER.warn("[BalancedMcHardcoreMain] The landing chunk took more than " + TIMEOUT_MS / 1000
                        + " s, using the default respawn position");
                search.fail(serverPlayer);
            } else {
                search.advance(serverPlayer);
            }
        }

        if (search.isFinished() && search.respawnWaiting) {

            search.respawnWaiting = false;
            // The same request the client sent. This time F_ServerGamePacketListenerMixin lets it through.
            serverPlayer.connection.handleClientCommand(
                    new ServerboundClientCommandPacket(ServerboundClientCommandPacket.Action.PERFORM_RESPAWN));
        }
    }

    /** True when the respawn request of a dead player must wait: the search runs. It answers the request later. */
    public static boolean holdRespawn(ServerPlayer serverPlayer) {

        RespawnSearch search = searches.get(serverPlayer.getUUID());

        if (search == null || search.isFinished() || serverPlayer.getHealth() > 0.0F || serverPlayer.wonGame) {
            return false;
        }

        search.respawnWaiting = true;
        return true;
    }

    /** The player left: drop the search and free its chunk. */
    public static void cancel(ServerPlayer serverPlayer) {

        RespawnSearch search = searches.remove(serverPlayer.getUUID());

        if (search != null) {
            search.releaseTicket(serverPlayer);
        }
    }

    /** Takes the search out for the respawn. A search that did not end gives its chunk back. */
    static RespawnSearch take(ServerPlayer serverPlayer) {

        RespawnSearch search = searches.remove(serverPlayer.getUUID());

        if (search != null && !search.isFinished()) {
            search.releaseTicket(serverPlayer);
        }

        return search;
    }

    boolean isFinished() {

        return this.result != null || this.failed;
    }

    /** The feet position, or null when the search found nothing. */
    BlockPos result() {

        return this.result;
    }

    boolean matches(ServerLevel level, BlockPos center) {

        return this.dimension == level.dimension() && this.center.equals(center);
    }

    private static boolean isRespawnBlock(BlockState state) {

        return state.is(BlockTags.BEDS) || state.is(Blocks.RESPAWN_ANCHOR);
    }

    private void advance(ServerPlayer serverPlayer) {

        ServerLevel level = serverPlayer.level().getServer().getLevel(this.dimension);

        if (level == null) {

            this.failed = true;
            return;
        }

        if (this.candidate != null) {

            // getChunkNow answers only once the chunk is complete, so the check below never waits.
            if (level.getChunkSource().getChunkNow(this.candidate.getX() >> 4, this.candidate.getZ() >> 4) == null) {
                return;
            }

            BlockPos safePosition = RespawnPositionHandler.findSafePosition(level, this.candidate.getX(), this.candidate.getZ());

            if (safePosition != null) {

                // The ticket stays until it expires: the landing chunk is still loaded when the player arrives.
                this.result = safePosition;
                this.candidate = null;
                LOGGER.info("[BalancedMcHardcoreMain] Landing spot ready after " + (Util.getMillis() - this.startedAt) + " ms: "
                        + safePosition);
                return;
            }

            this.releaseTicket(serverPlayer);
        }

        while (this.attempts < RespawnPositionHandler.MAX_POSITION_ATTEMPTS) {

            this.attempts++;
            BlockPos point = RespawnPositionHandler.candidatePoint(level, this.center, this.distance);

            if (point != null) {

                this.candidate = point;
                ChunkPos chunk = ChunkPos.containing(point);
                level.getChunkSource().addTicketWithRadius(TICKET, chunk, 0);
                return;
            }
        }

        this.failed = true;
    }

    private void fail(ServerPlayer serverPlayer) {

        this.releaseTicket(serverPlayer);
        this.failed = true;
    }

    private void releaseTicket(ServerPlayer serverPlayer) {

        if (this.candidate == null) {
            return;
        }

        ServerLevel level = serverPlayer.level().getServer().getLevel(this.dimension);

        if (level != null) {

            ChunkPos chunk = ChunkPos.containing(this.candidate);
            level.getChunkSource().removeTicketWithRadius(TICKET, chunk, 0);
        }

        this.candidate = null;
    }
}
