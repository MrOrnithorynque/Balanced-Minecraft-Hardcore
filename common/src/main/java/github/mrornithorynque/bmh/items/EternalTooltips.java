package github.mrornithorynque.bmh.items;

import java.util.function.Consumer;

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
    public static void addHeavy(Consumer<Component> lines) {

        lines.accept(Component.translatable("tooltip.bmh.heavy_with_eternity").withStyle(ChatFormatting.GOLD));
    }

    /**
     * Each tool names what it does beyond being a tool, in dark aqua, above the gold line. A worn-out
     * tool has no ability (EternalWear): a short red line takes the place of the ability line.
     */
    public static void addAbility(ItemStack stack, Consumer<Component> lines, String key) {

        if (EternalWear.isWornOut(stack)) {
            lines.accept(Component.translatable("tooltip.bmh.worn_out").withStyle(ChatFormatting.RED));
        } else {
            lines.accept(Component.translatable(key).withStyle(ChatFormatting.DARK_AQUA));
        }
    }

    /**
     * The durability that is left, and the 7% that each death removes (KeepItemAfterDeath). Vanilla shows
     * the durability only with advanced tooltips (F3+H).
     */
    public static void addDurability(ItemStack stack, Consumer<Component> lines) {

        int left = stack.getMaxDamage() - stack.getDamageValue();

        lines.accept(Component.translatable("tooltip.bmh.durability", left, stack.getMaxDamage()).withStyle(ChatFormatting.GRAY));
    }
}
