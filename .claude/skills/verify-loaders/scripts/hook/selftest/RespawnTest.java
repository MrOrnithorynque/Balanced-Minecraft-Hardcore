package github.mrornithorynque.bmh.selftest;

import java.util.List;

import github.mrornithorynque.bmh.handlers.JourneySignal;
import github.mrornithorynque.bmh.init.ItemInit;
import github.mrornithorynque.bmh.platform.Services;
import github.mrornithorynque.bmh.utilities.BMHGameRules;
import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.item.ItemStack;

/**
 * The distant respawn. The distance follows the formula of RespawnPositionHandler: config min and max, difficulty and
 * the Eternal Items carried. The player gets 200 ticks of immunity. The client gets the departure title at death and
 * the arrival title after the respawn, with the message only when displayTextOnRespawn is on. With
 * randomDeathSpawnPoint off, the player respawns near the world spawn with vanilla's 60 ticks and no journey title.
 */
final class RespawnTest implements SelfTest.Test {

    @Override
    public String name() {
        return "distant respawn";
    }

    @Override
    public void start(SelfTest.Ctx ctx) {
        ctx.rule(BMHGameRules.RULE_RANDOM_DEATH_SPAWN_POINT, true);
        ctx.rule(BMHGameRules.RULE_DISPLAY_TEXT_ON_RESPAWN, true);
        die(ctx, "far, with text", 0, true, true, 0);
        ctx.later(80, () -> {
            ctx.rule(BMHGameRules.RULE_DISPLAY_TEXT_ON_RESPAWN, false);
            // Two Eternal Items make the exile longer: 2 more in the formula.
            die(ctx, "far, no text, 2 Eternal Items", 8, true, false, 2);
            ctx.later(80, () -> {
                ctx.rule(BMHGameRules.RULE_RANDOM_DEATH_SPAWN_POINT, false);
                die(ctx, "rule off", 16, false, false, 0);
            });
        });
    }

    private static void die(SelfTest.Ctx ctx, String label, int dx, boolean far, boolean text, int eternal) {
        ServerPlayer player = ctx.player("bmh-respawn-" + dx, dx, 0);
        if (eternal >= 1) player.getInventory().setItem(0, new ItemStack(ItemInit.ETERNAL_SWORD.get()));
        if (eternal >= 2) player.getInventory().setItem(1, new ItemStack(ItemInit.ETERNAL_PICKAXE.get()));
        MockPlayers.sent(player);
        MockPlayers.kill(player);
        List<String> atDeath = ctx.titles(player);
        String departure = JourneySignal.DEPARTURE;
        ctx.check(label + ": departure title " + (far ? "sent" : "not sent") + " at death",
                atDeath.contains(departure) == far, "titles " + atDeath);
        ctx.later(5, () -> {
            ServerPlayer back = MockPlayers.respawn(ctx.server, ctx.live(player));
            int immunity = ctx.spawnInvulnerableTime(back);
            List<String> afterRespawn = ctx.titles(back);
            BlockPos spawn = MockPlayers.worldSpawn(ctx.level);
            double distance = Math.sqrt(Math.pow(back.getX() - spawn.getX(), 2) + Math.pow(back.getZ() - spawn.getZ(), 2));
            if (far) {
                int min = Services.PLATFORM.minDistanceFromBed();
                int max = Services.PLATFORM.maxDistanceFromBed();
                int easyLife = Math.min(9, 1 + eternal + ctx.level.getDifficulty().getId());
                int expected = min + (max - min) * easyLife / 9;
                ctx.check(label + ": respawn distance follows the formula", Math.abs(distance - expected) <= 3,
                        String.format("%.0f blocks from the world spawn, expected %d (min %d, max %d, difficulty %s)",
                                distance, expected, min, max, ctx.level.getDifficulty()));
                ctx.check(label + ": 200 ticks of immunity", immunity == 200, "spawnInvulnerableTime " + immunity);
                String arrival = JourneySignal.ARRIVAL + (text ? "(1 args)" : "");
                ctx.check(label + ": arrival title " + (text ? "with" : "without") + " the message", afterRespawn.contains(arrival),
                        "titles " + afterRespawn + ", expected " + arrival);
            } else {
                ctx.check(label + ": respawn near the world spawn", distance < 64, String.format("%.0f blocks", distance));
                ctx.check(label + ": vanilla immunity", immunity <= 60, "spawnInvulnerableTime " + immunity);
                ctx.check(label + ": no journey title", afterRespawn.stream().noneMatch(t -> t.startsWith("bmh.journey")),
                        "titles " + afterRespawn);
            }
        });
    }
}
