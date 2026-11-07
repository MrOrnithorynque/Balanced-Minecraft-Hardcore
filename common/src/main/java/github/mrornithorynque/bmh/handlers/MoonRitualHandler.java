package github.mrornithorynque.bmh.handlers;

import javax.annotation.Nullable;

import org.slf4j.Logger;

import com.mojang.logging.LogUtils;

import github.mrornithorynque.bmh.init.ItemInit;

import net.minecraft.core.particles.ParticleTypes;
import net.minecraft.network.chat.Component;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.sounds.SoundEvents;
import net.minecraft.sounds.SoundSource;
import net.minecraft.stats.Stats;
import net.minecraft.util.Mth;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.entity.HumanoidArm;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.Level;
import net.minecraft.world.phys.Vec3;

/**
 * The moon ritual wakes a Dormant Totem of Eternity. The player holds use with the totem and looks at the
 * moon at night. The ritual drains the levels one by one. At the end, the totem is a Totem of Eternity again.
 * The ritual must be done in one go: a release, a look away, a hidden moon, damage or no levels left end it.
 * A failed ritual keeps the levels it drained. The item (items/DormantTotemOfEternityItem) calls this class.
 */
public class MoonRitualHandler {

    /** The levels that the ritual costs. */
    public static final int RITUAL_LEVELS = 30;
    /** The length of the ritual, in ticks. */
    public static final int RITUAL_TICKS = 300;
    /** The ritual drains one level each time this number of ticks goes by. */
    public static final int TICKS_PER_LEVEL = RITUAL_TICKS / RITUAL_LEVELS;
    /** The largest angle between the look direction and the moon, in degrees. */
    public static final double AIM_DEGREES = 10.0;

    // The pitch of the chime at the first level and at the last one.
    private static final float CHIME_LOW_PITCH = 0.5F;
    private static final float CHIME_HIGH_PITCH = 1.9F;

    /** A player who fails must wait this time before the next try, so a held use key does not start again at once. */
    public static final int FAIL_COOLDOWN_TICKS = 20;

    private static final double AIM_COS = Math.cos(Math.toRadians(AIM_DEGREES));

    private static final Logger LOGGER = LogUtils.getLogger();

    /**
     * The unit vector from the player toward the moon. LevelRenderer turns the sky by -90 degrees around Y, then by
     * timeOfDay * 360 degrees around X, and draws the moon at y = -100. So the moon is at (sin t, -cos t, 0).
     */
    public static Vec3 moonDirection(Level level) {

        double angle = level.getTimeOfDay(1.0F) * Math.PI * 2.0;

        return new Vec3(Math.sin(angle), -Math.cos(angle), 0.0);
    }

    /** True when the look direction is within AIM_DEGREES of the moon. */
    public static boolean aimsAtMoon(Player player) {

        return player.getViewVector(1.0F).dot(moonDirection(player.level())) >= AIM_COS;
    }

    /** The angle between the look direction and the moon, in degrees, for the log. */
    public static double aimError(Player player) {

        double dot = Mth.clamp(player.getViewVector(1.0F).dot(moonDirection(player.level())), -1.0, 1.0);

        return Math.toDegrees(Math.acos(dot));
    }

    /** Null when a ritual can start or go on, else the lang key that says why not. The server decides. */
    @Nullable
    public static String blockedReason(Player player) {

        String hidden = AmuletReading.skyHiddenReason(player, "bmh.ritual.hidden");

        if (hidden != null) {
            return hidden;
        }

        if (!aimsAtMoon(player)) {
            return "bmh.ritual.aim";
        }

        return null;
    }

    /** The ritual starts. The server tells the log. */
    public static void start(ServerPlayer player) {

        player.level().playSound(null, player.getX(), player.getY(), player.getZ(), SoundEvents.RESPAWN_ANCHOR_CHARGE,
                SoundSource.PLAYERS, 0.8F, 1.3F);

        LOGGER.info("[BalancedMcHardcoreMain] Moon ritual started by " + player.getName().getString() + ": aimed "
                + String.format("%.1f", aimError(player)) + " degrees from the moon, " + player.experienceLevel + " levels");
    }

    /**
     * One tick of the ritual on the server. remaining goes from RITUAL_TICKS down to 1. False when the ritual fails
     * in this tick: the caller then stops the use.
     */
    public static boolean tick(ServerPlayer player, int remaining) {

        String reason = blockedReason(player);

        if (reason != null) {
            fail(player, "bmh.ritual.aim".equals(reason) ? "bmh.ritual.looked_away" : reason);
            return false;
        }

        if (player.hurtTime > 0) {
            fail(player, "bmh.ritual.hurt");
            return false;
        }

        int elapsed = RITUAL_TICKS - remaining + 1;

        if (elapsed % TICKS_PER_LEVEL == 0) {

            if (!player.getAbilities().instabuild) {

                if (player.experienceLevel < 1) {
                    fail(player, "bmh.ritual.levels");
                    return false;
                }

                player.giveExperienceLevels(-1);
            }

            // A chime at each level, higher and higher: the ritual climbs toward the end.
            float pitch = CHIME_LOW_PITCH + (CHIME_HIGH_PITCH - CHIME_LOW_PITCH) * elapsed / RITUAL_TICKS;
            player.level().playSound(null, player.getX(), player.getY(), player.getZ(), SoundEvents.AMETHYST_BLOCK_CHIME,
                    SoundSource.PLAYERS, 1.0F, pitch);
        }

        return true;
    }

    /** A release before the end is a failure too. */
    public static void release(ServerPlayer player) {

        fail(player, "bmh.ritual.released");
    }

    /**
     * The ritual fails: the totem stays dormant and the drained levels stay gone. A puff of smoke and a fizzle,
     * which nearby players see and hear too.
     */
    public static void fail(ServerPlayer player, String reason) {

        ServerLevel level = player.serverLevel();

        // The puff is at the raised totem for everyone else. The player sees it 2.5 blocks in front of the eyes: a
        // puff at the totem itself is so close to the camera that it covers the whole view.
        for (ServerPlayer viewer : level.players()) {
            Vec3 at = viewer == player
                    ? player.getEyePosition().add(player.getViewVector(1.0F).scale(2.5))
                    : raisedTotemPosition(player);
            level.sendParticles(viewer, ParticleTypes.POOF, false, at.x, at.y, at.z, 6, 0.1, 0.1, 0.1, 0.02);
        }
        level.playSound(null, player.getX(), player.getY(), player.getZ(), SoundEvents.FIRE_EXTINGUISH, SoundSource.PLAYERS,
                0.7F, 1.4F);
        player.displayClientMessage(Component.translatable(reason), true);
        player.getCooldowns().addCooldown(ItemInit.DORMANT_TOTEM_OF_ETERNITY.get(), FAIL_COOLDOWN_TICKS);

        LOGGER.info("[BalancedMcHardcoreMain] Moon ritual failed for " + player.getName().getString() + ": " + reason
                + ", aimed " + String.format("%.1f", aimError(player)) + " degrees from the moon, "
                + player.experienceLevel + " levels left");
    }

    /**
     * The ritual succeeds. The server shows the totem animation (entity event 35) and plays a chime. The caller
     * returns the new stack, and vanilla puts it in the hand.
     */
    public static ItemStack succeed(ServerPlayer player, Item dormant) {

        ServerLevel level = player.serverLevel();

        player.awardStat(Stats.ITEM_USED.get(dormant));
        BmhAdvancements.award(player, "eternity/moon_ritual");
        level.broadcastEntityEvent(player, (byte) 35);
        level.playSound(null, player.getX(), player.getY(), player.getZ(), SoundEvents.AMETHYST_BLOCK_RESONATE, SoundSource.PLAYERS,
                1.0F, 1.0F);
        player.displayClientMessage(Component.translatable("bmh.ritual.woken"), true);

        LOGGER.info("[BalancedMcHardcoreMain] Moon ritual done: the totem of " + player.getName().getString()
                + " is awake, aimed " + String.format("%.1f", aimError(player)) + " degrees from the moon, "
                + player.experienceLevel + " levels left");

        return new ItemStack(ItemInit.TOTEM_OF_ETERNITY.get());
    }

    /**
     * About where the totem is in the ritual pose seen from outside (UseAnim.BOW): both arms raised along the look, the
     * totem at the end of the arm that holds it. The shoulder is 0.24 blocks under the eyes and 0.31 blocks to the side.
     */
    public static Vec3 raisedTotemPosition(Player player) {

        Vec3 look = player.getViewVector(1.0F);
        float yaw = player.getYRot() * ((float) Math.PI / 180.0F);
        Vec3 right = new Vec3(-Math.cos(yaw), 0.0, -Math.sin(yaw));
        boolean rightArm = (player.getUsedItemHand() == InteractionHand.MAIN_HAND) == (player.getMainArm() == HumanoidArm.RIGHT);
        Vec3 shoulder = player.getEyePosition().add(0.0, -0.24, 0.0).add(right.scale(rightArm ? 0.31 : -0.31));

        return shoulder.add(look.scale(0.85));
    }
}
