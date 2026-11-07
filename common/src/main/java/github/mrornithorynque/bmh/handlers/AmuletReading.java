package github.mrornithorynque.bmh.handlers;

import java.util.Map;

import javax.annotation.Nullable;

import org.slf4j.Logger;

import com.mojang.logging.LogUtils;

import net.minecraft.core.BlockPos;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.FloatTag;
import net.minecraft.nbt.ListTag;
import net.minecraft.nbt.StringTag;
import net.minecraft.nbt.Tag;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.tags.FluidTags;
import net.minecraft.util.Mth;
import net.minecraft.world.effect.MobEffects;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.enchantment.Enchantment;
import net.minecraft.world.item.enchantment.EnchantmentHelper;
import net.minecraft.world.item.enchantment.Enchantments;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.block.Blocks;

/**
 * Star readings of the Navigator's Amulet: when the stars can be read, what a reading writes on the
 * stack (the contract the sky renderer reads) and the charges that limit readings.
 *
 * The contract, in the stack's NBT: the compound bmh_reading with azimuths (a list of floats, one per
 * point of the amulet, see AmuletPoints, then one for the world spawn: degrees clockwise from north, from the
 * player to the place), names (a list of strings in the same order, empty for the world spawn: the spyglass
 * shows them), spawn (int, the index of the world spawn in both lists; a reading from 1.7.0 or older has no
 * such tag) and time (long, game time of the reading). Next to it live
 * bmh_starlight (int, charges left) and bmh_night (long, the night those charges were counted for).
 * A stack that never read has neither charge tag: it is full.
 */
public class AmuletReading {

    public static final int MAX_CHARGES = 3;

    public static final String READING_TAG = "bmh_reading";
    public static final String AZIMUTHS_TAG = "azimuths";
    public static final String NAMES_TAG = "names";
    public static final String SPAWN_TAG = "spawn";
    public static final String TIME_TAG = "time";
    public static final String CHARGES_TAG = "bmh_starlight";
    public static final String NIGHT_TAG = "bmh_night";

    // 1.4.0's amulet (10 durability, Unbreaking allowed) stamped each use here.
    private static final String LEGACY_LAST_USE_TAG = "LastUseTime";
    private static final String LEGACY_DAMAGE_TAG = "Damage";
    private static final String ENCHANTMENTS_TAG = "Enchantments";

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
     * decided by how dark the sky is, and a thunderstorm darkens a day: rain is refused first. A sky out of
     * sight counts as hidden too (see sightBlocked): nothing is drawn then, so no charge is spent.
     */
    @Nullable
    public static String hiddenReason(ServerPlayer player) {

        return skyHiddenReason(player, "bmh.amulet.hidden");
    }

    /**
     * The same test for any sky reader (the moon ritual of the Dormant Totem uses it too), on either side.
     * The answer is the prefix followed by .dimension, .rain, .day or .roof, or null when the sky is in sight.
     */
    @Nullable
    public static String skyHiddenReason(Player player, String prefix) {

        Level level = player.level();

        if (level.dimension() != Level.OVERWORLD) {
            return prefix + ".dimension";
        }

        if (level.isRaining()) {
            return prefix + ".rain";
        }

        if (!level.isNight()) {
            return prefix + ".day";
        }

        if (!level.canSeeSky(BlockPos.containing(player.getEyePosition())) || sightBlocked(player)) {
            return prefix + ".roof";
        }

        return null;
    }

    /**
     * The cases where the sky renderer draws no sky for the player: a blinding effect, eyes in water or lava,
     * eyes inside powder snow. The refusal reuses the roof message: the sky is out of sight.
     */
    private static boolean sightBlocked(Player player) {

        return player.hasEffect(MobEffects.BLINDNESS)
                || player.hasEffect(MobEffects.DARKNESS)
                || player.isEyeInFluid(FluidTags.WATER)
                || player.isEyeInFluid(FluidTags.LAVA)
                || player.level().getBlockState(BlockPos.containing(player.getEyePosition())).is(Blocks.POWDER_SNOW);
    }

    /** Charges left on the stack. */
    public static int charges(ItemStack stack) {

        CompoundTag tag = stack.getTag();

        if (tag == null || !tag.contains(CHARGES_TAG, Tag.TAG_INT)) {
            return MAX_CHARGES;
        }

        return Mth.clamp(tag.getInt(CHARGES_TAG), 0, MAX_CHARGES);
    }

    /**
     * An amulet carried over from 1.4.0 drops what that version wrote and this one never reads: its wear,
     * its last-use stamp, and Unbreaking or Mending, which do nothing without durability. Curses stay.
     */
    public static void dropLegacyData(ItemStack stack) {

        CompoundTag tag = stack.getTag();

        if (tag == null || !(tag.contains(LEGACY_LAST_USE_TAG) || tag.contains(LEGACY_DAMAGE_TAG) || tag.contains(ENCHANTMENTS_TAG))) {
            return;
        }

        tag.remove(LEGACY_LAST_USE_TAG);
        tag.remove(LEGACY_DAMAGE_TAG);

        Map<Enchantment, Integer> enchantments = EnchantmentHelper.getEnchantments(stack);
        boolean unbreaking = enchantments.remove(Enchantments.UNBREAKING) != null;
        boolean mending = enchantments.remove(Enchantments.MENDING) != null;

        if (unbreaking || mending) {
            EnchantmentHelper.setEnchantments(enchantments, stack);
        }

        if (tag.isEmpty()) {
            stack.setTag(null);
        }
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
     * The points are read here and go nowhere but into the azimuths and their names: no coordinates, no distance.
     */
    public static void record(ServerPlayer player, ItemStack stack) {

        Level level = player.level();
        ListTag azimuths = new ListTag();
        ListTag names = new ListTag();

        for (AmuletPoints.Point target : AmuletPoints.get(player)) {
            azimuths.add(FloatTag.valueOf(azimuth(player.getX(), player.getZ(), target.pos().getX() + 0.5, target.pos().getZ() + 0.5)));
            names.add(StringTag.valueOf(target.name()));
        }

        // The world spawn comes last, with no name. Its index tells the sky renderer which line is the lighter one.
        BlockPos spawn = AmuletPoints.worldSpawn(player);
        int spawnIndex = azimuths.size();
        azimuths.add(FloatTag.valueOf(azimuth(player.getX(), player.getZ(), spawn.getX() + 0.5, spawn.getZ() + 0.5)));
        names.add(StringTag.valueOf(""));

        CompoundTag reading = new CompoundTag();
        reading.put(AZIMUTHS_TAG, azimuths);
        reading.put(NAMES_TAG, names);
        reading.putInt(SPAWN_TAG, spawnIndex);
        reading.putLong(TIME_TAG, level.getGameTime());

        CompoundTag tag = stack.getOrCreateTag();
        tag.put(READING_TAG, reading);

        if (!player.getAbilities().instabuild) {

            tag.putInt(CHARGES_TAG, charges(stack) - 1);
            tag.putLong(NIGHT_TAG, nightIndex(level.getDayTime()));
        }

        BmhAdvancements.award(player, "navigation/stargazer");

        LOGGER.info("[BalancedMcHardcoreMain] Star reading for " + player.getName().getString() + ": azimuths " + azimuths
                + ", " + charges(stack) + " charges left");
    }
}
