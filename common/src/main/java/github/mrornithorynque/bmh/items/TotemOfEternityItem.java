package github.mrornithorynque.bmh.items;

import net.minecraft.world.entity.player.Player;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStack;

public class TotemOfEternityItem extends Item implements IEternalItem {

    public TotemOfEternityItem(Item.Properties properties) {
        super(properties);
    }

    @Override
    public void reduceDurability(ItemStack stack, int percentage) {
        // no need here
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
