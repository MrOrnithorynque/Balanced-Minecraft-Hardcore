package github.mrornithorynque.bmh.handlers;

import github.mrornithorynque.bmh.items.EternalAxeItem;
import github.mrornithorynque.bmh.items.EternalPickaxeItem;
import github.mrornithorynque.bmh.items.EternalSwordItem;

import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.Items;

/**
 * The Eternal tools never break. Their durability stops at its last point, and there the tool is worn
 * out: it hits for half damage, mines at the speed of a stone tool and loses its ability. It still
 * mines every block it mined before. A repair at an anvil with gold blocks, or Mending, ends it.
 * A gold block repair always costs 15 levels and does not raise the prior work penalty of the tool,
 * so the anvil never refuses it as too expensive.
 */
public final class EternalWear {

    public static final float WORN_DAMAGE_FACTOR = 0.5F;

    // The mining speed of a stone tool.
    public static final float WORN_MINING_SPEED = 4.0F;

    // A repair of an Eternal tool with gold blocks at an anvil costs this many levels.
    public static final int GOLD_REPAIR_COST = 15;

    private EternalWear() {
    }

    public static boolean isEternalTool(ItemStack stack) {

        Item item = stack.getItem();

        return item instanceof EternalSwordItem || item instanceof EternalAxeItem || item instanceof EternalPickaxeItem;
    }

    public static boolean isWornOut(ItemStack stack) {

        return isEternalTool(stack) && stack.getDamageValue() >= stack.getMaxDamage() - 1;
    }

    /**
     * The damage that a use may add to an Eternal tool: never more than what leaves the last point.
     * A tool at 0 points from an older version of the mod gets its last point back.
     */
    public static int capDamage(ItemStack stack, int amount) {

        if (!isEternalTool(stack)) {
            return amount;
        }

        return Math.min(amount, stack.getMaxDamage() - 1 - stack.getDamageValue());
    }

    /** Each death removes a percentage of the durability of an Eternal tool, down to the last point. */
    public static void wear(ItemStack stack, int percentage) {

        if (percentage < 0 || percentage > 100) {
            throw new IllegalArgumentException("Percentage must be between 0 and 100");
        }

        int damage = (int) (stack.getMaxDamage() * (percentage / 100.0));

        stack.setDamageValue(Math.min(stack.getDamageValue() + damage, stack.getMaxDamage() - 1));
    }

    public static boolean isGoldRepair(ItemStack left, ItemStack right) {

        return isEternalTool(left) && right.is(Items.GOLD_BLOCK);
    }

    /** The prior work penalty that the anvil adds to the cost of the next use of this item. */
    public static int getRepairPenalty(ItemStack stack) {

        return stack.getBaseRepairCost();
    }

    public static void setRepairPenalty(ItemStack stack, int penalty) {

        if (penalty > 0) {
            stack.setRepairCost(penalty);
        } else if (stack.getTag() != null) {
            stack.getTag().remove("RepairCost");
        }
    }
}
