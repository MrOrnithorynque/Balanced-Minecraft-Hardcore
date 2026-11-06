package github.mrornithorynque.bmh.fabric;

import java.util.List;
import java.util.function.Supplier;

import github.mrornithorynque.bmh.BalancedMcHardcore;
import github.mrornithorynque.bmh.platform.IPlatformHelper;

import net.fabricmc.api.EnvType;
import net.fabricmc.fabric.api.creativetab.v1.FabricCreativeModeTab;
import net.fabricmc.loader.api.FabricLoader;
import net.minecraft.core.Registry;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.resources.ResourceKey;
import net.minecraft.resources.Identifier;
import net.minecraft.world.item.CreativeModeTab;
import net.minecraft.world.level.ItemLike;

public class FabricPlatformHelper implements IPlatformHelper {

    @Override
    @SuppressWarnings("unchecked")
    public <T, I extends T> Supplier<I> register(ResourceKey<? extends Registry<T>> registry, String name, Supplier<I> factory) {

        Registry<T> target = (Registry<T>) BuiltInRegistries.REGISTRY.getValue(registry.identifier());
        I object = Registry.register(target, Identifier.fromNamespaceAndPath(BalancedMcHardcore.MODID, name), factory.get());

        return () -> object;
    }

    @Override
    public CreativeModeTab.Builder creativeTabBuilder(List<Supplier<? extends ItemLike>> items) {

        return FabricCreativeModeTab.builder().displayItems((parameters, output) -> items.forEach(item -> output.accept(item.get())));
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

    @Override
    public void saveConfig(int minDistanceFromBed, int maxDistanceFromBed, int maxAmuletPoints) {

        FabricConfig.save(minDistanceFromBed, maxDistanceFromBed, maxAmuletPoints);
    }
}
