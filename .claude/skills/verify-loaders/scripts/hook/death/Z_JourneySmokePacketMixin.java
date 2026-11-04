package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import github.mrornithorynque.bmh.client.JourneySmoke;
import net.minecraft.client.Minecraft;
import net.minecraft.client.multiplayer.ClientPacketListener;

/**
 * THROWAWAY verification hook (never committed): logs the player position after each position packet and respawn
 * packet. TAIL runs on the render thread only: on the network thread these handlers leave by an exception.
 */
@Mixin(ClientPacketListener.class)
public class Z_JourneySmokePacketMixin {

    @Inject(method = "handleMovePlayer", at = @At("TAIL"))
    private void bmh$journeySmokePosition(CallbackInfo ci) {
        JourneySmoke.packet(Minecraft.getInstance(), "position packet");
    }

    @Inject(method = "handleRespawn", at = @At("TAIL"))
    private void bmh$journeySmokeRespawn(CallbackInfo ci) {
        JourneySmoke.packet(Minecraft.getInstance(), "respawn packet");
    }
}
