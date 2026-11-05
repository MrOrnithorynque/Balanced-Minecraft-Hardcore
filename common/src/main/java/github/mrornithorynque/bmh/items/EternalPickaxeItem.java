package github.mrornithorynque.bmh.items;

import java.util.List;

import github.mrornithorynque.bmh.handlers.EternalWear;

import net.minecraft.network.chat.Component;
import net.minecraft.world.item.DiggerItem;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.PickaxeItem;
import net.minecraft.world.item.Tier;
import net.minecraft.world.item.TooltipFlag;
import net.minecraft.world.level.block.state.BlockState;

public class EternalPickaxeItem extends PickaxeItem implements IEternalItem {

    public EternalPickaxeItem(Tier tier, int attackDamage, float attackSpeed, Properties properties) {
        super(tier, properties.attributes(DiggerItem.createAttributes(tier, attackDamage, attackSpeed)));
    }

    @Override
    public void appendHoverText(ItemStack stack, Item.TooltipContext context, List<Component> lines, TooltipFlag flag) {

        super.appendHoverText(stack, context, lines, flag);
        EternalTooltips.addAbility(lines, "tooltip.bmh.ability.eternal_pickaxe");
        EternalTooltips.addHeavy(lines);
        EternalTooltips.addDeathWear(lines);
        EternalTooltips.addWornOut(stack, lines);
    }

    @Override
    public void reduceDurability(ItemStack stack, int percentage) {

        EternalWear.wear(stack, percentage);
    }

    // A worn-out tool mines at the speed of a stone tool (EternalWear).
    @Override
    public float getDestroySpeed(ItemStack stack, BlockState state) {

        float speed = super.getDestroySpeed(stack, state);

        return EternalWear.isWornOut(stack) ? Math.min(speed, EternalWear.WORN_MINING_SPEED) : speed;
    }
}
