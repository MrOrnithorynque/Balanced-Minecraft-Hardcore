
package github.mrornithorynque.bmh.handlers;

import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.GameType;

import org.slf4j.Logger;

import com.mojang.logging.LogUtils;

import github.mrornithorynque.bmh.items.IEternalItem;
import github.mrornithorynque.bmh.items.TotemOfEternityItem;
import github.mrornithorynque.bmh.utilities.BMHGameRules;

import java.util.ArrayList;

public class EternalItemHandler {

    private static final Logger LOGGER = LogUtils.getLogger();

    // Called at the end of every server player tick.
    public static void onPlayerTick(ServerPlayer serverPlayer) {

        Player player = serverPlayer;

        TotemOfEternityHandler.dropExtraTotems(serverPlayer);

        if (!ruleApplies(serverPlayer))
            return;

        ArrayList<ItemStack> playerEternalItems = new ArrayList<>();

        for (int i = 0; i < player.getInventory().getContainerSize(); i++) {

            ItemStack newItem = player.getInventory().getItem(i);

            if (newItem.getItem() instanceof IEternalItem) {

                if (containsSameEternalItem(playerEternalItems, newItem)) {

                    player.drop(newItem.copy(), false);
                    newItem.shrink(1);

                    if (newItem.isEmpty()) {
                        player.getInventory().setItem(i, ItemStack.EMPTY);
                    }
                } else {

                    playerEternalItems.add(newItem.copy());
                }
            }
        }
    }

    // A player carries one copy of each Eternal Item: true cancels the pickup of the item on the ground.
    // The totem is always one per player. The other items follow the rule of onPlayerTick, or the tick
    // drops the copy, the player picks it up again, and this repeats forever.
    public static boolean shouldBlockPickup(Player player, ItemStack pickedUpItem) {

        if (!(pickedUpItem.getItem() instanceof IEternalItem))
            return false;

        if (!(pickedUpItem.getItem() instanceof TotemOfEternityItem)
                && !(player instanceof ServerPlayer serverPlayer && ruleApplies(serverPlayer)))
            return false;

        for (int i = 0; i < player.getInventory().getContainerSize(); i++) {

            if (player.getInventory().getItem(i).is(pickedUpItem.getItem())) {
                return true;
            }
        }

        return false;
    }

    private static boolean ruleApplies(ServerPlayer serverPlayer) {

        return serverPlayer.level().getGameRules().get(BMHGameRules.RULE_ONLY_CARRY_ONE_ETERNAL_ITEM_TYPE)
                && serverPlayer.gameMode.getGameModeForPlayer() == GameType.SURVIVAL;
    }

    private static boolean containsSameEternalItem(ArrayList<ItemStack> eternalItemList,
            ItemStack eternalItem) {

        for (ItemStack item : eternalItemList) {

            if (item != null && item.getItem() == eternalItem.getItem()) {
                return true;
            }

        }

        return false;
    }
}
