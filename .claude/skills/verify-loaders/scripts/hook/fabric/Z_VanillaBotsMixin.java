package github.mrornithorynque.bmh.fabric.mixin;

import java.util.Map;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * Throwaway verify hook, Fabric on 26.x only (lib.sh fabric_bot_hook copies it, fabric_bot_unhook removes it). The
 * bots are vanilla clients. Fabric API 0.116 (1.21.1) let a vanilla client in without a registry sync; the 26.1
 * Fabric API disconnects it unless every modded registry is optional. This makes Fabric API answer "all optional",
 * so the server lets the bot in without a sync, as on 1.21.1. It never ships: jarcheck.py runs before the hook.
 */
@Mixin(targets = "net.fabricmc.fabric.impl.registry.sync.RegistrySyncManager", remap = false)
public abstract class Z_VanillaBotsMixin {

    @Inject(method = "areAllRegistriesOptional", at = @At("HEAD"), cancellable = true, remap = false)
    private static void bmh$letVanillaBotsIn(Map<?, ?> registries, CallbackInfoReturnable<Boolean> cir) {

        cir.setReturnValue(true);
    }
}
