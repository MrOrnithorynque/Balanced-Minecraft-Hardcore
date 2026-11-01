package github.mrornithorynque.bmh.items;

import github.mrornithorynque.bmh.init.SoundInit;

import javax.annotation.Nonnull;

import github.mrornithorynque.bmh.client.LocalPlayerText;
import github.mrornithorynque.bmh.platform.Services;
import net.minecraft.Util;
import net.minecraft.core.Holder;
import net.minecraft.core.component.DataComponents;
import net.minecraft.network.chat.contents.TranslatableContents;
import net.minecraft.sounds.SoundEvents;
import net.minecraft.sounds.SoundSource;

import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.level.Level;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.InteractionResult;
import net.minecraft.world.InteractionResultHolder;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.component.CustomData;
import net.minecraft.world.item.context.UseOnContext;
import net.minecraft.world.item.enchantment.Enchantment;
import net.minecraft.world.item.enchantment.Enchantments;

public class NavigatorAmuletItem extends Item {

    private static final String LAST_USE_TAG = "LastUseTime";
    private static final int COOLDOWN_SECONDS = 5;

    public NavigatorAmuletItem(Item.Properties properties) {

        super(properties);
    }

    // The anvil asks through mixin.EnchantmentMixin, on both loaders; on Forge this is also IForgeItem's
    // enchanting table hook. No @Override: the method does not exist in vanilla.
    public boolean canApplyAtEnchantingTable(ItemStack stack, Enchantment enchantment) {
        // Enchantments are registry data since 1.21 and this one comes without its key: the vanilla
        // description is the translation key made from it.
        return enchantment.description().getContents() instanceof TranslatableContents contents
                && contents.getKey().equals(Util.makeDescriptionId("enchantment", Enchantments.UNBREAKING.location()));
    }

    // NeoForge's anvil asks the item (IItemExtension), not Enchantment.canEnchant, so the mixin never
    // sees it there. No @Override: the method does not exist in vanilla.
    public boolean supportsEnchantment(ItemStack stack, Holder<Enchantment> enchantment) {
        return enchantment.is(Enchantments.UNBREAKING);
    }

    @Override
    public InteractionResultHolder<ItemStack> use(@Nonnull Level level, @Nonnull Player player, @Nonnull InteractionHand interactionHand) {

        ItemStack itemStack = player.getItemInHand(interactionHand);

        if (!level.isClientSide) {
            long currentTime = level.getGameTime();
            long lastUseTime = itemStack.getOrDefault(DataComponents.CUSTOM_DATA, CustomData.EMPTY).copyTag().getLong(LAST_USE_TAG);

            if (currentTime >= lastUseTime + COOLDOWN_SECONDS * 20) {

                CustomData.update(DataComponents.CUSTOM_DATA, itemStack, tag -> tag.putLong(LAST_USE_TAG, currentTime));

                if (!player.level().isClientSide) {
                    itemStack.hurtAndBreak(1, player, LivingEntity.getSlotForHand(interactionHand));

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

                    if (Services.PLATFORM.isPhysicalClient()) {
                        LocalPlayerText.showToLocalPlayer(player, "Current coordinates: " +
                                player.blockPosition().getX() +
                                ", " +
                                player.blockPosition().getY() +
                                ", " +
                                player.blockPosition().getZ() +
                                ".");
                    }
                }
                return InteractionResultHolder.success(itemStack);
            } else {
                return InteractionResultHolder.fail(itemStack);
            }
        }

        return InteractionResultHolder.pass(itemStack);
    }
}
