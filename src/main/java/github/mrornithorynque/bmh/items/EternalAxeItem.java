package github.mrornithorynque.bmh.items;

import net.minecraft.world.item.AxeItem;
import net.minecraft.world.item.Tier;
import net.minecraftforge.event.entity.player.EntityItemPickupEvent;
import net.minecraftforge.eventbus.api.SubscribeEvent;
import net.minecraft.world.item.ItemStack;
import net.minecraftforge.fml.common.Mod;
import net.minecraft.world.entity.player.Player;

// @Mod.EventBusSubscriber
public class EternalAxeItem extends AxeItem implements IEternalItem {

    public EternalAxeItem(Tier tier, float attackDamage, float attackSpeed, Properties properties) {
        super(tier, attackDamage, attackSpeed, properties);
    }

    // @SubscribeEvent
    // public static void onItemPickup(EntityItemPickupEvent event) {
    //     Player player = (Player) event.getEntity();
    //     ItemStack pickedUpItem = event.getItem().getItem();

    //     if (pickedUpItem.getItem() instanceof EternalAxeItem) {
    //         for (ItemStack itemStack : player.getInventory().items) {
    //             if (!itemStack.isEmpty() && itemStack.getItem().getClass() == pickedUpItem.getItem().getClass()) {
    //                 event.setCanceled(true);
    //                 return;
    //             }
    //         }
    //     }
    // }
}
