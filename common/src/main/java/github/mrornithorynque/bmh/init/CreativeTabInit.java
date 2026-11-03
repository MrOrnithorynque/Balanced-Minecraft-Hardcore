package github.mrornithorynque.bmh.init;

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
            () -> Services.PLATFORM.creativeTabBuilder()
                    .title(Component.translatable("itemGroup.BMH_tab"))
                    .icon(() -> ItemInit.TOTEM_OF_ETERNITY.get().getDefaultInstance())
                    .displayItems((displayParams, output) -> {
                        output.accept(ItemInit.ETERNAL_AXE.get());
                        output.accept(ItemInit.ETERNAL_BUNDLE.get());
                        output.accept(ItemInit.ETERNAL_PICKAXE.get());
                        output.accept(ItemInit.ETERNAL_SWORD.get());
                        output.accept(ItemInit.NAVIGATOR_AMULET.get());
                        output.accept(ItemInit.TOTEM_OF_ETERNITY.get());
                        output.accept(ItemInit.DORMANT_TOTEM.get());
                        output.accept(ItemInit.ECHO_HEART.get());
                        output.accept(Items.BUNDLE);
                    })
                    .build());

    public static void init() {
        /* This method is empty but ensures static fields are initialized. */ }
}
