package github.mrornithorynque.bmh.init;

import net.minecraft.world.item.crafting.Ingredient;
import net.minecraftforge.common.ForgeTier;

public class BMHModTierInit {

    public static final ForgeTier ETERNAL = new ForgeTier(
        5,
        10000,
        5.0f,
        0.0f,
        80,
        TagInit.LIFE_TOTEM,
        () -> Ingredient.of(ItemInit.LIFE_TOTEM::get)
    );
}
