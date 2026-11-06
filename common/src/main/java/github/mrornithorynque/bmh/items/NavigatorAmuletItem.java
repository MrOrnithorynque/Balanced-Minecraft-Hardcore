package github.mrornithorynque.bmh.items;

import java.util.function.Consumer;

import github.mrornithorynque.bmh.handlers.AmuletPointSignal;
import github.mrornithorynque.bmh.handlers.AmuletPoints;
import github.mrornithorynque.bmh.handlers.AmuletReading;

import net.minecraft.ChatFormatting;
import net.minecraft.core.BlockPos;
import net.minecraft.network.chat.Component;
import net.minecraft.network.chat.Style;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.sounds.SoundEvents;
import net.minecraft.sounds.SoundSource;
import net.minecraft.tags.BlockTags;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.InteractionResult;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.EquipmentSlot;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.TooltipFlag;
import net.minecraft.world.item.component.TooltipDisplay;
import net.minecraft.world.item.context.UseOnContext;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.block.state.BlockState;

/**
 * A compass that only knows north (the needle is a client item property, see client/AmuletCompass),
 * and a navigator's tool: sneak and use it on a bed or a lodestone to name a point there (or rename or
 * remove it), sneak and use it anywhere else to list the points and remove one, use it at night under open sky to
 * read the stars toward every point and the world spawn. A reading costs one
 * of the 3 starlight charges, back at each nightfall. It has no durability and takes no enchantment.
 */
public class NavigatorAmuletItem extends Item {

    // Amethyst edge, the colour of the sky lines the reading draws.
    private static final int STARLIGHT_COLOR = 0xB58CFF;
    private static final Style STARLIGHT_STYLE = Style.EMPTY.withColor(STARLIGHT_COLOR);

    private static final int READING_COOLDOWN_TICKS = 40;
    // Keeps a held right click from opening the naming screen again and again.
    private static final int POINT_COOLDOWN_TICKS = 20;

    public NavigatorAmuletItem(Item.Properties properties) {

        super(properties);

        AmuletReading.logSelfCheck();
    }

    /**
     * Sneak and use on a bed or a lodestone, in the Overworld: the server opens the naming screen of that block's point.
     * The point changes only when the player confirms the screen (handlers/AmuletPointCommand).
     */
    @Override
    public InteractionResult useOn(UseOnContext context) {

        Player player = context.getPlayer();
        Level level = context.getLevel();
        BlockPos pos = context.getClickedPos();
        BlockState state = level.getBlockState(pos);
        boolean bed = state.is(BlockTags.BEDS);

        // Anything else falls through to use(): a reading. Both sides decide the same here, so the
        // client does not also send a use() for a click that opens the screen.
        if (player == null || !context.isSecondaryUseActive() || !AmuletPoints.canHold(state)) {
            return InteractionResult.PASS;
        }

        if (player instanceof ServerPlayer serverPlayer) {

            if (level.dimension() == Level.OVERWORLD) {

                AmuletPoints.Point point = AmuletPoints.at(serverPlayer, pos, state);
                AmuletPointSignal.open(serverPlayer, pos, point == null ? null : point.name(),
                        AmuletPoints.get(serverPlayer).size(), AmuletPoints.max(), bed);
                serverPlayer.getCooldowns().addCooldown(context.getItemInHand(), POINT_COOLDOWN_TICKS);
            } else {
                serverPlayer.sendOverlayMessage(Component.translatable("bmh.point.overworld"));
            }
        }

        return InteractionResult.SUCCESS;
    }

    /** A star reading. Everything is decided on the server: it writes the reading on the stack and tells the player. */
    @Override
    public InteractionResult use(Level level, Player player, InteractionHand interactionHand) {

        ItemStack itemStack = player.getItemInHand(interactionHand);

        // The client claims the click without deciding anything: on a PASS vanilla goes on with the other hand in the
        // same click (a shield, a bucket, an ender pearl) while the server does the reading.
        if (!(player instanceof ServerPlayer serverPlayer)) {
            return InteractionResult.CONSUME;
        }

        // Sneak and use on anything but a bed or a lodestone: the list of the points, to remove one whose block is gone.
        if (serverPlayer.isSecondaryUseActive()) {

            AmuletPointSignal.list(serverPlayer, AmuletPoints.get(serverPlayer), AmuletPoints.max());
            serverPlayer.getCooldowns().addCooldown(itemStack, POINT_COOLDOWN_TICKS);

            return InteractionResult.SUCCESS_SERVER;
        }

        String hiddenReason = AmuletReading.hiddenReason(serverPlayer);

        if (hiddenReason != null) {

            serverPlayer.sendOverlayMessage(Component.translatable(hiddenReason));
            return InteractionResult.FAIL;
        }

        AmuletReading.refill(itemStack, level);

        if (AmuletReading.charges(itemStack) <= 0 && !serverPlayer.getAbilities().instabuild) {

            serverPlayer.sendOverlayMessage(Component.translatable("bmh.amulet.empty"));
            return InteractionResult.FAIL;
        }

        AmuletReading.record(serverPlayer, itemStack);

        BlockPos pos = serverPlayer.blockPosition();

        level.playSound(null, player.getX(), player.getY(), player.getZ(),
                SoundEvents.AMETHYST_BLOCK_CHIME, SoundSource.PLAYERS, 1.0F, 1.0F);
        serverPlayer.sendOverlayMessage(Component.translatable("bmh.amulet.reading", pos.getX(), pos.getY(), pos.getZ()));
        serverPlayer.getCooldowns().addCooldown(itemStack, READING_COOLDOWN_TICKS);

        return InteractionResult.SUCCESS_SERVER;
    }

    // Once a second on the server: a 1.4.0 amulet sheds its old data, and the charges are back when a new night has begun.
    @Override
    public void inventoryTick(ItemStack stack, ServerLevel level, Entity entity, EquipmentSlot slot) {

        if (level.getGameTime() % 20 == 0) {
            AmuletReading.dropLegacyData(stack);
            AmuletReading.refill(stack, level);
        }
    }

    @Override
    public void appendHoverText(ItemStack stack, Item.TooltipContext context, TooltipDisplay display, Consumer<Component> tooltip,
            TooltipFlag flag) {

        tooltip.accept(Component.translatable("bmh.amulet.starlight", AmuletReading.charges(stack), AmuletReading.MAX_CHARGES)
                .withStyle(STARLIGHT_STYLE));
        tooltip.accept(Component.translatable("bmh.amulet.tooltip.read").withStyle(ChatFormatting.GRAY));
        tooltip.accept(Component.translatable("bmh.amulet.tooltip.home").withStyle(ChatFormatting.GRAY));
        tooltip.accept(Component.translatable("bmh.amulet.tooltip.list").withStyle(ChatFormatting.GRAY));
    }

    // The item bar is the charges, always shown: full is the state the player wants to see.
    @Override
    public boolean isBarVisible(ItemStack stack) {

        return true;
    }

    @Override
    public int getBarWidth(ItemStack stack) {

        return Math.round(13.0F * AmuletReading.charges(stack) / AmuletReading.MAX_CHARGES);
    }

    @Override
    public int getBarColor(ItemStack stack) {

        return STARLIGHT_COLOR;
    }
}
