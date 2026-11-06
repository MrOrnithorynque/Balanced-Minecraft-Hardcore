package github.mrornithorynque.bmh.items;

import java.util.Optional;
import java.util.function.Consumer;

import org.apache.commons.lang3.math.Fraction;

import net.minecraft.ChatFormatting;
import net.minecraft.core.component.DataComponents;
import net.minecraft.network.chat.Component;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.sounds.SoundEvents;
import net.minecraft.stats.Stats;
import net.minecraft.util.ARGB;
import net.minecraft.util.Mth;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.InteractionResult;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.SlotAccess;
import net.minecraft.world.entity.item.ItemEntity;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.inventory.ClickAction;
import net.minecraft.world.inventory.Slot;
import net.minecraft.world.inventory.tooltip.BundleTooltip;
import net.minecraft.world.inventory.tooltip.TooltipComponent;
import net.minecraft.world.item.TooltipFlag;
import net.minecraft.world.level.Level;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.Items;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.ItemUtils;
import net.minecraft.world.item.component.BundleContents;
import net.minecraft.world.item.component.TooltipDisplay;

import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

/**
 * Vanilla's bundle, minus what an Eternal Bundle must refuse (see canInsert). The contents live in
 * the same BUNDLE_CONTENTS component as vanilla's, which ItemInit gives the item.
 */
public class EternalBundleItem extends Item implements IEternalItem {

    public static final int MAX_WEIGHT = 64;
    private static final Logger LOGGER = LogManager.getLogger();
    // The color of the 1.21.1 bundle bar: red 0.4, green 0.4, blue 1.0.
    private static final int BAR_COLOR = ARGB.color(102, 102, 255);

    public EternalBundleItem(Item.Properties properties) {

        super(properties);
    }

    @Override
    public void reduceDurability(ItemStack stack, int percentage) {
        // No need here
    }

    public boolean overrideStackedOnOther(
            ItemStack stack,
            Slot slot,
            ClickAction clickAction,
            Player player) {

        if (stack.getCount() != 1 || clickAction != ClickAction.SECONDARY) {
            return false;
        }

        BundleContents.Mutable contents = new BundleContents.Mutable(getContents(stack));
        ItemStack itemStack = slot.getItem();

        if (itemStack.isEmpty()) {

            ItemStack removed = contents.removeOne();

            if (removed != null) {
                this.playRemoveOneSound(player);
                contents.tryInsert(slot.safeInsert(removed));
                LOGGER.info("[BalancedMcHardcoreMain] Removed one item from the stack.");
            }
        } else if (canInsert(itemStack)) {

            int added = contents.tryTransfer(slot, player);

            if (added > 0) {
                this.playInsertSound(player);
                LOGGER.info("[BalancedMcHardcoreMain] Added " + added + " items to the stack.");
            }
        }

        stack.set(DataComponents.BUNDLE_CONTENTS, contents.toImmutable());

        return true;
    }

    public boolean overrideOtherStackedOnMe(
            ItemStack currentStack,
            ItemStack otherStack,
            Slot slot,
            ClickAction clickAction,
            Player player,
            SlotAccess slotAccess) {

        if (currentStack.getCount() != 1) {
            return false;
        }

        if (clickAction == ClickAction.SECONDARY && slot.allowModification(player)) {

            BundleContents.Mutable contents = new BundleContents.Mutable(getContents(currentStack));

            if (otherStack.isEmpty()) {

                ItemStack removed = contents.removeOne();

                if (removed != null) {
                    this.playRemoveOneSound(player);
                    slotAccess.set(removed);
                    LOGGER.info("[BalancedMcHardcoreMain] Removed one item from the stack.");
                }
            } else if (canInsert(otherStack)) {

                // tryInsert takes the items out of otherStack itself.
                int added = contents.tryInsert(otherStack);

                if (added > 0) {
                    this.playInsertSound(player);
                    LOGGER.info("[BalancedMcHardcoreMain] Added " + added + " items to the stack.");
                }
            }

            currentStack.set(DataComponents.BUNDLE_CONTENTS, contents.toImmutable());

            return true;
        }

        return false;
    }

    // Drops everything at once, as the bundle of 1.21.1 did. Vanilla's bundle now drops one item at a time while used.
    @Override
    public InteractionResult use(
            Level level,
            Player player,
            InteractionHand interactionHand) {

        ItemStack stack = player.getItemInHand(interactionHand);

        if (dropContents(stack, player)) {

            this.playDropContentsSound(player);
            player.awardStat(Stats.ITEM_USED.get(this));

            return InteractionResult.SUCCESS;
        } else {
            return InteractionResult.FAIL;
        }
    }

    public boolean isBarVisible(ItemStack stack) {

        return !getContents(stack).isEmpty();
    }

    public int getBarWidth(ItemStack stack) {

        return Math.min(1 + Mth.mulAndTruncate(getWeight(stack), 12), 13);
    }

    public int getBarColor(ItemStack stack) {

        return BAR_COLOR;
    }

    // Must be checked before taking anything out of a slot, otherwise a rejected
    // item is removed from the slot and never put back (issue #2).
    private static boolean canInsert(ItemStack stack) {

        return !stack.isEmpty()
                && stack.getItem().canFitInsideContainerItems()
                && !(stack.getItem() instanceof IEternalItem)
                && stack.getItem() != Items.ENDER_CHEST;
    }

    private static BundleContents getContents(ItemStack stack) {

        return stack.getOrDefault(DataComponents.BUNDLE_CONTENTS, BundleContents.EMPTY);
    }

    // The weight of the contents, where 1 is a full bundle. An invalid weight counts as full, as in vanilla's BundleItem.
    private static Fraction getWeight(ItemStack stack) {

        return getContents(stack).weight().result().orElse(Fraction.ONE);
    }

    private static boolean dropContents(ItemStack stack, Player player) {

        BundleContents contents = getContents(stack);

        if (contents.isEmpty()) {
            return false;
        }

        stack.set(DataComponents.BUNDLE_CONTENTS, BundleContents.EMPTY);

        if (player instanceof ServerPlayer) {
            contents.itemCopyStream().forEach(itemStack -> player.drop(itemStack, true));
        }

        return true;
    }

    public Optional<TooltipComponent> getTooltipImage(ItemStack stack) {

        return Optional.of(new BundleTooltip(getContents(stack)));
    }

    @Override
    public void appendHoverText(
            ItemStack stack,
            Item.TooltipContext context,
            TooltipDisplay display,
            Consumer<Component> componentList,
            TooltipFlag tooltipFlag) {

        componentList.accept(Component.translatable("item.minecraft.bundle.fullness",
                Mth.mulAndTruncate(getWeight(stack), MAX_WEIGHT), MAX_WEIGHT)
                .withStyle(ChatFormatting.GRAY));
        EternalTooltips.addHeavy(componentList);
    }

    public void onDestroyed(ItemEntity itemEntity) {

        BundleContents contents = getContents(itemEntity.getItem());

        itemEntity.getItem().set(DataComponents.BUNDLE_CONTENTS, BundleContents.EMPTY);
        ItemUtils.onContainerDestroyed(itemEntity, contents.itemCopyStream());
    }

    private void playRemoveOneSound(Entity entity) {

        entity.playSound(SoundEvents.BUNDLE_REMOVE_ONE, 0.8F,
                0.8F + entity.level().getRandom().nextFloat() * 0.4F);
    }

    private void playInsertSound(Entity entity) {

        entity.playSound(SoundEvents.BUNDLE_INSERT, 0.8F, 0.8F + entity.level().getRandom().nextFloat() * 0.4F);
    }

    private void playDropContentsSound(Entity entity) {

        entity.playSound(SoundEvents.BUNDLE_DROP_CONTENTS, 0.8F,
                0.8F + entity.level().getRandom().nextFloat() * 0.4F);
    }
}
