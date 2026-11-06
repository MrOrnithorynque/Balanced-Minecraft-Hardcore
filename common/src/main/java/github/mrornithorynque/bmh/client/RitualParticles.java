package github.mrornithorynque.bmh.client;

import github.mrornithorynque.bmh.handlers.MoonRitualHandler;

import net.minecraft.client.Minecraft;
import net.minecraft.core.particles.ParticleTypes;
import net.minecraft.util.RandomSource;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.level.Level;
import net.minecraft.world.phys.Vec3;

/**
 * The light of the moon ritual: particles drift from all around into the totem. The enchanting table particle flies
 * to an exact point and goes out there, so the light ends in the totem and stays out of the view. Its pale blue fits
 * the moon. The particles start on the far side of the totem from the eyes, so none passes in front of the camera. It ends 1.2 blocks below the point it is given (vanilla drops it into the enchanting table's book), so the
 * point given is 1.2 blocks above the totem.
 * <p>
 * In first person the totem is where RitualHandRenderer draws it, in front of the eyes. Other players and the third
 * person views see the totem in the raised hands.
 */
public final class RitualParticles {

    // The particles start between these distances from the totem.
    private static final double MIN_DISTANCE = 1.2;
    private static final double MAX_DISTANCE = 2.4;
    // Vanilla's enchanting table particle ends this far below the point it is given.
    private static final double DROP = 1.2;
    private static final int PER_TICK = 1;

    private RitualParticles() {
    }

    /** Particles for one tick of the ritual. Every client calls it for every player in a ritual. */
    public static void spawn(Level level, Player player) {

        Minecraft minecraft = Minecraft.getInstance();
        boolean firstPerson = player == minecraft.player && minecraft.options.getCameraType().isFirstPerson();
        Vec3 totem = firstPerson ? RitualHandRenderer.totemPosition(player) : MoonRitualHandler.raisedTotemPosition(player);
        Vec3 away = totem.subtract(minecraft.gameRenderer.getMainCamera().position()).normalize();
        RandomSource random = player.getRandom();

        for (int i = 0; i < PER_TICK; i++) {

            Vec3 direction = new Vec3(random.nextGaussian(), random.nextGaussian(), random.nextGaussian()).normalize();

            // Keep the start on the far side of the totem, as seen from the camera.
            if (direction.dot(away) < 0.0) {
                direction = direction.subtract(away.scale(2.0 * direction.dot(away)));
            }

            Vec3 start = totem.add(direction.scale(MIN_DISTANCE + random.nextDouble() * (MAX_DISTANCE - MIN_DISTANCE)));

            level.addParticle(ParticleTypes.ENCHANT, totem.x, totem.y + DROP, totem.z,
                    start.x - totem.x, start.y - totem.y - DROP, start.z - totem.z);
        }
    }
}
