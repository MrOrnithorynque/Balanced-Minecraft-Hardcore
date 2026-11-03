package github.mrornithorynque.bmh.init;

import java.util.function.Supplier;

import github.mrornithorynque.bmh.items.*;
import github.mrornithorynque.bmh.platform.Services;
import net.minecraft.core.registries.Registries;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.Rarity;

public class ItemInit {

        public static final Supplier<Item> ETERNAL_BUNDLE = register("eternal_bundle",
                        () -> new EternalBundleItem(
                                        new Item.Properties()
                                                        .stacksTo(1)
                                                        .fireResistant()
                                                        .rarity(Rarity.EPIC)));

        public static final Supplier<Item> ETERNAL_AXE = register("eternal_axe",
                        () -> new EternalAxeItem(
                                        BMHModTierInit.ETERNAL,
                                        17.0f,
                                        -3.2f,
                                        new Item.Properties()
                                                        .stacksTo(1)
                                                        .fireResistant()
                                                        .rarity(Rarity.EPIC)));

        public static final Supplier<Item> ETERNAL_PICKAXE = register("eternal_pickaxe",
                        () -> new EternalPickaxeItem(
                                        BMHModTierInit.ETERNAL,
                                        7,
                                        -2f,
                                        new Item.Properties()
                                                        .stacksTo(1)
                                                        .fireResistant()
                                                        .rarity(Rarity.EPIC)));

        public static final Supplier<Item> ETERNAL_SWORD = register("eternal_sword",
                        () -> new EternalSwordItem(
                                        BMHModTierInit.ETERNAL,
                                        10,
                                        -1.4f,
                                        new Item.Properties()
                                                        .stacksTo(1)
                                                        .fireResistant()
                                                        .rarity(Rarity.EPIC)));

        public static final Supplier<Item> NAVIGATOR_AMULET = register("navigator_amulet",
                        () -> new NavigatorAmuletItem(
                                        new Item.Properties()
                                                        .stacksTo(1)
                                                        .rarity(Rarity.COMMON)));

        public static final Supplier<Item> TOTEM_OF_ETERNITY = register("totem_of_eternity",
                        () -> new TotemOfEternityItem(
                                        new Item.Properties()
                                                        .stacksTo(1)
                                                        .fireResistant()
                                                        .rarity(Rarity.EPIC)));

        private static Supplier<Item> register(String name, Supplier<Item> factory) {

                return Services.PLATFORM.register(Registries.ITEM, name, factory);
        }

        public static void init() {
                /* This method is empty but ensures static fields are initialized. */ }
}
