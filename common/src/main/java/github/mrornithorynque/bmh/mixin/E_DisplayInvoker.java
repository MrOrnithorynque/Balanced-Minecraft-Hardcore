package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Invoker;

import net.minecraft.world.entity.Display;

/** Display.setGlowColorOverride is private on every Minecraft version. */
@Mixin(Display.class)
public interface E_DisplayInvoker {

    @Invoker("setGlowColorOverride")
    void bmh$setGlowColorOverride(int color);
}
