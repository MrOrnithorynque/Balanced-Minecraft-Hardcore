package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import github.mrornithorynque.bmh.items.TotemOfEternityItem;

import net.minecraft.client.multiplayer.ClientPacketListener;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.Items;

/**
 * Shows the Totem of Eternity in the totem animation when it saves the player. Vanilla looks for a Totem of Undying in
 * the hands only. The server sends the animation before it sends the Dormant Totem, so the client still sees the totem.
 */
@Mixin(ClientPacketListener.class)
public abstract class I_ClientPacketListenerTotemMixin {

    @Inject(method = "findTotem", at = @At("RETURN"), cancellable = true)
    private static void bmh$totemOfEternity(Player player, CallbackInfoReturnable<ItemStack> cir) {

        for (InteractionHand hand : InteractionHand.values()) {
            if (player.getItemInHand(hand).is(Items.TOTEM_OF_UNDYING)) {
                return;
            }
        }

        for (int i = 0; i < player.getInventory().getContainerSize(); i++) {

            ItemStack stack = player.getInventory().getItem(i);

            if (stack.getItem() instanceof TotemOfEternityItem) {
                cir.setReturnValue(stack.copy());
                return;
            }
        }
    }
}
