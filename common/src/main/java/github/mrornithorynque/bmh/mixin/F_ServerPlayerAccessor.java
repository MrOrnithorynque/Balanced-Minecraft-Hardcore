package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

import net.minecraft.server.level.ServerPlayer;

/** ServerPlayer.spawnInvulnerableTime is private on every Minecraft version. Vanilla starts it at 60 ticks. */
@Mixin(ServerPlayer.class)
public interface F_ServerPlayerAccessor {

    @Accessor("spawnInvulnerableTime")
    void bmh$setSpawnInvulnerableTime(int ticks);
}
