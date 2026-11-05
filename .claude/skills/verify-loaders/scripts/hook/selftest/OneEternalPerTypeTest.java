package github.mrornithorynque.bmh.selftest;

import java.util.List;

import github.mrornithorynque.bmh.init.ItemInit;
import github.mrornithorynque.bmh.utilities.BMHGameRules;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.item.ItemEntity;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.Items;
import net.minecraft.world.level.GameType;

/**
 * onlyCarryOneEternalItemType. A second copy of an Eternal Item falls out and stays on the ground: this is the pickup
 * loop of issue 5. Dirt dropped at the same time is picked up, which shows that the player can pick things up. With the
 * rule off, or in creative, the second copy stays in the inventory.
 */
final class OneEternalPerTypeTest implements SelfTest.Test {

    @Override
    public String name() {
        return "one eternal per type";
    }

    @Override
    public void start(SelfTest.Ctx ctx) {
        ctx.rule(BMHGameRules.RULE_ONLY_CARRY_ONE_ETERNAL_ITEM_TYPE, true);
        loop(ctx, "sword", ItemInit.ETERNAL_SWORD.get(), 0);
        loop(ctx, "bundle", ItemInit.ETERNAL_BUNDLE.get(), 8);
        loop(ctx, "totem", ItemInit.TOTEM_OF_ETERNITY.get(), 16);
        offhand(ctx, 24);

        ctx.later(120, () -> {
            ctx.rule(BMHGameRules.RULE_ONLY_CARRY_ONE_ETERNAL_ITEM_TYPE, false);
            kept(ctx, "rule off", 32, GameType.SURVIVAL);
            ctx.later(20, () -> {
                ctx.rule(BMHGameRules.RULE_ONLY_CARRY_ONE_ETERNAL_ITEM_TYPE, true);
                kept(ctx, "creative", 40, GameType.CREATIVE);
            });
        });
    }

    // Two copies in the main inventory: the tick drops one, then the touch must not take it back.
    private static void loop(SelfTest.Ctx ctx, String label, Item item, int dx) {
        ServerPlayer player = ctx.player("bmh-one-" + label, dx, 0);
        player.getInventory().items.set(0, new ItemStack(item));
        player.getInventory().items.set(1, new ItemStack(item));
        // Every copy seen on the ground during the first ticks, to tell how one left if it is gone at the check.
        java.util.Set<ItemEntity> seen = new java.util.LinkedHashSet<>();
        for (int t = 1; t <= 4; t++) {
            ctx.later(t, () -> seen.addAll(ctx.level.getEntitiesOfClass(ItemEntity.class, ctx.live(player).getBoundingBox().inflate(64), e -> e.getItem().is(item))));
        }
        ctx.later(5, () -> {
            ctx.check(label + ": the second copy falls out", count(ctx, player, item) == 1, "in inventory " + count(ctx, player, item));
            // In the loop, the touch takes this entity and the next tick drops a new one: a count alone looks the same.
            List<ItemEntity> ground = ground(ctx, player, item);
            ground.forEach(ItemEntity::setNoPickUpDelay);
            if (ground.size() != 1) {
                // Where did it go? Every copy within 64 blocks, for the detail of the check below.
                ServerPlayer at = ctx.live(player);
                StringBuilder seen2 = new StringBuilder();
                for (ItemEntity e : ctx.level.getEntitiesOfClass(ItemEntity.class, at.getBoundingBox().inflate(64), e -> e.getItem().is(item))) {
                    seen2.append(String.format(" [%.1f %.1f %.1f alive=%s age=%d]", e.getX() - at.getX(), e.getY() - at.getY(), e.getZ() - at.getZ(), e.isAlive(), e.getAge()));
                }
                for (ItemEntity e : seen) {
                    seen2.append(" [seen, removed=" + e.getRemovalReason() + " age=" + e.getAge() + "]");
                }
                ctx.check(label + ": the dropped copy lies near the player", false, ground.size() + " within 6 blocks; within 64:" + seen2);
            }
            // drop() throws the copy about 2 blocks forward: the player walks onto it, as a real player does.
            ServerPlayer walker = ctx.live(player);
            if (ground.size() == 1) walker.teleportTo(ground.get(0).getX(), ground.get(0).getY(), ground.get(0).getZ());
            ItemEntity dirt = new ItemEntity(ctx.level, walker.getX(), walker.getY(), walker.getZ(), new ItemStack(Items.DIRT));
            dirt.setNoPickUpDelay();
            ctx.level.addFreshEntity(dirt);
            ctx.later(60, () -> {
                boolean same = ground.size() == 1 && ground.get(0).isAlive();
                ctx.check(label + ": the dropped copy is never picked up", same && count(ctx, player, item) == 1,
                        "first dropped entity " + (ground.size() == 1 ? (ground.get(0).isAlive() ? "still on the ground" : "picked up")
                                : ground.size() + " entities") + ", in inventory " + count(ctx, player, item));
                ctx.check(label + ": dirt next to it is picked up (control)", ctx.live(player).getInventory().countItem(Items.DIRT) == 1,
                        "dirt in inventory " + ctx.live(player).getInventory().countItem(Items.DIRT));
            });
        });
    }

    // One copy in the off hand, one on the ground under the player: the pickup check covers every slot.
    private static void offhand(SelfTest.Ctx ctx, int dx) {
        Item sword = ItemInit.ETERNAL_SWORD.get();
        ServerPlayer player = ctx.player("bmh-one-offhand", dx, 0);
        player.getInventory().offhand.set(0, new ItemStack(sword));
        ItemEntity copy = new ItemEntity(ctx.level, player.getX(), player.getY(), player.getZ(), new ItemStack(sword));
        copy.setNoPickUpDelay();
        ctx.level.addFreshEntity(copy);
        ctx.later(60, () -> ctx.check("off hand: a copy on the ground is never picked up", copy.isAlive() && count(ctx, player, sword) == 1,
                "copy " + (copy.isAlive() ? "still on the ground" : "picked up") + ", in inventory " + count(ctx, player, sword)));
    }

    // Two copies stay where the rule does not apply.
    private static void kept(SelfTest.Ctx ctx, String label, int dx, GameType mode) {
        Item sword = ItemInit.ETERNAL_SWORD.get();
        ServerPlayer player = ctx.player("bmh-one-" + label.replace(' ', '-'), dx, 0);
        player.setGameMode(mode);
        player.getInventory().items.set(0, new ItemStack(sword));
        player.getInventory().items.set(1, new ItemStack(sword));
        ctx.later(10, () -> ctx.check(label + ": both copies stay", count(ctx, player, sword) == 2, "in inventory " + count(ctx, player, sword)));
    }

    private static int count(SelfTest.Ctx ctx, ServerPlayer player, Item item) {
        return ctx.live(player).getInventory().countItem(item);
    }

    private static List<ItemEntity> ground(SelfTest.Ctx ctx, ServerPlayer player, Item item) {
        return ctx.level.getEntitiesOfClass(ItemEntity.class, ctx.live(player).getBoundingBox().inflate(6),
                e -> e.isAlive() && e.getItem().is(item));
    }
}
