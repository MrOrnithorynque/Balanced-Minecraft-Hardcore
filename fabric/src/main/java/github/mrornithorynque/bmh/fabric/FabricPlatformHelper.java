package github.mrornithorynque.bmh.fabric;

import java.util.function.Supplier;

import github.mrornithorynque.bmh.BalancedMcHardcore;
import github.mrornithorynque.bmh.platform.IPlatformHelper;

import net.fabricmc.api.EnvType;
import net.fabricmc.fabric.api.gamerule.v1.GameRuleFactory;
import net.fabricmc.fabric.api.gamerule.v1.GameRuleRegistry;
import net.fabricmc.fabric.api.itemgroup.v1.FabricItemGroup;
import net.fabricmc.loader.api.FabricLoader;
import net.minecraft.core.Registry;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.resources.ResourceKey;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.world.item.CreativeModeTab;
import net.minecraft.world.level.GameRules;

public class FabricPlatformHelper implements IPlatformHelper {

    @Override
    @SuppressWarnings("unchecked")
    public <T, I extends T> Supplier<I> register(ResourceKey<? extends Registry<T>> registry, String name, Supplier<I> factory) {

        Registry<T> target = (Registry<T>) BuiltInRegistries.REGISTRY.get(registry.location());
        I object = Registry.register(target, ResourceLocation.fromNamespaceAndPath(BalancedMcHardcore.MODID, name), factory.get());

        return () -> object;
    }

    @Override
    public CreativeModeTab.Builder creativeTabBuilder() {

        return FabricItemGroup.builder();
    }

    @Override
    public GameRules.Key<GameRules.BooleanValue> registerBooleanGameRule(String name, GameRules.Category category, boolean defaultValue) {

        return GameRuleRegistry.register(name, category, GameRuleFactory.createBooleanRule(defaultValue));
    }

    @Override
    public boolean isPhysicalClient() {

        return FabricLoader.getInstance().getEnvironmentType() == EnvType.CLIENT;
    }

    @Override
    public int minDistanceFromBed() {

        return FabricConfig.get().minDistanceFromBed;
    }

    @Override
    public int maxDistanceFromBed() {

        return FabricConfig.get().maxDistanceFromBed;
    }

    @Override
    public int maxAmuletPoints() {

        return FabricConfig.get().maxAmuletPoints;
    }
}
