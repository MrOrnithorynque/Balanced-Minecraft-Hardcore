package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.ModifyArg;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import github.mrornithorynque.bmh.handlers.AxeSmashHandler;
import github.mrornithorynque.bmh.handlers.EchoStrikeHandler;
import github.mrornithorynque.bmh.items.EternalAxeItem;
import github.mrornithorynque.bmh.items.EternalSwordItem;

import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.item.Item;

/**
 * The three moments of a player's melee hit that the Eternal Sword's echo and the Eternal Axe's smash
 * need, the same on Forge, Fabric and NeoForge:
 * <ul>
 * <li>the head: vanilla resets the attack strength early and Forge late, so this is the one place
 * that reads the same value everywhere, and the main hand is the weapon of the whole swing;</li>
 * <li>the one {@code Entity.hurt} call on the target (the sweep uses {@code LivingEntity.hurt}): the
 * damage the hit is about to deal, scaled here for the axe;</li>
 * <li>the {@code causeFoodExhaustion} call, which only the branch of a landed hit reaches.</li>
 * </ul>
 */
@Mixin(Player.class)
public abstract class E_PlayerAttackMixin {

    // Set at the head of every attack. A null weapon means no ability: a client, or nothing to do.
    @Unique
    private Item bmh$weapon;

    @Unique
    private float bmh$attackStrength;

    @Unique
    private float bmh$hitDamage;

    @Unique
    private float bmh$smashMultiplier = 1.0F;

    @Inject(method = "attack", at = @At("HEAD"))
    private void bmh$beginAttack(Entity target, CallbackInfo ci) {

        Player self = (Player) (Object) this;

        bmh$weapon = null;
        bmh$hitDamage = 0.0F;
        bmh$smashMultiplier = 1.0F;

        // The client also runs Player.attack, for its own prediction: abilities are the server's.
        if (self instanceof ServerPlayer) {
            bmh$weapon = self.getMainHandItem().getItem();
            bmh$attackStrength = self.getAttackStrengthScale(0.5F);
        }
    }

    @ModifyArg(method = "attack", at = @At(value = "INVOKE", target = "Lnet/minecraft/world/entity/Entity;hurt(Lnet/minecraft/world/damagesource/DamageSource;F)Z"), index = 1)
    private float bmh$scaleHit(float damage) {

        if (bmh$weapon instanceof EternalAxeItem) {
            bmh$smashMultiplier = AxeSmashHandler.multiplier((Player) (Object) this);
            damage *= bmh$smashMultiplier;
        }

        bmh$hitDamage = damage;

        return damage;
    }

    @Inject(method = "attack", at = @At(value = "INVOKE", target = "Lnet/minecraft/world/entity/player/Player;causeFoodExhaustion(F)V"))
    private void bmh$hitLanded(Entity target, CallbackInfo ci) {

        if (bmh$weapon == null || !((Object) this instanceof ServerPlayer attacker)) {
            return;
        }

        if (bmh$weapon instanceof EternalSwordItem) {
            EchoStrikeHandler.onSwordHit(attacker, target, bmh$hitDamage, bmh$attackStrength);
        } else if (bmh$smashMultiplier > 1.0F) {
            AxeSmashHandler.onSmash(attacker, target, bmh$smashMultiplier);
        }
    }
}
