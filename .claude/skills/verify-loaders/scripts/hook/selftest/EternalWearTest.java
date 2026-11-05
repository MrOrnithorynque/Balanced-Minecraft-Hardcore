package github.mrornithorynque.bmh.selftest;

import github.mrornithorynque.bmh.handlers.EchoSenseHandler;
import github.mrornithorynque.bmh.handlers.EternalWear;
import github.mrornithorynque.bmh.init.ItemInit;
import github.mrornithorynque.bmh.items.IEternalItem;
import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.Display;
import net.minecraft.world.entity.EntityType;
import net.minecraft.world.entity.animal.IronGolem;
import net.minecraft.world.inventory.AnvilMenu;
import net.minecraft.world.inventory.ClickType;
import net.minecraft.world.inventory.ContainerLevelAccess;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.Items;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.phys.AABB;

/**
 * The Eternal tools never break (EternalWear). At the last point of durability a hit leaves the tool in the hand, a
 * tool at 0 points from an older version gets its last point back, and a netherite sword (the control) breaks. A
 * worn-out sword hits for about half of a fresh one, a worn-out axe does not smash, a worn-out pickaxe mines at stone
 * speed and lights no ore. A death stops at the last point. A gold block repair at the anvil costs 15 levels, keeps
 * the prior work penalty of the tool, and works even when vanilla would say too expensive.
 */
final class EternalWearTest implements SelfTest.Test {

    @Override
    public String name() {
        return "eternal wear";
    }

    @Override
    public void start(SelfTest.Ctx ctx) {
        neverBreaks(ctx);
        weaker(ctx);
        echoSense(ctx);
        deathWear(ctx);
        anvilCost(ctx);
    }

    private static ItemStack worn(ItemStack stack) {
        stack.setDamageValue(stack.getMaxDamage() - 1);
        return stack;
    }

    private static void neverBreaks(SelfTest.Ctx ctx) {
        ServerPlayer eternal = holding(ctx, "bmh-wear-last", -12, worn(new ItemStack(ItemInit.ETERNAL_SWORD.get())));
        ItemStack legacy = new ItemStack(ItemInit.ETERNAL_SWORD.get());
        legacy.setDamageValue(legacy.getMaxDamage());
        ServerPlayer old = holding(ctx, "bmh-wear-zero", -8, legacy);
        ServerPlayer control = holding(ctx, "bmh-wear-ctrl", -4, worn(new ItemStack(Items.NETHERITE_SWORD)));
        IronGolem a = golem(ctx, -12, 2);
        IronGolem b = golem(ctx, -8, 2);
        IronGolem c = golem(ctx, -4, 2);
        ctx.later(30, () -> {
            ServerPlayer p1 = ctx.live(eternal);
            ServerPlayer p2 = ctx.live(old);
            ServerPlayer p3 = ctx.live(control);
            p1.attack(a);
            p2.attack(b);
            p3.attack(c);
            ItemStack s1 = p1.getMainHandItem();
            ItemStack s2 = p2.getMainHandItem();
            ctx.check("a hit at the last point keeps the sword", s1.is(ItemInit.ETERNAL_SWORD.get())
                    && s1.getDamageValue() == s1.getMaxDamage() - 1, "hand " + s1 + ", damage " + s1.getDamageValue());
            ctx.check("a sword at 0 points gets its last point back", s2.is(ItemInit.ETERNAL_SWORD.get())
                    && s2.getDamageValue() == s2.getMaxDamage() - 1, "hand " + s2 + ", damage " + s2.getDamageValue());
            ctx.check("control: a netherite sword at its last point breaks", p3.getMainHandItem().isEmpty(),
                    "hand " + p3.getMainHandItem());
            a.discard();
            b.discard();
            c.discard();
        });
    }

    private static void weaker(SelfTest.Ctx ctx) {
        ServerPlayer fresh = holding(ctx, "bmh-wear-fresh", 0, new ItemStack(ItemInit.ETERNAL_SWORD.get()));
        ServerPlayer tired = holding(ctx, "bmh-wear-tired", 4, worn(new ItemStack(ItemInit.ETERNAL_SWORD.get())));
        ServerPlayer axe = holding(ctx, "bmh-wear-axe", 8, worn(new ItemStack(ItemInit.ETERNAL_AXE.get())));
        IronGolem g1 = golem(ctx, 0, 2);
        IronGolem g2 = golem(ctx, 4, 2);
        IronGolem g3 = golem(ctx, 8, 2);
        IronGolem g4 = golem(ctx, 8, -2);
        float[] damage = new float[4];
        ctx.later(30, () -> {
            ServerPlayer f = ctx.live(fresh);
            ServerPlayer t = ctx.live(tired);
            ServerPlayer x = ctx.live(axe);
            f.setOnGround(true);
            t.setOnGround(true);
            f.attack(g1);
            t.attack(g2);
            damage[0] = g1.getMaxHealth() - g1.getHealth();
            damage[1] = g2.getMaxHealth() - g2.getHealth();
            ctx.check("a worn-out sword hits for about half", damage[0] > 0 && damage[1] > 0.4f * damage[0]
                    && damage[1] < 0.6f * damage[0], "fresh " + damage[0] + ", worn " + damage[1]);
            x.fallDistance = 0;
            x.setOnGround(true);
            x.attack(g3);
            damage[2] = g3.getMaxHealth() - g3.getHealth();
            ctx.later(30, () -> {
                ServerPlayer air = ctx.live(axe);
                air.fallDistance = 10;
                air.setOnGround(false);
                air.attack(g4);
                damage[3] = g4.getMaxHealth() - g4.getHealth();
                // A critical hit (x1.5) still lands; the smash (about x2.26 more) does not.
                // The axe must still be in the hand: a broken axe would also "not smash".
                ctx.check("a worn-out axe does not smash (< 2x)", air.getMainHandItem().is(ItemInit.ETERNAL_AXE.get())
                        && damage[2] > 0 && damage[3] > damage[2] && damage[3] < 2.0f * damage[2],
                        "hand " + air.getMainHandItem() + ", standing " + damage[2] + ", falling " + damage[3]);
                ctx.check("a worn-out axe does not soften the fall", air.fallDistance == 10f, "fallDistance " + air.fallDistance);
                g1.discard();
                g2.discard();
                g3.discard();
                g4.discard();
            });
        });
        ItemStack pick = new ItemStack(ItemInit.ETERNAL_PICKAXE.get());
        float freshSpeed = pick.getDestroySpeed(Blocks.STONE.defaultBlockState());
        float wornSpeed = worn(pick).getDestroySpeed(Blocks.STONE.defaultBlockState());
        ctx.check("a worn-out pickaxe mines at stone speed", freshSpeed > 4.0f && wornSpeed == 4.0f,
                "fresh " + freshSpeed + ", worn " + wornSpeed);
    }

    private static void echoSense(SelfTest.Ctx ctx) {
        int fresh = lit(ctx, "bmh-wear-ore1", -20, new ItemStack(ItemInit.ETERNAL_PICKAXE.get()));
        int tired = lit(ctx, "bmh-wear-ore2", 20, worn(new ItemStack(ItemInit.ETERNAL_PICKAXE.get())));
        ctx.check("control: a fresh pickaxe lights the ore nearby", fresh > 0, "displays " + fresh);
        ctx.check("a worn-out pickaxe lights no ore", tired == 0, "displays " + tired);
    }

    // Breaks a diamond ore next to another one and counts the Echo Sense displays around them.
    private static int lit(SelfTest.Ctx ctx, String name, int dx, ItemStack pickaxe) {
        ServerPlayer player = holding(ctx, name, dx, pickaxe);
        BlockPos ore = ctx.origin.offset(dx, 1, 2);
        BlockPos other = ctx.origin.offset(dx, 1, 4);
        ctx.level.setBlockAndUpdate(ore, Blocks.DIAMOND_ORE.defaultBlockState());
        ctx.level.setBlockAndUpdate(other, Blocks.DIAMOND_ORE.defaultBlockState());
        player.gameMode.destroyBlock(ore);
        return ctx.level.getEntitiesOfClass(Display.BlockDisplay.class, new AABB(ore).inflate(10),
                e -> e.getTags().contains(EchoSenseHandler.ECHO_TAG)).size();
    }

    private static void deathWear(SelfTest.Ctx ctx) {
        ItemStack sword = new ItemStack(ItemInit.ETERNAL_SWORD.get());
        sword.setDamageValue(sword.getMaxDamage() - 10);
        ((IEternalItem) sword.getItem()).reduceDurability(sword, 7);
        ctx.check("a death stops at the last point", sword.getDamageValue() == sword.getMaxDamage() - 1,
                "damage " + sword.getDamageValue() + " of " + sword.getMaxDamage());
    }

    private static void anvilCost(SelfTest.Ctx ctx) {
        ServerPlayer player = ctx.player("bmh-wear-anvil", 16, 0);
        player.giveExperienceLevels(30);
        ctx.level.setBlockAndUpdate(ctx.origin.offset(16, 1, 2), Blocks.ANVIL.defaultBlockState());
        AnvilMenu menu = new AnvilMenu(9, player.getInventory(), ContainerLevelAccess.create(ctx.level, ctx.origin.offset(16, 1, 2)));
        ItemStack pickaxe = new ItemStack(ItemInit.ETERNAL_PICKAXE.get());
        pickaxe.setDamageValue(100);
        menu.getSlot(0).set(pickaxe);
        menu.getSlot(1).set(new ItemStack(Items.GOLD_BLOCK, 1));
        ItemStack result = menu.getSlot(2).getItem().copy();
        ctx.check("anvil: one gold block repairs the pickaxe for 15 levels", result.getDamageValue() == 0 && menu.getCost() == 15,
                "damage 100 -> " + result.getDamageValue() + ", cost " + menu.getCost());
        menu.clicked(2, 0, ClickType.QUICK_MOVE, player);
        ctx.check("anvil: taking it costs 15 levels", player.experienceLevel == 15, "levels " + player.experienceLevel);
        menu.removed(player);
        // A penalty of 39 makes vanilla refuse any repair (39 + 1 >= 40): the gold repair still works.
        player.giveExperienceLevels(15);
        AnvilMenu worn = new AnvilMenu(11, player.getInventory(), ContainerLevelAccess.create(ctx.level, ctx.origin.offset(16, 1, 2)));
        ItemStack used = new ItemStack(ItemInit.ETERNAL_PICKAXE.get());
        used.setDamageValue(100);
        EternalWear.setRepairPenalty(used, 39);
        worn.getSlot(0).set(used);
        worn.getSlot(1).set(new ItemStack(Items.GOLD_BLOCK, 1));
        ItemStack fixed = worn.getSlot(2).getItem().copy();
        ctx.check("anvil: a penalty of 39 still repairs for 15 levels", !fixed.isEmpty() && fixed.getDamageValue() == 0
                && worn.getCost() == 15, "result " + fixed + ", cost " + worn.getCost());
        ctx.check("anvil: the gold repair keeps the penalty", EternalWear.getRepairPenalty(fixed) == 39
                && EternalWear.getRepairPenalty(worn.getSlot(0).getItem()) == 39,
                "result " + EternalWear.getRepairPenalty(fixed) + ", left " + EternalWear.getRepairPenalty(worn.getSlot(0).getItem()));
        worn.removed(player);
        AnvilMenu control = new AnvilMenu(10, player.getInventory(), ContainerLevelAccess.create(ctx.level, ctx.origin.offset(16, 1, 2)));
        ItemStack netherite = new ItemStack(Items.NETHERITE_PICKAXE);
        netherite.setDamageValue(100);
        control.getSlot(0).set(netherite);
        control.getSlot(1).set(new ItemStack(Items.NETHERITE_INGOT, 1));
        ctx.check("control: a netherite repair keeps its vanilla cost", control.getCost() > 0 && control.getCost() < 15,
                "cost " + control.getCost());
        control.removed(player);
    }

    private static ServerPlayer holding(SelfTest.Ctx ctx, String name, int dx, ItemStack stack) {
        ServerPlayer player = ctx.player(name, dx, 0);
        player.getInventory().items.set(0, stack);
        player.getInventory().selected = 0;
        return player;
    }

    private static IronGolem golem(SelfTest.Ctx ctx, int dx, int dz) {
        IronGolem golem = EntityType.IRON_GOLEM.create(ctx.level);
        golem.moveTo(ctx.origin.getX() + dx + 0.5, ctx.origin.getY() + 1, ctx.origin.getZ() + dz + 0.5, 0, 0);
        golem.setNoAi(true);
        ctx.level.addFreshEntity(golem);
        return golem;
    }
}
