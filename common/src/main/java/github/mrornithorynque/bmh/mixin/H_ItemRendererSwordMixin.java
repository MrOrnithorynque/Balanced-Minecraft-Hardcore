package github.mrornithorynque.bmh.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyVariable;

import github.mrornithorynque.bmh.client.EternalSwordModel;

import net.minecraft.client.renderer.entity.ItemRenderer;
import net.minecraft.client.resources.model.BakedModel;
import net.minecraft.world.item.ItemDisplayContext;
import net.minecraft.world.item.ItemStack;

/**
 * Draws the Eternal Sword with its 20x20 model in the hands (client/EternalSwordModel). Only render() knows where the
 * item is drawn, and vanilla changes the trident's model at the same place. Forge and NeoForge change the model only
 * after that point, so one mixin covers the three loaders.
 */
@Mixin(ItemRenderer.class)
public abstract class H_ItemRendererSwordMixin {

    @ModifyVariable(method = "render", at = @At("HEAD"), argsOnly = true)
    private BakedModel bmh$eternalSwordInHand(BakedModel model, ItemStack stack, ItemDisplayContext context) {

        return EternalSwordModel.inHandModel(model, stack, context);
    }
}
