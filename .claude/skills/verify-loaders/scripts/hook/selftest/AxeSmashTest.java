package github.mrornithorynque.bmh.selftest;

import github.mrornithorynque.bmh.init.ItemInit;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.EntityType;
import net.minecraft.world.entity.animal.IronGolem;
import net.minecraft.world.item.ItemStack;

/**
 * The Eternal Axe's smash (AxeSmashHandler). A full-strength hit after a 10 block fall deals at least 2.5 times the
 * damage of the same hit standing. The smash gives about x2.26 at 10 blocks, and the vanilla critical (x1.5) comes on
 * top. The blow also halves the fall distance. Iron golems take the hits: they have 100 health and stay in peaceful.
 */
final class AxeSmashTest implements SelfTest.Test {

    @Override
    public String name() {
        return "axe smash";
    }

    @Override
    public void start(SelfTest.Ctx ctx) {
        ServerPlayer player = ctx.player("bmh-axe", 0, 0);
        player.getInventory().items.set(0, new ItemStack(ItemInit.ETERNAL_AXE.get()));
        player.getInventory().selected = 0;
        IronGolem standing = golem(ctx, 2);
        IronGolem falling = golem(ctx, -2);
        float[] damage = new float[2];
        ctx.later(30, () -> {
            ServerPlayer now = ctx.live(player);
            now.fallDistance = 0;
            now.setOnGround(true);
            now.attack(standing);
            damage[0] = standing.getMaxHealth() - standing.getHealth();
            ctx.later(30, () -> {
                ServerPlayer air = ctx.live(player);
                air.fallDistance = 10;
                air.setOnGround(false);
                air.attack(falling);
                damage[1] = falling.getMaxHealth() - falling.getHealth();
                ctx.check("standing hit lands", damage[0] > 0, "damage " + damage[0]);
                ctx.check("falling hit smashes (>= 2.5x)", damage[1] >= 2.5f * damage[0],
                        "standing " + damage[0] + ", falling " + damage[1]);
                ctx.check("the smash halves the fall", air.fallDistance == 5f, "fallDistance " + air.fallDistance);
                standing.discard();
                falling.discard();
            });
        });
    }

    private static IronGolem golem(SelfTest.Ctx ctx, int dz) {
        IronGolem golem = EntityType.IRON_GOLEM.create(ctx.level);
        golem.moveTo(ctx.origin.getX() + 0.5, ctx.origin.getY() + 1, ctx.origin.getZ() + dz + 0.5, 0, 0);
        golem.setNoAi(true);
        ctx.level.addFreshEntity(golem);
        return golem;
    }
}
