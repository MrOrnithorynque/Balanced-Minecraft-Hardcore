package github.mrornithorynque.bmh.items;

import java.util.List;

import github.mrornithorynque.bmh.handlers.EternalWear;

import net.minecraft.ChatFormatting;
import net.minecraft.network.chat.Component;
import net.minecraft.world.item.ItemStack;

/**
 * The tooltip lines every Eternal Item shares. Static text: a client cannot read the server's
 * game rules and config, so these never show the real numbers of a world.
 */
public final class EternalTooltips {

    private EternalTooltips() {
    }

    /** Every Eternal Item pushes the respawn farther away while it is carried. */
    public static void addHeavy(List<Component> lines) {

        lines.add(Component.translatable("tooltip.bmh.heavy_with_eternity").withStyle(ChatFormatting.GOLD));
    }

    /**
     * Each tool names what it does beyond being a tool, in dark aqua, above the gold line. A worn-out
     * tool has no ability (EternalWear): a short red line takes the place of the ability line.
     */
    public static void addAbility(ItemStack stack, List<Component> lines, String key) {

        if (EternalWear.isWornOut(stack)) {
            lines.add(Component.translatable("tooltip.bmh.worn_out").withStyle(ChatFormatting.RED));
        } else {
            lines.add(Component.translatable(key).withStyle(ChatFormatting.DARK_AQUA));
        }
    }

    /**
     * The 7% of durability that each death removes (KeepItemAfterDeath). The durability that is left is
     * not here: vanilla shows it with advanced tooltips (F3+H), like for every other item.
     */
    public static void addDeathWear(List<Component> lines) {

        lines.add(Component.translatable("tooltip.bmh.death_wear").withStyle(ChatFormatting.GRAY));
    }
}
