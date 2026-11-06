package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import github.mrornithorynque.bmh.handlers.EternalWear;
import github.mrornithorynque.bmh.handlers.TotemAwakeningHandler;
import github.mrornithorynque.bmh.items.IEternalItem;

import net.minecraft.world.entity.player.Inventory;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.inventory.AnvilMenu;
import net.minecraft.world.inventory.ContainerLevelAccess;
import net.minecraft.world.inventory.DataSlot;
import net.minecraft.world.inventory.ItemCombinerMenu;
import net.minecraft.world.inventory.ItemCombinerMenuSlotDefinition;
import net.minecraft.world.inventory.MenuType;
import net.minecraft.world.item.ItemStack;

/**
 * A Dormant Totem on the left and an Echo Heart on the right give the Totem of Eternity, for 39
 * levels, consuming both. The name field is ignored: the result is a fresh stack, and vanilla's
 * rename (done by setItemName on the old result) is overwritten by the createResult it calls next.
 * Common to the three loaders: no loader has to fire an event for it, and Forge's AnvilUpdateEvent
 * is skipped for this one combination only.
 * <p>
 * A repair of an Eternal tool with gold blocks always costs 15 levels and keeps the prior work penalty
 * of the tool. The head puts the penalty aside, so vanilla never finds the repair too expensive. The
 * return puts the penalty back on the tool and on the result, and sets the cost. The client does the same.
 * <p>
 * When the player takes the result, the Eternal Item on the left must go. Some mod combinations stop vanilla
 * from emptying the left slot (Easy Anvils with Stackables), and the item stays there as a free copy (GitHub
 * issue 5). onTake checks the slot right after vanilla empties it, before the anvil can break.
 * <p>
 * NeoForge moves the vanilla body of createResult into createResultInternal and keeps createResult as a final
 * wrapper that calls it and then fires AnvilUpdateEvent. HEAD and RETURN of createResult still frame the vanilla code.
 */
@Mixin(AnvilMenu.class)
public abstract class B_AnvilMenuMixin extends ItemCombinerMenu {

    // How many of the right slot's items onTake removes. 0 would empty the slot too (it holds one).
    @Shadow
    private int repairItemCountCost;

    @Shadow
    @Final
    private DataSlot cost;

    // The penalty of the tool during a gold block repair, -1 when the inputs are something else.
    @Unique
    private int bmh$goldRepairPenalty = -1;

    // Never called: Mixin only needs it to compile against ItemCombinerMenu.
    private B_AnvilMenuMixin(MenuType<?> type, int containerId, Inventory inventory, ContainerLevelAccess access,
            ItemCombinerMenuSlotDefinition slotDefinition) {
        super(type, containerId, inventory, access, slotDefinition);
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

    @Inject(method = "createResult", at = @At("HEAD"))
    private void bmh$beginGoldRepair(CallbackInfo ci) {

        ItemStack left = this.inputSlots.getItem(0);

        bmh$goldRepairPenalty = -1;

        if (EternalWear.isGoldRepair(left, this.inputSlots.getItem(1))) {
            bmh$goldRepairPenalty = EternalWear.getRepairPenalty(left);
            EternalWear.setRepairPenalty(left, 0);
        }
    }

    @Inject(method = "createResult", at = @At("RETURN"))
    private void bmh$goldRepairCost(CallbackInfo ci) {

        if (bmh$goldRepairPenalty < 0) {
            return;
        }

        int penalty = bmh$goldRepairPenalty;
        ItemStack result = this.resultSlots.getItem(0);

        bmh$goldRepairPenalty = -1;
        EternalWear.setRepairPenalty(this.inputSlots.getItem(0), penalty);

        if (result.isEmpty()) {
            return;
        }

        EternalWear.setRepairPenalty(result, penalty);
        this.cost.set(EternalWear.GOLD_REPAIR_COST);
        this.broadcastChanges();
    }

    // Vanilla empties the left slot right before it calls access.execute, which can break the anvil. There the
    // anvil block entity of Easy Anvils drops what it holds, so the check must come before that call. In 26.1
    // vanilla empties the left slot after the right slot, so the first read of repairItemCountCost (the point
    // that 1.21.1 used) now comes before it.
    @Inject(method = "onTake", at = @At(value = "INVOKE",
            target = "Lnet/minecraft/world/inventory/ContainerLevelAccess;execute(Ljava/util/function/BiConsumer;)V"))
    private void bmh$takeEternalFromLeft(Player player, ItemStack stack, CallbackInfo ci) {

        if (this.inputSlots.getItem(0).getItem() instanceof IEternalItem) {
            this.inputSlots.setItem(0, ItemStack.EMPTY);
        }
    }
}
