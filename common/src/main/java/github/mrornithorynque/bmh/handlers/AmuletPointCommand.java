package github.mrornithorynque.bmh.handlers;

import java.util.HashMap;
import java.util.Map;
import java.util.UUID;

import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.arguments.IntegerArgumentType;
import com.mojang.brigadier.arguments.StringArgumentType;
import com.mojang.brigadier.context.CommandContext;
import com.mojang.brigadier.exceptions.CommandSyntaxException;

import github.mrornithorynque.bmh.items.NavigatorAmuletItem;

import net.minecraft.commands.CommandSourceStack;
import net.minecraft.commands.Commands;
import net.minecraft.commands.arguments.coordinates.BlockPosArgument;
import net.minecraft.core.BlockPos;
import net.minecraft.network.chat.Component;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.sounds.SoundEvents;
import net.minecraft.sounds.SoundSource;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.block.state.BlockState;

/**
 * The command the naming screen sends when the player confirms it: /bmh point save x y z name, or
 * /bmh point remove x y z (both on a bed or a lodestone), and from the list screen /bmh point delete x y z (no block
 * needed). BMH has no network channel, so the screen sends a vanilla command packet
 * (client/amulet/PointScreen). A player can also type the command. So the server checks everything
 * here, as it would check a packet: an amulet in hand, the Overworld, a bed or a lodestone within reach,
 * a clean name, the cap, and one change per second at most. mixin.J_CommandsMixin registers it.
 */
public class AmuletPointCommand {

    // The screen opens on a click: the player stands next to the block. A little more than vanilla reach.
    private static final double MAX_DISTANCE = 8.0;
    private static final long MIN_TICKS_BETWEEN_CHANGES = 20;

    private static final Map<UUID, Long> LAST_CHANGE = new HashMap<>();

    public static void register(CommandDispatcher<CommandSourceStack> dispatcher) {

        dispatcher.register(Commands.literal("bmh").then(Commands.literal("point")
                .then(Commands.literal("save").then(Commands.argument("pos", BlockPosArgument.blockPos())
                        .then(Commands.argument("name", StringArgumentType.greedyString())
                                .executes(AmuletPointCommand::save))))
                .then(Commands.literal("remove").then(Commands.argument("pos", BlockPosArgument.blockPos())
                        .executes(AmuletPointCommand::remove)))
                .then(Commands.literal("delete").then(Commands.argument("x", IntegerArgumentType.integer())
                        .then(Commands.argument("y", IntegerArgumentType.integer())
                                .then(Commands.argument("z", IntegerArgumentType.integer())
                                        .executes(AmuletPointCommand::delete)))))));
    }

    /**
     * Removes the point at these coordinates from the list screen (client/amulet/PointListScreen). It needs no block:
     * the bed or the lodestone of the point may be gone, and its chunk may be far away and not loaded. It touches
     * only the player's own points, so it needs an amulet in hand and nothing else. The server sends the list again.
     */
    private static int delete(CommandContext<CommandSourceStack> context) throws CommandSyntaxException {

        ServerPlayer player = context.getSource().getPlayerOrException();
        BlockPos pos = new BlockPos(IntegerArgumentType.getInteger(context, "x"), IntegerArgumentType.getInteger(context, "y"),
                IntegerArgumentType.getInteger(context, "z"));

        if (!holdsAmulet(player)) {
            player.displayClientMessage(Component.translatable("bmh.point.no_amulet"), true);
            return 0;
        }

        AmuletPoints.Point point = null;

        for (AmuletPoints.Point candidate : AmuletPoints.get(player)) {
            if (candidate.pos().equals(pos)) {
                point = candidate;
            }
        }

        if (point == null || !AmuletPoints.removeAt(player, pos)) {
            return 0;
        }

        player.displayClientMessage(Component.translatable("bmh.point.removed", point.name()), true);
        AmuletPointSignal.list(player, AmuletPoints.get(player), AmuletPoints.max());

        return 1;
    }

    private static boolean holdsAmulet(ServerPlayer player) {

        return player.getItemInHand(InteractionHand.MAIN_HAND).getItem() instanceof NavigatorAmuletItem
                || player.getItemInHand(InteractionHand.OFF_HAND).getItem() instanceof NavigatorAmuletItem;
    }

    private static int save(CommandContext<CommandSourceStack> context) throws CommandSyntaxException {

        ServerPlayer player = context.getSource().getPlayerOrException();
        BlockPos pos = BlockPosArgument.getLoadedBlockPos(context, "pos");

        if (!allowed(player, pos)) {
            return 0;
        }

        String name = AmuletPoints.cleanName(StringArgumentType.getString(context, "name"));

        if (name.isEmpty()) {
            player.displayClientMessage(Component.translatable("bmh.point.no_name"), true);
            return 0;
        }

        BlockState state = player.serverLevel().getBlockState(pos);
        boolean renamed = AmuletPoints.at(player, pos, state) != null;
        int used = AmuletPoints.get(player).size();

        if (!renamed && used >= AmuletPoints.max()) {
            player.displayClientMessage(Component.translatable("bmh.point.full", AmuletPoints.max()), true);
            return 0;
        }

        AmuletPoints.put(player, pos, state, name);
        LAST_CHANGE.put(player.getUUID(), player.serverLevel().getGameTime());
        player.serverLevel().playSound(null, player.getX(), player.getY(), player.getZ(),
                SoundEvents.AMETHYST_BLOCK_CHIME, SoundSource.PLAYERS, 1.0F, 0.7F);
        player.displayClientMessage(Component.translatable(renamed ? "bmh.point.renamed" : "bmh.point.saved", name), true);

        return 1;
    }

    private static int remove(CommandContext<CommandSourceStack> context) throws CommandSyntaxException {

        ServerPlayer player = context.getSource().getPlayerOrException();
        BlockPos pos = BlockPosArgument.getLoadedBlockPos(context, "pos");

        if (!allowed(player, pos)) {
            return 0;
        }

        BlockState state = player.serverLevel().getBlockState(pos);
        AmuletPoints.Point point = AmuletPoints.at(player, pos, state);

        if (point == null || !AmuletPoints.remove(player, pos, state)) {
            return 0;
        }

        LAST_CHANGE.put(player.getUUID(), player.serverLevel().getGameTime());
        player.displayClientMessage(Component.translatable("bmh.point.removed", point.name()), true);

        return 1;
    }

    // The checks that a save and a remove share. A refused change tells the player why, on the action bar.
    private static boolean allowed(ServerPlayer player, BlockPos pos) {

        if (!holdsAmulet(player)) {
            player.displayClientMessage(Component.translatable("bmh.point.no_amulet"), true);
            return false;
        }

        if (player.serverLevel().dimension() != Level.OVERWORLD) {
            player.displayClientMessage(Component.translatable("bmh.point.overworld"), true);
            return false;
        }

        if (!player.isAlive() || player.isSpectator()
                || player.distanceToSqr(pos.getX() + 0.5, pos.getY() + 0.5, pos.getZ() + 0.5) > MAX_DISTANCE * MAX_DISTANCE
                || !AmuletPoints.canHold(player.serverLevel().getBlockState(pos))) {
            player.displayClientMessage(Component.translatable("bmh.point.too_far"), true);
            return false;
        }

        Long last = LAST_CHANGE.get(player.getUUID());

        return last == null || player.serverLevel().getGameTime() - last >= MIN_TICKS_BETWEEN_CHANGES
                || player.serverLevel().getGameTime() < last;
    }
}
