package github.mrornithorynque.bmh.init;

import net.minecraft.world.item.crafting.Ingredient;
import net.minecraftforge.common.ForgeTier;

public class BMHModTierInit {

    public static final ForgeTier ETERNAL = new ForgeTier(
            5,
            10000,
            15.0f,
            0.0f,
            80,
            TagInit.TOTEM_OF_ETERNITY,
            () -> Ingredient.of(ItemInit.TOTEM_OF_ETERNITY::get));
}
