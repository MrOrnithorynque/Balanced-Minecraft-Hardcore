package github.mrornithorynque.bmh.handlers;

import org.slf4j.Logger;

import com.mojang.logging.LogUtils;

import net.minecraft.core.particles.ParticleTypes;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.sounds.SoundEvents;
import net.minecraft.sounds.SoundSource;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.player.Player;

/**
 * The Eternal Axe's smash: a hit landed while falling hits harder the longer the fall, and the
 * blow breaks the fall. No tree felling.
 *
 * The server does not track a player's velocity reliably on 1.20.x, so the speed comes from the
 * fall: free fall gains 0.08 blocks/tick per tick, so after {@code d} blocks the speed is
 * {@code sqrt(2 * 0.08 * d)} blocks per tick. Damage is multiplied by {@code 1 + speed}, capped at 3.
 *
 * Fed by {@code E_PlayerAttackMixin}: the multiplier before the hit, the effects once it landed.
 */
public class AxeSmashHandler {

    private static final Logger LOGGER = LogUtils.getLogger();

    private static final float MIN_FALL_DISTANCE = 1.5F;
    private static final float MAX_MULTIPLIER = 3.0F;
    // Vanilla gravity, blocks per tick squared.
    private static final double GRAVITY = 0.08D;

    /** The multiplier a hit would get right now: 1 when the player is not smashing. */
    public static float multiplier(Player player) {

        if (!isSmashing(player)) {
            return 1.0F;
        }

        double speed = Math.sqrt(2.0D * GRAVITY * player.fallDistance);

        return (float) Math.min(MAX_MULTIPLIER, 1.0D + speed);
    }

    private static boolean isSmashing(Player player) {

        return player.fallDistance > MIN_FALL_DISTANCE
                && !player.onGround()
                && !player.isFallFlying()
                && !player.isInWater()
                && !player.onClimbable()
                && !player.isPassenger();
    }

    /** Called once the smashing hit has landed: breaks the fall and makes the impact felt. */
    public static void onSmash(ServerPlayer attacker, Entity target, float multiplier) {

        LOGGER.debug("[BalancedMcHardcoreMain] Smash: {} hits x{} after a {} block fall",
                attacker.getGameProfile().getName(), multiplier, attacker.fallDistance);

        BmhAdvancements.award(attacker, "eternity/smash");

        // Half the fall distance, about half the fall damage.
        attacker.fallDistance *= 0.5F;

        ServerLevel level = attacker.serverLevel();
        int dust = Math.round(10.0F * multiplier);

        level.playSound(null, target.getX(), target.getY(), target.getZ(),
                SoundEvents.ANVIL_LAND, SoundSource.PLAYERS, 0.6F, 0.7F);
        level.playSound(null, target.getX(), target.getY(), target.getZ(),
                SoundEvents.PLAYER_ATTACK_KNOCKBACK, SoundSource.PLAYERS, 1.0F, 0.6F);

        level.sendParticles(ParticleTypes.EXPLOSION, target.getX(), target.getY() + 0.1D, target.getZ(),
                1, 0.0D, 0.0D, 0.0D, 0.0D);
        level.sendParticles(ParticleTypes.CLOUD, target.getX(), target.getY() + 0.1D, target.getZ(),
                dust, 0.5D, 0.05D, 0.5D, 0.15D);
        level.sendParticles(ParticleTypes.CRIT, target.getX(), target.getY(0.5D), target.getZ(),
                dust, 0.4D, 0.4D, 0.4D, 0.5D);
    }
}
