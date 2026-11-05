package github.mrornithorynque.bmh.items;

import java.util.List;

import javax.annotation.Nullable;

import github.mrornithorynque.bmh.handlers.EternalWear;

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
        EternalTooltips.addAbility(lines, "tooltip.bmh.ability.eternal_sword");
        EternalTooltips.addHeavy(lines);
        EternalTooltips.addDeathWear(lines);
        EternalTooltips.addWornOut(stack, lines);
    }

    @Override
    public void reduceDurability(ItemStack stack, int percentage) {

        EternalWear.wear(stack, percentage);
    }
}
