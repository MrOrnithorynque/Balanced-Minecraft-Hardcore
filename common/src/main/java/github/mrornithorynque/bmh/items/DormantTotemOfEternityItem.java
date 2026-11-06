package github.mrornithorynque.bmh.items;

import java.util.List;

import javax.annotation.Nullable;

import net.minecraft.ChatFormatting;
import net.minecraft.network.chat.Component;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.TooltipFlag;
import net.minecraft.world.level.Level;

/**
 * The Totem of Eternity after it saved a player. It cannot save anyone, but it keeps the eternity: it stays with the
 * player through death, and it makes the respawn farther like every Eternal Item. An Echo Heart at the anvil awakens
 * it again (TotemAwakeningHandler).
 */
public class DormantTotemOfEternityItem extends Item implements IEternalItem {

    public DormantTotemOfEternityItem(Item.Properties properties) {
        super(properties);
    }

    @Override
    public void reduceDurability(ItemStack stack, int percentage) {
        // The totem has no durability.
    }

    @Override
    public void appendHoverText(ItemStack stack, @Nullable Level level, List<Component> lines, TooltipFlag flag) {

        super.appendHoverText(stack, level, lines, flag);
        lines.add(Component.translatable("tooltip.bmh.eternity").withStyle(ChatFormatting.GRAY));
        lines.add(Component.translatable("tooltip.bmh.dormant_totem_of_eternity").withStyle(ChatFormatting.GRAY));
        EternalTooltips.addHeavy(lines);
    }
}
