package github.mrornithorynque.bmh.item;

import github.mrornithorynque.bmh.utilities.IEternalItem;
import net.minecraft.world.item.AxeItem;
import net.minecraft.world.item.Tier;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.Rarity;

public class EternalAxeItem extends AxeItem implements IEternalItem {

    public EternalAxeItem(Tier tier, float attackDamage, float attackSpeed, Properties properties) {
        super(tier, attackDamage, attackSpeed, properties);
    }
}
