package github.mrornithorynque.bmh.handlers;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

import org.jspecify.annotations.Nullable;

import com.mojang.serialization.Codec;
import com.mojang.serialization.MapCodec;
import com.mojang.serialization.codecs.RecordCodecBuilder;

import github.mrornithorynque.bmh.platform.Services;

import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.tags.BlockTags;
import net.minecraft.world.level.block.BedBlock;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.block.state.properties.BedPart;
import net.minecraft.world.level.storage.ValueInput;
import net.minecraft.world.level.storage.ValueOutput;

/**
 * The points of the Navigator's Amulet: named beds and lodestones of the Overworld. A star reading points to all of them.
 * They live on the player: mixin.C_ServerPlayerMixin saves them with the rest of the player's data and copies them
 * across death. AmuletPointCommand changes them when the player confirms the naming screen.
 */
public class AmuletPoints {

    public static final int MAX_NAME_LENGTH = 24;

    private static final String POINTS_TAG = "bmh_points";
    // 1.5.0 to 1.6.x kept one Home here. It becomes a point named Home when the player loads, and the next save drops it.
    private static final String LEGACY_HOME_TAG = "bmh_home";
    private static final String LEGACY_HOME_NAME = "Home";

    public record Point(String name, BlockPos pos) {
    }

    // The saved form of a position, the same as before 26.1: x, y and z fields, 0 when one is missing.
    private static final MapCodec<BlockPos> POSITION_MAP_CODEC = RecordCodecBuilder.mapCodec(instance -> instance.group(
            Codec.INT.optionalFieldOf("x", 0).forGetter(BlockPos::getX),
            Codec.INT.optionalFieldOf("y", 0).forGetter(BlockPos::getY),
            Codec.INT.optionalFieldOf("z", 0).forGetter(BlockPos::getZ)).apply(instance, BlockPos::new));
    private static final Codec<BlockPos> POSITION_CODEC = POSITION_MAP_CODEC.codec();
    // The saved form of a point: its name and its position fields in one compound.
    private static final Codec<Point> POINT_CODEC = RecordCodecBuilder.create(instance -> instance.group(
            Codec.STRING.optionalFieldOf("name", "").forGetter(Point::name),
            POSITION_MAP_CODEC.forGetter(Point::pos)).apply(instance, Point::new));

    /** Implemented by ServerPlayer through C_ServerPlayerMixin. */
    public interface Holder {

        List<Point> bmh$getPoints();

        void bmh$setPoints(List<Point> points);
    }

    public static List<Point> get(ServerPlayer player) {

        return List.copyOf(((Holder) player).bmh$getPoints());
    }

    /** The most points a player can keep (mod config). A player who has more keeps them, but cannot add one. */
    public static int max() {

        return Services.PLATFORM.maxAmuletPoints();
    }

    /**
     * The world spawn: the first place of the world. A reading always points to it, besides the named points, with a
     * lighter line and no name. It is not a point: it does not count toward the cap and no one removes it.
     */
    public static BlockPos worldSpawn(ServerPlayer player) {

        return player.level().getRespawnData().pos();
    }

    /** Removes the point at exactly this position, with no block needed (the block may be gone). False when there is none. */
    public static boolean removeAt(ServerPlayer player, BlockPos pos) {

        List<Point> points = new ArrayList<>(((Holder) player).bmh$getPoints());
        boolean removed = points.removeIf(point -> point.pos().equals(pos));

        if (removed) {
            ((Holder) player).bmh$setPoints(points);
        }

        return removed;
    }

    /** True for the blocks that can hold a point. */
    public static boolean canHold(BlockState state) {

        return state.is(BlockTags.BEDS) || state.is(Blocks.LODESTONE);
    }

    /** The block a point keeps: the head of a bed, else the block itself. */
    public static BlockPos anchor(BlockPos pos, BlockState state) {

        if (state.is(BlockTags.BEDS) && state.getValue(BedBlock.PART) == BedPart.FOOT) {
            return pos.relative(state.getValue(BedBlock.FACING)).immutable();
        }

        return pos.immutable();
    }

    /** The point on this block, or null. A bed matches on either half: a Home from 1.6.x kept the half the player clicked. */
    @Nullable
    public static Point at(ServerPlayer player, BlockPos pos, BlockState state) {

        BlockPos other = state.is(BlockTags.BEDS) ? pos.relative(BedBlock.getConnectedDirection(state)) : pos;

        for (Point point : ((Holder) player).bmh$getPoints()) {
            if (point.pos().equals(pos) || point.pos().equals(other)) {
                return point;
            }
        }

        return null;
    }

    /** Renames the point on this block, or adds a new point. The caller checks the cap first. */
    public static void put(ServerPlayer player, BlockPos pos, BlockState state, String name) {

        Point existing = at(player, pos, state);
        List<Point> points = new ArrayList<>(((Holder) player).bmh$getPoints());

        if (existing != null) {
            points.set(points.indexOf(existing), new Point(name, existing.pos()));
        } else {
            points.add(new Point(name, anchor(pos, state)));
        }

        ((Holder) player).bmh$setPoints(points);
    }

    /** Removes the point on this block. False when there is none. */
    public static boolean remove(ServerPlayer player, BlockPos pos, BlockState state) {

        Point existing = at(player, pos, state);

        if (existing == null) {
            return false;
        }

        List<Point> points = new ArrayList<>(((Holder) player).bmh$getPoints());
        points.remove(existing);
        ((Holder) player).bmh$setPoints(points);

        return true;
    }

    /**
     * The name as the server keeps it: no formatting codes, no control characters, no spaces at the ends, at most
     * MAX_NAME_LENGTH characters. An empty result is not a name.
     */
    public static String cleanName(String raw) {

        StringBuilder name = new StringBuilder();

        for (int i = 0; i < raw.length(); i++) {

            char c = raw.charAt(i);

            if (c == '§') {
                // A formatting code is the section sign and the next character: skip both.
                i++;
            } else if (!Character.isISOControl(c)) {
                name.append(c);
            }
        }

        String trimmed = name.toString().trim();

        return trimmed.length() > MAX_NAME_LENGTH ? trimmed.substring(0, MAX_NAME_LENGTH).trim() : trimmed;
    }

    // Used by the mixin to save the points with the player. The list is always written, empty or not.
    public static void write(List<Point> points, ValueOutput output) {

        ValueOutput.TypedOutputList<Point> list = output.list(POINTS_TAG, POINT_CODEC);

        for (Point point : points) {
            list.add(point);
        }
    }

    // Used by the mixin to load the points. A player saved before points existed brings their Home as a point named Home.
    public static List<Point> read(ValueInput input) {

        List<Point> points = new ArrayList<>();
        Optional<ValueInput.TypedInputList<Point>> list = input.list(POINTS_TAG, POINT_CODEC);

        if (list.isPresent()) {
            list.get().stream().forEach(points::add);
        } else {
            input.read(LEGACY_HOME_TAG, POSITION_CODEC).ifPresent(pos -> points.add(new Point(LEGACY_HOME_NAME, pos)));
        }

        return points;
    }
}
