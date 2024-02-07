package github.mrornithorynque.bmh.item;

import github.mrornithorynque.bmh.utilities.IEternalItem;
import net.minecraft.world.item.AxeItem;
import net.minecraft.world.item.Tier;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.PickaxeItem;
import net.minecraft.world.item.Rarity;

public class EternalPickaxeItem extends PickaxeItem implements IEternalItem {

    public EternalPickaxeItem(Tier tier, int attackDamage, float attackSpeed, Properties properties) {
        super(tier, attackDamage, attackSpeed, properties);
    }
}
