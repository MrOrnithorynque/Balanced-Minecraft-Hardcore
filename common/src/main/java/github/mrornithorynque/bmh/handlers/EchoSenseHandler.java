package github.mrornithorynque.bmh.handlers;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.function.Predicate;

import org.joml.Vector3f;
import org.slf4j.Logger;

import com.mojang.logging.LogUtils;

import com.mojang.math.Transformation;

import github.mrornithorynque.bmh.BalancedMcHardcore;
import github.mrornithorynque.bmh.mixin.E_BlockDisplayInvoker;
import github.mrornithorynque.bmh.mixin.E_DisplayInvoker;

import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.Identifier;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.sounds.SoundEvents;
import net.minecraft.sounds.SoundSource;
import net.minecraft.tags.BlockTags;
import net.minecraft.tags.TagKey;
import net.minecraft.world.entity.Display;
import net.minecraft.world.entity.EntityType;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.phys.AABB;

/**
 * Echo Sense, the Eternal Pickaxe's ability: breaking an ore makes the same ore within 8 blocks glow
 * through walls for 5 seconds. The glow is made of block display entities the server spawns, so the
 * client needs no code.
 *
 * A display is saved with its chunk: a player who quits within 5 seconds would leave the ore glowing
 * for good. Every display therefore carries the scoreboard tags {@code bmh_echo} and
 * {@code bmh_echo_until_<expiry game time>}, and {@code E_DisplayMixin} discards it from
 * {@code Display.tick} once that time has passed or its ore is gone, which also covers displays
 * reloaded from disk.
 *
 * At most 64 displays live around one spot at a time (the ones within reach of a new reading are counted,
 * and a display met again only gets a longer glow, so a vein mined for a while keeps glowing a little
 * longer than 5 seconds).
 */
public class EchoSenseHandler {

    private static final Logger LOGGER = LogUtils.getLogger();

    /** Every ore Echo Sense reacts to. Lives in data/bmh/tags/block (data/bmh/tags/blocks before 1.21). */
    public static final TagKey<Block> ECHO_SENSE_ORES = TagKey.create(Registries.BLOCK,
            Identifier.fromNamespaceAndPath(BalancedMcHardcore.MODID, "echo_sense_ores"));

    /** Marks a display as Echo Sense's; {@code UNTIL_PREFIX + expiry game time} says when it ends. */
    public static final String ECHO_TAG = "bmh_echo";
    private static final String UNTIL_PREFIX = "bmh_echo_until_";

    private static final int RADIUS = 8;
    private static final int MAX_DISPLAYS = 64;
    private static final int LIFETIME_TICKS = 100;
    // Ticks between two readings of the same player: a mod that breaks a whole vein at once reads once.
    public static final int COOLDOWN_TICKS = 4;
    private static final int GLOW_COLOR = 0x00F0FF;

    // "The same ore" is the same vanilla ore tag, so the stone and deepslate variants light up together.
    // Nether quartz ore and ancient debris have no such tag: they match their own block (nether gold ore is
    // in #minecraft:gold_ores, so it lights the gold ores).
    private static final List<TagKey<Block>> ORE_FAMILIES = List.of(
            BlockTags.COAL_ORES,
            BlockTags.COPPER_ORES,
            BlockTags.IRON_ORES,
            BlockTags.GOLD_ORES,
            BlockTags.REDSTONE_ORES,
            BlockTags.LAPIS_ORES,
            BlockTags.DIAMOND_ORES,
            BlockTags.EMERALD_ORES);

    private record Ore(BlockPos pos, BlockState state) {
    }

    /** Logs once at server start how many blocks the tag holds: a tag in the wrong folder loads empty, without an error. */
    public static void logTagSize() {

        int size = BuiltInRegistries.BLOCK.get(ECHO_SENSE_ORES).map(tag -> tag.size()).orElse(-1);

        if (size > 0) {
            LOGGER.info("[BalancedMcHardcoreMain] Echo Sense: tag {} lists {} blocks.", ECHO_SENSE_ORES.location(), size);
        } else {
            LOGGER.error("[BalancedMcHardcoreMain] Echo Sense: tag {} is empty or missing, the Eternal Pickaxe will find nothing.",
                    ECHO_SENSE_ORES.location());
        }
    }

    /**
     * Called after a player broke {@code pos} with the Eternal Pickaxe.
     *
     * @param broken what stood at {@code pos} before the break
     */
    public static void onOreBroken(ServerPlayer player, BlockPos pos, BlockState broken) {

        if (!broken.is(ECHO_SENSE_ORES)) {
            return;
        }

        ServerLevel level = player.level();
        long expiry = level.getGameTime() + LIFETIME_TICKS;

        // Displays already shining around here only need a longer glow, not a twin.
        Map<Long, Display.BlockDisplay> shining = new HashMap<>();

        for (Display.BlockDisplay display : level.getEntitiesOfClass(Display.BlockDisplay.class,
                new AABB(pos).inflate(RADIUS + 1), entity -> entity.entityTags().contains(ECHO_TAG))) {
            shining.put(display.blockPosition().asLong(), display);
        }

        int budget = Math.max(0, MAX_DISPLAYS - shining.size());
        int spawned = 0;
        int extended = 0;

        for (Ore ore : findSameOre(level, pos, sameOre(broken))) {

            Display.BlockDisplay display = shining.get(ore.pos().asLong());

            if (display != null) {
                setExpiry(display, expiry);
                extended++;
            } else if (budget > 0) {
                spawn(level, ore, expiry);
                budget--;
                spawned++;
            }
        }

        if (spawned > 0) {
            level.playSound(null, pos.getX() + 0.5D, pos.getY() + 0.5D, pos.getZ() + 0.5D,
                    SoundEvents.AMETHYST_BLOCK_CHIME, SoundSource.PLAYERS, 0.8F, 0.7F);
        }

        LOGGER.debug("[BalancedMcHardcoreMain] Echo Sense: {} lit {} ores of the same kind, {} already shining.",
                player.getGameProfile().name(), spawned, extended);
    }

    /** Called from {@code Display.tick} on both sides: discards an Echo Sense display whose time is up or whose ore is gone. */
    public static void discardIfExpired(Display display) {

        if (display.level().isClientSide()) {
            return;
        }

        Set<String> tags = display.entityTags();

        if (tags.isEmpty() || !tags.contains(ECHO_TAG)) {
            return;
        }

        // It ends with its time, or with its ore: a lit ore that was mined must not stay behind as a ghost.
        if (display.level().getGameTime() >= expiryOf(tags)
                || !display.level().getBlockState(display.blockPosition()).is(ECHO_SENSE_ORES)) {
            display.discard();
        }
    }

    private static Predicate<BlockState> sameOre(BlockState broken) {

        for (TagKey<Block> family : ORE_FAMILIES) {
            if (broken.is(family)) {
                return state -> state.is(family);
            }
        }

        Block block = broken.getBlock();

        return state -> state.is(block);
    }

    // Every position within RADIUS blocks of the center that holds the same ore, nearest first.
    private static List<Ore> findSameOre(ServerLevel level, BlockPos center, Predicate<BlockState> sameOre) {

        List<Ore> found = new ArrayList<>();
        BlockPos.MutableBlockPos cursor = new BlockPos.MutableBlockPos();

        for (int dx = -RADIUS; dx <= RADIUS; dx++) {
            for (int dy = -RADIUS; dy <= RADIUS; dy++) {
                for (int dz = -RADIUS; dz <= RADIUS; dz++) {

                    if (dx * dx + dy * dy + dz * dz > RADIUS * RADIUS) {
                        continue;
                    }

                    cursor.set(center.getX() + dx, center.getY() + dy, center.getZ() + dz);

                    // Never load a chunk for a glow.
                    if (!level.isLoaded(cursor)) {
                        continue;
                    }

                    BlockState state = level.getBlockState(cursor);

                    if (sameOre.test(state)) {
                        found.add(new Ore(cursor.immutable(), state));
                    }
                }
            }
        }

        found.sort(Comparator.comparingDouble(ore -> ore.pos().distSqr(center)));

        return found;
    }

    private static void spawn(ServerLevel level, Ore ore, long expiry) {

        Display.BlockDisplay display = new Display.BlockDisplay(EntityType.BLOCK_DISPLAY, level);

        // A block display draws its block from the entity's position to one block further.
        display.snapTo(ore.pos().getX(), ore.pos().getY(), ore.pos().getZ(), 0.0F, 0.0F);

        ((E_BlockDisplayInvoker) display).bmh$setBlockState(ore.state());
        ((E_DisplayInvoker) display).bmh$setGlowColorOverride(GLOW_COLOR);
        // Drawn a hair inside its block. An entity inside an opaque block is lit with the block's own light,
        // which is none: at full size it would turn an exposed ore dark and shimmer against the real faces.
        // Inside, the real ore keeps its look and the glow outline still shows through walls.
        ((E_DisplayInvoker) display).bmh$setTransformation(
                new Transformation(new Vector3f(0.02F, 0.02F, 0.02F), null, new Vector3f(0.96F, 0.96F, 0.96F), null));

        display.setGlowingTag(true);
        display.addTag(ECHO_TAG);
        setExpiry(display, expiry);

        level.addFreshEntity(display);
    }

    private static void setExpiry(Display display, long expiry) {

        for (String tag : new ArrayList<>(display.entityTags())) {
            if (tag.startsWith(UNTIL_PREFIX)) {
                display.removeTag(tag);
            }
        }

        display.addTag(UNTIL_PREFIX + expiry);
    }

    // A display without a readable expiry is a leftover: it counts as expired.
    private static long expiryOf(Set<String> tags) {

        for (String tag : tags) {
            if (tag.startsWith(UNTIL_PREFIX)) {
                try {
                    return Long.parseLong(tag.substring(UNTIL_PREFIX.length()));
                } catch (NumberFormatException e) {
                    return 0L;
                }
            }
        }

        return 0L;
    }
}
