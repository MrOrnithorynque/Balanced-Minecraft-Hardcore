package com.MrOrnithorynque.bmh.init;

import com.MrOrnithorynque.bmh.BalancedMcHardcoreMain;

import net.minecraft.world.effect.MobEffectInstance;
import net.minecraft.world.effect.MobEffects;
import net.minecraft.world.food.FoodProperties;
import net.minecraft.world.item.CreativeModeTab;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.Rarity;
import net.minecraftforge.registries.DeferredRegister;
import net.minecraftforge.registries.ForgeRegistries;
import net.minecraftforge.registries.RegistryObject;

public class ItemInit
{
    public static final DeferredRegister<Item> ITEMS =
            DeferredRegister.create(ForgeRegistries.ITEMS, BalancedMcHardcoreMain.MODID);

    public static final RegistryObject<Item> TEST_ITEM =
            ITEMS.register("test_item", () -> new Item(new Item.Properties()
                .stacksTo(64)
                .food(new FoodProperties.Builder()
                    .nutrition(5)
                    .saturationMod(0.2f)
                    .effect(() -> new MobEffectInstance(MobEffects.ABSORPTION, 2400, 0), 1.0f)
                    .build())
                .rarity(Rarity.EPIC))
            );
}
