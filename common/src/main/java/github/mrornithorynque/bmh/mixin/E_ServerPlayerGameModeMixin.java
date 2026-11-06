package github.mrornithorynque.bmh.mixin;

import java.util.ArrayDeque;
import java.util.Deque;

import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import github.mrornithorynque.bmh.handlers.EchoSenseHandler;
import github.mrornithorynque.bmh.handlers.EternalWear;
import github.mrornithorynque.bmh.items.EternalPickaxeItem;

import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.server.level.ServerPlayerGameMode;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;

/**
 * Where a player's block really breaks: the head remembers what stood there, and a {@code true}
 * return (Forge's break event and protection mods can still refuse) hands the ore to Echo Sense.
 */
@Mixin(ServerPlayerGameMode.class)
public abstract class E_ServerPlayerGameModeMixin {

    @Shadow
    @Final
    protected ServerPlayer player;

    // One entry per destroyBlock call in progress: a mod that breaks a whole vein from inside a break
    // event calls it again before the first call returns.
    @Unique
    private final Deque<BlockState> bmh$brokenStates = new ArrayDeque<>();

    @Unique
    private long bmh$nextSense;

    @Inject(method = "destroyBlock", at = @At("HEAD"))
    private void bmh$rememberBlock(BlockPos pos, CallbackInfoReturnable<Boolean> cir) {

        // A return that never came (an exception) leaves its entry behind: forget the old ones.
        if (bmh$brokenStates.size() > 8) {
            bmh$brokenStates.clear();
        }

        ItemStack tool = player.getMainHandItem();

        bmh$brokenStates.push(tool.getItem() instanceof EternalPickaxeItem && !EternalWear.isWornOut(tool)
                ? player.level().getBlockState(pos)
                : Blocks.AIR.defaultBlockState());
    }

    @Inject(method = "destroyBlock", at = @At("RETURN"))
    private void bmh$senseOre(BlockPos pos, CallbackInfoReturnable<Boolean> cir) {

        BlockState broken = bmh$brokenStates.poll();

        if (broken == null || !cir.getReturnValueZ() || !broken.is(EchoSenseHandler.ECHO_SENSE_ORES)) {
            return;
        }

        long now = player.level().getGameTime();

        if (now >= bmh$nextSense) {
            bmh$nextSense = now + EchoSenseHandler.COOLDOWN_TICKS;
            EchoSenseHandler.onOreBroken(player, pos, broken);
        }
    }
}
