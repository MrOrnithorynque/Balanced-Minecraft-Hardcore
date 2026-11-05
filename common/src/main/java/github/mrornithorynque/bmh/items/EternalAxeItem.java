package github.mrornithorynque.bmh.items;

import java.util.List;

import javax.annotation.Nullable;

import org.slf4j.Logger;

import com.mojang.logging.LogUtils;

import github.mrornithorynque.bmh.handlers.EternalWear;

import net.minecraft.network.chat.Component;
import net.minecraft.world.item.AxeItem;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.Tier;
import net.minecraft.world.item.TooltipFlag;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.Level;

public class EternalAxeItem extends AxeItem implements IEternalItem {

    private static final Logger LOGGER = LogUtils.getLogger();

    public EternalAxeItem(Tier tier, float attackDamage, float attackSpeed, Properties properties) {
        super(tier, attackDamage, attackSpeed, properties);
    }

    @Override
    public void appendHoverText(ItemStack stack, @Nullable Level level, List<Component> lines, TooltipFlag flag) {

        super.appendHoverText(stack, level, lines, flag);
        EternalTooltips.addAbility(lines, "tooltip.bmh.ability.eternal_axe");
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
