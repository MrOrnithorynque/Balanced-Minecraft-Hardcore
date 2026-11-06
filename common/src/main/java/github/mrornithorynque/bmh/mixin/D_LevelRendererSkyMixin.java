package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import com.mojang.blaze3d.vertex.PoseStack;

import github.mrornithorynque.bmh.client.sky.StarReadingRenderer;
import net.minecraft.client.renderer.SkyRenderer;

/**
 * Draws the Navigator's Amulet star reading right after vanilla draws the stars: same pose, same blend, and only when
 * the stars show at all. Since 26.1 the sky lives in SkyRenderer, not in LevelRenderer. The class keeps its old name
 * because the mixin config lists it by name.
 */
@Mixin(SkyRenderer.class)
public abstract class D_LevelRendererSkyMixin {

    // SkyRenderer.renderStars (26.1.2, every loader). renderSunMoonAndStars calls it only when the star brightness is
    // above 0, with the pose after YP(-90) and XP(star angle). At its end the pose is still on poseStack: the reading
    // uses the stars' exact pose and does not rebuild it.
    @Inject(method = "renderStars(FLcom/mojang/blaze3d/vertex/PoseStack;)V", at = @At("TAIL"))
    private void bmh$drawStarReading(float starBrightness, PoseStack poseStack, CallbackInfo ci) {

        StarReadingRenderer.render(poseStack, starBrightness);
    }
}
