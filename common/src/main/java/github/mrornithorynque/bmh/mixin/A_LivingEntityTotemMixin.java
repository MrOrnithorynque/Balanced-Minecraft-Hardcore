package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

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
}
