
package github.mrornithorynque.bmh.handlers;

import net.minecraft.world.entity.player.Player;
import net.minecraft.world.item.ItemStack;
import net.minecraftforge.event.TickEvent.PlayerTickEvent;
import net.minecraftforge.eventbus.api.SubscribeEvent;
import net.minecraftforge.fml.common.Mod;
import java.util.HashMap;
import java.util.Map;

import org.slf4j.Logger;

import com.mojang.logging.LogUtils;

import github.mrornithorynque.bmh.items.IEternalItem;

@Mod.EventBusSubscriber
public class EternalItemHandler {

    private static final Map<Player, ItemStack[]> playerInventorySnapshots = new HashMap<>();

    private static final Logger LOGGER = LogUtils.getLogger();

    @SubscribeEvent
    public static void onPlayerTick(PlayerTickEvent event) {
        Player player = event.player;

        LOGGER.info("PlayerTickEvent: " + player.getName().getString() + " " + event.phase.toString() + " "
                + event.side.toString());

        if (event.side.isServer() && event.phase == PlayerTickEvent.Phase.END) {
            ItemStack[] currentInventory = player.getInventory().items.toArray(new ItemStack[0]);
            ItemStack[] previousInventory = playerInventorySnapshots.get(player);

            LOGGER.info("Current inventory: " + currentInventory);

            // Check if this is the first tick snapshot or inventory has changed
            if (previousInventory == null || !areInventoriesIdentical(previousInventory, currentInventory)) {

                playerInventorySnapshots.put(player, currentInventory.clone());

                for (ItemStack newItem : currentInventory) {

                    LOGGER.info("Checking item: " + newItem);

                    // Check if the item is an instance of IEternalItem and is not the same as in
                    // the previous snapshot
                    if (newItem.getItem() instanceof IEternalItem
                            && (previousInventory == null || !containsItem(previousInventory, newItem))) {

                        LOGGER.info("Eternal item found: " + newItem);

                        boolean foundDuplicate = false;

                        // Iterate through the inventory to check for duplicates of the new Eternal item
                        for (ItemStack existingItem : currentInventory) {

                            LOGGER.info("Checking existing item: " + existingItem);

                            if (existingItem != newItem && existingItem.getItem() == newItem.getItem()) {

                                foundDuplicate = true;
                                break;
                            }
                        }

                        if (foundDuplicate) {
                            LOGGER.info("Duplicate found, handling restriction");
                            LOGGER.info("Dropping item: " + newItem);
                            // Handle the restriction, e.g., drop the item, notify the player, etc.
                            // This is a placeholder action
                            player.drop(newItem, false);
                            break; // Assuming only one new item needs to be checked per tick
                        }
                    }
                }
            }
        }
    }

    private static boolean areInventoriesIdentical(ItemStack[] inv1, ItemStack[] inv2) {
        if (inv1.length != inv2.length)
            return false;
        LOGGER.info("Inventory lengths are equal");

        for (int i = 0; i < inv1.length; i++) {
            LOGGER.info("Checking item " + i + ": " + inv1[i] + " " + inv2[i]);
            if (!ItemStack.matches(inv1[i], inv2[i]))
                return false;
            LOGGER.info("Items are equal");
        }

        return true;
    }

    private static boolean containsItem(ItemStack[] inventory, ItemStack item) {
        for (ItemStack invItem : inventory) {
            LOGGER.info("Checking item: " + invItem);
            if (ItemStack.matches(invItem, item))
                return true;
            LOGGER.info("Items are not equal");
        }
        return false;
    }
}
