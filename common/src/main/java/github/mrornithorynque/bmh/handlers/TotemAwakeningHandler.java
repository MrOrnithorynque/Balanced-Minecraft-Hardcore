package github.mrornithorynque.bmh.handlers;

import github.mrornithorynque.bmh.init.ItemInit;

import net.minecraft.world.item.ItemStack;

/**
 * The anvil step of the Totem of Eternity: a Hollow Totem or a Dormant Totem of Eternity on the left
 * and an Echo Heart on the right give the Totem of Eternity. B_AnvilMenuMixin calls it from
 * AnvilMenu.createResult.
 */
public class TotemAwakeningHandler {

    /** The most a survival anvil allows: 40 levels and above is "Too Expensive". */
    public static final int COST = 39;

    /** The awakened totem, or EMPTY when the two stacks are not a totem to awaken and an Echo Heart. */
    public static ItemStack awaken(ItemStack left, ItemStack right) {

        boolean asleep = left.is(ItemInit.HOLLOW_TOTEM.get()) || left.is(ItemInit.DORMANT_TOTEM_OF_ETERNITY.get());

        if (asleep && right.is(ItemInit.ECHO_HEART.get())) {
            return new ItemStack(ItemInit.TOTEM_OF_ETERNITY.get());
        }

        return ItemStack.EMPTY;
    }
}
