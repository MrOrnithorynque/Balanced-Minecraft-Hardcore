package github.mrornithorynque.bmh.handlers;

import net.minecraft.world.entity.player.Player;
import net.minecraft.world.item.ItemStack;
import net.minecraftforge.event.entity.player.PlayerEvent;
import net.minecraftforge.eventbus.api.SubscribeEvent;
import net.minecraftforge.fml.common.Mod;

public class KeepItemAfterDeath {

    @SubscribeEvent
    public static void onPlayerClone(PlayerEvent.Clone event) {

        if (event.isWasDeath()) {

            Player originalPlayer = event.getOriginal();
            Player newPlayer = (Player) event.getEntity();

            for (int i = 0; i < originalPlayer.getInventory().getContainerSize(); i++) {

                ItemStack stack = originalPlayer.getInventory().getItem(i);

                if (isItemToKeep(stack)) {

                    originalPlayer.getInventory().removeItemNoUpdate(i);

                    newPlayer.getInventory().add(stack);
                }
            }
        }
    }

    private static boolean isItemToKeep(ItemStack stack) {

        // Implement your logic to determine if this is the item that should be kept
        // For example, check for a specific item:
        // return stack.getItem() == Items.YOUR_SPECIAL_ITEM;
        // Or check for a tag or NBT data that you've assigned to the item.

        return false;
    }
}