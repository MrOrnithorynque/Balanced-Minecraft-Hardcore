package github.mrornithorynque.bmh.init;

import github.mrornithorynque.bmh.BalancedMcHardcoreMain;
import github.mrornithorynque.bmh.items.*;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.Rarity;

import net.minecraftforge.registries.DeferredRegister;
import net.minecraftforge.registries.ForgeRegistries;
import net.minecraftforge.registries.RegistryObject;

import net.minecraftforge.fml.common.Mod;

@Mod.EventBusSubscriber(modid = BalancedMcHardcoreMain.MODID, bus = Mod.EventBusSubscriber.Bus.MOD)
public class ItemInit {

        public static final DeferredRegister<Item> ITEMS = DeferredRegister.create(ForgeRegistries.ITEMS,
                        BalancedMcHardcoreMain.MODID);

        public static final RegistryObject<Item> ETERNAL_BUNDLE = ITEMS.register("eternal_bundle",
                        () -> new EternalBundleItem(
                                        new Item.Properties()
                                                        .stacksTo(1)
                                                        .fireResistant()
                                                        .rarity(Rarity.EPIC)));

        public static final RegistryObject<Item> ETERNAL_AXE = ITEMS.register("eternal_axe",
                        () -> new EternalAxeItem(
                                        BMHModTierInit.ETERNAL,
                                        17.0f,
                                        -3.5f,
                                        new Item.Properties()
                                                        .stacksTo(1)
                                                        .fireResistant()
                                                        .rarity(Rarity.EPIC)));

        public static final RegistryObject<Item> ETERNAL_PICKAXE = ITEMS.register("eternal_pickaxe",
                        () -> new EternalPickaxeItem(
                                        BMHModTierInit.ETERNAL,
                                        7,
                                        -2f,
                                        new Item.Properties()
                                                        .stacksTo(1)
                                                        .fireResistant()
                                                        .rarity(Rarity.EPIC)));

        public static final RegistryObject<Item> ETERNAL_SWORD = ITEMS.register("eternal_sword",
                        () -> new EternalSwordItem(
                                        BMHModTierInit.ETERNAL,
                                        10,
                                        -1f,
                                        new Item.Properties()
                                                        .stacksTo(1)
                                                        .fireResistant()
                                                        .rarity(Rarity.EPIC)));

        public static final RegistryObject<Item> NAVIGATOR_AMULET = ITEMS.register("navigator_amulet",
                        () -> new NavigatorAmuletItem(
                                        new Item.Properties()
                                                        .stacksTo(1)
                                                        .durability(10)
                                                        .rarity(Rarity.COMMON)));

        public static final RegistryObject<Item> TOTEM_OF_ETERNITY = ITEMS.register("totem_of_eternity",
                        () -> new TotemOfEternityItem(
                                        new Item.Properties()
                                                        .stacksTo(1)
                                                        .fireResistant()
                                                        .rarity(Rarity.EPIC)));
}
