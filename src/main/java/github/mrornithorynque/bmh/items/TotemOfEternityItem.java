package github.mrornithorynque.bmh.items;

import net.minecraft.world.entity.player.Player;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStack;

import net.minecraftforge.fml.common.Mod;
import net.minecraftforge.event.entity.player.EntityItemPickupEvent;
import net.minecraftforge.eventbus.api.SubscribeEvent;

@Mod.EventBusSubscriber
public class TotemOfEternityItem extends Item implements IEternalItem {

    public TotemOfEternityItem(Item.Properties properties) {
        super(properties);
    }

    @SubscribeEvent
    public static void onItemPickup(EntityItemPickupEvent event) {
        Player player = (Player) event.getEntity();
        ItemStack pickedUpItem = event.getItem().getItem();

        if (pickedUpItem.getItem() instanceof TotemOfEternityItem) {
            for (ItemStack itemStack : player.getInventory().items) {
                if (!itemStack.isEmpty() && itemStack.getItem().getClass() == pickedUpItem.getItem().getClass()) {
                    event.setCanceled(true);
                    return;
                }
            }
        }
    }
}
