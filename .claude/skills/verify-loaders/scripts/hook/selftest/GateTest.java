package github.mrornithorynque.bmh.selftest;

import net.minecraft.server.level.ServerPlayer;

/** The mock player itself: it joins, ticks, dies, respawns and leaves without an exception on this loader. */
final class GateTest implements SelfTest.Test {

    @Override
    public String name() {
        return "gate";
    }

    @Override
    public void start(SelfTest.Ctx ctx) {
        ServerPlayer player = ctx.player("bmh-gate", 0, 0);
        ctx.check("joined", ctx.live(player) != null, "players online " + ctx.server.getPlayerCount());
        // Only Player.tick (doTick, which fires the loaders' player tick events) refills the attack charge.
        player.resetAttackStrengthTicker();
        float charge = player.getAttackStrengthScale(0);
        int ticks = player.tickCount;
        ctx.later(20, () -> {
            ServerPlayer now = ctx.live(player);
            ctx.check("ticks", now != null && now.getAttackStrengthScale(0) > charge,
                    "attack charge " + charge + " -> " + (now == null ? "gone" : now.getAttackStrengthScale(0))
                            + ", level tickCount " + ticks + " -> " + (now == null ? "gone" : now.tickCount));
            now.kill();
            ctx.later(5, () -> {
                ServerPlayer dead = ctx.live(player);
                ctx.check("dies", dead != null && dead.isDeadOrDying(), "dead " + (dead != null && dead.isDeadOrDying()));
                MockPlayers.respawn(ctx.server, dead);
                ctx.later(5, () -> {
                    ServerPlayer alive = ctx.live(player);
                    ctx.check("respawns", alive != null && alive != dead && alive.isAlive(), "new object " + (alive != dead) + " alive " + (alive != null && alive.isAlive()));
                    ctx.server.getPlayerList().remove(alive);
                    ctx.later(2, () -> ctx.check("leaves", ctx.live(player) == null, "players online " + ctx.server.getPlayerCount()));
                });
            });
        });
    }
}
