package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Invoker;

import com.mojang.math.Transformation;

import net.minecraft.world.entity.Display;

/** Display.setGlowColorOverride and Display.setTransformation are private on every Minecraft version. */
@Mixin(Display.class)
public interface E_DisplayInvoker {

    @Invoker("setGlowColorOverride")
    void bmh$setGlowColorOverride(int color);

    @Invoker("setTransformation")
    void bmh$setTransformation(Transformation transformation);
}
