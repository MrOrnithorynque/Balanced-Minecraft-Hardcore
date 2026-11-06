package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import github.mrornithorynque.bmh.items.TotemOfEternityItem;

import net.minecraft.client.multiplayer.ClientPacketListener;
import net.minecraft.core.component.DataComponents;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.item.ItemStack;

/**
 * Shows the Totem of Eternity in the totem animation when it saves the player. Vanilla looks for a death protection
 * item (a Totem of Undying) in the hands only. The server sends the animation before it sends the Dormant Totem, so
 * the client still sees the totem.
 */
@Mixin(ClientPacketListener.class)
public abstract class I_ClientPacketListenerTotemMixin {

    // ClientPacketListener.findTotem (26.1.2) returns the first hand item with the death protection component, else
    // a new Totem of Undying. Since 1.21.2 that component, not the item, is what saves a player.
    @Inject(method = "findTotem(Lnet/minecraft/world/entity/player/Player;)Lnet/minecraft/world/item/ItemStack;",
            at = @At("RETURN"), cancellable = true)
    private static void bmh$totemOfEternity(Player player, CallbackInfoReturnable<ItemStack> cir) {

        // Vanilla found the item in a hand that saved the player: keep it.
        for (InteractionHand hand : InteractionHand.values()) {
            if (player.getItemInHand(hand).has(DataComponents.DEATH_PROTECTION)) {
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
