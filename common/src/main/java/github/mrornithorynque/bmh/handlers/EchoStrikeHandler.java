package github.mrornithorynque.bmh.handlers;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.UUID;

import org.joml.Vector3f;
import org.slf4j.Logger;

import com.mojang.logging.LogUtils;

import net.minecraft.core.particles.DustParticleOptions;
import net.minecraft.core.particles.ParticleTypes;
import net.minecraft.resources.ResourceKey;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.sounds.SoundEvents;
import net.minecraft.sounds.SoundSource;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.entity.OwnableEntity;
import net.minecraft.world.entity.decoration.ArmorStand;
import net.minecraft.world.entity.monster.Enemy;
import net.minecraft.world.entity.npc.AbstractVillager;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.level.Level;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;

/**
 * Echo Strike, the Eternal Sword's ability: a full-strength hit that lands echoes a few ticks later
 * onto the nearest other enemy around the target, for half the damage. It jumps to another
 * mob on purpose: a second hit on the same target would land inside its invulnerability frames
 * and add nothing.
 *
 * Fed by {@code E_PlayerAttackMixin} (the hit) and {@code E_ServerPlayerMixin} (the clock). The echo
 * hurts its victim directly and never goes through {@code Player.attack}, so it cannot echo again.
 */
public class EchoStrikeHandler {

    private static final Logger LOGGER = LogUtils.getLogger();

    private static final float MIN_ATTACK_STRENGTH = 0.9F;
    private static final float DAMAGE_RATIO = 0.5F;
    private static final int DELAY_TICKS = 5;
    private static final double RANGE = 5.0D;
    // A fake player is never ticked by the level: do not let its echoes pile up.
    private static final int MAX_PENDING_ECHOES = 8;

    private static final DustParticleOptions CYAN = new DustParticleOptions(new Vector3f(0.1F, 0.9F, 1.0F), 1.0F);

    /** An echo on its way: where the hit landed, the damage it will deal, and when. */
    public record PendingEcho(ResourceKey<Level> dimension, UUID targetId, Vec3 origin, float damage, long dueGameTime) {
    }

    /**
     * Called when a hit of the Eternal Sword has landed.
     *
     * @param damage         what the hit handed to {@code hurt}, crit and enchantments included
     * @param attackStrength the attacker's attack strength when the swing started
     */
    public static void onSwordHit(ServerPlayer attacker, Entity target, float damage, float attackStrength) {

        if (attackStrength < MIN_ATTACK_STRENGTH || damage <= 0.0F || !(target instanceof LivingEntity)) {
            return;
        }

        List<PendingEcho> pending = ((EchoStrikeQueue) attacker).bmh$pendingEchoes();

        if (pending.size() >= MAX_PENDING_ECHOES) {
            pending.remove(0);
        }

        ServerLevel level = attacker.serverLevel();

        pending.add(new PendingEcho(
                level.dimension(),
                target.getUUID(),
                center(target),
                damage * DAMAGE_RATIO,
                level.getGameTime() + DELAY_TICKS));
    }

    /** Called at the start of every tick of the attacker: lands the echoes that are due. */
    public static void deliverDueEchoes(ServerPlayer attacker) {

        List<PendingEcho> pending = ((EchoStrikeQueue) attacker).bmh$pendingEchoes();
        ServerLevel level = attacker.serverLevel();
        long now = level.getGameTime();

        // Out of the queue first: delivering hurts a mob, and a mod listening to that may start another swing.
        List<PendingEcho> due = new ArrayList<>();

        for (Iterator<PendingEcho> iterator = pending.iterator(); iterator.hasNext();) {

            PendingEcho echo = iterator.next();

            if (echo.dueGameTime() <= now) {
                iterator.remove();
                due.add(echo);
            }
        }

        for (PendingEcho echo : due) {

            // The attacker may have changed dimension since the hit: the echo is lost with it.
            if (level.dimension().equals(echo.dimension())) {
                deliver(attacker, level, echo);
            }
        }
    }

    private static void deliver(ServerPlayer attacker, ServerLevel level, PendingEcho echo) {

        // The target may have died or been knocked away since the hit: follow it while it lives.
        Entity target = level.getEntity(echo.targetId());
        Vec3 origin = target != null && target.isAlive() ? center(target) : echo.origin();

        LivingEntity victim = nearestVictim(attacker, level, origin, echo.targetId());

        if (victim == null) {
            return;
        }

        Vec3 end = center(victim);

        // Invulnerability frames can absorb the echo: then there is nothing to show either.
        if (!victim.hurt(level.damageSources().playerAttack(attacker), echo.damage())) {
            return;
        }

        drawLine(level, origin, end);
        level.playSound(null, end.x, end.y, end.z, SoundEvents.AMETHYST_BLOCK_RESONATE, SoundSource.PLAYERS, 0.6F, 1.6F);

        LOGGER.debug("[BalancedMcHardcoreMain] Echo Strike: {} echoes {} onto {} ({} blocks away)",
                attacker.getGameProfile().getName(), echo.damage(), victim.getName().getString(), origin.distanceTo(end));
    }

    private static LivingEntity nearestVictim(ServerPlayer attacker, ServerLevel level, Vec3 origin, UUID targetId) {

        AABB area = new AABB(origin, origin).inflate(RANGE);
        LivingEntity nearest = null;
        double nearestDistance = RANGE * RANGE;

        for (LivingEntity candidate : level.getEntitiesOfClass(LivingEntity.class, area,
                entity -> isValidVictim(attacker, entity, targetId))) {

            double distance = center(candidate).distanceToSqr(origin);

            if (distance <= nearestDistance) {
                nearest = candidate;
                nearestDistance = distance;
            }
        }

        return nearest;
    }

    // An enemy (a hostile mob), minus what the Eternal Sword must never turn on. The brief lists the spared
    // kinds (players, villagers, armor stands, the attacker's pets) and says "enemy": an Enemy is neither,
    // and animals, golems and allays are not hurt by a fight next to them.
    private static boolean isValidVictim(ServerPlayer attacker, LivingEntity entity, UUID targetId) {

        if (!(entity instanceof Enemy) || entity == attacker || entity.getUUID().equals(targetId)) {
            return false;
        }

        if (!entity.isAlive() || entity.isInvulnerable() || !entity.isAttackable()) {
            return false;
        }

        if (entity instanceof Player || entity instanceof ArmorStand || entity instanceof AbstractVillager) {
            return false;
        }

        // The attacker's own tamed animals (wolves, cats, parrots, horses...).
        if (entity instanceof OwnableEntity pet && attacker.getUUID().equals(pet.getOwnerUUID())) {
            return false;
        }

        // What the attacker is riding, and teammates' pets and mounts.
        return entity != attacker.getVehicle() && !attacker.hasPassenger(entity) && !entity.isAlliedTo(attacker);
    }

    private static void drawLine(ServerLevel level, Vec3 from, Vec3 to) {

        double length = from.distanceTo(to);
        int steps = Math.max(2, (int) Math.ceil(length / 0.25D));

        for (int i = 0; i <= steps; i++) {

            double t = (double) i / steps;

            level.sendParticles(CYAN,
                    from.x + (to.x - from.x) * t,
                    from.y + (to.y - from.y) * t,
                    from.z + (to.z - from.z) * t,
                    1, 0.0D, 0.0D, 0.0D, 0.0D);
        }

        level.sendParticles(ParticleTypes.ELECTRIC_SPARK, to.x, to.y, to.z, 6, 0.2D, 0.2D, 0.2D, 0.1D);
    }

    private static Vec3 center(Entity entity) {

        return entity.position().add(0.0D, entity.getBbHeight() * 0.5D, 0.0D);
    }
}
