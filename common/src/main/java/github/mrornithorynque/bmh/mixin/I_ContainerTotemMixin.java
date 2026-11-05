package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;

import github.mrornithorynque.bmh.handlers.TotemOfEternityHandler;
import github.mrornithorynque.bmh.items.TotemOfEternityItem;

import net.minecraft.world.Container;
import net.minecraft.world.entity.vehicle.AbstractMinecartContainer;
import net.minecraft.world.entity.vehicle.ChestBoat;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.block.ChestBlock;
import net.minecraft.world.level.block.entity.BaseContainerBlockEntity;
import net.minecraft.world.level.block.entity.ChestBlockEntity;
import net.minecraft.world.level.block.state.BlockState;

/**
 * One Totem of Eternity per container, for hoppers and droppers. Vanilla's hopper asks Container.canPlaceItem, and so
 * do the item handler wrappers of Forge and NeoForge, so one mixin covers the three loaders. Mixin cannot inject into
 * the default method of an interface, so this mixin adds canPlaceItem to the classes that use the default: every
 * container block (chest, barrel, shulker box, hopper, dispenser, dropper), the container minecarts and the chest boat.
 * The build renames the method with Container's, because the mixin implements Container. A hopper asks one half of a
 * double chest, so a chest counts the totems of both halves. The menus use Slot.mayPlace instead (I_SlotTotemMixin).
 */
@Mixin({ BaseContainerBlockEntity.class, AbstractMinecartContainer.class, ChestBoat.class })
public abstract class I_ContainerTotemMixin implements Container {

    @Override
    public boolean canPlaceItem(int slot, ItemStack stack) {

        if (!(stack.getItem() instanceof TotemOfEternityItem)) {
            return true;
        }

        if ((Object) this instanceof ChestBlockEntity chest && chest.getLevel() != null
                && chest.getBlockState().getBlock() instanceof ChestBlock block) {

            BlockState state = chest.getBlockState();
            Container wholeChest = ChestBlock.getContainer(block, state, chest.getLevel(), chest.getBlockPos(), true);

            if (wholeChest != null) {
                return !TotemOfEternityHandler.holdsAnotherTotem(this, wholeChest, slot, stack);
            }
        }

        return !TotemOfEternityHandler.holdsAnotherTotem(this, slot, stack);
    }
}
