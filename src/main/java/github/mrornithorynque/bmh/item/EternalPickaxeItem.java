package github.mrornithorynque.bmh.item;

import github.mrornithorynque.bmh.utilities.IEternalItem;
import net.minecraft.world.item.Tier;
import net.minecraft.world.item.PickaxeItem;

public class EternalPickaxeItem extends PickaxeItem implements IEternalItem {

    public EternalPickaxeItem(Tier tier, int attackDamage, float attackSpeed, Properties properties) {
        super(tier, attackDamage, attackSpeed, properties);
    }
}
