package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Invoker;

import net.minecraft.world.entity.Display;
import net.minecraft.world.level.block.state.BlockState;

/** BlockDisplay.setBlockState is private on every Minecraft version. */
@Mixin(Display.BlockDisplay.class)
public interface E_BlockDisplayInvoker {

    @Invoker("setBlockState")
    void bmh$setBlockState(BlockState state);
}
