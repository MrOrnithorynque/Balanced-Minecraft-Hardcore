package github.mrornithorynque.bmh.items;

import github.mrornithorynque.bmh.init.SoundInit;

import javax.annotation.Nonnull;

import github.mrornithorynque.utilities.HexColor;
import github.mrornithorynque.utilities.TextDrawer;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.sounds.SoundEvents;
import net.minecraft.sounds.SoundSource;

import net.minecraft.world.entity.player.Player;
import net.minecraft.world.level.Level;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.InteractionResult;
import net.minecraft.world.InteractionResultHolder;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.context.UseOnContext;
import net.minecraft.world.item.enchantment.Enchantment;
import net.minecraft.world.item.enchantment.Enchantments;

public class NavigatorAmuletItem extends Item {

    private static final String LAST_USE_TAG = "LastUseTime";
    private static final int COOLDOWN_SECONDS = 5;

    public NavigatorAmuletItem(Item.Properties properties) {

        super(properties);
    }

    @Override
    public boolean canApplyAtEnchantingTable(ItemStack stack, Enchantment enchantment) {
        return enchantment == Enchantments.UNBREAKING;
    }

    @Override
    public InteractionResultHolder<ItemStack> use(@Nonnull Level level, @Nonnull Player player, @Nonnull InteractionHand interactionHand) {

        ItemStack itemStack = player.getItemInHand(interactionHand);

        if (!level.isClientSide) {
            CompoundTag tag = itemStack.getOrCreateTag();

            long currentTime = level.getGameTime();
            long lastUseTime = tag.getLong(LAST_USE_TAG);

            if (currentTime >= lastUseTime + COOLDOWN_SECONDS * 20) {

                tag.putLong(LAST_USE_TAG, currentTime);
                itemStack.setTag(tag);

                if (!player.level().isClientSide) {
                    itemStack.hurtAndBreak(1, player, (p) -> p.broadcastBreakEvent(interactionHand));

                    level.playSeededSound(
                            null,
                            player.blockPosition().getX(),
                            player.blockPosition().getY(),
                            player.blockPosition().getZ(),
                            SoundEvents.ENDER_EYE_DEATH,
                            SoundSource.PLAYERS,
                            1.0F,
                            1.0F,
                            0);

                    TextDrawer.getInstance().drawString(
                            "Current coordinates: " +
                                    player.blockPosition().getX() +
                                    ", " +
                                    player.blockPosition().getY() +
                                    ", " +
                                    player.blockPosition().getZ() +
                                    ".",
                            TextDrawer.ScreenPosition.CENTER,
                            HexColor.WHITE.getValue(),
                            5000);
                }
                return InteractionResultHolder.success(itemStack);
            } else {
                return InteractionResultHolder.fail(itemStack);
            }
        }

        return InteractionResultHolder.pass(itemStack);
    }
}
