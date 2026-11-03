package github.mrornithorynque.bmh.init;

import net.minecraft.world.item.Items;
import net.minecraft.world.item.Tier;
import net.minecraft.world.item.crafting.Ingredient;

public class BMHModTierInit {

    public static final Tier ETERNAL = new Tier() {

        @Override
        public int getUses() {
            return 10000;
        }

        @Override
        public float getSpeed() {
            return 15.0f;
        }

        @Override
        public float getAttackDamageBonus() {
            return 0.0f;
        }

        @Override
        public int getLevel() {
            return 5;
        }

        @Override
        public int getEnchantmentValue() {
            return 80;
        }

        @Override
        public Ingredient getRepairIngredient() {
            return Ingredient.of(Items.GOLD_BLOCK);
        }
    };
}
