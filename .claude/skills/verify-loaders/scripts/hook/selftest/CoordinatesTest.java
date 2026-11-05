package github.mrornithorynque.bmh.selftest;

import github.mrornithorynque.bmh.utilities.BMHGameRules;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.level.GameRules;
import net.minecraft.world.level.GameType;

/**
 * removeCoordinatesInSurvival (DisableDebugInfo): while a survival player is online, vanilla's reducedDebugInfo is
 * forced on (no coordinates on F3). Once nobody is in survival, the old value comes back.
 */
final class CoordinatesTest implements SelfTest.Test {

    @Override
    public String name() {
        return "coordinates hidden";
    }

    @Override
    public void start(SelfTest.Ctx ctx) {
        ctx.rule(BMHGameRules.RULE_REMOVE_COORDINATES_IN_SURVIVAL, false);
        ServerPlayer player = ctx.player("bmh-coords", 0, 0);
        ctx.later(5, () -> {
            ctx.rule(GameRules.RULE_REDUCEDDEBUGINFO, false);
            ctx.rule(BMHGameRules.RULE_REMOVE_COORDINATES_IN_SURVIVAL, true);
            ctx.later(5, () -> {
                ctx.check("survival player: reducedDebugInfo forced on", reduced(ctx), "reducedDebugInfo " + reduced(ctx));
                ctx.live(player).setGameMode(GameType.CREATIVE);
                ctx.later(5, () -> ctx.check("nobody in survival: reducedDebugInfo back off", !reduced(ctx),
                        "reducedDebugInfo " + reduced(ctx)));
            });
        });
    }

    private static boolean reduced(SelfTest.Ctx ctx) {
        return ctx.server.getGameRules().getBoolean(GameRules.RULE_REDUCEDDEBUGINFO);
    }
}
