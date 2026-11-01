package github.mrornithorynque.bmh.init;

import net.minecraft.tags.BlockTags;
import net.minecraft.tags.TagKey;
import net.minecraft.world.item.Tier;
import net.minecraft.world.item.crafting.Ingredient;
import net.minecraft.world.level.block.Block;

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

        // Mines everything netherite mines (the old level 5, one above netherite, meant no more).
        @Override
        public TagKey<Block> getIncorrectBlocksForDrops() {
            return BlockTags.INCORRECT_FOR_NETHERITE_TOOL;
        }

        @Override
        public int getEnchantmentValue() {
            return 80;
        }

        @Override
        public Ingredient getRepairIngredient() {
            return Ingredient.of(ItemInit.TOTEM_OF_ETERNITY.get());
        }
    };
}
