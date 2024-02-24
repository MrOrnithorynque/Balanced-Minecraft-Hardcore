
package github.mrornithorynque.bmh.handlers;

import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.GameType;
import net.minecraftforge.event.TickEvent.PlayerTickEvent;
import net.minecraftforge.eventbus.api.SubscribeEvent;
import net.minecraftforge.fml.common.Mod;

import org.slf4j.Logger;

import com.mojang.logging.LogUtils;

import github.mrornithorynque.bmh.items.IEternalItem;
import github.mrornithorynque.bmh.utilities.BMHGameRules;

import java.util.ArrayList;
import javax.annotation.Nonnull;

@Mod.EventBusSubscriber
public class EternalItemHandler {

    private static final Logger LOGGER = LogUtils.getLogger();

    @SubscribeEvent
    public static void onPlayerTick(PlayerTickEvent event) {

        Player player = event.player;

        if (player instanceof ServerPlayer) {

            ServerPlayer serverPlayer = (ServerPlayer) player;

            if (!serverPlayer.serverLevel().getGameRules()
                    .getBoolean(BMHGameRules.RULE_ONLY_CARRY_ONE_ETERNAL_ITEM_TYPE)) {
                return;
            }

            if (!(serverPlayer.gameMode.getGameModeForPlayer() == GameType.SURVIVAL)) {
                return;
            }
        }

        if (event.side.isServer() && event.phase == PlayerTickEvent.Phase.END) {

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
    }

    private static boolean containsSameEternalItem(@Nonnull ArrayList<ItemStack> eternalItemList,
            @Nonnull ItemStack eternalItem) {

        for (ItemStack item : eternalItemList) {

            if (item != null && item.getItem() == eternalItem.getItem()) {
                return true;
            }

        }

        return false;
    }
}
