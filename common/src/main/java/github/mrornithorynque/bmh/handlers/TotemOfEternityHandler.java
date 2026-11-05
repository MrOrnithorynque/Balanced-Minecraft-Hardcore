package github.mrornithorynque.bmh.handlers;

import org.slf4j.Logger;

import com.mojang.logging.LogUtils;

import github.mrornithorynque.bmh.init.ItemInit;
import github.mrornithorynque.bmh.items.TotemOfEternityItem;

import net.minecraft.server.level.ServerPlayer;
import net.minecraft.stats.Stats;
import net.minecraft.world.Container;
import net.minecraft.world.effect.MobEffectInstance;
import net.minecraft.world.effect.MobEffects;
import net.minecraft.world.entity.player.Inventory;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.gameevent.GameEvent;

/**
 * The Totem of Eternity saves a player from death once, from any slot of the inventory, and then turns back into a
 * Dormant Totem. A container holds one Totem of Eternity at most.
 */
public class TotemOfEternityHandler {

    private static final Logger LOGGER = LogUtils.getLogger();

    // Called by mixin/A_LivingEntityTotemMixin when no Totem of Undying in a hand saved the player.
    // It does what vanilla's totem does, without the Fire Resistance (see that mixin).
    public static boolean trySave(ServerPlayer player) {

        Inventory inventory = player.getInventory();

        for (int i = 0; i < inventory.getContainerSize(); i++) {

            ItemStack stack = inventory.getItem(i);

            if (stack.getItem() instanceof TotemOfEternityItem) {

                inventory.setItem(i, new ItemStack(ItemInit.DORMANT_TOTEM.get()));
                player.awardStat(Stats.ITEM_USED.get(stack.getItem()));
                player.gameEvent(GameEvent.ITEM_INTERACT_FINISH);

                player.setHealth(1.0F);
                player.removeAllEffects();
                player.addEffect(new MobEffectInstance(MobEffects.REGENERATION, 900, 1));
                player.addEffect(new MobEffectInstance(MobEffects.ABSORPTION, 100, 1));
                player.level().broadcastEntityEvent(player, (byte) 35);

                LOGGER.info("[BalancedMcHardcoreMain] The Totem of Eternity saved " + player.getName().getString()
                        + " and is now dormant");
                return true;
            }
        }

        return false;
    }

    // True when the stack is a Totem of Eternity and the container already holds another one in a different slot.
    // A swap inside one container moves the same stack, so the check skips that stack.
    public static boolean holdsAnotherTotem(Container container, int slot, ItemStack placed) {

        if (!(placed.getItem() instanceof TotemOfEternityItem)) {
            return false;
        }

        for (int i = 0; i < container.getContainerSize(); i++) {

            ItemStack stack = container.getItem(i);

            if (i != slot && stack != placed && stack.getItem() instanceof TotemOfEternityItem) {
                return true;
            }
        }

        return false;
    }
}
