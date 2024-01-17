package github.mrornithorynque.bmh.item;

import net.minecraft.client.renderer.EffectInstance;
import net.minecraft.sounds.SoundEvents;
import net.minecraft.sounds.SoundSource;
import net.minecraft.world.effect.MobEffects;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.Level;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.player.Player; // Import the Player class from the correct package
import net.minecraft.network.chat.Component;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.InteractionResultHolder;

public class LifeTotemItem extends Item {

    public LifeTotemItem(Properties properties) {
        super(properties);
    }

    // public void openInventory(PlayerEntity player, ItemStack totemStack) {
    //     TotemInventory inventory = new TotemInventory(totemStack);
    // }

    private static class TotemInventory {
        private ItemStack[] inventory;
        private ItemStack totemStack;

        public TotemInventory(ItemStack totemStack) {
            this.totemStack = totemStack;
            this.inventory = new ItemStack[3]; // 3 emplacements
            // loadFromNBT();
        }

        // private void loadFromNBT() {
        //     CompoundNBT nbt = totemStack.getOrCreateTag();
        //     // Charger l'inventaire à partir de NBT
        // }

        // public void saveToNBT() {
        //     CompoundNBT nbt = totemStack.getOrCreateTag();
        //     // Enregistrer l'inventaire dans NBT
        // }

    }
}

