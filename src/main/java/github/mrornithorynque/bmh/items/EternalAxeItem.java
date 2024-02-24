package github.mrornithorynque.bmh.items;

import net.minecraft.world.item.AxeItem;
import net.minecraft.world.item.Tier;

public class EternalAxeItem extends AxeItem implements IEternalItem {

    public EternalAxeItem(Tier tier, float attackDamage, float attackSpeed, Properties properties) {
        super(tier, attackDamage, attackSpeed, properties);
    }
}
