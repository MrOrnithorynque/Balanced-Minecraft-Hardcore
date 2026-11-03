package github.mrornithorynque.bmh.mixin;

import org.joml.Matrix4f;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Slice;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import com.mojang.blaze3d.vertex.PoseStack;
import com.mojang.math.Axis;

import github.mrornithorynque.bmh.client.sky.StarReadingRenderer;
import net.minecraft.client.Camera;
import net.minecraft.client.multiplayer.ClientLevel;
import net.minecraft.client.renderer.LevelRenderer;

/**
 * Draws the Navigator's Amulet star reading right after vanilla draws the stars: same pose, same blend, and only when
 * the stars show at all.
 */
@Mixin(LevelRenderer.class)
public abstract class D_LevelRendererSkyMixin {

    @Shadow
    private ClientLevel level;

    // The star section of renderSky (1.21.1, every loader): starBuffer.bind(), starBuffer.drawWithShader(...),
    // VertexBuffer.unbind(), then the fog setup. We go right after that unbind().
    @Inject(method = "renderSky(Lorg/joml/Matrix4f;Lorg/joml/Matrix4f;FLnet/minecraft/client/Camera;ZLjava/lang/Runnable;)V",
            slice = @Slice(from = @At(value = "FIELD", target = "Lnet/minecraft/client/renderer/LevelRenderer;starBuffer:Lcom/mojang/blaze3d/vertex/VertexBuffer;")),
            at = @At(value = "INVOKE", target = "Lcom/mojang/blaze3d/vertex/VertexBuffer;unbind()V", ordinal = 0, shift = At.Shift.AFTER))
    private void bmh$drawStarReading(Matrix4f frustumMatrix, Matrix4f projectionMatrix, float partialTick, Camera camera,
            boolean isFoggy, Runnable skyFogSetup, CallbackInfo ci) {

        // Since 1.20.5 the stars' pose lives in a local PoseStack, out of a mixin's reach: rebuilt the way vanilla
        // builds it, from the same matrix and the same two rotations.
        PoseStack poseStack = new PoseStack();
        poseStack.mulPose(frustumMatrix);
        poseStack.mulPose(Axis.YP.rotationDegrees(-90.0F));
        poseStack.mulPose(Axis.XP.rotationDegrees(this.level.getTimeOfDay(partialTick) * 360.0F));
        StarReadingRenderer.render(this.level, poseStack.last().pose(), partialTick);
    }
}
