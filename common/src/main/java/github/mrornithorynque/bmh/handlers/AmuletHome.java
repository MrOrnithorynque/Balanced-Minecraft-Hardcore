package github.mrornithorynque.bmh.handlers;

import javax.annotation.Nullable;

import net.minecraft.core.BlockPos;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.Tag;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.level.Level;

/**
 * The player's Home: the point the Navigator's Amulet reads the stars toward. It lives on the player:
 * mixin.C_ServerPlayerMixin saves it with the rest of the player's data and copies it across death.
 */
public class AmuletHome {

    private static final String HOME_TAG = "bmh_home";

    /** Implemented by ServerPlayer through C_ServerPlayerMixin. */
    public interface Holder {

        @Nullable
        BlockPos bmh$getHome();

        void bmh$setHome(@Nullable BlockPos home);
    }

    /**
     * Home as a reading uses it: the block the player set, else their respawn point if it is in the
     * Overworld, else the world spawn.
     */
    public static BlockPos get(ServerPlayer player) {

        BlockPos home = ((Holder) player).bmh$getHome();

        if (home != null) {
            return home;
        }

        BlockPos respawn = player.getRespawnPosition();

        if (respawn != null && player.getRespawnDimension() == Level.OVERWORLD) {
            return respawn;
        }

        return player.serverLevel().getSharedSpawnPos();
    }

    public static void set(ServerPlayer player, BlockPos pos) {

        ((Holder) player).bmh$setHome(pos.immutable());
    }

    // Used by the mixin to save Home into the player's NBT. No Home, nothing written.
    public static void write(@Nullable BlockPos home, CompoundTag playerTag) {

        if (home == null) {
            return;
        }

        CompoundTag tag = new CompoundTag();
        tag.putInt("x", home.getX());
        tag.putInt("y", home.getY());
        tag.putInt("z", home.getZ());
        playerTag.put(HOME_TAG, tag);
    }

    @Nullable
    public static BlockPos read(CompoundTag playerTag) {

        if (!playerTag.contains(HOME_TAG, Tag.TAG_COMPOUND)) {
            return null;
        }

        CompoundTag tag = playerTag.getCompound(HOME_TAG);

        return new BlockPos(tag.getInt("x"), tag.getInt("y"), tag.getInt("z"));
    }
}
