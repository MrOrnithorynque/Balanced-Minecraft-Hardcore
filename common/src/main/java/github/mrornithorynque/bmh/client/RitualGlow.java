package github.mrornithorynque.bmh.client;

import net.minecraft.client.Minecraft;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.item.ItemStack;

/**
 * The Dormant Totem shimmers like an enchanted item while a moon ritual charges it. The renderer asks the item for its
 * glint with the stack only, so this looks for a player who is using that very stack.
 */
public final class RitualGlow {

    private RitualGlow() {
    }

    public static boolean isCharging(ItemStack stack) {

        Minecraft minecraft = Minecraft.getInstance();

        if (minecraft.level == null) {
            return false;
        }

        for (Player player : minecraft.level.players()) {
            if (player.isUsingItem() && player.getUseItem() == stack) {
                return true;
            }
        }

        return false;
    }
}
