package github.mrornithorynque.bmh.items;

import java.util.function.Consumer;

import github.mrornithorynque.bmh.handlers.EternalWear;

import net.minecraft.network.chat.Component;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.ToolMaterial;
import net.minecraft.world.item.TooltipFlag;
import net.minecraft.world.item.component.TooltipDisplay;
import net.minecraft.world.level.block.state.BlockState;

/** A pickaxe is a plain item with the pickaxe properties of its material, as vanilla's pickaxes are. */
public class EternalPickaxeItem extends Item implements IEternalItem {

    public EternalPickaxeItem(ToolMaterial material, float attackDamage, float attackSpeed, Item.Properties properties) {
        super(properties.pickaxe(material, attackDamage, attackSpeed));
    }

    @Override
    public void appendHoverText(ItemStack stack, Item.TooltipContext context, TooltipDisplay display, Consumer<Component> lines,
            TooltipFlag flag) {

        super.appendHoverText(stack, context, display, lines, flag);
        EternalTooltips.addAbility(stack, lines, "tooltip.bmh.ability.eternal_pickaxe");
        EternalTooltips.addHeavy(lines);
        EternalTooltips.addDurability(stack, lines);
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

    // A worn-out tool mines at the speed of a stone tool (EternalWear).
    @Override
    public float getDestroySpeed(ItemStack stack, BlockState state) {

        float speed = super.getDestroySpeed(stack, state);

        return EternalWear.isWornOut(stack) ? Math.min(speed, EternalWear.WORN_MINING_SPEED) : speed;
    }
}
