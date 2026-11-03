package github.mrornithorynque.bmh.mixin;

import java.util.ArrayList;
import java.util.List;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import github.mrornithorynque.bmh.handlers.EchoStrikeHandler;
import github.mrornithorynque.bmh.handlers.EchoStrikeQueue;

import net.minecraft.server.level.ServerPlayer;

/**
 * Gives every player a queue of waiting Echo Strike echoes and the clock that delivers them. They
 * belong to the player, so nothing outlives the world and no loader tick event is needed.
 */
@Mixin(ServerPlayer.class)
public abstract class E_ServerPlayerMixin implements EchoStrikeQueue {

    @Unique
    private final List<EchoStrikeHandler.PendingEcho> bmh$pendingEchoes = new ArrayList<>();

    @Override
    public List<EchoStrikeHandler.PendingEcho> bmh$pendingEchoes() {

        return bmh$pendingEchoes;
    }

    @Inject(method = "tick", at = @At("HEAD"))
    private void bmh$deliverEchoes(CallbackInfo ci) {

        if (!bmh$pendingEchoes.isEmpty()) {
            EchoStrikeHandler.deliverDueEchoes((ServerPlayer) (Object) this);
        }
    }
}
