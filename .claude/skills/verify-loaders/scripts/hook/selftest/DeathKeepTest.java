package github.mrornithorynque.bmh.selftest;

import github.mrornithorynque.bmh.init.ItemInit;
import github.mrornithorynque.bmh.utilities.BMHGameRules;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.player.Inventory;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.Items;

/**
 * Death keeps the Eternal Items, the bundle with its contents, and drops the rest. damageEternalItemWhenDead takes 7 %
 * of the tools' durability at each death. With the rule off, they keep it. The distant respawn is off here, because it
 * has its own test, so the respawn stays near the spawn.
 */
final class DeathKeepTest implements SelfTest.Test {

    @Override
    public String name() {
        return "death keep and wear";
    }

    @Override
    public void start(SelfTest.Ctx ctx) {
        ctx.rule(BMHGameRules.RULE_RANDOM_DEATH_SPAWN_POINT, false);
        ctx.rule(BMHGameRules.RULE_DAMAGE_ETERNAL_ITEM_WHEN_DEAD, true);
        die(ctx, "wear on", 0, true);
        ctx.later(60, () -> {
            ctx.rule(BMHGameRules.RULE_DAMAGE_ETERNAL_ITEM_WHEN_DEAD, false);
            die(ctx, "wear off", 8, false);
        });
    }

    private static void die(SelfTest.Ctx ctx, String label, int dx, boolean wear) {
        ServerPlayer player = ctx.player("bmh-death-" + label.replace(' ', '-'), dx, 0);
        ItemStack sword = new ItemStack(ItemInit.ETERNAL_SWORD.get());
        ItemStack bundle = new ItemStack(ItemInit.ETERNAL_BUNDLE.get());
        MockPlayers.fillBundle(bundle, new ItemStack(Items.DIAMOND, 3));
        ItemStack bundleBefore = bundle.copy();
        int max = sword.getMaxDamage();
        player.getInventory().setItem(0, sword);
        player.getInventory().setItem(1, bundle);
        player.getInventory().setItem(2, new ItemStack(Items.DIRT, 16));
        MockPlayers.kill(player);
        ctx.later(5, () -> {
            ServerPlayer back = MockPlayers.respawn(ctx.server, ctx.live(player));
            ctx.later(5, () -> {
                ServerPlayer now = ctx.live(back);
                ItemStack kept = ItemStack.EMPTY;
                ItemStack keptBundle = ItemStack.EMPTY;
                for (int slot = 0; slot < Inventory.INVENTORY_SIZE; slot++) {
                    ItemStack stack = now.getInventory().getItem(slot);
                    if (stack.is(ItemInit.ETERNAL_SWORD.get())) kept = stack;
                    if (stack.is(ItemInit.ETERNAL_BUNDLE.get())) keptBundle = stack;
                }
                ctx.check(label + ": the Eternal Sword is kept", !kept.isEmpty(), "sword " + kept);
                ctx.check(label + ": the dirt is dropped", now.getInventory().countItem(Items.DIRT) == 0,
                        "dirt in inventory " + now.getInventory().countItem(Items.DIRT));
                ctx.check(label + ": the Eternal Bundle keeps its contents", MockPlayers.sameData(keptBundle, bundleBefore),
                        "bundle " + keptBundle);
                int expected = wear ? (int) (max * 0.07) : 0;
                ctx.check(label + ": durability lost " + (wear ? "7 %" : "none"), kept.getDamageValue() == expected,
                        "damage " + kept.getDamageValue() + " of " + max + ", expected " + expected);
            });
        });
    }
}
