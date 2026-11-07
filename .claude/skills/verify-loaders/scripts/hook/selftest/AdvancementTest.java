package github.mrornithorynque.bmh.selftest;

import java.util.ArrayList;
import java.util.List;

import github.mrornithorynque.bmh.handlers.AmuletPoints;
import github.mrornithorynque.bmh.handlers.AmuletReading;
import github.mrornithorynque.bmh.handlers.BmhAdvancements;
import github.mrornithorynque.bmh.handlers.EchoSenseHandler;
import github.mrornithorynque.bmh.handlers.MoonRitualHandler;
import github.mrornithorynque.bmh.handlers.TotemOfEternityHandler;
import github.mrornithorynque.bmh.init.ItemInit;
import github.mrornithorynque.bmh.platform.Services;
import github.mrornithorynque.bmh.utilities.BMHGameRules;
import net.minecraft.advancements.CriteriaTriggers;
import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.stats.Stats;
import net.minecraft.world.Difficulty;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.block.Blocks;

/**
 * The BMH advancement tab (BmhAdvancements). Every advancement of the list is known to the server. The helper awards
 * once: the first call completes the advancement and the second does nothing. A missing path does nothing. The vanilla
 * triggers fire for the items. Every award point in the code awards its advancement: a distant respawn, the totem save,
 * the moon ritual, the three Eternal abilities, the star reading and the named places.
 */
final class AdvancementTest implements SelfTest.Test {

    private static final String[] ALL = {
            "root",
            "journey/exile", "journey/far_away", "journey/here_we_go_again", "journey/home_at_last", "journey/long_way_home",
            "eternity/hollow_totem", "eternity/echo_heart", "eternity/awakening", "eternity/saved", "eternity/moon_ritual",
            "eternity/eternal_tool", "eternity/echo_strike", "eternity/smash", "eternity/echo_sense", "eternity/everything_stays",
            "navigation/amulet", "navigation/stargazer", "navigation/named_place", "navigation/own_sky" };

    @Override
    public String name() {
        return "advancements";
    }

    @Override
    public void start(SelfTest.Ctx ctx) {
        List<String> missing = new ArrayList<>();
        for (String path : ALL) {
            if (!BmhAdvancements.exists(ctx.server, path)) missing.add(path);
        }
        ctx.check("all " + ALL.length + " advancements are loaded", missing.isEmpty(), "missing " + missing);

        itemsAndHelper(ctx);
        awardPoints(ctx);
        exile(ctx);
        tenthDeath(ctx);
    }

    // The helper awards once, and the vanilla triggers see the items.
    private static void itemsAndHelper(SelfTest.Ctx ctx) {
        ServerPlayer player = ctx.player("bmh-adv-items", 0, 0);

        ctx.check("a fresh player has not read the sky", !BmhAdvancements.isDone(player, "navigation/stargazer"), "done at the start");
        boolean first = BmhAdvancements.award(player, "navigation/stargazer");
        boolean second = BmhAdvancements.award(player, "navigation/stargazer");
        ctx.check("the first award completes it", first && BmhAdvancements.isDone(player, "navigation/stargazer"), "first " + first);
        ctx.check("the second award does nothing", !second, "second " + second);
        ctx.check("a missing advancement does nothing", !BmhAdvancements.award(player, "no/such/thing"), "award returned true");

        CriteriaTriggers.TICK.trigger(player);
        ctx.check("the root is awarded by the tick trigger", BmhAdvancements.isDone(player, "root"), "root not done");

        // One Totem of Eternity at most, and all five Eternal Items at once, as "Everything that stays" needs.
        player.getInventory().setItem(0, new ItemStack(ItemInit.HOLLOW_TOTEM.get()));
        player.getInventory().setItem(1, new ItemStack(ItemInit.ECHO_HEART.get()));
        player.getInventory().setItem(2, new ItemStack(ItemInit.TOTEM_OF_ETERNITY.get()));
        player.getInventory().setItem(3, new ItemStack(ItemInit.ETERNAL_SWORD.get()));
        player.getInventory().setItem(4, new ItemStack(ItemInit.ETERNAL_AXE.get()));
        player.getInventory().setItem(5, new ItemStack(ItemInit.ETERNAL_PICKAXE.get()));
        player.getInventory().setItem(6, new ItemStack(ItemInit.ETERNAL_BUNDLE.get()));
        player.getInventory().setItem(7, new ItemStack(ItemInit.NAVIGATOR_AMULET.get()));
        player.containerMenu.broadcastChanges();
        ctx.later(5, () -> {
            ServerPlayer now = ctx.live(player);
            for (String path : new String[] { "eternity/hollow_totem", "eternity/echo_heart", "eternity/awakening",
                    "eternity/eternal_tool", "eternity/everything_stays", "navigation/amulet" }) {
                ctx.check("holding the item completes " + path, BmhAdvancements.isDone(now, path), "not done");
            }
        });
    }

    // The code points that call BmhAdvancements.award, one by one.
    private static void awardPoints(SelfTest.Ctx ctx) {
        ServerPlayer player = ctx.player("bmh-adv-points", 0, 10);
        BlockPos pos = ctx.origin.offset(0, 1, 10);

        // The star reading.
        AmuletReading.record(player, new ItemStack(ItemInit.NAVIGATOR_AMULET.get()));
        ctx.check("a star reading awards navigation/stargazer", BmhAdvancements.isDone(player, "navigation/stargazer"), "not done");

        // The named places: one earns the first advancement, three the second.
        AmuletPoints.put(player, pos, Blocks.LODESTONE.defaultBlockState(), "One");
        ctx.check("one named place awards navigation/named_place", BmhAdvancements.isDone(player, "navigation/named_place"), "not done");
        ctx.check("one named place does not award navigation/own_sky", !BmhAdvancements.isDone(player, "navigation/own_sky"), "done");
        AmuletPoints.put(player, pos.east(5), Blocks.LODESTONE.defaultBlockState(), "Two");
        AmuletPoints.put(player, pos.east(10), Blocks.LODESTONE.defaultBlockState(), "Three");
        ctx.check("three named places award navigation/own_sky", BmhAdvancements.isDone(player, "navigation/own_sky"), "not done");

        // The moon ritual and the totem save.
        MoonRitualHandler.succeed(player, ItemInit.DORMANT_TOTEM_OF_ETERNITY.get());
        ctx.check("the moon ritual awards eternity/moon_ritual", BmhAdvancements.isDone(player, "eternity/moon_ritual"), "not done");
        player.getInventory().setItem(20, new ItemStack(ItemInit.TOTEM_OF_ETERNITY.get()));
        boolean saved = TotemOfEternityHandler.trySave(player);
        ctx.check("the totem save awards eternity/saved", saved && BmhAdvancements.isDone(player, "eternity/saved"), "saved " + saved);

        // Echo Sense: an ore of the same kind next to the broken one shines.
        BlockPos oreSpot = ctx.origin.offset(0, 1, 16);
        ctx.level.setBlockAndUpdate(oreSpot.east(2), Blocks.DIAMOND_ORE.defaultBlockState());
        EchoSenseHandler.onOreBroken(player, oreSpot, Blocks.DIAMOND_ORE.defaultBlockState());
        ctx.check("Echo Sense awards eternity/echo_sense", BmhAdvancements.isDone(player, "eternity/echo_sense"), "not done");
        ctx.level.setBlockAndUpdate(oreSpot.east(2), Blocks.AIR.defaultBlockState());

        // The sword's echo and the axe's smash.
        ServerPlayer fighter = ctx.player("bmh-adv-fight", 0, 20);
        fighter.getInventory().setItem(0, new ItemStack(ItemInit.ETERNAL_SWORD.get()));
        fighter.getInventory().setItem(1, new ItemStack(ItemInit.ETERNAL_AXE.get()));
        MockPlayers.select(fighter, 0);
        // The echo picks an Enemy, so husks (they stay in easy, not in peaceful).
        Difficulty before = ctx.level.getDifficulty();
        ctx.server.setDifficulty(Difficulty.EASY, true);
        LivingEntity near = MockPlayers.husk(ctx.level, ctx.origin.getX() + 0.5, ctx.origin.getY() + 1, ctx.origin.getZ() + 22.5);
        LivingEntity far = MockPlayers.husk(ctx.level, ctx.origin.getX() + 0.5, ctx.origin.getY() + 1, ctx.origin.getZ() + 25.5);
        ctx.later(30, () -> {
            ServerPlayer now = ctx.live(fighter);
            now.fallDistance = 0;
            now.setOnGround(true);
            now.attack(near);
            ctx.later(15, () -> {
                ctx.check("the sword's echo awards eternity/echo_strike", BmhAdvancements.isDone(ctx.live(fighter), "eternity/echo_strike"),
                        "not done, the second golem lost " + (far.getMaxHealth() - far.getHealth()) + " health");
                ServerPlayer axe = ctx.live(fighter);
                MockPlayers.select(axe, 1);
                ctx.later(30, () -> {
                    ServerPlayer air = ctx.live(fighter);
                    air.fallDistance = 10;
                    air.setOnGround(false);
                    // A fresh husk: the sword may have killed the first one.
                    LivingEntity target = MockPlayers.husk(ctx.level, ctx.origin.getX() + 0.5, ctx.origin.getY() + 1, ctx.origin.getZ() + 22.5);
                    air.attack(target);
                    ctx.check("the axe's smash awards eternity/smash", BmhAdvancements.isDone(air, "eternity/smash"), "not done");
                    target.discard();
                    near.discard();
                    far.discard();
                    ctx.server.setDifficulty(before, true);
                });
            });
        });
    }

    // A distant respawn awards the exile, and "Far, far away" when the exile is 30000 blocks or more. The formula gives
    // less than 30000 on peaceful with the default config and more on hard, so both answers are checked.
    private static void exile(SelfTest.Ctx ctx) {
        ctx.rule(BMHGameRules.RULE_RANDOM_DEATH_SPAWN_POINT, true);
        Difficulty before = ctx.level.getDifficulty();
        ctx.later(60, () -> {
            ctx.server.setDifficulty(Difficulty.PEACEFUL, true);
            die(ctx, "peaceful", 30, () -> ctx.later(20, () -> {
                ctx.server.setDifficulty(Difficulty.HARD, true);
                die(ctx, "hard", 36, () -> ctx.server.setDifficulty(before, true));
            }));
        });
    }

    private static void die(SelfTest.Ctx ctx, String label, int dz, Runnable then) {
        ServerPlayer player = ctx.player("bmh-adv-exile-" + label, 0, dz);
        player.getInventory().setItem(0, new ItemStack(ItemInit.ETERNAL_SWORD.get()));
        player.getInventory().setItem(1, new ItemStack(ItemInit.ETERNAL_PICKAXE.get()));
        player.getInventory().setItem(2, new ItemStack(ItemInit.ETERNAL_AXE.get()));
        MockPlayers.sent(player);
        MockPlayers.kill(player);
        ctx.later(5, () -> {
            ServerPlayer back = MockPlayers.respawn(ctx.server, ctx.live(player));
            int min = Services.PLATFORM.minDistanceFromBed();
            int max = Services.PLATFORM.maxDistanceFromBed();
            int easyLife = Math.min(9, 1 + 3 + ctx.level.getDifficulty().getId());
            int expected = min + (max - min) * easyLife / 9;
            ctx.check(label + ": a distant respawn awards journey/exile", BmhAdvancements.isDone(back, "journey/exile"), "not done");
            boolean far = BmhAdvancements.isDone(back, "journey/far_away");
            ctx.check(label + ": the first death does not award the hidden journey/here_we_go_again",
                    !BmhAdvancements.isDone(back, "journey/here_we_go_again"), "done");
            ctx.check(label + ": journey/far_away follows the distance (expected " + expected + ")", far == (expected >= 30000),
                    "far_away " + far);
            then.run();
        });
    }

    // The tenth death awards the hidden advancement: the statistic starts at 9, so one more death makes 10.
    private static void tenthDeath(SelfTest.Ctx ctx) {
        ServerPlayer player = ctx.player("bmh-adv-tenth", 0, 42);
        player.getStats().setValue(player, Stats.CUSTOM.get(Stats.DEATHS), 9);
        MockPlayers.kill(player);
        ctx.later(5, () -> {
            ServerPlayer back = MockPlayers.respawn(ctx.server, ctx.live(player));
            ctx.check("the tenth death awards journey/here_we_go_again", BmhAdvancements.isDone(back, "journey/here_we_go_again"),
                    "deaths " + back.getStats().getValue(Stats.CUSTOM.get(Stats.DEATHS)));
        });
    }
}
