package github.mrornithorynque.bmh.items;

import java.util.List;

import net.minecraft.ChatFormatting;
import net.minecraft.network.chat.Component;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.TooltipFlag;

/**
 * A plain item with one gray line of help in its tooltip. The Dormant Totem and the Echo Heart use
 * it: nothing else tells a player that they meet at an anvil.
 */
public class HintItem extends Item {

    private final String hintKey;

    public HintItem(Item.Properties properties, String hintKey) {
        super(properties);
        this.hintKey = hintKey;
    }

    @Override
    public void appendHoverText(ItemStack stack, Item.TooltipContext context, List<Component> lines, TooltipFlag flag) {

        super.appendHoverText(stack, context, lines, flag);
        lines.add(Component.translatable(this.hintKey).withStyle(ChatFormatting.GRAY));
    }
}
