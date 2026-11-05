package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import github.mrornithorynque.bmh.handlers.TotemOfEternityHandler;

import net.minecraft.server.level.ServerPlayer;
import net.minecraft.tags.DamageTypeTags;
import net.minecraft.world.damagesource.DamageSource;
import net.minecraft.world.effect.MobEffects;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.entity.player.Player;

/**
 * Totem of Undying nerf: a totem still saves a player with Regeneration and Absorption, but no longer
 * grants its 40 seconds of Fire Resistance. Mobs keep vanilla's totem.
 *
 * Vanilla clears the effects and adds the three of the totem after it has consumed the item, so the end of
 * the method is the one place that sees the result. Forge and NeoForge fire their totem event before that,
 * and Fabric has none: one mixin covers the three loaders.
 */
@Mixin(LivingEntity.class)
public abstract class A_LivingEntityTotemMixin {

    @Inject(method = "checkTotemDeathProtection", at = @At("RETURN"))
    private void bmh$dropTotemFireResistance(DamageSource source, CallbackInfoReturnable<Boolean> cir) {

        // The totem clears the player's effects before it adds its three (NeoForge clears the ones a totem cures,
        // Fire Resistance included), so any Fire Resistance left at this point is the totem's.
        if (cir.getReturnValueZ() && (Object) this instanceof Player player) {
            player.removeEffect(MobEffects.FIRE_RESISTANCE);
        }
    }

    // When no Totem of Undying in a hand saved the player, a Totem of Eternity anywhere in the inventory can. It runs
    // after vanilla's check, so a cheap totem in a hand is used first. Damage that bypasses invulnerability (the void,
    // /kill) still kills, as with vanilla's totem. The Fire Resistance of this save stays: the injection above runs
    // first, and it sees false, so it removes nothing.
    @Inject(method = "checkTotemDeathProtection", at = @At("RETURN"), cancellable = true)
    private void bmh$totemOfEternitySave(DamageSource source, CallbackInfoReturnable<Boolean> cir) {

        if (!cir.getReturnValueZ() && (Object) this instanceof ServerPlayer player
                && !source.is(DamageTypeTags.BYPASSES_INVULNERABILITY) && TotemOfEternityHandler.trySave(player)) {
            cir.setReturnValue(true);
        }
    }
}
