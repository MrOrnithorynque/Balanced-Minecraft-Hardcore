package github.mrornithorynque.bmh.items;

import org.slf4j.Logger;

import com.mojang.logging.LogUtils;

import net.minecraft.world.item.AxeItem;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.Tier;

public class EternalAxeItem extends AxeItem implements IEternalItem {

    private static final Logger LOGGER = LogUtils.getLogger();

    public EternalAxeItem(Tier tier, float attackDamage, float attackSpeed, Properties properties) {
        super(tier, attackDamage, attackSpeed, properties);
    }

    @Override
    public void reduceDurability(ItemStack stack, int percentage) {

        if (percentage < 0 || percentage > 100) {
            throw new IllegalArgumentException("Percentage must be between 0 and 100");
        }

        int damage = (int) (stack.getMaxDamage() * (percentage / 100.0));

        stack.setDamageValue(Math.min(stack.getDamageValue() + damage, stack.getMaxDamage()));

        if (stack.getDamageValue() >= stack.getMaxDamage()) {
        }
    }
}
