package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;

import github.mrornithorynque.bmh.handlers.TotemOfEternityHandler;

import net.minecraft.world.Container;
import net.minecraft.world.entity.vehicle.AbstractMinecartContainer;
import net.minecraft.world.entity.vehicle.ChestBoat;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.block.entity.BaseContainerBlockEntity;

/**
 * One Totem of Eternity per container, for hoppers and droppers. Vanilla's hopper asks Container.canPlaceItem, and so
 * do the item handler wrappers of Forge and NeoForge, so one mixin covers the three loaders. Mixin cannot inject into
 * the default method of an interface, so this mixin adds canPlaceItem to the classes that use the default: every
 * container block (chest, barrel, shulker box, hopper, dispenser, dropper), the container minecarts and the chest boat.
 * The build renames the method with Container's, because the mixin implements Container. A double chest asks each half.
 * The menus use Slot.mayPlace instead (I_SlotTotemMixin).
 */
@Mixin({ BaseContainerBlockEntity.class, AbstractMinecartContainer.class, ChestBoat.class })
public abstract class I_ContainerTotemMixin implements Container {

    @Override
    public boolean canPlaceItem(int slot, ItemStack stack) {

        return !TotemOfEternityHandler.holdsAnotherTotem(this, slot, stack);
    }
}
