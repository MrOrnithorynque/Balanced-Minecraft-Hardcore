package github.mrornithorynque.bmh.init;

import github.mrornithorynque.bmh.BalancedMcHardcoreMain;

import java.util.function.Supplier;
import java.util.List;
import java.util.ArrayList;

import net.minecraftforge.eventbus.api.SubscribeEvent;
import net.minecraftforge.event.BuildCreativeModeTabContentsEvent;

import net.minecraftforge.registries.DeferredRegister;
import net.minecraftforge.registries.RegistryObject;

import net.minecraftforge.api.distmarker.Dist;
import net.minecraftforge.fml.common.Mod;

import net.minecraft.core.registries.Registries;
import net.minecraft.network.chat.Component;

import net.minecraft.world.item.CreativeModeTab;
import net.minecraft.world.item.CreativeModeTabs;
import net.minecraft.world.level.ItemLike;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.Items;

@Mod.EventBusSubscriber(modid = BalancedMcHardcoreMain.MODID, bus = Mod.EventBusSubscriber.Bus.MOD, value = Dist.CLIENT)
public class CreativeTabInit {

    public static final DeferredRegister<CreativeModeTab> CREATIVE_MODE_TABS = DeferredRegister
            .create(Registries.CREATIVE_MODE_TAB, BalancedMcHardcoreMain.MODID);

    public static final List<Supplier<? extends ItemLike>> CREATIVE_MODE_TAB_ITEMS = new ArrayList<>();

    public static final RegistryObject<CreativeModeTab> BMH_CREATIVE_TAB = CREATIVE_MODE_TABS
            .register("bmh_creative_tab", () -> CreativeModeTab.builder()
                    .title(Component.translatable("itemGroup.BMH_tab"))
                    .icon(ItemInit.TOTEM_OF_ETERNITY.get()::getDefaultInstance)
                    .displayItems((displayParams, output) -> CREATIVE_MODE_TAB_ITEMS
                            .forEach(itemLike -> output.accept(itemLike.get())))
                    .build());

    public static <T extends Item> RegistryObject<T> addToTab(RegistryObject<T> itemLike) {

        CREATIVE_MODE_TAB_ITEMS.add(itemLike);
        return itemLike;
    }

    @SubscribeEvent
    public static void buildContents(BuildCreativeModeTabContentsEvent event) {

        if (event.getTabKey() == CreativeModeTabs.BUILDING_BLOCKS) {
        }

        if (event.getTab() == BMH_CREATIVE_TAB.get()) {

            event.accept(ItemInit.ETERNAL_AXE);
            event.accept(ItemInit.ETERNAL_BUNDLE);
            event.accept(ItemInit.ETERNAL_PICKAXE);
            event.accept(ItemInit.ETERNAL_SWORD);
            event.accept(ItemInit.TOTEM_OF_ETERNITY);
            event.accept(Items.BUNDLE::asItem);
        }
    }
}
