package github.mrornithorynque.bmh.mixin;

import org.joml.Matrix4f;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Slice;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import com.mojang.blaze3d.vertex.PoseStack;

import github.mrornithorynque.bmh.client.sky.StarReadingRenderer;
import net.minecraft.client.Camera;
import net.minecraft.client.multiplayer.ClientLevel;
import net.minecraft.client.renderer.LevelRenderer;

/**
 * Draws the Navigator's Amulet star reading right after vanilla draws the stars: same pose (the stars' rotation is
 * still on the stack), same blend, and only when the stars show at all.
 */
@Mixin(LevelRenderer.class)
public abstract class D_LevelRendererSkyMixin {

    @Shadow
    private ClientLevel level;

    // The star section of renderSky (identical from 1.20 to 1.20.4, on every loader): starBuffer.bind(),
    // starBuffer.drawWithShader(...), VertexBuffer.unbind(), then the fog setup. We go right after that unbind().
    @Inject(method = "renderSky(Lcom/mojang/blaze3d/vertex/PoseStack;Lorg/joml/Matrix4f;FLnet/minecraft/client/Camera;ZLjava/lang/Runnable;)V",
            slice = @Slice(from = @At(value = "FIELD", target = "Lnet/minecraft/client/renderer/LevelRenderer;starBuffer:Lcom/mojang/blaze3d/vertex/VertexBuffer;")),
            at = @At(value = "INVOKE", target = "Lcom/mojang/blaze3d/vertex/VertexBuffer;unbind()V", ordinal = 0, shift = At.Shift.AFTER))
    private void bmh$drawStarReading(PoseStack poseStack, Matrix4f projectionMatrix, float partialTick, Camera camera,
            boolean isFoggy, Runnable skyFogSetup, CallbackInfo ci) {

        StarReadingRenderer.render(this.level, poseStack.last().pose(), partialTick);
    }
}
