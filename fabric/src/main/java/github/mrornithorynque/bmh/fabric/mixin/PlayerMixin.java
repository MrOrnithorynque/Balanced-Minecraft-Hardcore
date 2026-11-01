package github.mrornithorynque.bmh.fabric.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import github.mrornithorynque.bmh.handlers.SleepHandler;

import net.minecraft.world.entity.player.Player;

/** Where Forge fires PlayerWakeUpEvent: the head of Player.stopSleepInBed, every way out of bed included. */
@Mixin(Player.class)
public abstract class PlayerMixin {

    @Inject(method = "stopSleepInBed", at = @At("HEAD"))
    private void bmh$onWakeUp(boolean wakeImmediately, boolean updateLevelForSleepingPlayers, CallbackInfo ci) {

        SleepHandler.onPlayerWakeUp((Player) (Object) this);
    }
}
