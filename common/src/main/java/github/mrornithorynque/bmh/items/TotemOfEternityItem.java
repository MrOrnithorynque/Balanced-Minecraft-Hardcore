package github.mrornithorynque.bmh.items;

import java.util.List;

import net.minecraft.ChatFormatting;
import net.minecraft.network.chat.Component;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.TooltipFlag;

public class TotemOfEternityItem extends Item implements IEternalItem {

    public TotemOfEternityItem(Item.Properties properties) {
        super(properties);
    }

    @Override
    public void reduceDurability(ItemStack stack, int percentage) {
        // no need here
    }

    // The glint and the "Eternity" line stand in for an enchantment. It is not one: the item itself
    // is what survives death, so a grindstone has nothing to strip.
    @Override
    public boolean isFoil(ItemStack stack) {
        return true;
    }

    @Override
    public void appendHoverText(ItemStack stack, Item.TooltipContext context, List<Component> lines, TooltipFlag flag) {

        super.appendHoverText(stack, context, lines, flag);
        lines.add(Component.translatable("tooltip.bmh.eternity").withStyle(ChatFormatting.GRAY));
        EternalTooltips.addHeavy(lines);
    }

    // A player carries one totem at most: true cancels the pickup of the item on the ground.
    public static boolean shouldBlockPickup(Player player, ItemStack pickedUpItem) {

        if (pickedUpItem.getItem() instanceof TotemOfEternityItem) {
            for (ItemStack itemStack : player.getInventory().items) {
                if (!itemStack.isEmpty() && (itemStack.getItem().getClass() == pickedUpItem.getItem().getClass())) {

                    return true;
                }
            }
        }

        return false;
    }
}
