package github.mrornithorynque.bmh.items;

import java.util.List;

import org.slf4j.Logger;

import com.mojang.logging.LogUtils;

import net.minecraft.network.chat.Component;
import net.minecraft.world.item.AxeItem;
import net.minecraft.world.item.DiggerItem;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.Tier;
import net.minecraft.world.item.TooltipFlag;

public class EternalAxeItem extends AxeItem implements IEternalItem {

    private static final Logger LOGGER = LogUtils.getLogger();

    public EternalAxeItem(Tier tier, float attackDamage, float attackSpeed, Properties properties) {
        super(tier, properties.attributes(DiggerItem.createAttributes(tier, attackDamage, attackSpeed)));
    }

    @Override
    public void appendHoverText(ItemStack stack, Item.TooltipContext context, List<Component> lines, TooltipFlag flag) {

        super.appendHoverText(stack, context, lines, flag);
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
