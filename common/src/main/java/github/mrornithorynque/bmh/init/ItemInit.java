package github.mrornithorynque.bmh.init;

import java.util.function.Supplier;

import github.mrornithorynque.bmh.items.*;
import github.mrornithorynque.bmh.platform.Services;
import net.minecraft.core.component.DataComponents;
import net.minecraft.core.registries.Registries;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.Rarity;
import net.minecraft.world.item.component.BundleContents;

public class ItemInit {

        public static final Supplier<Item> ETERNAL_BUNDLE = register("eternal_bundle",
                        () -> new EternalBundleItem(
                                        new Item.Properties()
                                                        .stacksTo(1)
                                                        .component(DataComponents.BUNDLE_CONTENTS, BundleContents.EMPTY)
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

        // The Totem of Eternity after it saved a player. It is an Eternal Item: it stays with the player
        // through death. Only the moon ritual wakes it again (handlers/MoonRitualHandler).
        // The id is sleeping_totem: an id that contains dormant_totem or totem_of_eternity makes the
        // inventory checks of the test bots match the wrong item.
        public static final Supplier<Item> DORMANT_TOTEM_OF_ETERNITY = register("sleeping_totem",
                        () -> new DormantTotemOfEternityItem(
                                        new Item.Properties()
                                                        .stacksTo(1)
                                                        .fireResistant()
                                                        .rarity(Rarity.EPIC)));

        // The two halves of the Totem of Eternity. Not Eternal Items: they drop on death like any item.
        // The Hollow Totem keeps the id dormant_totem: worlds from before 1.9.0 hold it under that id.
        public static final Supplier<Item> HOLLOW_TOTEM = register("dormant_totem",
                        () -> new HintItem(
                                        new Item.Properties()
                                                        .stacksTo(1)
                                                        .fireResistant()
                                                        .rarity(Rarity.RARE),
                                        "tooltip.bmh.dormant_totem"));

        public static final Supplier<Item> ECHO_HEART = register("echo_heart",
                        () -> new HintItem(
                                        new Item.Properties()
                                                        .stacksTo(1)
                                                        .fireResistant()
                                                        .rarity(Rarity.RARE),
                                        "tooltip.bmh.echo_heart"));

        private static Supplier<Item> register(String name, Supplier<Item> factory) {

                return Services.PLATFORM.register(Registries.ITEM, name, factory);
        }

        public static void init() {
                /* This method is empty but ensures static fields are initialized. */ }
}
