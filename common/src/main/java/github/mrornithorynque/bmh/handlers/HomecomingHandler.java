package github.mrornithorynque.bmh.handlers;

import java.util.ArrayList;
import java.util.List;

import org.jspecify.annotations.Nullable;

import com.mojang.serialization.Codec;
import com.mojang.serialization.codecs.RecordCodecBuilder;

import github.mrornithorynque.bmh.init.SoundInit;
import github.mrornithorynque.bmh.utilities.BMHGameRules;

import net.minecraft.core.BlockPos;
import net.minecraft.network.chat.Component;
import net.minecraft.network.protocol.game.ClientboundSetSubtitleTextPacket;
import net.minecraft.network.protocol.game.ClientboundSetTitleTextPacket;
import net.minecraft.network.protocol.game.ClientboundSetTitlesAnimationPacket;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.sounds.SoundSource;
import net.minecraft.world.level.storage.ValueInput;
import net.minecraft.world.level.storage.ValueOutput;
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

    // The saved form is the same as before 26.1. Every field is required, so every field is always written: an
    // optionalFieldOf with a default would leave out a value equal to the default (a point at z = 0 once lost its z).
    private static final Codec<Home> HOME_CODEC = RecordCodecBuilder.create(instance -> instance.group(
            Codec.STRING.fieldOf("name").forGetter(Home::name),
            Codec.INT.fieldOf("x").forGetter(home -> home.pos().getX()),
            Codec.INT.fieldOf("y").forGetter(home -> home.pos().getY()),
            Codec.INT.fieldOf("z").forGetter(home -> home.pos().getZ()))
            .apply(instance, (name, x, y, z) -> new Home(name, new BlockPos(x, y, z))));
    private static final Codec<Record> RECORD_CODEC = RecordCodecBuilder.create(instance -> instance.group(
            Codec.STRING.fieldOf("dimension").forGetter(Record::dimension),
            Codec.LONG.fieldOf("arrival").forGetter(Record::arrivalTime),
            Codec.INT.fieldOf("distance").forGetter(Record::distance),
            HOME_CODEC.listOf().fieldOf("homes").forGetter(Record::homes))
            .apply(instance, Record::new));

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

        ServerLevel level = player.level();
        List<Home> homes = new ArrayList<>();
        homes.add(new Home("", exile.center()));

        if (level.dimension() == Level.OVERWORLD) {
            for (AmuletPoints.Point point : AmuletPoints.get(player)) {
                homes.add(new Home(point.name(), point.pos()));
            }
        }

        ((Holder) player).bmh$setHomecoming(
                new Record(level.dimension().identifier().toString(), level.getGameTime(), exile.distance(), homes));
    }

    /** The once-a-second check. Cheap when there is no record. */
    public static void tick(ServerPlayer player) {

        Record record = get(player);

        if (record == null || !record.dimension().equals(player.level().dimension().identifier().toString())) {
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

        ServerLevel level = player.level();

        ((Holder) player).bmh$setHomecoming(null);

        // The advancements do not depend on the text rule: the player is home either way.
        BmhAdvancements.award(player, "journey/home_at_last");

        if (record.distance() >= BmhAdvancements.FAR_AWAY_BLOCKS) {
            BmhAdvancements.award(player, "journey/long_way_home");
        }

        // The text follows the same rule as the text of the journey. The sound does not.
        if (level.getGameRules().get(BMHGameRules.RULE_DISPLAY_TEXT_ON_RESPAWN)) {
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

    // Used by the mixin to save the record with the player. No record writes nothing.
    public static void write(@Nullable Record record, ValueOutput output) {

        if (record != null) {
            output.store(TAG, RECORD_CODEC, record);
        }
    }

    // Used by the mixin to load the record, or null.
    @Nullable
    public static Record read(ValueInput input) {

        return input.read(TAG, RECORD_CODEC).orElse(null);
    }
}
