package github.mrornithorynque.bmh.handlers;

import javax.annotation.Nullable;

import org.slf4j.Logger;

import com.mojang.logging.LogUtils;

import net.minecraft.core.BlockPos;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.Tag;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.util.Mth;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.Level;

/**
 * Star readings of the Navigator's Amulet: when the stars can be read, what a reading writes on the
 * stack (the contract the sky renderer reads) and the charges that limit readings.
 *
 * The contract, in the stack's NBT: the compound bmh_reading with azimuth (float, degrees clockwise
 * from north, from the player to Home) and time (long, game time of the reading). Next to it live
 * bmh_starlight (int, charges left) and bmh_night (long, the night those charges were counted for).
 * A stack that never read has neither charge tag: it is full.
 */
public class AmuletReading {

    public static final int MAX_CHARGES = 3;

    public static final String READING_TAG = "bmh_reading";
    public static final String AZIMUTH_TAG = "azimuth";
    public static final String TIME_TAG = "time";
    public static final String CHARGES_TAG = "bmh_starlight";
    public static final String NIGHT_TAG = "bmh_night";

    private static final long DAY_LENGTH = 24000L;
    // The night index goes up at this time of the day. Reading itself waits for isNight(), a little later.
    private static final long NIGHTFALL = 12000L;

    private static final Logger LOGGER = LogUtils.getLogger();

    /**
     * Degrees clockwise from north, in [0, 360): 0 is north (-Z), 90 is east (+X), 180 south, 270 west.
     * The sky renderer reads the same convention.
     */
    public static float azimuth(double fromX, double fromZ, double toX, double toZ) {

        double degrees = Math.toDegrees(Math.atan2(toX - fromX, fromZ - toZ));
        float azimuth = (float) ((degrees % 360.0 + 360.0) % 360.0);

        // A tiny negative angle rounds up to 360 once it is a float.
        return azimuth >= 360.0F ? 0.0F : azimuth;
    }

    public static void logSelfCheck() {

        float north = azimuth(0, 0, 0, -100);
        float east = azimuth(0, 0, 100, 0);

        LOGGER.info("[BalancedMcHardcoreMain] Azimuth self-check: Home (0, -100) seen from (0, 0) gives " + north
                + " (expected 0), Home (100, 0) gives " + east + " (expected 90)");

        if (north != 0.0F || east != 90.0F) {
            LOGGER.error("[BalancedMcHardcoreMain] Azimuth self-check failed: the sky renderer would point the wrong way");
        }
    }

    /** Goes up by one at each nightfall, so a stack knows which night its charges were counted for. */
    public static long nightIndex(long dayTime) {

        return Math.floorDiv(dayTime + DAY_LENGTH - NIGHTFALL, DAY_LENGTH);
    }

    /**
     * Null when the stars can be read here and now, else the lang key saying why not. isNight() is
     * decided by how dark the sky is, and a thunderstorm darkens a day: rain is refused first.
     */
    @Nullable
    public static String hiddenReason(ServerPlayer player) {

        ServerLevel level = player.serverLevel();

        if (level.dimension() != Level.OVERWORLD) {
            return "bmh.amulet.hidden.dimension";
        }

        if (level.isRaining()) {
            return "bmh.amulet.hidden.rain";
        }

        if (!level.isNight()) {
            return "bmh.amulet.hidden.day";
        }

        if (!level.canSeeSky(BlockPos.containing(player.getEyePosition()))) {
            return "bmh.amulet.hidden.roof";
        }

        return null;
    }

    /** Charges left on the stack. */
    public static int charges(ItemStack stack) {

        CompoundTag tag = stack.getTag();

        if (tag == null || !tag.contains(CHARGES_TAG, Tag.TAG_INT)) {
            return MAX_CHARGES;
        }

        return Mth.clamp(tag.getInt(CHARGES_TAG), 0, MAX_CHARGES);
    }

    /** Charges come back at each nightfall: a stack counted for another night is full again. */
    public static void refill(ItemStack stack, Level level) {

        CompoundTag tag = stack.getTag();

        if (tag == null || !tag.contains(NIGHT_TAG, Tag.TAG_LONG)) {
            return;
        }

        long night = nightIndex(level.getDayTime());

        if (tag.getLong(NIGHT_TAG) != night) {

            tag.putInt(CHARGES_TAG, MAX_CHARGES);
            tag.putLong(NIGHT_TAG, night);
        }
    }

    /**
     * Writes a reading on the stack: the contract, and one charge less (creative readings are free).
     * Home is read here and goes nowhere but into the azimuth: no coordinates, no distance.
     */
    public static void record(ServerPlayer player, ItemStack stack) {

        Level level = player.level();
        BlockPos home = AmuletHome.get(player);
        float azimuth = azimuth(player.getX(), player.getZ(), home.getX() + 0.5, home.getZ() + 0.5);

        CompoundTag reading = new CompoundTag();
        reading.putFloat(AZIMUTH_TAG, azimuth);
        reading.putLong(TIME_TAG, level.getGameTime());

        CompoundTag tag = stack.getOrCreateTag();
        tag.put(READING_TAG, reading);

        if (!player.getAbilities().instabuild) {

            tag.putInt(CHARGES_TAG, charges(stack) - 1);
            tag.putLong(NIGHT_TAG, nightIndex(level.getDayTime()));
        }

        LOGGER.info("[BalancedMcHardcoreMain] Star reading for " + player.getName().getString() + ": azimuth " + azimuth
                + ", " + charges(stack) + " charges left");
    }
}
