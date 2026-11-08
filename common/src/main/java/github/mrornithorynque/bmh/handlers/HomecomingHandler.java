package github.mrornithorynque.bmh.handlers;

import java.util.ArrayList;
import java.util.List;

import javax.annotation.Nullable;

import github.mrornithorynque.bmh.init.SoundInit;
import github.mrornithorynque.bmh.utilities.BMHGameRules;

import net.minecraft.core.BlockPos;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.ListTag;
import net.minecraft.nbt.Tag;
import net.minecraft.network.chat.Component;
import net.minecraft.network.protocol.game.ClientboundSetSubtitleTextPacket;
import net.minecraft.network.protocol.game.ClientboundSetTitleTextPacket;
import net.minecraft.network.protocol.game.ClientboundSetTitlesAnimationPacket;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.sounds.SoundSource;
import net.minecraft.world.level.Level;

/**
 * The end of an exile. When a player gets back near a home after a distant respawn, the server shows a title with the
 * distance and the time away and rings the bell. Then the record is gone.
 * The homes are fixed at the moment of the exile: the center of the exile and the amulet points the player had then.
 * The record lives on the player: mixin.C_ServerPlayerMixin saves it and copies it across death. The check runs once
 * a second from mixin.E_ServerPlayerMixin. All of it is server side. The client gets vanilla title packets only.
 */
public final class HomecomingHandler {

    /** The title of a home that is the center of the exile. A named point shows its own name. */
    public static final String TITLE_HOME = "bmh.homecoming.home";
    /** The subtitle. Arguments: the blocks and the nights. */
    public static final String SUBTITLE = "bmh.homecoming.subtitle";
    /** The subtitle for exactly one night. Argument: the blocks. */
    public static final String SUBTITLE_ONE_NIGHT = "bmh.homecoming.subtitle.one_night";
    /** The subtitle for less than one night. Argument: the blocks. */
    public static final String SUBTITLE_NO_NIGHTS = "bmh.homecoming.subtitle.no_nights";

    /** A player is home within this horizontal distance of a home, in blocks. */
    public static final int RADIUS = 24;
    /** The check runs this often, in ticks. */
    public static final int CHECK_INTERVAL = 20;

    private static final String TAG = "bmh_homecoming";
    private static final long TICKS_PER_NIGHT = 24000L;
    private static final float BELL_PITCH = 0.8F;

    /** A place that ends the exile. The name is empty for the center of the exile. */
    public record Home(String name, BlockPos pos) {
    }

    /** The exile a player is on: the dimension, the game time of the arrival, the distance and the homes. */
    public record Record(String dimension, long arrivalTime, int distance, List<Home> homes) {
    }

    /** Implemented by ServerPlayer through C_ServerPlayerMixin. */
    public interface Holder {

        @Nullable
        Record bmh$getHomecoming();

        void bmh$setHomecoming(@Nullable Record record);
    }

    private HomecomingHandler() {
    }

    @Nullable
    public static Record get(ServerPlayer player) {

        return ((Holder) player).bmh$getHomecoming();
    }

    /**
     * Called after a distant respawn. A new exile replaces the old record. The center is where the exile was measured
     * from. The amulet points are Overworld points, so they count only when the exile is in the Overworld.
     */
    public static void onExile(ServerPlayer player, RespawnPositionHandler.Exile exile) {

        ServerLevel level = player.serverLevel();
        List<Home> homes = new ArrayList<>();
        homes.add(new Home("", exile.center()));

        if (level.dimension() == Level.OVERWORLD) {
            for (AmuletPoints.Point point : AmuletPoints.get(player)) {
                homes.add(new Home(point.name(), point.pos()));
            }
        }

        ((Holder) player).bmh$setHomecoming(
                new Record(level.dimension().location().toString(), level.getGameTime(), exile.distance(), homes));
    }

    /** The once-a-second check. Cheap when there is no record. */
    public static void tick(ServerPlayer player) {

        Record record = get(player);

        if (record == null || !record.dimension().equals(player.serverLevel().dimension().location().toString())) {
            return;
        }

        Home home = reached(record, player.getX(), player.getZ());

        if (home != null) {
            arrive(player, record, home);
        }
    }

    /** The nearest home within the radius, or null. The first home wins a tie. */
    @Nullable
    public static Home reached(Record record, double x, double z) {

        Home nearest = null;
        double nearestSquared = (double) RADIUS * RADIUS;

        for (Home home : record.homes()) {

            double dx = x - (home.pos().getX() + 0.5D);
            double dz = z - (home.pos().getZ() + 0.5D);
            double squared = dx * dx + dz * dz;

            if (squared <= nearestSquared) {
                nearest = home;
                nearestSquared = squared - 1.0E-9D;
            }
        }

        return nearest;
    }

    /** Whole nights between the arrival of the exile and now. */
    public static long nights(Record record, long gameTime) {

        return Math.max(0L, (gameTime - record.arrivalTime()) / TICKS_PER_NIGHT);
    }

    private static void arrive(ServerPlayer player, Record record, Home home) {

        ServerLevel level = player.serverLevel();

        ((Holder) player).bmh$setHomecoming(null);

        // The text follows the same rule as the text of the journey. The sound does not.
        if (level.getGameRules().getBoolean(BMHGameRules.RULE_DISPLAY_TEXT_ON_RESPAWN)) {
            showTitle(player, record, home, nights(record, level.getGameTime()));
        }

        // No excluded player: the player hears the bell too. A client plays no sound of its own for this.
        level.playSound(null, player.blockPosition(), SoundInit.BELL_SOUND.get(), SoundSource.PLAYERS, 1.0F, BELL_PITCH);
    }

    private static void showTitle(ServerPlayer player, Record record, Home home, long nights) {

        // A fake player of another mod can have no connection.
        if (player.connection == null) {
            return;
        }

        String blocks = String.valueOf(record.distance());
        Component subtitle;

        if (nights >= 2) {
            subtitle = Component.translatableWithFallback(SUBTITLE, "%s blocks and %s nights since you left", blocks, nights);
        } else if (nights == 1) {
            subtitle = Component.translatableWithFallback(SUBTITLE_ONE_NIGHT, "%s blocks and 1 night since you left", blocks);
        } else {
            subtitle = Component.translatableWithFallback(SUBTITLE_NO_NIGHTS, "%s blocks since you left", blocks);
        }

        Component title = home.name().isEmpty()
                ? Component.translatableWithFallback(TITLE_HOME, "Home")
                : Component.literal(home.name());

        // Same order as the /title command: the title packet is what makes the client show both.
        player.connection.send(new ClientboundSetTitlesAnimationPacket(10, 70, 20));
        player.connection.send(new ClientboundSetSubtitleTextPacket(subtitle));
        player.connection.send(new ClientboundSetTitleTextPacket(title));
    }

    // Used by the mixin to save the record into the player's NBT. No record writes nothing.
    public static void write(@Nullable Record record, CompoundTag playerTag) {

        if (record == null) {
            return;
        }

        CompoundTag tag = new CompoundTag();
        tag.putString("dimension", record.dimension());
        tag.putLong("arrival", record.arrivalTime());
        tag.putInt("distance", record.distance());

        ListTag homes = new ListTag();

        for (Home home : record.homes()) {

            CompoundTag homeTag = new CompoundTag();
            homeTag.putString("name", home.name());
            homeTag.putInt("x", home.pos().getX());
            homeTag.putInt("y", home.pos().getY());
            homeTag.putInt("z", home.pos().getZ());
            homes.add(homeTag);
        }

        tag.put("homes", homes);
        playerTag.put(TAG, tag);
    }

    // Used by the mixin to load the record, or null.
    @Nullable
    public static Record read(CompoundTag playerTag) {

        if (!playerTag.contains(TAG, Tag.TAG_COMPOUND)) {
            return null;
        }

        CompoundTag tag = playerTag.getCompound(TAG);
        ListTag list = tag.getList("homes", Tag.TAG_COMPOUND);
        List<Home> homes = new ArrayList<>();

        for (int i = 0; i < list.size(); i++) {

            CompoundTag homeTag = list.getCompound(i);
            homes.add(new Home(homeTag.getString("name"),
                    new BlockPos(homeTag.getInt("x"), homeTag.getInt("y"), homeTag.getInt("z"))));
        }

        return new Record(tag.getString("dimension"), tag.getLong("arrival"), tag.getInt("distance"), homes);
    }
}
