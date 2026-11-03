package github.mrornithorynque.bmh.client;

import github.mrornithorynque.bmh.init.ItemInit;
import github.mrornithorynque.bmh.items.EternalBundleItem;

import net.minecraft.client.renderer.item.ClampedItemPropertyFunction;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.world.item.Item;

/**
 * The item properties our models switch on. Client only: it is loaded from the loaders' client hooks and from
 * nowhere else, a dedicated server must never see it. Every loader hands over its own way to register a property
 * (ItemProperties on Forge and NeoForge, Fabric API's FabricModelPredicateProviderRegistry), so a property is
 * written once for the three of them.
 */
public class ClientItemProperties {

    @FunctionalInterface
    public interface Registrar {

        void register(Item item, ResourceLocation id, ClampedItemPropertyFunction function);
    }

    public static void registerAll(Registrar registrar) {

        // The Eternal Bundle's model switches on "filled", but vanilla registers that property for Items.BUNDLE only.
        registrar.register(ItemInit.ETERNAL_BUNDLE.get(), new ResourceLocation("filled"),
                (stack, level, entity, seed) -> EternalBundleItem.getFullnessDisplay(stack));
    }
}
