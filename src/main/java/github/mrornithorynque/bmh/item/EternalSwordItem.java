package github.mrornithorynque.bmh.item;

import github.mrornithorynque.bmh.utilities.IEternalItem;
import net.minecraft.world.item.SwordItem;
import net.minecraft.world.item.Tier;

public class EternalSwordItem extends SwordItem implements IEternalItem {

    public EternalSwordItem(Tier tier, int attackDamage, float attackSpeed, Properties properties) {
        super(tier, attackDamage, attackSpeed, properties);
    }
}
