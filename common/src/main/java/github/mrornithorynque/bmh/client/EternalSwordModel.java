package github.mrornithorynque.bmh.client;

import github.mrornithorynque.bmh.init.ItemInit;

import net.minecraft.client.renderer.item.ClampedItemPropertyFunction;
import net.minecraft.client.resources.model.BakedModel;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.world.item.ItemDisplayContext;
import net.minecraft.world.item.ItemStack;

/**
 * The Eternal Sword has two sprites: a 16x16 one in the inventory, on the ground and in item frames, and a bigger
 * 20x20 one in the hands. Vanilla does the same for the trident. The item model is the 16x16 one, and its override
 * on "bmh:in_hand" gives the 20x20 model. mixin/H_ItemRendererSwordMixin asks for that override when it draws the
 * sword in a hand. Client class: ClientItemProperties registers the property, a dedicated server never loads it.
 */
public class EternalSwordModel {

    public static final ResourceLocation IN_HAND = ResourceLocation.fromNamespaceAndPath("bmh", "in_hand");

    // Only the render thread reads and writes it, and only inside inHandModel.
    private static boolean resolvingInHand = false;

    public static ClampedItemPropertyFunction inHand() {

        return (stack, level, entity, seed) -> resolvingInHand ? 1.0F : 0.0F;
    }

    public static BakedModel inHandModel(BakedModel model, ItemStack stack, ItemDisplayContext context) {

        if (!isHand(context) || !stack.is(ItemInit.ETERNAL_SWORD.get())) {
            return model;
        }

        resolvingInHand = true;
        BakedModel inHand = model.getOverrides().resolve(model, stack, null, null, 0);
        resolvingInHand = false;

        return inHand != null ? inHand : model;
    }

    private static boolean isHand(ItemDisplayContext context) {

        return context == ItemDisplayContext.FIRST_PERSON_RIGHT_HAND || context == ItemDisplayContext.FIRST_PERSON_LEFT_HAND
                || context == ItemDisplayContext.THIRD_PERSON_RIGHT_HAND || context == ItemDisplayContext.THIRD_PERSON_LEFT_HAND;
    }
}
