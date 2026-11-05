package github.mrornithorynque.bmh.items;

import java.util.List;

import javax.annotation.Nullable;

import net.minecraft.ChatFormatting;
import net.minecraft.network.chat.Component;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.TooltipFlag;
import net.minecraft.world.level.Level;

public class TotemOfEternityItem extends Item implements IEternalItem {

    public TotemOfEternityItem(Item.Properties properties) {
        super(properties);
    }

    @Override
    public void reduceDurability(ItemStack stack, int percentage) {
        // no need here
    }

    // The glint and the "Eternity" line stand in for an enchantment. It is not one: the item itself
    // is what survives death, so a grindstone has nothing to strip.
    @Override
    public boolean isFoil(ItemStack stack) {
        return true;
    }

    @Override
    public void appendHoverText(ItemStack stack, @Nullable Level level, List<Component> lines, TooltipFlag flag) {

        super.appendHoverText(stack, level, lines, flag);
        lines.add(Component.translatable("tooltip.bmh.eternity").withStyle(ChatFormatting.GRAY));
        lines.add(Component.translatable("tooltip.bmh.totem_of_eternity_save").withStyle(ChatFormatting.GRAY));
        EternalTooltips.addHeavy(lines);
    }
}
