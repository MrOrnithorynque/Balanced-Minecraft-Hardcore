package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.ArrayList;
import java.util.List;

import github.mrornithorynque.bmh.handlers.AmuletPoints;
import github.mrornithorynque.bmh.handlers.HomecomingHandler;

import org.jspecify.annotations.Nullable;

import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.level.storage.ValueInput;
import net.minecraft.world.level.storage.ValueOutput;

/**
 * Gives every ServerPlayer the points of the Navigator's Amulet (see AmuletPoints) and the record of an exile that
 * has not ended (see HomecomingHandler): saved with the player,
 * and carried over to the new player on death and when coming back from the End.
 */
@Mixin(ServerPlayer.class)
public abstract class C_ServerPlayerMixin implements AmuletPoints.Holder, HomecomingHandler.Holder {

    @Unique
    private List<AmuletPoints.Point> bmh$points = new ArrayList<>();

    @Unique
    private HomecomingHandler.@Nullable Record bmh$homecoming;

    @Override
    public HomecomingHandler.@Nullable Record bmh$getHomecoming() {

        return this.bmh$homecoming;
    }

    @Override
    public void bmh$setHomecoming(HomecomingHandler.@Nullable Record record) {

        this.bmh$homecoming = record;
    }

    @Override
    public List<AmuletPoints.Point> bmh$getPoints() {

        return this.bmh$points;
    }

    @Override
    public void bmh$setPoints(List<AmuletPoints.Point> points) {

        this.bmh$points = new ArrayList<>(points);
    }

    @Inject(method = "addAdditionalSaveData", at = @At("TAIL"))
    private void bmh$savePoints(ValueOutput output, CallbackInfo ci) {

        AmuletPoints.write(this.bmh$points, output);
        HomecomingHandler.write(this.bmh$homecoming, output);
    }

    @Inject(method = "readAdditionalSaveData", at = @At("TAIL"))
    private void bmh$loadPoints(ValueInput input, CallbackInfo ci) {

        this.bmh$points = AmuletPoints.read(input);
        this.bmh$homecoming = HomecomingHandler.read(input);
    }

    @Inject(method = "restoreFrom", at = @At("TAIL"))
    private void bmh$copyPoints(ServerPlayer oldPlayer, boolean alive, CallbackInfo ci) {

        this.bmh$points = new ArrayList<>(((AmuletPoints.Holder) oldPlayer).bmh$getPoints());
        this.bmh$homecoming = ((HomecomingHandler.Holder) oldPlayer).bmh$getHomecoming();
    }
}
