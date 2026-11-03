package github.mrornithorynque.bmh.items;

import java.util.List;

import net.minecraft.network.chat.Component;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.SwordItem;
import net.minecraft.world.item.Tier;
import net.minecraft.world.item.TooltipFlag;

public class EternalSwordItem extends SwordItem implements IEternalItem {

    public EternalSwordItem(Tier tier, int attackDamage, float attackSpeed, Properties properties) {
        super(tier, properties.attributes(SwordItem.createAttributes(tier, attackDamage, attackSpeed)));
    }

    @Override
    public void appendHoverText(ItemStack stack, Item.TooltipContext context, List<Component> lines, TooltipFlag flag) {

        super.appendHoverText(stack, context, lines, flag);
        EternalTooltips.addAbility(lines, "tooltip.bmh.ability.eternal_sword");
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
