package github.mrornithorynque.bmh.init;

import github.mrornithorynque.bmh.BalancedMcHardcoreMain;

import java.util.function.Supplier;
import java.util.List;
import java.util.ArrayList;

import net.minecraftforge.registries.DeferredRegister;
import net.minecraftforge.registries.RegistryObject;
import net.minecraft.core.registries.Registries;
import net.minecraft.network.chat.Component;
import net.minecraft.world.item.CreativeModeTab;
import net.minecraft.world.level.ItemLike;

public class CreativeTabInit {

    public static final DeferredRegister<CreativeModeTab> CREATIVE_MODE_TABS =
        DeferredRegister.create(Registries.CREATIVE_MODE_TAB, BalancedMcHardcoreMain.MODID);

    public static final List<Supplier<? extends ItemLike>> CREATIVE_MODE_TAB_ITEMS = new ArrayList<>();

    public static final RegistryObject<CreativeModeTab> BMH_CREATIVE_TAB =
        CREATIVE_MODE_TABS.register("bmh_creative_tab", () -> CreativeModeTab.builder()
            .title(Component.translatable("itemGroup.BMH_tab"))
            .icon(ItemInit.TEST_ITEM.get()::getDefaultInstance)
            .displayItems((displayParams, output) ->
                CREATIVE_MODE_TAB_ITEMS.forEach(itemLike -> output.accept(itemLike.get())))
            .build()
        );
}
