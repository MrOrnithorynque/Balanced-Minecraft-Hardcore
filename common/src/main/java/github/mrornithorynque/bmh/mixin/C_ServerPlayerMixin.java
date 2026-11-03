package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import github.mrornithorynque.bmh.handlers.AmuletHome;

import net.minecraft.core.BlockPos;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.server.level.ServerPlayer;

/**
 * Gives every ServerPlayer the Navigator's Amulet Home (see AmuletHome): saved with the player,
 * and carried over to the new player on death and when coming back from the End.
 */
@Mixin(ServerPlayer.class)
public abstract class C_ServerPlayerMixin implements AmuletHome.Holder {

    @Unique
    private BlockPos bmh$home;

    @Override
    public BlockPos bmh$getHome() {

        return this.bmh$home;
    }

    @Override
    public void bmh$setHome(BlockPos home) {

        this.bmh$home = home;
    }

    @Inject(method = "addAdditionalSaveData", at = @At("TAIL"))
    private void bmh$saveHome(CompoundTag tag, CallbackInfo ci) {

        AmuletHome.write(this.bmh$home, tag);
    }

    @Inject(method = "readAdditionalSaveData", at = @At("TAIL"))
    private void bmh$loadHome(CompoundTag tag, CallbackInfo ci) {

        this.bmh$home = AmuletHome.read(tag);
    }

    @Inject(method = "restoreFrom", at = @At("TAIL"))
    private void bmh$copyHome(ServerPlayer oldPlayer, boolean alive, CallbackInfo ci) {

        this.bmh$home = ((AmuletHome.Holder) oldPlayer).bmh$getHome();
    }
}
