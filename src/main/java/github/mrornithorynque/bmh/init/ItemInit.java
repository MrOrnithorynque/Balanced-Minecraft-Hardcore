package github.mrornithorynque.bmh.init;

import github.mrornithorynque.bmh.BalancedMcHardcoreMain;
import github.mrornithorynque.bmh.item.EternalAxeItem;
import github.mrornithorynque.bmh.item.LifeTotemItem;
import net.minecraft.world.effect.MobEffectInstance;
import net.minecraft.world.effect.MobEffects;
import net.minecraft.world.food.FoodProperties;
import net.minecraft.world.item.BlockItem;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.Rarity;

import net.minecraftforge.registries.DeferredRegister;
import net.minecraftforge.registries.ForgeRegistries;
import net.minecraftforge.registries.RegistryObject;

import net.minecraftforge.fml.common.Mod;
import net.minecraft.world.item.Tiers;

@Mod.EventBusSubscriber(modid = BalancedMcHardcoreMain.MODID, bus = Mod.EventBusSubscriber.Bus.MOD)
public class ItemInit {

    public static final DeferredRegister<Item> ITEMS =
        DeferredRegister.create(ForgeRegistries.ITEMS, BalancedMcHardcoreMain.MODID);

    public static final RegistryObject<Item> TEST_ITEM = ITEMS.register("test_item", () -> {
        FoodProperties foodProperties = new FoodProperties.Builder()
            .nutrition(5)
            .saturationMod(0.2f)
            .effect(() -> new MobEffectInstance(MobEffects.ABSORPTION, 2400, 0), 1.0f)
            .build();

        Item.Properties properties = new Item.Properties()
            .stacksTo(64)
            .food(foodProperties)
            .rarity(Rarity.EPIC);

        return new Item(properties);
    });

    public static final RegistryObject<Item> LIFE_TOTEM =
        ITEMS.register("life_totem", () -> new LifeTotemItem(new Item.Properties()
            .stacksTo(1)
            .fireResistant()
            .rarity(Rarity.EPIC))
        );

    public static final RegistryObject<Item> ETERNAL_AXE =
        ITEMS.register("eternal_axe", () -> new EternalAxeItem(
            BMHModTierInit.ETERNAL,
            10.0f,
            0.3f,
            new Item.Properties()
                .stacksTo(1)
                .fireResistant()
                .rarity(Rarity.EPIC)
        ));

    public static final RegistryObject<BlockItem> TEST_BLOCK_ITEM =
        ITEMS.register("test_block", () -> new BlockItem(BlockInit.TEST_BLOCK.get(),
            new Item.Properties()
                .rarity(Rarity.EPIC))
        );
}
