package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.ModifyArg;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import github.mrornithorynque.bmh.handlers.RespawnPositionHandler;

import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.server.players.PlayerList;
import net.minecraft.world.entity.Entity;

/**
 * Puts the distant respawn position into vanilla's respawn itself. The respawn event fires after vanilla sends the
 * spawn point to the client, so a teleport from there showed the spawn point first.
 */
@Mixin(PlayerList.class)
public abstract class F_PlayerListRespawnMixin {

    // Set at the start of a respawn, cleared once the new player has its position.
    @Unique
    private boolean bmh$positionPending;
    @Unique
    private boolean bmh$endConquered;

    @Inject(method = "respawn", at = @At("HEAD"))
    private void bmh$startRespawn(ServerPlayer serverPlayer, boolean keepEverything,
            CallbackInfoReturnable<ServerPlayer> cir) {

        this.bmh$positionPending = true;
        // keepEverything is true only when the player comes back from the End.
        this.bmh$endConquered = keepEverything;
    }

    // respawn calls noCollision once per turn of the loop that lifts the new player out of blocks, on every 1.20.x
    // version and loader. At the first call, vanilla has placed the new player (bed, anchor or world spawn) and has
    // sent no position yet.
    @ModifyArg(method = "respawn", at = @At(value = "INVOKE", target = "Lnet/minecraft/server/level/ServerLevel;noCollision(Lnet/minecraft/world/entity/Entity;)Z"))
    private Entity bmh$distantRespawn(Entity entity) {

        if (this.bmh$positionPending && entity instanceof ServerPlayer newPlayer) {
            this.bmh$positionPending = false;

            BlockPos respawnPosition = RespawnPositionHandler.findRespawnPosition(newPlayer, newPlayer.serverLevel(),
                    false, this.bmh$endConquered);

            if (respawnPosition != null) {
                newPlayer.moveTo(respawnPosition.getX() + 0.5, respawnPosition.getY(), respawnPosition.getZ() + 0.5);
            }
        }

        return entity;
    }
}
