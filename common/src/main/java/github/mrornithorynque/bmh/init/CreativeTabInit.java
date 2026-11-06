package github.mrornithorynque.bmh.init;

import java.util.List;
import java.util.function.Supplier;

import github.mrornithorynque.bmh.platform.Services;

import net.minecraft.core.registries.Registries;
import net.minecraft.network.chat.Component;

import net.minecraft.world.item.CreativeModeTab;
import net.minecraft.world.item.Items;

public class CreativeTabInit {

    public static final Supplier<CreativeModeTab> BMH_CREATIVE_TAB = Services.PLATFORM.register(
            Registries.CREATIVE_MODE_TAB,
            "bmh_creative_tab",
            () -> Services.PLATFORM.creativeTabBuilder(List.of(
                            ItemInit.ETERNAL_AXE,
                            ItemInit.ETERNAL_BUNDLE,
                            ItemInit.ETERNAL_PICKAXE,
                            ItemInit.ETERNAL_SWORD,
                            ItemInit.NAVIGATOR_AMULET,
                            ItemInit.TOTEM_OF_ETERNITY,
                            ItemInit.DORMANT_TOTEM,
                            ItemInit.ECHO_HEART,
                            () -> Items.BUNDLE))
                    .title(Component.translatable("itemGroup.BMH_tab"))
                    .icon(() -> ItemInit.TOTEM_OF_ETERNITY.get().getDefaultInstance())
                    .build());

    public static void init() {
        /* This method is empty but ensures static fields are initialized. */ }
}
