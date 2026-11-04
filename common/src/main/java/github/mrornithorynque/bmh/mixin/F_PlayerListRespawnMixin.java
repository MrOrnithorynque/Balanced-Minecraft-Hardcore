package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.ModifyVariable;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import github.mrornithorynque.bmh.handlers.RespawnPositionHandler;

import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.server.players.PlayerList;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.level.portal.DimensionTransition;
import net.minecraft.world.phys.Vec3;

/**
 * Puts the distant respawn position into vanilla's respawn itself. The respawn event fires after vanilla sends the
 * spawn point to the client, so a teleport from there showed the spawn point first.
 */
@Mixin(PlayerList.class)
public abstract class F_PlayerListRespawnMixin {

    // Set at the start of a respawn, cleared when the position is chosen.
    @Unique
    private ServerPlayer bmh$deadPlayer;
    @Unique
    private boolean bmh$endConquered;

    @Inject(method = "respawn", at = @At("HEAD"))
    private void bmh$startRespawn(ServerPlayer serverPlayer, boolean keepInventory, Entity.RemovalReason reason,
            CallbackInfoReturnable<ServerPlayer> cir) {

        this.bmh$deadPlayer = serverPlayer;
        // keepInventory is true only when the player comes back from the End.
        this.bmh$endConquered = keepInventory;
    }

    // Right after vanilla's own search: a respawn anchor still loses a charge, and a missing bed still shows its message.
    // This changes the result and does not replace the call, so other mods can change it too. On NeoForge,
    // PlayerRespawnPositionEvent gets this result next.
    @ModifyVariable(method = "respawn", at = @At(value = "INVOKE_ASSIGN", target = "Lnet/minecraft/server/level/ServerPlayer;findRespawnPositionAndUseSpawnBlock(ZLnet/minecraft/world/level/portal/DimensionTransition$PostDimensionTransition;)Lnet/minecraft/world/level/portal/DimensionTransition;"))
    private DimensionTransition bmh$distantRespawn(DimensionTransition transition) {

        ServerPlayer serverPlayer = this.bmh$deadPlayer;
        this.bmh$deadPlayer = null;

        if (serverPlayer == null) {
            return transition;
        }

        BlockPos respawnPosition = RespawnPositionHandler.findRespawnPosition(serverPlayer, transition.newLevel(),
                transition.missingRespawnBlock(), this.bmh$endConquered);

        if (respawnPosition == null) {
            return transition;
        }

        return new DimensionTransition(transition.newLevel(), Vec3.atBottomCenterOf(respawnPosition), transition.speed(),
                transition.yRot(), transition.xRot(), transition.missingRespawnBlock(), transition.postDimensionTransition());
    }
}
