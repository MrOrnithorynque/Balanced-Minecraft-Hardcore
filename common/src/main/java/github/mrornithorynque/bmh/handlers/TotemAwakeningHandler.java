package github.mrornithorynque.bmh.handlers;

import github.mrornithorynque.bmh.init.ItemInit;

import net.minecraft.world.item.ItemStack;

/**
 * The anvil step of the Totem of Eternity: a Dormant Totem on the left and an Echo Heart on the
 * right give the Totem of Eternity. B_AnvilMenuMixin calls it from AnvilMenu.createResult.
 */
public class TotemAwakeningHandler {

    /** The most a survival anvil allows: 40 levels and above is "Too Expensive". */
    public static final int COST = 39;

    /** The awakened totem, or EMPTY when the two stacks are not a Dormant Totem and an Echo Heart. */
    public static ItemStack awaken(ItemStack left, ItemStack right) {

        if (left.is(ItemInit.DORMANT_TOTEM.get()) && right.is(ItemInit.ECHO_HEART.get())) {
            return new ItemStack(ItemInit.TOTEM_OF_ETERNITY.get());
        }

        return ItemStack.EMPTY;
    }
}
