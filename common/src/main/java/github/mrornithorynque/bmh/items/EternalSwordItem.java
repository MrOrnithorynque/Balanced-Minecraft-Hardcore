package github.mrornithorynque.bmh.items;

import java.util.List;

import javax.annotation.Nullable;

import net.minecraft.network.chat.Component;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.SwordItem;
import net.minecraft.world.item.Tier;
import net.minecraft.world.item.TooltipFlag;
import net.minecraft.world.level.Level;

public class EternalSwordItem extends SwordItem implements IEternalItem {

    public EternalSwordItem(Tier tier, int attackDamage, float attackSpeed, Properties properties) {
        super(tier, attackDamage, attackSpeed, properties);
    }

    @Override
    public void appendHoverText(ItemStack stack, @Nullable Level level, List<Component> lines, TooltipFlag flag) {

        super.appendHoverText(stack, level, lines, flag);
        EternalTooltips.addHeavy(lines);
        EternalTooltips.addDeathWear(lines);
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
