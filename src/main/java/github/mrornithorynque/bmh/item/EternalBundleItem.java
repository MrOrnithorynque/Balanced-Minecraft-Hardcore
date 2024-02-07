package github.mrornithorynque.bmh.item;

import java.util.List;
import java.util.Optional;
import java.util.stream.Stream;

import javax.annotation.Nonnull;

import github.mrornithorynque.bmh.utilities.IEternalItem;
import net.minecraft.ChatFormatting;
import net.minecraft.core.NonNullList;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.ListTag;
import net.minecraft.nbt.Tag;
import net.minecraft.network.chat.Component;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.sounds.SoundEvents;
import net.minecraft.stats.Stats;
import net.minecraft.util.Mth;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.InteractionResultHolder;
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
import net.minecraft.world.item.BlockItem;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.Items;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.ItemUtils;

public class EternalBundleItem extends Item implements IEternalItem {

    private static final String TAG_ITEMS = "Items";
    public static final int MAX_WEIGHT = 64;
    private static final int BUNDLE_IN_BUNDLE_WEIGHT = 4;
    private static final int BAR_COLOR = Mth.color(0.4F, 0.4F, 1.0F);

    public EternalBundleItem(Item.Properties p_150726_) {

        super(p_150726_);
    }

    public static float getFullnessDisplay(ItemStack p_150767_) {

        return (float) getContentWeight(p_150767_) / 64.0F;
    }

    public boolean overrideStackedOnOther(
            @Nonnull ItemStack stack,
            @Nonnull Slot slot,
            @Nonnull ClickAction clickAction,
            @Nonnull Player player) {

        if (stack.getCount() != 1 || clickAction != ClickAction.SECONDARY) {

            return false;
        } else {

            ItemStack itemstack = slot.getItem();
            if (itemstack.isEmpty()) {

                this.playRemoveOneSound(player);
                removeOne(stack).ifPresent((p_150740_) -> {
                    add(stack, slot.safeInsert(p_150740_));
                });
            } else if (itemstack.getItem().canFitInsideContainerItems()) {

                int i = (64 - getContentWeight(stack)) / getWeight(itemstack);
                int j = add(stack, slot.safeTake(itemstack.getCount(), i, player));
                if (j > 0) {

                    this.playInsertSound(player);
                }
            }

            return true;
        }
    }

    public boolean overrideOtherStackedOnMe(
            @Nonnull ItemStack currentStack,
            @Nonnull ItemStack otherStack,
            @Nonnull Slot slot,
            @Nonnull ClickAction clickAction,
            @Nonnull Player player,
            @Nonnull SlotAccess slotAccess) {

        if (currentStack.getCount() != 1)

            return false;
        if (clickAction == ClickAction.SECONDARY && slot.allowModification(player)) {

            if (otherStack.isEmpty()) {

                removeOne(currentStack).ifPresent((p_186347_) -> {
                    this.playRemoveOneSound(player);
                    slotAccess.set(p_186347_);
                });
            } else {

                int i = add(currentStack, otherStack);
                if (i > 0) {

                    this.playInsertSound(player);
                    otherStack.shrink(i);
                }
            }

            return true;
        } else {

            return false;
        }
    }

    public InteractionResultHolder<ItemStack> use(
            @Nonnull Level level,
            @Nonnull Player player,
            @Nonnull InteractionHand interactionHand) {

        ItemStack itemstack = player.getItemInHand(interactionHand);

        if (dropContents(itemstack, player)) {

            this.playDropContentsSound(player);
            player.awardStat(Stats.ITEM_USED.get(this));
            return InteractionResultHolder.sidedSuccess(itemstack, level.isClientSide());
        } else {

            return InteractionResultHolder.fail(itemstack);
        }
    }

    public boolean isBarVisible(@Nonnull ItemStack stack) {

        return getContentWeight(stack) > 0;
    }

    public int getBarWidth(@Nonnull ItemStack stack) {

        return Math.min(1 + 12 * getContentWeight(stack) / 64, 13);
    }

    public int getBarColor(@Nonnull ItemStack stack) {

        return BAR_COLOR;
    }

    private static int add(ItemStack currentStack, ItemStack otherStack) {

        if (!otherStack.isEmpty() && otherStack.getItem().canFitInsideContainerItems()) {

            CompoundTag compoundtag = currentStack.getOrCreateTag();

            if (!compoundtag.contains("Items")) {

                compoundtag.put("Items", new ListTag());
            }

            int i = getContentWeight(currentStack);
            int j = getWeight(otherStack);
            int k = Math.min(otherStack.getCount(), (64 - i) / j);

            if (k == 0) {

                return 0;
            } else {

                ListTag listtag = compoundtag.getList("Items", 10);
                Optional<CompoundTag> optional = getMatchingItem(otherStack, listtag);

                if (optional.isPresent()) {

                    CompoundTag compoundtag1 = optional.get();
                    ItemStack itemstack = ItemStack.of(compoundtag1);
                    itemstack.grow(k);
                    itemstack.save(compoundtag1);
                    listtag.remove(compoundtag1);
                    listtag.add(0, (Tag) compoundtag1);
                } else {

                    ItemStack itemstack1 = otherStack.copyWithCount(k);
                    CompoundTag compoundtag2 = new CompoundTag();
                    itemstack1.save(compoundtag2);
                    listtag.add(0, (Tag) compoundtag2);
                }

                return k;
            }
        } else {
            return 0;
        }
    }

    private static Optional<CompoundTag> getMatchingItem(ItemStack stack, ListTag listTag) {

        return stack.is(Items.BUNDLE) ? Optional.empty()
                : listTag.stream().filter(CompoundTag.class::isInstance).map(CompoundTag.class::cast)
                        .filter((p_186350_) -> {
                            return ItemStack.isSameItemSameTags(ItemStack.of(p_186350_), stack);
                        }).findFirst();
    }

    private static int getWeight(ItemStack stack) {

        if (stack.is(Items.BUNDLE)) {

            return 4 + getContentWeight(stack);
        } else {

            if ((stack.is(Items.BEEHIVE) || stack.is(Items.BEE_NEST)) && stack.hasTag()) {
                CompoundTag compoundtag = BlockItem.getBlockEntityData(stack);
                if (compoundtag != null && !compoundtag.getList("Bees", 10).isEmpty()) {
                    return 64;
                }
            }

            return 64 / stack.getMaxStackSize();
        }
    }

    private static int getContentWeight(ItemStack stack) {

        return getContents(stack).mapToInt((p_186356_) -> {
            return getWeight(p_186356_) * p_186356_.getCount();
        }).sum();
    }

    private static Optional<ItemStack> removeOne(ItemStack stack) {

        CompoundTag compoundtag = stack.getOrCreateTag();
        if (!compoundtag.contains("Items")) {
            return Optional.empty();
        } else {
            ListTag listtag = compoundtag.getList("Items", 10);
            if (listtag.isEmpty()) {
                return Optional.empty();
            } else {
                int i = 0;
                CompoundTag compoundtag1 = listtag.getCompound(0);
                ItemStack itemstack = ItemStack.of(compoundtag1);
                listtag.remove(0);
                if (listtag.isEmpty()) {
                    stack.removeTagKey("Items");
                }

                return Optional.of(itemstack);
            }
        }
    }

    private static boolean dropContents(ItemStack stack, Player player) {

        CompoundTag compoundtag = stack.getOrCreateTag();
        if (!compoundtag.contains("Items")) {

            return false;
        } else {
            if (player instanceof ServerPlayer) {

                ListTag listtag = compoundtag.getList("Items", 10);

                for (int i = 0; i < listtag.size(); ++i) {

                    CompoundTag compoundtag1 = listtag.getCompound(i);
                    ItemStack itemstack = ItemStack.of(compoundtag1);
                    player.drop(itemstack, true);
                }
            }

            stack.removeTagKey("Items");
            return true;
        }
    }

    private static Stream<ItemStack> getContents(ItemStack stack) {

        CompoundTag compoundtag = stack.getTag();
        if (compoundtag == null) {

            return Stream.empty();
        } else {

            ListTag listtag = compoundtag.getList("Items", 10);
            return listtag.stream().map(CompoundTag.class::cast).map(ItemStack::of);
        }
    }

    public Optional<TooltipComponent> getTooltipImage(@Nonnull ItemStack stack) {

        NonNullList<ItemStack> nonnulllist = NonNullList.create();
        getContents(stack).forEach(nonnulllist::add);
        return Optional.of(new BundleTooltip(nonnulllist, getContentWeight(stack)));
    }

    public void appendHoverText(
            @Nonnull ItemStack stack,
            Level level,
            @Nonnull List<Component> p_150751_,
            @Nonnull TooltipFlag p_150752_) {

        p_150751_.add(Component.translatable("item.minecraft.bundle.fullness", getContentWeight(stack), 64)
                .withStyle(ChatFormatting.GRAY));
    }

    public void onDestroyed(@Nonnull ItemEntity itemEntity) {

        ItemUtils.onContainerDestroyed(itemEntity, getContents(itemEntity.getItem()));
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