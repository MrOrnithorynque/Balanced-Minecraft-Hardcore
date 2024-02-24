package github.mrornithorynque.bmh.items;

import net.minecraft.world.item.PickaxeItem;
import net.minecraft.world.item.Tier;

public class EternalPickaxeItem extends PickaxeItem implements IEternalItem {

    public EternalPickaxeItem(Tier tier, int attackDamage, float attackSpeed, Properties properties) {
        super(tier, attackDamage, attackSpeed, properties);
    }
}
