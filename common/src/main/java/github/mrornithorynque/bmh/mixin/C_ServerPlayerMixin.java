package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.ArrayList;
import java.util.List;

import github.mrornithorynque.bmh.handlers.AmuletPoints;

import net.minecraft.nbt.CompoundTag;
import net.minecraft.server.level.ServerPlayer;

/**
 * Gives every ServerPlayer the points of the Navigator's Amulet (see AmuletPoints): saved with the player,
 * and carried over to the new player on death and when coming back from the End.
 */
@Mixin(ServerPlayer.class)
public abstract class C_ServerPlayerMixin implements AmuletPoints.Holder {

    @Unique
    private List<AmuletPoints.Point> bmh$points = new ArrayList<>();

    @Override
    public List<AmuletPoints.Point> bmh$getPoints() {

        return this.bmh$points;
    }

    @Override
    public void bmh$setPoints(List<AmuletPoints.Point> points) {

        this.bmh$points = new ArrayList<>(points);
    }

    @Inject(method = "addAdditionalSaveData", at = @At("TAIL"))
    private void bmh$savePoints(CompoundTag tag, CallbackInfo ci) {

        AmuletPoints.write(this.bmh$points, tag);
    }

    @Inject(method = "readAdditionalSaveData", at = @At("TAIL"))
    private void bmh$loadPoints(CompoundTag tag, CallbackInfo ci) {

        this.bmh$points = AmuletPoints.read(tag);
    }

    @Inject(method = "restoreFrom", at = @At("TAIL"))
    private void bmh$copyPoints(ServerPlayer oldPlayer, boolean alive, CallbackInfo ci) {

        this.bmh$points = new ArrayList<>(((AmuletPoints.Holder) oldPlayer).bmh$getPoints());
    }
}
