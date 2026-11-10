package github.mrornithorynque.bmh.items;

import java.util.function.Consumer;

import github.mrornithorynque.bmh.handlers.EternalWear;

import net.minecraft.network.chat.Component;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.ToolMaterial;
import net.minecraft.world.item.TooltipFlag;
import net.minecraft.world.item.component.TooltipDisplay;

/**
 * A sword is a plain item with the sword properties of its material (tool rules, attributes, one durability point per
 * hit), as vanilla's swords are. The "minecraft:swords" tag makes it sweep (Player.isSweepAttack).
 */
public class EternalSwordItem extends Item implements IEternalItem {

    public EternalSwordItem(ToolMaterial material, float attackDamage, float attackSpeed, Item.Properties properties) {
        super(properties.sword(material, attackDamage, attackSpeed));
    }

    @Override
    public void appendHoverText(ItemStack stack, Item.TooltipContext context, TooltipDisplay display, Consumer<Component> lines,
            TooltipFlag flag) {

        super.appendHoverText(stack, context, display, lines, flag);
        EternalTooltips.addAbility(stack, lines, "tooltip.bmh.ability.eternal_sword");
        EternalTooltips.addHeavy(lines);
        EternalTooltips.addDeathWear(lines);
    }

    @Override
    public void reduceDurability(ItemStack stack, int percentage) {

        EternalWear.wear(stack, percentage);
    }

    // The bar keeps one pixel at the last points of durability, so a worn-out tool still shows it.
    @Override
    public int getBarWidth(ItemStack stack) {

        return Math.max(1, super.getBarWidth(stack));
    }
}
