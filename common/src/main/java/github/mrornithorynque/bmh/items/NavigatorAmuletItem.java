package github.mrornithorynque.bmh.items;

import java.util.List;

import javax.annotation.Nonnull;

import github.mrornithorynque.bmh.handlers.AmuletHome;
import github.mrornithorynque.bmh.handlers.AmuletReading;

import net.minecraft.ChatFormatting;
import net.minecraft.core.BlockPos;
import net.minecraft.network.chat.Component;
import net.minecraft.network.chat.Style;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.sounds.SoundEvents;
import net.minecraft.sounds.SoundSource;
import net.minecraft.tags.BlockTags;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.InteractionResult;
import net.minecraft.world.InteractionResultHolder;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.TooltipFlag;
import net.minecraft.world.item.context.UseOnContext;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;

/**
 * A compass that only knows north (the needle is a client item property, see client/AmuletCompass),
 * and a navigator's tool: sneak and use it on a bed or a lodestone to set Home, use it at night under
 * open sky to read the stars toward Home. A reading costs one of the 3 starlight charges, back at
 * each nightfall. It has no durability and takes no enchantment.
 */
public class NavigatorAmuletItem extends Item {

    // Amethyst edge, the colour of the sky lines the reading draws.
    private static final int STARLIGHT_COLOR = 0xB58CFF;
    private static final Style STARLIGHT_STYLE = Style.EMPTY.withColor(STARLIGHT_COLOR);

    private static final int READING_COOLDOWN_TICKS = 40;
    // Keeps a held right click from setting Home, with its chime, four times a second.
    private static final int HOME_COOLDOWN_TICKS = 20;

    public NavigatorAmuletItem(Item.Properties properties) {

        super(properties);

        AmuletReading.logSelfCheck();
    }

    /** Sneak and use on a bed or a lodestone, in the Overworld: that block is now Home. */
    @Override
    public InteractionResult useOn(@Nonnull UseOnContext context) {

        Player player = context.getPlayer();
        Level level = context.getLevel();
        BlockPos pos = context.getClickedPos();
        BlockState state = level.getBlockState(pos);
        boolean bed = state.is(BlockTags.BEDS);

        // Anything else falls through to use(): a reading. Both sides decide the same here, so the
        // client does not also send a use() for a click that just set Home.
        if (player == null || !context.isSecondaryUseActive() || !(bed || state.is(Blocks.LODESTONE))) {
            return InteractionResult.PASS;
        }

        if (player instanceof ServerPlayer serverPlayer) {

            if (level.dimension() == Level.OVERWORLD) {

                AmuletHome.set(serverPlayer, pos);
                level.playSound(null, player.getX(), player.getY(), player.getZ(),
                        SoundEvents.AMETHYST_BLOCK_CHIME, SoundSource.PLAYERS, 1.0F, 0.7F);
                serverPlayer.displayClientMessage(
                        Component.translatable(bed ? "bmh.amulet.home_set.bed" : "bmh.amulet.home_set.lodestone"), true);
                serverPlayer.getCooldowns().addCooldown(this, HOME_COOLDOWN_TICKS);
            } else {
                serverPlayer.displayClientMessage(Component.translatable("bmh.amulet.home_overworld"), true);
            }
        }

        return InteractionResult.sidedSuccess(level.isClientSide);
    }

    /** A star reading. Everything is decided on the server: it writes the reading on the stack and tells the player. */
    @Override
    public InteractionResultHolder<ItemStack> use(@Nonnull Level level, @Nonnull Player player, @Nonnull InteractionHand interactionHand) {

        ItemStack itemStack = player.getItemInHand(interactionHand);

        if (!(player instanceof ServerPlayer serverPlayer)) {
            return InteractionResultHolder.pass(itemStack);
        }

        String hiddenReason = AmuletReading.hiddenReason(serverPlayer);

        if (hiddenReason != null) {

            serverPlayer.displayClientMessage(Component.translatable(hiddenReason), true);
            return InteractionResultHolder.fail(itemStack);
        }

        AmuletReading.refill(itemStack, level);

        if (AmuletReading.charges(itemStack) <= 0 && !serverPlayer.getAbilities().instabuild) {

            serverPlayer.displayClientMessage(Component.translatable("bmh.amulet.empty"), true);
            return InteractionResultHolder.fail(itemStack);
        }

        AmuletReading.record(serverPlayer, itemStack);

        BlockPos pos = serverPlayer.blockPosition();

        level.playSound(null, player.getX(), player.getY(), player.getZ(),
                SoundEvents.AMETHYST_BLOCK_CHIME, SoundSource.PLAYERS, 1.0F, 1.0F);
        serverPlayer.displayClientMessage(Component.translatable("bmh.amulet.reading", pos.getX(), pos.getY(), pos.getZ()), true);
        serverPlayer.getCooldowns().addCooldown(this, READING_COOLDOWN_TICKS);

        return InteractionResultHolder.success(itemStack);
    }

    // Once a second on the server: the charges are back when a new night has begun.
    @Override
    public void inventoryTick(@Nonnull ItemStack stack, @Nonnull Level level, @Nonnull Entity entity, int slot, boolean selected) {

        if (!level.isClientSide && level.getGameTime() % 20 == 0) {
            AmuletReading.refill(stack, level);
        }
    }

    @Override
    public void appendHoverText(@Nonnull ItemStack stack, @Nonnull Item.TooltipContext context, @Nonnull List<Component> tooltip, @Nonnull TooltipFlag flag) {

        tooltip.add(Component.translatable("bmh.amulet.starlight", AmuletReading.charges(stack), AmuletReading.MAX_CHARGES)
                .withStyle(STARLIGHT_STYLE));
        tooltip.add(Component.translatable("bmh.amulet.tooltip.read").withStyle(ChatFormatting.GRAY));
        tooltip.add(Component.translatable("bmh.amulet.tooltip.home").withStyle(ChatFormatting.GRAY));
    }

    // The item bar is the charges, always shown: full is the state the player wants to see.
    @Override
    public boolean isBarVisible(@Nonnull ItemStack stack) {

        return true;
    }

    @Override
    public int getBarWidth(@Nonnull ItemStack stack) {

        return Math.round(13.0F * AmuletReading.charges(stack) / AmuletReading.MAX_CHARGES);
    }

    @Override
    public int getBarColor(@Nonnull ItemStack stack) {

        return STARLIGHT_COLOR;
    }
}
