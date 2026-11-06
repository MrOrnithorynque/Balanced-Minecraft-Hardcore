package github.mrornithorynque.bmh.selftest;

import github.mrornithorynque.bmh.utilities.BMHGameRules;
import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.effect.MobEffectInstance;
import net.minecraft.world.level.block.Blocks;

/**
 * foodAndHealthRegenOnWakeUp (SleepHandler). Waking up trades food for health by tiers. A well fed player at full
 * health also gets Haste II and Resistance II for 3 minutes, with no particles. With the rule off, nothing changes.
 * Peaceful refills food and health on every player tick, so each case checks right after the wake-up, in the same step.
 */
final class SleepTest implements SelfTest.Test {

    @Override
    public String name() {
        return "sleep buff";
    }

    @Override
    public void start(SelfTest.Ctx ctx) {
        ctx.rule(BMHGameRules.RULE_FOOD_AND_HEALTH_REGEN_ON_WAKE_UP, true);
        ServerPlayer full = wake(ctx, "full", 0, 20, 20f);
        ctx.check("full: food 20 -> 16", full.getFoodData().getFoodLevel() == 16, "food " + full.getFoodData().getFoodLevel());
        MobEffectInstance haste = MockPlayers.haste(full);
        MobEffectInstance resistance = MockPlayers.resistance(full);
        ctx.check("full: Haste II, 3 minutes, no particles", haste != null && haste.getAmplifier() == 1 && haste.getDuration() > 3500
                && !haste.isVisible(), "haste " + haste);
        ctx.check("full: Resistance II, 3 minutes, no particles", resistance != null && resistance.getAmplifier() == 1
                && resistance.getDuration() > 3500 && !resistance.isVisible(), "resistance " + resistance);

        ServerPlayer half = wake(ctx, "half", 8, 12, 10f);
        ctx.check("half: health 10 -> 14", half.getHealth() == 14f, "health " + half.getHealth());
        ctx.check("half: food 12 -> 10", half.getFoodData().getFoodLevel() == 10, "food " + half.getFoodData().getFoodLevel());
        ctx.check("half: no Haste below full health", MockPlayers.haste(half) == null, "haste " + MockPlayers.haste(half));

        ctx.rule(BMHGameRules.RULE_FOOD_AND_HEALTH_REGEN_ON_WAKE_UP, false);
        ServerPlayer off = wake(ctx, "rule off", 16, 20, 10f);
        ctx.check("rule off: health and food stay", off.getHealth() == 10f && off.getFoodData().getFoodLevel() == 20,
                "health " + off.getHealth() + ", food " + off.getFoodData().getFoodLevel());
    }

    // A bed under the player, the vanilla sleep and wake-up: the loaders' wake-up hooks fire as in a real night.
    private static ServerPlayer wake(SelfTest.Ctx ctx, String label, int dx, int food, float health) {
        ServerPlayer player = ctx.player("bmh-sleep-" + dx, dx, 0);
        BlockPos bed = player.blockPosition();
        ctx.level.setBlockAndUpdate(bed, Blocks.RED_BED.defaultBlockState());
        player.getFoodData().setFoodLevel(food);
        player.setHealth(health);
        player.startSleeping(bed);
        player.stopSleepInBed(true, true);
        return player;
    }
}
