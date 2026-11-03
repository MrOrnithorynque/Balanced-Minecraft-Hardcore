package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import github.mrornithorynque.bmh.handlers.TotemAwakeningHandler;

import net.minecraft.world.entity.player.Inventory;
import net.minecraft.world.inventory.AnvilMenu;
import net.minecraft.world.inventory.ContainerLevelAccess;
import net.minecraft.world.inventory.DataSlot;
import net.minecraft.world.inventory.ItemCombinerMenu;
import net.minecraft.world.inventory.MenuType;
import net.minecraft.world.item.ItemStack;

/**
 * A Dormant Totem on the left and an Echo Heart on the right give the Totem of Eternity, for 39
 * levels, consuming both. The name field is ignored: the result is a fresh stack, and vanilla's
 * rename (done by setItemName on the old result) is overwritten by the createResult it calls next.
 * Common to the three loaders: no loader has to fire an event for it, and Forge's AnvilUpdateEvent
 * is skipped for this one combination only.
 */
@Mixin(AnvilMenu.class)
public abstract class B_AnvilMenuMixin extends ItemCombinerMenu {

    // How many of the right slot's items onTake removes. 0 would empty the slot too (it holds one).
    @Shadow
    private int repairItemCountCost;

    @Shadow
    @Final
    private DataSlot cost;

    // Never called: Mixin only needs it to compile against ItemCombinerMenu.
    private B_AnvilMenuMixin(MenuType<?> type, int containerId, Inventory inventory, ContainerLevelAccess access) {
        super(type, containerId, inventory, access);
    }

    @Inject(method = "createResult", at = @At("HEAD"), cancellable = true)
    private void bmh$awakenTotem(CallbackInfo ci) {

        ItemStack awakened = TotemAwakeningHandler.awaken(this.inputSlots.getItem(0), this.inputSlots.getItem(1));

        if (awakened.isEmpty()) {
            return;
        }

        this.cost.set(TotemAwakeningHandler.COST);
        this.repairItemCountCost = 1;
        this.resultSlots.setItem(0, awakened);
        this.broadcastChanges();
        ci.cancel();
    }
}
