package github.mrornithorynque.bmh.handlers;

import java.util.ArrayList;
import java.util.UUID;

import org.slf4j.Logger;

import java.util.HashMap;
import java.util.Map;
import java.util.List;

import com.mojang.logging.LogUtils;

import github.mrornithorynque.bmh.init.BMHModTierInit;
import github.mrornithorynque.bmh.items.IEternalItem;
import net.minecraft.world.entity.player.Player;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.Tier;
import net.minecraft.world.item.TieredItem;
import net.minecraft.world.level.GameType;
import net.minecraft.world.item.Item;

import net.minecraftforge.eventbus.api.SubscribeEvent;
import net.minecraftforge.event.entity.living.LivingDeathEvent;
import net.minecraftforge.event.entity.player.PlayerEvent;

import net.minecraftforge.fml.common.Mod;

@Mod.EventBusSubscriber
public class KeepItemAfterDeath {

    private static final Logger LOGGER = LogUtils.getLogger();

    private static final Map<UUID, List<ItemStack>> itemsToKeep = new HashMap<>();

    @SubscribeEvent
    public static void onPlayerDeath(LivingDeathEvent event) {
        if (event.getEntity() instanceof ServerPlayer) {

            ServerPlayer serverPlayer = (ServerPlayer) event.getEntity();
            UUID playerId = serverPlayer.getUUID();
            List<ItemStack> savedItems = new ArrayList<>();

            for (int i = 0; i < serverPlayer.getInventory().getContainerSize(); i++) {
                ItemStack stack = serverPlayer.getInventory().getItem(i);

                if (isItemToKeep(stack)) {

                    if (serverPlayer.gameMode.getGameModeForPlayer() == GameType.SURVIVAL) {

                        IEternalItem eternalItem = (IEternalItem) stack.getItem();
                        eternalItem.reduceDurability(stack, 7);
                    }

                    savedItems.add(stack.copy());
                    serverPlayer.getInventory().removeItem(stack);
                }
            }

            if (!savedItems.isEmpty()) {
                itemsToKeep.put(playerId, savedItems);
            }
        }
    }

    @SubscribeEvent
    public static void onPlayerRespawn(PlayerEvent.PlayerRespawnEvent event) {

        Player player = (Player) event.getEntity();
        UUID playerId = player.getUUID();

        if (itemsToKeep.containsKey(playerId)) {
            List<ItemStack> savedItems = itemsToKeep.get(playerId);

            for (ItemStack stack : savedItems) {

                if (stack != null) {
                    player.getInventory().add(stack);
                }
            }

            itemsToKeep.remove(playerId);
        }
    }

    private static boolean isItemToKeep(ItemStack stack) {

        Item item = stack.getItem();

        return item instanceof IEternalItem;

        // if (item instanceof TieredItem) {

        //     TieredItem tieredItem = (TieredItem) item;
        //     Tier tier = tieredItem.getTier();

        //     return tier == BMHModTierInit.ETERNAL;
        // }

        // if (item instanceof IEternalItem) {
        //     return true;
        // }

        // return false;
    }
}