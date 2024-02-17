package github.mrornithorynque.bmh.items;

import net.minecraft.world.entity.player.Player;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.SwordItem;
import net.minecraft.world.item.Tier;

import net.minecraftforge.fml.common.Mod;
import net.minecraftforge.event.entity.player.EntityItemPickupEvent;
import net.minecraftforge.eventbus.api.SubscribeEvent;

// @Mod.EventBusSubscriber
public class EternalSwordItem extends SwordItem implements IEternalItem {

    public EternalSwordItem(Tier tier, int attackDamage, float attackSpeed, Properties properties) {
        super(tier, attackDamage, attackSpeed, properties);
    }

    // @SubscribeEvent
    // public static void onItemPickup(EntityItemPickupEvent event) {
    //     Player player = (Player) event.getEntity();
    //     ItemStack pickedUpItem = event.getItem().getItem();

    //     if (pickedUpItem.getItem() instanceof EternalSwordItem) {
    //         for (ItemStack itemStack : player.getInventory().items) {
    //             if (!itemStack.isEmpty() && itemStack.getItem().getClass() == pickedUpItem.getItem().getClass()) {
    //                 event.setCanceled(true);
    //                 return;
    //             }
    //         }
    //     }
    // }
}
