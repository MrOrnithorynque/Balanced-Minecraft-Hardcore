package github.mrornithorynque.bmh.neoforge;

import java.util.HashMap;
import java.util.Map;
import java.util.function.Supplier;

import github.mrornithorynque.bmh.BalancedMcHardcore;
import github.mrornithorynque.bmh.platform.IPlatformHelper;

import net.minecraft.core.Registry;
import net.minecraft.resources.ResourceKey;
import net.minecraft.world.item.CreativeModeTab;
import net.minecraft.world.level.GameRules;

import net.neoforged.bus.api.IEventBus;
import net.neoforged.fml.loading.FMLEnvironment;
import net.neoforged.neoforge.registries.DeferredRegister;

public class NeoForgePlatformHelper implements IPlatformHelper {

    // One DeferredRegister per registry, filled while the common init classes load.
    private static final Map<ResourceKey<?>, DeferredRegister<?>> REGISTERS = new HashMap<>();

    public static void registerAll(IEventBus modEventBus) {

        REGISTERS.values().forEach(register -> register.register(modEventBus));
    }

    @Override
    @SuppressWarnings("unchecked")
    public <T, I extends T> Supplier<I> register(ResourceKey<? extends Registry<T>> registry, String name, Supplier<I> factory) {

        DeferredRegister<T> register = (DeferredRegister<T>) REGISTERS.computeIfAbsent(registry,
                key -> DeferredRegister.create(registry, BalancedMcHardcore.MODID));

        return register.register(name, factory);
    }

    @Override
    public CreativeModeTab.Builder creativeTabBuilder() {

        return CreativeModeTab.builder();
    }

    @Override
    public GameRules.Key<GameRules.BooleanValue> registerBooleanGameRule(String name, GameRules.Category category, boolean defaultValue) {

        return GameRules.register(name, category, GameRules.BooleanValue.create(defaultValue));
    }

    @Override
    public boolean isPhysicalClient() {

        return FMLEnvironment.dist.isClient();
    }

    @Override
    public int minDistanceFromBed() {

        return NeoForgeConfig.minDistanceFromBed.get();
    }

    @Override
    public int maxDistanceFromBed() {

        return NeoForgeConfig.maxDistanceFromBed.get();
    }
}
