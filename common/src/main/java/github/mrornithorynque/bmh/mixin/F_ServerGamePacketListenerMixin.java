package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import github.mrornithorynque.bmh.handlers.RespawnSearch;

import net.minecraft.network.DisconnectionDetails;
import net.minecraft.network.protocol.game.ServerboundClientCommandPacket;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.server.network.ServerGamePacketListenerImpl;

/**
 * Lets the server wait for the landing chunk of a distant respawn without blocking its thread (RespawnSearch). The
 * connection ticks while the player is dead, so it drives the search and answers the held respawn request.
 */
@Mixin(ServerGamePacketListenerImpl.class)
public abstract class F_ServerGamePacketListenerMixin {

    // Public in vanilla: the connection's current player. The respawn replaces it.
    @Shadow
    public ServerPlayer player;

    @Inject(method = "handleClientCommand", at = @At("HEAD"), cancellable = true)
    private void bmh$holdRespawn(ServerboundClientCommandPacket packet, CallbackInfo ci) {

        // The network thread passes the packet on to the server thread, which calls this method again.
        if (packet.getAction() == ServerboundClientCommandPacket.Action.PERFORM_RESPAWN
                && this.player.server.isSameThread() && RespawnSearch.holdRespawn(this.player)) {
            ci.cancel();
        }
    }

    @Inject(method = "tick", at = @At("RETURN"))
    private void bmh$tickSearch(CallbackInfo ci) {

        RespawnSearch.tick(this.player);
    }

    @Inject(method = "onDisconnect", at = @At("HEAD"))
    private void bmh$dropSearch(DisconnectionDetails details, CallbackInfo ci) {

        RespawnSearch.cancel(this.player);
    }
}
