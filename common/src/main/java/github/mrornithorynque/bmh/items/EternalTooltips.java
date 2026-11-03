package github.mrornithorynque.bmh.items;

import java.util.List;

import net.minecraft.ChatFormatting;
import net.minecraft.network.chat.Component;

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

    /** Each tool names what it does beyond being a tool, in dark aqua, above the gold line. */
    public static void addAbility(List<Component> lines, String key) {

        lines.add(Component.translatable(key).withStyle(ChatFormatting.DARK_AQUA));
    }

    /** The three tools lose 7% of their durability at each death (KeepItemAfterDeath). */
    public static void addDeathWear(List<Component> lines) {

        lines.add(Component.translatable("tooltip.bmh.death_wear").withStyle(ChatFormatting.GRAY));
    }
}
